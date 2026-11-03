// SPDX-License-Identifier: GPL-3.0-or-later
//! The corpus of real design files, measured and copied without ever being changed.
//!
//! A corpus is a folder of files that somebody really made (libraries `.lbr`, schematics `.sch`, boards `.brd`). This module does two things with it and nothing else:
//!
//! * [`inventory`] reads each file once and counts them by **level** (library, schematic, board) and by **family and version** (XML with the version its header gives, the legacy
//!   binary form, anything else). The report holds counts, sizes and one digest of the whole set; it holds no file name, no folder name and no content, so it can be kept
//!   next to the code. The digest is the SHA-256 of the sorted list of the SHA-256 of every file: it names the set without naming anything in it.
//! * [`copy`] makes a working copy of the design files in another folder, with a `SHA256SUMS` of the copy, and reads each copy back to prove it is what was read. The folder
//!   it reads from is opened for reading only: the originals are never written, renamed or deleted, and a copy inside the original folder (or the original inside the copy)
//!   is refused.
//!
//! What it does not do: it does not parse a design file. The version comes from the first bytes of an XML header, and the legacy binary form is told only by its first byte;
//! whether a file really is what it claims is the job of a reader, and the results of the readers come to the laboratory as classified results.

use crate::config::Config;
use crate::error::LabError;
use crate::hash::{sha256_hex, sha256_reader};
use crate::manifest::{check_path, Manifest, MANIFEST_NAME};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

/// The level of a design file, by its extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// A library (`.lbr`).
    Library,
    /// A schematic (`.sch`).
    Schematic,
    /// A board (`.brd`).
    Board,
}

impl Kind {
    /// The kind of an extension (any letter case), when the configuration lists it.
    pub fn of_extension(extension: &str, config: &Config) -> Option<Kind> {
        let lower = extension.to_ascii_lowercase();
        if !config.corpus_extensions.contains(&lower) {
            return None;
        }
        match lower.as_str() {
            "lbr" => Some(Kind::Library),
            "sch" => Some(Kind::Schematic),
            "brd" => Some(Kind::Board),
            _ => None,
        }
    }

    /// The word used in reports.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Library => "library",
            Kind::Schematic => "schematic",
            Kind::Board => "board",
        }
    }
}

/// What the first bytes of a file say it is.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family {
    /// XML whose `<eagle version="...">` header gives this version.
    Xml(String),
    /// The legacy binary form, told only by its first byte (0x10). Whether it is one is for a reader to say.
    LegacyBinary,
    /// Anything else: empty, text that is not an XML design, or bytes that start otherwise.
    Other,
}

impl Family {
    /// The word used in reports.
    pub fn label(&self) -> String {
        match self {
            Family::Xml(version) => format!("xml {version}"),
            Family::LegacyBinary => "legacy-binary".to_string(),
            Family::Other => "other".to_string(),
        }
    }
}

/// Tell the family from the first bytes of a file.
pub fn family_of(head: &[u8]) -> Family {
    let head = head.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(head);
    if head.first() == Some(&0x10) {
        return Family::LegacyBinary;
    }
    let trimmed = {
        let start = head
            .iter()
            .position(|b| !b.is_ascii_whitespace())
            .unwrap_or(head.len());
        &head[start..]
    };
    if !trimmed.starts_with(b"<?xml") {
        return Family::Other;
    }
    let marker = b"<eagle version=\"";
    let Some(at) = trimmed
        .windows(marker.len())
        .position(|window| window == marker)
    else {
        return Family::Other;
    };
    let rest = &trimmed[at + marker.len()..];
    let Some(end) = rest.iter().position(|b| *b == b'"') else {
        return Family::Other;
    };
    let version = &rest[..end];
    let well_formed = !version.is_empty()
        && version.len() <= 16
        && version.iter().all(|b| b.is_ascii_digit() || *b == b'.')
        && version[0].is_ascii_digit();
    if !well_formed {
        return Family::Other;
    }
    match std::str::from_utf8(version) {
        Ok(text) => Family::Xml(text.to_string()),
        Err(_) => Family::Other,
    }
}

/// What the walk found.
struct Found {
    /// The design files, as `/` separated relative paths, sorted.
    files: Vec<(String, Kind)>,
    /// Every plain file seen, design or not.
    total_files: u64,
    /// Links, devices and names that are not UTF-8: left alone and counted.
    skipped_unsupported: u64,
    /// Design files whose path is longer than the configuration allows or is not a safe path.
    skipped_path: u64,
}

fn io_error(path: &Path) -> impl FnOnce(std::io::Error) -> LabError + '_ {
    move |source| LabError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn walk(root: &Path, config: &Config) -> Result<Found, LabError> {
    let mut found = Found {
        files: Vec::new(),
        total_files: 0,
        skipped_unsupported: 0,
        skipped_path: 0,
    };
    let mut pending = vec![(root.to_path_buf(), String::new())];
    while let Some((dir, prefix)) = pending.pop() {
        let Ok(read) = fs::read_dir(&dir) else {
            // a folder that cannot be read is counted, not fatal: a corpus is somebody's disk
            found.skipped_unsupported += 1;
            continue;
        };
        for item in read {
            let Ok(item) = item else {
                found.skipped_unsupported += 1;
                continue;
            };
            let Ok(name) = item.file_name().into_string() else {
                found.skipped_unsupported += 1;
                continue;
            };
            let Ok(file_type) = item.file_type() else {
                found.skipped_unsupported += 1;
                continue;
            };
            let relative = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            if file_type.is_symlink() {
                found.skipped_unsupported += 1;
            } else if file_type.is_dir() {
                pending.push((item.path(), relative));
            } else if file_type.is_file() {
                found.total_files += 1;
                let kind = Path::new(&name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .and_then(|e| Kind::of_extension(e, config));
                if let Some(kind) = kind {
                    if relative == MANIFEST_NAME || check_path(&relative, config, 0).is_err() {
                        found.skipped_path += 1;
                    } else if found.files.len() >= config.max_entries {
                        return Err(LabError::TooLarge { what: "entries" });
                    } else {
                        found.files.push((relative, kind));
                    }
                }
            } else {
                found.skipped_unsupported += 1;
            }
        }
    }
    found.files.sort();
    Ok(found)
}

/// How many design files of one level and family, and how many bytes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cell {
    /// Files.
    pub files: u64,
    /// Bytes.
    pub bytes: u64,
}

/// The count of a corpus by level and by family and version. It holds no name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    /// Every plain file seen under the folder, design or not.
    pub total_files: u64,
    /// The design files read.
    pub design_files: u64,
    /// Bytes of the design files read.
    pub design_bytes: u64,
    /// Links, devices, unreadable folders and names that are not UTF-8, left alone.
    pub skipped_unsupported: u64,
    /// Design files left out for a path that is too long or not safe.
    pub skipped_path: u64,
    /// Design files that could not be read.
    pub unreadable: u64,
    /// Files and bytes by level and family.
    pub cells: BTreeMap<(Kind, Family), Cell>,
    /// SHA-256 of the sorted SHA-256 hashes of the design files: names the set, names nothing in it.
    pub set_digest: String,
}

impl Inventory {
    /// The report as JSON, with no name and no path.
    pub fn to_json(&self) -> Result<String, LabError> {
        let cells: Vec<serde_json::Value> = self
            .cells
            .iter()
            .map(|((kind, family), cell)| {
                serde_json::json!({
                    "level": kind.name(), "family": family.label(), "files": cell.files, "bytes": cell.bytes,
                })
            })
            .collect();
        let value = serde_json::json!({
            "total_files": self.total_files, "design_files": self.design_files, "design_bytes": self.design_bytes,
            "skipped_unsupported": self.skipped_unsupported, "skipped_path": self.skipped_path, "unreadable": self.unreadable,
            "cells": cells, "set_digest": self.set_digest,
        });
        serde_json::to_string_pretty(&value).map_err(|_| LabError::BadBaseline)
    }
}

/// Read the first bytes of a file and its whole hash in one pass. Gives the family, the hash and the size.
fn read_once(path: &Path, config: &Config) -> std::io::Result<(Family, String, u64)> {
    let mut file = fs::File::open(path)?;
    let mut head = vec![0u8; config.corpus_head_bytes];
    let mut filled = 0;
    while filled < head.len() {
        let n = file.read(&mut head[filled..])?;
        if n == 0 {
            break;
        }
        filled += n;
    }
    head.truncate(filled);
    let family = family_of(&head);
    let mut counted = 0u64;
    let hash = sha256_reader(CountingReader {
        inner: Cursor::new(head).chain(file),
        count: &mut counted,
    })?;
    Ok((family, hash, counted))
}

struct CountingReader<'a, R: Read> {
    inner: R,
    count: &'a mut u64,
}

impl<R: Read> Read for CountingReader<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        *self.count += n as u64;
        Ok(n)
    }
}

/// Count the design files under `root` by level and by family and version. The folder is only read.
pub fn inventory(root: &Path, config: &Config) -> Result<Inventory, LabError> {
    let found = walk(root, config)?;
    let mut inventory = Inventory {
        total_files: found.total_files,
        design_files: 0,
        design_bytes: 0,
        skipped_unsupported: found.skipped_unsupported,
        skipped_path: found.skipped_path,
        unreadable: 0,
        cells: BTreeMap::new(),
        set_digest: String::new(),
    };
    let mut hashes = Vec::with_capacity(found.files.len());
    for (relative, kind) in &found.files {
        match read_once(&root.join(relative), config) {
            Ok((family, hash, size)) => {
                inventory.design_files += 1;
                inventory.design_bytes += size;
                let cell = inventory.cells.entry((*kind, family)).or_default();
                cell.files += 1;
                cell.bytes += size;
                hashes.push(hash);
            }
            Err(_) => inventory.unreadable += 1,
        }
    }
    hashes.sort();
    let mut joined = String::with_capacity(hashes.len() * 65);
    for hash in &hashes {
        joined.push_str(hash);
        joined.push('\n');
    }
    inventory.set_digest = sha256_hex(joined.as_bytes());
    Ok(inventory)
}

/// What a copy did. It holds no name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyReport {
    /// Design files copied and checked.
    pub copied: u64,
    /// Bytes copied.
    pub bytes: u64,
    /// Design files left out for a path that is too long or not safe.
    pub skipped_path: u64,
    /// Design files left out because another one differs from them only in letter case.
    pub skipped_duplicate: u64,
    /// Links, devices, unreadable folders and names that are not UTF-8, left alone.
    pub skipped_unsupported: u64,
    /// SHA-256 of the `SHA256SUMS` written in the copy.
    pub manifest_sha256: String,
    /// SHA-256 of the sorted SHA-256 hashes of the files, as in the inventory.
    pub set_digest: String,
}

/// The real path a folder has or would have: the nearest folder that exists, resolved, with the rest of the name put back.
fn resolve(path: &Path) -> Result<PathBuf, LabError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(io_error(path))?.join(path)
    };
    let mut tail = Vec::new();
    let mut at = absolute.as_path();
    loop {
        if let Ok(real) = fs::canonicalize(at) {
            let mut out = real;
            for part in tail.iter().rev() {
                out.push(part);
            }
            return Ok(out);
        }
        match (at.parent(), at.file_name()) {
            (Some(parent), Some(name)) => {
                tail.push(name.to_os_string());
                at = parent;
            }
            _ => return Err(LabError::UnsafeTarget),
        }
    }
}

fn inside(inner: &Path, outer: &Path) -> bool {
    inner.starts_with(outer)
}

/// Make a working copy of the design files of `src` in `dst`, with a `SHA256SUMS`, reading each copy back to prove it. `dst` must not exist or must be an empty
/// folder, and neither folder may be inside the other. `src` is only read.
pub fn copy(src: &Path, dst: &Path, config: &Config) -> Result<CopyReport, LabError> {
    let src_real = fs::canonicalize(src).map_err(io_error(src))?;
    // the target is judged before anything is made: a refused copy leaves no folder behind, least of all inside the original
    let dst_real = resolve(dst)?;
    if inside(&dst_real, &src_real) || inside(&src_real, &dst_real) {
        return Err(LabError::UnsafeTarget);
    }
    if dst.exists() {
        let mut entries = fs::read_dir(dst).map_err(io_error(dst))?;
        if entries.next().is_some() {
            return Err(LabError::UnsafeTarget);
        }
    }
    fs::create_dir_all(dst).map_err(io_error(dst))?;
    let found = walk(src, config)?;
    let mut manifest = Manifest::new();
    let mut folded = BTreeSet::new();
    let mut report = CopyReport {
        copied: 0,
        bytes: 0,
        skipped_path: found.skipped_path,
        skipped_duplicate: 0,
        skipped_unsupported: found.skipped_unsupported,
        manifest_sha256: String::new(),
        set_digest: String::new(),
    };
    let mut hashes = Vec::new();
    for (relative, _) in &found.files {
        if !folded.insert(relative.to_lowercase()) {
            report.skipped_duplicate += 1;
            continue;
        }
        let from = src.join(relative);
        let to = dst.join(relative);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(io_error(parent))?;
        }
        let hash = copy_file(&from, &to)?;
        let back = fs::File::open(&to)
            .and_then(sha256_reader)
            .map_err(io_error(&to))?;
        if back != hash {
            return Err(LabError::CopyMismatch);
        }
        let size = fs::metadata(&to).map_err(io_error(&to))?.len();
        manifest.insert(relative, &hash, config, 0)?;
        hashes.push(hash);
        report.copied += 1;
        report.bytes += size;
    }
    let text = manifest.render();
    let manifest_path: PathBuf = dst.join(MANIFEST_NAME);
    fs::write(&manifest_path, text.as_bytes()).map_err(io_error(&manifest_path))?;
    report.manifest_sha256 = sha256_hex(text.as_bytes());
    hashes.sort();
    let mut joined = String::new();
    for hash in &hashes {
        joined.push_str(hash);
        joined.push('\n');
    }
    report.set_digest = sha256_hex(joined.as_bytes());
    Ok(report)
}

/// Copy one file, reading the source only, and give the SHA-256 of what was read.
fn copy_file(from: &Path, to: &Path) -> Result<String, LabError> {
    use sha2::{Digest, Sha256};
    let mut input = fs::File::open(from).map_err(io_error(from))?;
    let mut output = fs::File::create(to).map_err(io_error(to))?;
    let mut hasher = Sha256::new();
    let mut block = vec![0u8; 64 * 1024];
    loop {
        let n = input.read(&mut block).map_err(io_error(from))?;
        if n == 0 {
            break;
        }
        hasher.update(&block[..n]);
        output.write_all(&block[..n]).map_err(io_error(to))?;
    }
    output.sync_all().map_err(io_error(to))?;
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verify::{verify_copies, EntryStatus};

    const XML9: &[u8] = b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE eagle SYSTEM \"eagle.dtd\">\n<eagle version=\"9.5.2\">\n<drawing/></eagle>\n";
    const XML6: &[u8] = b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<!DOCTYPE eagle SYSTEM \"eagle.dtd\">\n<eagle version=\"6.0.0\">\n</eagle>\n";

    fn cfg() -> Config {
        Config::default()
    }

    fn write(root: &Path, relative: &str, bytes: &[u8]) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn sample(root: &Path) {
        write(root, "customer_secret/board_one.brd", XML9);
        write(root, "customer_secret/board_one.sch", XML9);
        write(root, "old/Legacy.BRD", &[0x10, 0x80, 0x00, 0x01, 0x02]);
        write(root, "old/parts.lbr", XML6);
        write(root, "old/deep/er/parts2.lbr", XML6);
        write(root, "notes.txt", b"not a design");
        write(root, "junk.sch", b"plain text that is not a design");
        write(root, "empty.brd", b"");
    }

    #[test]
    fn the_family_comes_from_the_first_bytes() {
        assert_eq!(family_of(XML9), Family::Xml("9.5.2".into()));
        assert_eq!(family_of(XML6), Family::Xml("6.0.0".into()));
        let mut bom = vec![0xEF, 0xBB, 0xBF];
        bom.extend_from_slice(XML9);
        assert_eq!(family_of(&bom), Family::Xml("9.5.2".into()));
        assert_eq!(
            family_of(b"\n\n  <?xml version=\"1.0\"?><eagle version=\"7.7.0\">"),
            Family::Xml("7.7.0".into())
        );
        assert_eq!(family_of(&[0x10, 0x80, 1, 2]), Family::LegacyBinary);
        for other in [
            &b""[..],
            b"hello",
            b"<?xml version=\"1.0\"?><svg/>",
            b"<?xml version=\"1.0\"?><eagle version=\"9.5.2",
            b"<?xml version=\"1.0\"?><eagle version=\"9.5.2<x\">",
            b"<?xml version=\"1.0\"?><eagle version=\"\">",
            b"<?xml version=\"1.0\"?><eagle version=\"12345678901234567\">",
            b"<eagle version=\"9.5.2\">",
        ] {
            assert_eq!(family_of(other), Family::Other, "{other:?}");
        }
    }

    #[test]
    fn the_inventory_counts_by_level_and_family_and_ignores_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        sample(dir.path());
        let inv = inventory(dir.path(), &cfg()).unwrap();
        assert_eq!(inv.total_files, 8);
        assert_eq!(inv.design_files, 7, "notes.txt is not a design file");
        let get = |k, f: Family| inv.cells.get(&(k, f)).cloned().unwrap_or_default().files;
        assert_eq!(get(Kind::Board, Family::Xml("9.5.2".into())), 1);
        assert_eq!(get(Kind::Schematic, Family::Xml("9.5.2".into())), 1);
        assert_eq!(get(Kind::Library, Family::Xml("6.0.0".into())), 2);
        assert_eq!(
            get(Kind::Board, Family::LegacyBinary),
            1,
            "the extension in capitals counts"
        );
        assert_eq!(get(Kind::Schematic, Family::Other), 1);
        assert_eq!(
            get(Kind::Board, Family::Other),
            1,
            "an empty file is read and is `other`"
        );
        assert_eq!(inv.unreadable, 0);
        assert_eq!(
            inv.design_bytes,
            (XML9.len() * 2 + XML6.len() * 2 + 5 + 31) as u64
        );
    }

    #[test]
    fn the_report_holds_no_name_and_no_content() {
        let dir = tempfile::tempdir().unwrap();
        sample(dir.path());
        let json = inventory(dir.path(), &cfg()).unwrap().to_json().unwrap();
        for leak in [
            "customer",
            "secret",
            "board_one",
            "Legacy",
            "parts",
            "deep",
            "drawing",
            "notes",
            dir.path().to_str().unwrap(),
        ] {
            assert!(!json.contains(leak), "{leak} in {json}");
        }
        assert!(json.contains("\"level\": \"board\"") && json.contains("xml 9.5.2"));
    }

    #[test]
    fn the_digest_names_the_content_not_the_names() {
        let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        write(a.path(), "x/one.brd", XML9);
        write(a.path(), "two.lbr", XML6);
        write(b.path(), "other/name.brd", XML9);
        write(b.path(), "again/three.lbr", XML6);
        let (da, db) = (
            inventory(a.path(), &cfg()).unwrap().set_digest,
            inventory(b.path(), &cfg()).unwrap().set_digest,
        );
        assert_eq!(da, db, "the same files under other names are the same set");
        write(
            b.path(),
            "again/three.lbr",
            b"<?xml version=\"1.0\"?><eagle version=\"6.0.1\">",
        );
        assert_ne!(da, inventory(b.path(), &cfg()).unwrap().set_digest);
        assert_eq!(da.len(), 64);
    }

    #[test]
    fn a_copy_is_byte_for_byte_with_a_manifest_that_both_copies_verify_against() {
        let (src, one, two) = (
            tempfile::tempdir().unwrap(),
            tempfile::tempdir().unwrap(),
            tempfile::tempdir().unwrap(),
        );
        sample(src.path());
        let (d1, d2) = (one.path().join("copy"), two.path().join("copy"));
        let r1 = copy(src.path(), &d1, &cfg()).unwrap();
        let r2 = copy(src.path(), &d2, &cfg()).unwrap();
        assert_eq!(
            (r1.copied, r1.skipped_path, r1.skipped_duplicate),
            (7, 0, 0)
        );
        assert_eq!(r1.set_digest, r2.set_digest);
        assert_eq!(r1.manifest_sha256, r2.manifest_sha256);
        assert_eq!(
            r1.set_digest,
            inventory(src.path(), &cfg()).unwrap().set_digest
        );
        assert_eq!(
            fs::read(d1.join("customer_secret/board_one.brd")).unwrap(),
            XML9
        );
        assert!(
            !d1.join("notes.txt").exists(),
            "only the design files are copied"
        );
        // the redundant copies agree with the manifest
        let manifest =
            Manifest::parse(&fs::read_to_string(d1.join(MANIFEST_NAME)).unwrap(), &cfg()).unwrap();
        let report = verify_copies(&manifest, &[d1.clone(), d2.clone()], &cfg()).unwrap();
        assert!(report.is_redundant_and_clean(&cfg()));
        // and a changed byte in one copy is found
        fs::write(d2.join("old/parts.lbr"), b"changed").unwrap();
        let report = verify_copies(&manifest, &[d1, d2], &cfg()).unwrap();
        assert!(!report.is_redundant_and_clean(&cfg()));
        assert!(report.copies[1]
            .entries
            .iter()
            .any(|(_, s)| *s != EntryStatus::Ok));
    }

    #[test]
    fn the_originals_are_never_touched() {
        let (src, out) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        sample(src.path());
        // the originals are read-only, as a golden set should be
        let mut files = Vec::new();
        for (relative, _) in &walk(src.path(), &cfg()).unwrap().files {
            let path = src.path().join(relative);
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_readonly(true);
            fs::set_permissions(&path, permissions).unwrap();
            files.push(path);
        }
        let before = Manifest::from_dir(src.path(), &cfg()).unwrap().render();
        let times: Vec<_> = files
            .iter()
            .map(|p| fs::metadata(p).unwrap().modified().unwrap())
            .collect();
        copy(src.path(), &out.path().join("c"), &cfg()).unwrap();
        inventory(src.path(), &cfg()).unwrap();
        assert_eq!(
            before,
            Manifest::from_dir(src.path(), &cfg()).unwrap().render(),
            "same bytes, same names"
        );
        let after: Vec<_> = files
            .iter()
            .map(|p| fs::metadata(p).unwrap().modified().unwrap())
            .collect();
        assert_eq!(times, after, "not even the modification time moved");
        for p in &files {
            assert!(fs::metadata(p).unwrap().permissions().readonly());
            let mut permissions = fs::metadata(p).unwrap().permissions();
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            fs::set_permissions(p, permissions).unwrap();
        }
        assert!(
            !src.path().join(MANIFEST_NAME).exists(),
            "nothing was written in the original folder"
        );
    }

    #[test]
    fn a_copy_will_not_go_inside_the_original_nor_the_original_inside_the_copy_nor_over_files() {
        let (src, out) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        sample(src.path());
        assert!(matches!(
            copy(src.path(), &src.path().join("copy"), &cfg()),
            Err(LabError::UnsafeTarget)
        ));
        assert!(matches!(
            copy(src.path(), src.path(), &cfg()),
            Err(LabError::UnsafeTarget)
        ));
        assert!(matches!(
            copy(&src.path().join("old"), src.path(), &cfg()),
            Err(LabError::UnsafeTarget)
        ));
        // a target that already holds something is not mixed in
        write(out.path(), "already/here.txt", b"x");
        assert!(matches!(
            copy(src.path(), out.path(), &cfg()),
            Err(LabError::UnsafeTarget)
        ));
        // the refusals left the original folder as it was
        assert!(
            !src.path().join("copy").exists(),
            "a refused copy leaves nothing behind"
        );
        assert!(matches!(
            copy(
                &src.path().join("missing"),
                &out.path().join("fresh"),
                &cfg()
            ),
            Err(LabError::Io { .. })
        ));
    }

    #[test]
    fn long_paths_and_names_that_differ_only_in_case_are_left_out_and_counted() {
        let (src, out) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        write(src.path(), "ok.brd", XML9);
        let long = format!("{}.brd", "n".repeat(300));
        write(src.path(), &long, XML9);
        let config = Config {
            max_path_bytes: 100,
            ..Config::default()
        };
        let report = copy(src.path(), &out.path().join("c"), &config).unwrap();
        assert_eq!((report.copied, report.skipped_path), (1, 1));
        let inv = inventory(src.path(), &config).unwrap();
        assert_eq!((inv.design_files, inv.skipped_path), (1, 1));
        // two files that differ only in case: the second is left out where case does not tell them apart
        let (src2, out2) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        write(src2.path(), "a/Board.brd", XML9);
        if fs::write(src2.path().join("a/BOARD.brd"), XML6).is_ok()
            && fs::read_dir(src2.path().join("a")).unwrap().count() == 2
        {
            let r = copy(src2.path(), &out2.path().join("c"), &cfg()).unwrap();
            assert_eq!((r.copied, r.skipped_duplicate), (1, 1));
        }
    }

    #[test]
    fn the_extensions_and_the_size_of_the_head_are_settings() {
        let dir = tempfile::tempdir().unwrap();
        sample(dir.path());
        let only_boards = Config::parse("corpus_extensions = brd\n").unwrap();
        assert_eq!(inventory(dir.path(), &only_boards).unwrap().design_files, 3);
        // a head too short to hold the header makes the XML files `other`: the setting matters
        let short = Config::parse("corpus_head_bytes = 8\n").unwrap();
        let inv = inventory(dir.path(), &short).unwrap();
        assert!(inv.cells.keys().all(|(_, f)| !matches!(f, Family::Xml(_))));
        assert_eq!(
            inv.design_files, 7,
            "the whole file is hashed whatever the head"
        );
        assert!(Config::parse("corpus_extensions = brd,exe\n").is_err());
        assert!(Config::parse("corpus_head_bytes = 0\n").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn links_are_left_alone_and_counted() {
        let (src, outside) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        write(src.path(), "real.brd", XML9);
        write(outside.path(), "other.brd", XML9);
        std::os::unix::fs::symlink(
            outside.path().join("other.brd"),
            src.path().join("link.brd"),
        )
        .unwrap();
        std::os::unix::fs::symlink(outside.path(), src.path().join("dirlink")).unwrap();
        let inv = inventory(src.path(), &cfg()).unwrap();
        assert_eq!((inv.design_files, inv.skipped_unsupported), (1, 2));
    }
}

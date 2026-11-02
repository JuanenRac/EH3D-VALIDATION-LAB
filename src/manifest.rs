// SPDX-License-Identifier: GPL-3.0-or-later
//! The manifest of a golden set: one line `<sha256>  <relative path>` per file, in the plain
//! `SHA256SUMS` form that standard checksum tools also read. Written sorted, so the same set
//! always gives the same bytes.

use crate::config::Config;
use crate::error::LabError;
use crate::hash::{is_sha256_hex, sha256_reader};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// Name of the manifest file inside a golden set; it is never listed in itself.
pub const MANIFEST_NAME: &str = "SHA256SUMS";

/// Check that a path is a safe relative path: `/` separated, no empty, `.` or `..` part, no
/// backslash, colon or control character, and within the configured length.
pub fn check_path(path: &str, config: &Config, line: usize) -> Result<(), LabError> {
    let bad = LabError::BadPath { line };
    if path.is_empty() || path.len() > config.max_path_bytes {
        return Err(bad);
    }
    if path
        .chars()
        .any(|c| c.is_control() || c == '\\' || c == ':')
    {
        return Err(bad);
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(bad);
    }
    Ok(())
}

/// The files of a golden set and the SHA-256 of each.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Manifest {
    entries: BTreeMap<String, String>,
    folded: BTreeSet<String>,
}

impl Manifest {
    /// An empty manifest.
    pub fn new() -> Manifest {
        Manifest::default()
    }

    /// Number of files listed.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when no file is listed.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The listed files with their hashes, sorted by path.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().map(|(p, h)| (p.as_str(), h.as_str()))
    }

    /// True when the path is listed.
    pub fn contains(&self, path: &str) -> bool {
        self.entries.contains_key(path)
    }

    /// Add a file. Refuses unsafe paths, bad hashes, a full manifest and duplicates (paths that
    /// differ only in letter case count as the same on Windows).
    pub fn insert(
        &mut self,
        path: &str,
        sha256: &str,
        config: &Config,
        line: usize,
    ) -> Result<(), LabError> {
        check_path(path, config, line)?;
        if !is_sha256_hex(sha256) {
            return Err(LabError::ManifestSyntax { line });
        }
        if self.entries.len() >= config.max_entries {
            return Err(LabError::TooLarge { what: "entries" });
        }
        if !self.folded.insert(path.to_lowercase()) {
            return Err(LabError::DuplicatePath { line });
        }
        self.entries.insert(path.to_string(), sha256.to_string());
        Ok(())
    }

    /// Parse the text of a manifest.
    pub fn parse(text: &str, config: &Config) -> Result<Manifest, LabError> {
        if text.len() as u64 > config.max_manifest_bytes {
            return Err(LabError::TooLarge {
                what: "manifest bytes",
            });
        }
        let mut manifest = Manifest::new();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            if raw.is_empty() {
                continue;
            }
            let (hash, path) = raw
                .split_once("  ")
                .ok_or(LabError::ManifestSyntax { line })?;
            manifest.insert(path, hash, config, line)?;
        }
        Ok(manifest)
    }

    /// The text of the manifest: sorted by path, `\n` line ends, nothing else.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for (path, hash) in &self.entries {
            out.push_str(hash);
            out.push_str("  ");
            out.push_str(path);
            out.push('\n');
        }
        out
    }

    /// Build the manifest of a directory by hashing every file in it (the manifest file itself excluded).
    pub fn from_dir(root: &Path, config: &Config) -> Result<Manifest, LabError> {
        let mut manifest = Manifest::new();
        for path in list_files(root, config)? {
            let full = root.join(&path);
            let file = fs::File::open(&full).map_err(|source| LabError::Io {
                path: full.clone(),
                source,
            })?;
            let hash = sha256_reader(file).map_err(|source| LabError::Io { path: full, source })?;
            manifest.insert(&path, &hash, config, 0)?;
        }
        Ok(manifest)
    }
}

/// Every plain file under `root` as a `/` separated relative path, sorted, without the manifest
/// file at the top. A link, a device or a name that is not UTF-8 is refused, never followed.
pub fn list_files(root: &Path, config: &Config) -> Result<Vec<String>, LabError> {
    let mut files = Vec::new();
    let mut pending = vec![(root.to_path_buf(), String::new())];
    while let Some((dir, prefix)) = pending.pop() {
        let read = fs::read_dir(&dir).map_err(|source| LabError::Io {
            path: dir.clone(),
            source,
        })?;
        for item in read {
            let item = item.map_err(|source| LabError::Io {
                path: dir.clone(),
                source,
            })?;
            let name = item
                .file_name()
                .into_string()
                .map_err(|_| LabError::UnsupportedEntry)?;
            let relative = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            let kind = item.file_type().map_err(|source| LabError::Io {
                path: item.path(),
                source,
            })?;
            if kind.is_symlink() {
                return Err(LabError::UnsupportedEntry);
            } else if kind.is_dir() {
                pending.push((item.path(), relative));
            } else if kind.is_file() {
                if relative == MANIFEST_NAME {
                    continue;
                }
                if files.len() >= config.max_entries {
                    return Err(LabError::TooLarge { what: "entries" });
                }
                files.push(relative);
            } else {
                return Err(LabError::UnsupportedEntry);
            }
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::sha256_hex;

    fn cfg() -> Config {
        Config::default()
    }

    #[test]
    fn render_and_parse_round_trip_and_are_sorted() {
        let mut m = Manifest::new();
        m.insert("b/two.txt", &sha256_hex(b"2"), &cfg(), 0).unwrap();
        m.insert("a.txt", &sha256_hex(b"1"), &cfg(), 0).unwrap();
        let text = m.render();
        assert!(text.starts_with(&sha256_hex(b"1")));
        assert_eq!(Manifest::parse(&text, &cfg()).unwrap(), m);
        assert_eq!(Manifest::parse(&text, &cfg()).unwrap().render(), text);
    }

    #[test]
    fn unsafe_paths_are_refused() {
        let h = sha256_hex(b"x");
        for path in [
            "",
            "/abs",
            "../up",
            "a/../b",
            "a//b",
            "a\\b",
            "c:x",
            "a/./b",
            "tab\there",
            "dir/",
        ] {
            let mut m = Manifest::new();
            assert!(
                matches!(
                    m.insert(path, &h, &cfg(), 3),
                    Err(LabError::BadPath { line: 3 })
                ),
                "{path:?} must be refused"
            );
        }
        let long = "x".repeat(cfg().max_path_bytes + 1);
        assert!(Manifest::new().insert(&long, &h, &cfg(), 1).is_err());
    }

    #[test]
    fn duplicates_including_case_only_differences_are_refused() {
        let h = sha256_hex(b"x");
        let text = format!("{h}  File.txt\n{h}  file.TXT\n");
        assert!(matches!(
            Manifest::parse(&text, &cfg()),
            Err(LabError::DuplicatePath { line: 2 })
        ));
    }

    #[test]
    fn malformed_lines_name_their_line() {
        let h = sha256_hex(b"x");
        for (text, line) in [
            ("nothing here\n".to_string(), 1),
            (format!("{h} single-space.txt\n"), 1),
            (format!("{}  upper.txt\n", h.to_uppercase()), 1),
            (format!("{h}  ok.txt\n{}  short.txt\n", &h[..10]), 2),
        ] {
            assert!(
                matches!(Manifest::parse(&text, &cfg()), Err(LabError::ManifestSyntax { line: l }) if l == line),
                "{text:?}"
            );
        }
    }

    #[test]
    fn limits_are_enforced() {
        let h = sha256_hex(b"x");
        let small = Config {
            max_entries: 1,
            ..Config::default()
        };
        let text = format!("{h}  a\n{h}  b\n");
        assert!(matches!(
            Manifest::parse(&text, &small),
            Err(LabError::TooLarge { .. })
        ));
        let tiny = Config {
            max_manifest_bytes: 10,
            ..Config::default()
        };
        assert!(matches!(
            Manifest::parse(&text, &tiny),
            Err(LabError::TooLarge { .. })
        ));
    }

    #[test]
    fn from_dir_hashes_every_file_and_skips_its_own_manifest() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("b.bin"), b"bee").unwrap();
        fs::write(dir.path().join("a.txt"), b"ay").unwrap();
        fs::write(dir.path().join(MANIFEST_NAME), b"ignored").unwrap();
        let m = Manifest::from_dir(dir.path(), &cfg()).unwrap();
        let listed: Vec<_> = m.iter().collect();
        assert_eq!(
            listed,
            vec![
                ("a.txt", sha256_hex(b"ay").as_str()),
                ("sub/b.bin", sha256_hex(b"bee").as_str())
            ]
        );
    }

    #[test]
    fn a_missing_directory_is_an_io_error_not_a_panic() {
        let dir = tempfile::tempdir().unwrap();
        let gone = dir.path().join("absent");
        assert!(matches!(
            Manifest::from_dir(&gone, &cfg()),
            Err(LabError::Io { .. })
        ));
    }
}

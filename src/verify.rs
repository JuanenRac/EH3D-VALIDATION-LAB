// SPDX-License-Identifier: GPL-3.0-or-later
//! Verification of redundant copies of a golden set against its manifest.
//!
//! The laboratory can tell that two folders hold the same bytes; it cannot tell that they are on
//! two different media. Putting the copies on separate media is the owner's job.

use crate::config::Config;
use crate::error::LabError;
use crate::hash::sha256_reader;
use crate::manifest::{list_files, Manifest};
use std::fs;
use std::path::{Path, PathBuf};

/// What was found for one listed file in one copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryStatus {
    /// The file exists and its SHA-256 is the listed one.
    Ok,
    /// The file does not exist.
    Missing,
    /// The file exists but its SHA-256 differs.
    Mismatch,
    /// The entry exists but cannot be read as a plain file.
    Unreadable,
}

/// The result for one copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyReport {
    /// Every listed file with what was found, sorted by path.
    pub entries: Vec<(String, EntryStatus)>,
    /// Files present in the copy that the manifest does not list, sorted.
    pub extra: Vec<String>,
}

impl CopyReport {
    /// True when every listed file is intact and, unless extras are allowed, nothing else is there.
    pub fn is_clean(&self, config: &Config) -> bool {
        self.entries
            .iter()
            .all(|(_, status)| *status == EntryStatus::Ok)
            && (config.allow_extra_files || self.extra.is_empty())
    }
}

/// The result for all the copies of one golden set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyReport {
    /// One report per copy, in the order given.
    pub copies: Vec<CopyReport>,
}

impl VerifyReport {
    /// True when enough copies were given and every one of them is clean.
    pub fn is_redundant_and_clean(&self, config: &Config) -> bool {
        self.copies.len() >= config.min_copies && self.copies.iter().all(|c| c.is_clean(config))
    }
}

/// Check one copy (a folder) against the manifest.
pub fn verify_copy(
    manifest: &Manifest,
    root: &Path,
    config: &Config,
) -> Result<CopyReport, LabError> {
    let mut entries = Vec::with_capacity(manifest.len());
    for (path, expected) in manifest.iter() {
        let full = root.join(path);
        let status = match fs::symlink_metadata(&full) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => EntryStatus::Missing,
            Err(_) => EntryStatus::Unreadable,
            Ok(meta) if !meta.is_file() => EntryStatus::Unreadable,
            Ok(_) => match fs::File::open(&full).and_then(sha256_reader) {
                Ok(actual) if actual == expected => EntryStatus::Ok,
                Ok(_) => EntryStatus::Mismatch,
                Err(_) => EntryStatus::Unreadable,
            },
        };
        entries.push((path.to_string(), status));
    }
    let extra = list_files(root, config)?
        .into_iter()
        .filter(|path| !manifest.contains(path))
        .collect();
    Ok(CopyReport { entries, extra })
}

/// Check every copy. The same folder given twice is refused, because it would count twice.
pub fn verify_copies(
    manifest: &Manifest,
    roots: &[PathBuf],
    config: &Config,
) -> Result<VerifyReport, LabError> {
    let mut seen = Vec::with_capacity(roots.len());
    for root in roots {
        let canonical = fs::canonicalize(root).map_err(|source| LabError::Io {
            path: root.clone(),
            source,
        })?;
        if seen.contains(&canonical) {
            return Err(LabError::DuplicateCopy);
        }
        seen.push(canonical);
    }
    let copies = roots
        .iter()
        .map(|root| verify_copy(manifest, root, config))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(VerifyReport { copies })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn golden(dir: &Path) {
        fs::create_dir_all(dir.join("set")).unwrap();
        fs::write(dir.join("set").join("one.txt"), b"first").unwrap();
        fs::write(dir.join("two.txt"), b"second").unwrap();
    }

    fn copy_of(source: &Path, target: &Path) {
        fs::create_dir_all(target.join("set")).unwrap();
        fs::copy(
            source.join("set").join("one.txt"),
            target.join("set").join("one.txt"),
        )
        .unwrap();
        fs::copy(source.join("two.txt"), target.join("two.txt")).unwrap();
    }

    #[test]
    fn identical_copies_are_redundant_and_clean() {
        let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        golden(a.path());
        copy_of(a.path(), b.path());
        let config = Config::default();
        let manifest = Manifest::from_dir(a.path(), &config).unwrap();
        let report = verify_copies(
            &manifest,
            &[a.path().to_path_buf(), b.path().to_path_buf()],
            &config,
        )
        .unwrap();
        assert!(report.is_redundant_and_clean(&config));
    }

    #[test]
    fn one_changed_byte_is_reported_as_a_mismatch_in_that_copy_only() {
        let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        golden(a.path());
        copy_of(a.path(), b.path());
        let config = Config::default();
        let manifest = Manifest::from_dir(a.path(), &config).unwrap();
        fs::write(b.path().join("two.txt"), b"secone").unwrap();
        let report = verify_copies(
            &manifest,
            &[a.path().to_path_buf(), b.path().to_path_buf()],
            &config,
        )
        .unwrap();
        assert!(!report.is_redundant_and_clean(&config));
        assert!(report.copies[0].is_clean(&config));
        assert!(report.copies[1]
            .entries
            .contains(&("two.txt".to_string(), EntryStatus::Mismatch)));
    }

    #[test]
    fn a_missing_file_and_an_extra_file_are_both_failures() {
        let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        golden(a.path());
        copy_of(a.path(), b.path());
        let config = Config::default();
        let manifest = Manifest::from_dir(a.path(), &config).unwrap();
        fs::remove_file(b.path().join("set").join("one.txt")).unwrap();
        fs::write(b.path().join("stray.txt"), b"?").unwrap();
        let report = verify_copy(&manifest, b.path(), &config).unwrap();
        assert!(report
            .entries
            .contains(&("set/one.txt".to_string(), EntryStatus::Missing)));
        assert_eq!(report.extra, vec!["stray.txt".to_string()]);
        assert!(!report.is_clean(&config));
        let lenient = Config {
            allow_extra_files: true,
            ..Config::default()
        };
        assert!(!report.is_clean(&lenient), "the missing file still fails");
    }

    #[test]
    fn extras_alone_pass_only_when_allowed() {
        let a = tempfile::tempdir().unwrap();
        golden(a.path());
        let config = Config::default();
        let manifest = Manifest::from_dir(a.path(), &config).unwrap();
        fs::write(a.path().join("notes.txt"), b"n").unwrap();
        let report = verify_copy(&manifest, a.path(), &config).unwrap();
        assert!(!report.is_clean(&config));
        assert!(report.is_clean(&Config {
            allow_extra_files: true,
            ..Config::default()
        }));
    }

    #[test]
    fn a_single_copy_is_not_redundant_with_the_default_setting() {
        let a = tempfile::tempdir().unwrap();
        golden(a.path());
        let config = Config::default();
        let manifest = Manifest::from_dir(a.path(), &config).unwrap();
        let report = verify_copies(&manifest, &[a.path().to_path_buf()], &config).unwrap();
        assert!(report.copies[0].is_clean(&config));
        assert!(!report.is_redundant_and_clean(&config));
        assert!(report.is_redundant_and_clean(&Config {
            min_copies: 1,
            ..Config::default()
        }));
    }

    #[test]
    fn the_same_folder_twice_does_not_count_as_two_copies() {
        let a = tempfile::tempdir().unwrap();
        golden(a.path());
        let config = Config::default();
        let manifest = Manifest::from_dir(a.path(), &config).unwrap();
        let again = a.path().join(".");
        assert!(matches!(
            verify_copies(&manifest, &[a.path().to_path_buf(), again], &config),
            Err(LabError::DuplicateCopy)
        ));
    }
}

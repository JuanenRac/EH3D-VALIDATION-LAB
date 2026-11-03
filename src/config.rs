// SPDX-License-Identifier: GPL-3.0-or-later
//! Every tuned value of the laboratory lives here and is read from a plain `key = value` file.
//! The defaults are the conservative ones; a file only has to name what it changes.

use crate::error::LabError;

/// The settings of a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Fewest copies that must agree with the manifest for a golden set to count as redundant.
    pub min_copies: usize,
    /// When false, a file in a copy that the manifest does not list is a failure.
    pub allow_extra_files: bool,
    /// Most entries a manifest or a directory may hold.
    pub max_entries: usize,
    /// Longest path, in bytes, a manifest may hold.
    pub max_path_bytes: usize,
    /// Largest manifest file, in bytes.
    pub max_manifest_bytes: u64,
    /// Largest results or baseline file, in bytes.
    pub max_results_bytes: u64,
    /// Text mixed into the hash that stands for a case in reports, so short names cannot be guessed.
    pub report_salt: String,
    /// When true, a case that passed in the baseline and fails now fails the gate even if the rate holds.
    pub fail_on_new_failure: bool,
    /// When true, a case of the baseline that is missing now fails the gate.
    pub fail_on_missing_case: bool,
    /// The extensions of the files a corpus run reads and copies (any of `lbr`, `sch`, `brd`).
    pub corpus_extensions: Vec<String>,
    /// How many bytes of the start of a design file are looked at to tell its family and version.
    pub corpus_head_bytes: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            min_copies: 2,
            allow_extra_files: false,
            max_entries: 100_000,
            max_path_bytes: 240,
            max_manifest_bytes: 64 * 1024 * 1024,
            max_results_bytes: 64 * 1024 * 1024,
            report_salt: String::new(),
            fail_on_new_failure: true,
            fail_on_missing_case: true,
            corpus_extensions: vec!["lbr".into(), "sch".into(), "brd".into()],
            corpus_head_bytes: 4096,
        }
    }
}

impl Config {
    /// Parse a configuration text. Blank lines and lines starting with `#` are ignored.
    pub fn parse(text: &str) -> Result<Config, LabError> {
        let mut config = Config::default();
        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let trimmed = raw.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let (key, value) = trimmed
                .split_once('=')
                .ok_or(LabError::ConfigSyntax { line })?;
            let key = key.trim();
            let value = value.trim();
            let bad = || LabError::ConfigBadValue {
                line,
                key: key.to_string(),
            };
            match key {
                "min_copies" => config.min_copies = value.parse().map_err(|_| bad())?,
                "allow_extra_files" => {
                    config.allow_extra_files = parse_bool(value).ok_or_else(bad)?
                }
                "max_entries" => config.max_entries = value.parse().map_err(|_| bad())?,
                "max_path_bytes" => config.max_path_bytes = value.parse().map_err(|_| bad())?,
                "max_manifest_bytes" => {
                    config.max_manifest_bytes = value.parse().map_err(|_| bad())?
                }
                "max_results_bytes" => {
                    config.max_results_bytes = value.parse().map_err(|_| bad())?
                }
                "report_salt" => config.report_salt = value.to_string(),
                "fail_on_new_failure" => {
                    config.fail_on_new_failure = parse_bool(value).ok_or_else(bad)?
                }
                "fail_on_missing_case" => {
                    config.fail_on_missing_case = parse_bool(value).ok_or_else(bad)?
                }
                "corpus_extensions" => {
                    let list: Vec<String> = value
                        .split(',')
                        .map(|e| e.trim().to_ascii_lowercase())
                        .collect();
                    if list.is_empty()
                        || list
                            .iter()
                            .any(|e| !matches!(e.as_str(), "lbr" | "sch" | "brd"))
                    {
                        return Err(bad());
                    }
                    config.corpus_extensions = list;
                }
                "corpus_head_bytes" => {
                    let n: usize = value.parse().map_err(|_| bad())?;
                    if !(16..=1_048_576).contains(&n) {
                        return Err(bad());
                    }
                    config.corpus_head_bytes = n;
                }
                _ => {
                    return Err(LabError::ConfigUnknownKey {
                        line,
                        key: key.to_string(),
                    })
                }
            }
        }
        Ok(config)
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_gives_the_conservative_defaults() {
        let config = Config::parse("").unwrap();
        assert_eq!(config, Config::default());
        assert_eq!(config.min_copies, 2);
        assert!(!config.allow_extra_files);
        assert!(config.fail_on_new_failure);
    }

    #[test]
    fn a_file_changes_only_what_it_names() {
        let config = Config::parse("# comment\nmin_copies = 3\n\nreport_salt = abc\n").unwrap();
        assert_eq!(config.min_copies, 3);
        assert_eq!(config.report_salt, "abc");
        assert_eq!(config.max_entries, Config::default().max_entries);
    }

    #[test]
    fn unknown_keys_and_bad_values_are_refused_with_the_line() {
        assert!(matches!(
            Config::parse("min_copies = 2\ncolour = red"),
            Err(LabError::ConfigUnknownKey { line: 2, .. })
        ));
        assert!(matches!(
            Config::parse("allow_extra_files = yes"),
            Err(LabError::ConfigBadValue { line: 1, .. })
        ));
        assert!(matches!(
            Config::parse("min_copies = -1"),
            Err(LabError::ConfigBadValue { line: 1, .. })
        ));
        assert!(matches!(
            Config::parse("just words"),
            Err(LabError::ConfigSyntax { line: 1 })
        ));
    }
}

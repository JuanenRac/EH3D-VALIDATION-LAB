// SPDX-License-Identifier: GPL-3.0-or-later
//! Results of a corpus run: a level of agreement for each case and, for every failure, one of three classes.
//! A failure without a class cannot be built, so an unclassified failure never reaches a gate.

use crate::config::Config;
use crate::error::LabError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

/// A level of agreement with the reference, `L0` (reads without error) to `L5` (manufacturing outputs equivalent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Level(u8);

impl Level {
    /// The highest level.
    pub const MAX: u8 = 5;

    /// A level from its number, or `None` above `L5`.
    pub fn new(number: u8) -> Option<Level> {
        (number <= Level::MAX).then_some(Level(number))
    }

    /// The number of the level.
    pub fn number(self) -> u8 {
        self.0
    }

    /// Parse `L0` to `L5`.
    pub fn parse(text: &str) -> Option<Level> {
        let digits = text.strip_prefix('L')?;
        if digits.len() != 1 {
            return None;
        }
        Level::new(digits.parse().ok()?)
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "L{}", self.0)
    }
}

impl From<Level> for String {
    fn from(level: Level) -> String {
        level.to_string()
    }
}

impl TryFrom<String> for Level {
    type Error = String;
    fn try_from(text: String) -> Result<Level, String> {
        Level::parse(&text).ok_or_else(|| "level must be L0 to L5".to_string())
    }
}

/// Why a case failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureClass {
    /// An error of the design tool under test.
    PhoenixError,
    /// A limitation of the format being read or written.
    FormatLimitation,
    /// An error in the file itself.
    FileError,
}

impl FailureClass {
    /// Parse `phoenix-error`, `format-limitation` or `file-error`.
    pub fn parse(text: &str) -> Option<FailureClass> {
        match text {
            "phoenix-error" => Some(FailureClass::PhoenixError),
            "format-limitation" => Some(FailureClass::FormatLimitation),
            "file-error" => Some(FailureClass::FileError),
            _ => None,
        }
    }
}

/// What happened to one case at one level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// The case reached the level.
    Pass,
    /// The case did not reach the level, for the given reason.
    Fail(FailureClass),
}

/// One line of a results file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseResult {
    /// The name of the case. It is private: reports only ever carry its opaque hash.
    pub case: String,
    /// The level measured.
    pub level: Level,
    /// The outcome.
    pub outcome: Outcome,
}

/// Parse a results file: `case<TAB>level<TAB>pass` or `case<TAB>level<TAB>fail:<class>`, one per line.
pub fn parse_results(text: &str, config: &Config) -> Result<Vec<CaseResult>, LabError> {
    if text.len() as u64 > config.max_results_bytes {
        return Err(LabError::TooLarge {
            what: "results bytes",
        });
    }
    let mut results = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        if raw.trim().is_empty() || raw.starts_with('#') {
            continue;
        }
        let mut parts = raw.split('\t');
        let (case, level, outcome) = match (parts.next(), parts.next(), parts.next(), parts.next())
        {
            (Some(c), Some(l), Some(o), None) if !c.is_empty() => (c, l, o),
            _ => return Err(LabError::ResultsSyntax { line }),
        };
        let level = Level::parse(level).ok_or(LabError::ResultsSyntax { line })?;
        let outcome = match outcome {
            "pass" => Outcome::Pass,
            "fail" => return Err(LabError::UnclassifiedFailure { line }),
            other => match other.strip_prefix("fail:") {
                Some(class) => Outcome::Fail(
                    FailureClass::parse(class).ok_or(LabError::UnclassifiedFailure { line })?,
                ),
                None => return Err(LabError::ResultsSyntax { line }),
            },
        };
        if results.len() >= config.max_entries {
            return Err(LabError::TooLarge { what: "results" });
        }
        if !seen.insert((case.to_string(), level)) {
            return Err(LabError::DuplicateResult { line });
        }
        results.push(CaseResult {
            case: case.to_string(),
            level,
            outcome,
        });
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_run_from_l0_to_l5_and_nothing_else() {
        for n in 0..=5u8 {
            assert_eq!(Level::parse(&format!("L{n}")).unwrap().number(), n);
        }
        for bad in ["L6", "l1", "1", "L", "L10", "L-1", " L1", "LL"] {
            assert!(Level::parse(bad).is_none(), "{bad:?}");
        }
        assert!(Level::new(3).unwrap() < Level::new(4).unwrap());
    }

    #[test]
    fn a_results_file_parses_with_every_class() {
        let text = "a\tL0\tpass\nb\tL3\tfail:phoenix-error\nc\tL4\tfail:format-limitation\n# note\n\nd\tL5\tfail:file-error\n";
        let r = parse_results(text, &Config::default()).unwrap();
        assert_eq!(r.len(), 4);
        assert_eq!(r[1].outcome, Outcome::Fail(FailureClass::PhoenixError));
        assert_eq!(r[3].level.number(), 5);
    }

    #[test]
    fn a_failure_without_a_known_class_is_refused() {
        let config = Config::default();
        assert!(matches!(
            parse_results("a\tL1\tfail\n", &config),
            Err(LabError::UnclassifiedFailure { line: 1 })
        ));
        assert!(matches!(
            parse_results("a\tL1\tpass\nb\tL1\tfail:my-fault\n", &config),
            Err(LabError::UnclassifiedFailure { line: 2 })
        ));
        assert!(matches!(
            parse_results("a\tL1\tfail:\n", &config),
            Err(LabError::UnclassifiedFailure { line: 1 })
        ));
    }

    #[test]
    fn malformed_and_repeated_lines_are_refused() {
        let config = Config::default();
        for text in [
            "onlyname\n",
            "a\tL1\n",
            "a\tL9\tpass\n",
            "a\tL1\tmaybe\n",
            "\tL1\tpass\n",
            "a\tL1\tpass\textra\n",
        ] {
            assert!(
                matches!(
                    parse_results(text, &config),
                    Err(LabError::ResultsSyntax { line: 1 })
                ),
                "{text:?}"
            );
        }
        assert!(matches!(
            parse_results("a\tL1\tpass\na\tL1\tpass\n", &config),
            Err(LabError::DuplicateResult { line: 2 })
        ));
        // the same case at two levels is fine
        assert_eq!(
            parse_results("a\tL1\tpass\na\tL2\tpass\n", &config)
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn the_size_limit_applies() {
        let config = Config {
            max_results_bytes: 5,
            ..Config::default()
        };
        assert!(matches!(
            parse_results("a\tL1\tpass\n", &config),
            Err(LabError::TooLarge { .. })
        ));
    }
}

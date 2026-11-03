// SPDX-License-Identifier: GPL-3.0-or-later
//! The oracle of golden files: what a reference program said about a case, compared with what the tool under test says.
//!
//! A **golden file** is written once, while the reference program still works, and kept next to the private projects (see the decision on the golden-file oracle). It has one
//! line per check: `case<TAB>check<TAB>expected`. The `check` is `L<n>:<name>`, where `L<n>` is the level the check belongs to (`L0` to `L5`) and `<name>` says what is
//! compared (`opens-clean`, `drc-errors`, `output-sha256`, ...). The value is any text without a tab; this module never interprets it, it only compares.
//!
//! An **observed file** has the same lines for the tool under test, plus a fourth column on a line that is expected to differ: `case<TAB>check<TAB>value<TAB>class`, with one
//! of the three classes of a failure. A difference without a class is refused, as everywhere else in the laboratory: the person who looks at the difference decides whether it
//! is an error of the tool, a limitation of the format or an error in the file. A check that the golden file has and the observed file lacks is a failure of the tool
//! (`phoenix-error`) unless the observed file says otherwise with a line whose value is `-` and a class.
//!
//! The result is the list of [`CaseResult`] that the gate already understands: a case passes a level when every golden check of that level agrees.
//! The originals of a golden set are never opened for writing here: the module reads text that the caller already has.

use crate::classify::{CaseResult, FailureClass, Level, Outcome};
use crate::config::Config;
use crate::error::LabError;
use std::collections::{BTreeMap, BTreeSet};

/// One line of a golden or observed file.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Line {
    case: String,
    level: Level,
    check: String,
    value: String,
    class: Option<FailureClass>,
}

/// What a comparison found, as counts that carry no case name, check name or value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OracleSummary {
    /// Golden checks that agreed.
    pub agreed: usize,
    /// Golden checks that differed or were missing.
    pub differed: usize,
    /// Observed checks the golden file has no line for (ignored).
    pub extra: usize,
}

fn parse_lines(text: &str, config: &Config, observed: bool) -> Result<Vec<Line>, LabError> {
    if text.len() as u64 > config.max_results_bytes {
        return Err(LabError::TooLarge {
            what: "oracle bytes",
        });
    }
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        if raw.trim().is_empty() || raw.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = raw.split('\t').collect();
        let max = if observed { 4 } else { 3 };
        if parts.len() < 3 || parts.len() > max || parts[0].is_empty() {
            return Err(LabError::OracleSyntax { line });
        }
        let (level_text, name) = parts[1]
            .split_once(':')
            .ok_or(LabError::OracleSyntax { line })?;
        let level = Level::parse(level_text).ok_or(LabError::OracleSyntax { line })?;
        if name.is_empty() {
            return Err(LabError::OracleSyntax { line });
        }
        let class = match parts.get(3) {
            None => None,
            Some(text) => {
                Some(FailureClass::parse(text).ok_or(LabError::UnclassifiedFailure { line })?)
            }
        };
        if out.len() >= config.max_entries {
            return Err(LabError::TooLarge {
                what: "oracle lines",
            });
        }
        if !seen.insert((parts[0].to_string(), parts[1].to_string())) {
            return Err(LabError::DuplicateResult { line });
        }
        out.push(Line {
            case: parts[0].to_string(),
            level,
            check: parts[1].to_string(),
            value: parts[2].to_string(),
            class,
        });
    }
    Ok(out)
}

/// Compare an observed file with a golden file. Returns the results by case and level (sorted) and the counts.
///
/// A difference needs a class in the observed file (see the module text); the first difference without one stops the comparison with `UnclassifiedFailure` and the line of
/// the observed file, so an unclassified difference never reaches a gate.
pub fn compare(
    golden: &str,
    observed: &str,
    config: &Config,
) -> Result<(Vec<CaseResult>, OracleSummary), LabError> {
    let golden = parse_lines(golden, config, false)?;
    let observed_lines = parse_lines(observed, config, true)?;
    let mut observed: BTreeMap<(&str, &str), (&Line, usize)> = BTreeMap::new();
    for (index, line) in observed_lines.iter().enumerate() {
        observed.insert((line.case.as_str(), line.check.as_str()), (line, index));
    }
    let mut summary = OracleSummary::default();
    // (case, level) -> the worst class found so far, None while everything agrees
    let mut by_case: BTreeMap<(String, Level), Option<FailureClass>> = BTreeMap::new();
    let mut golden_keys = BTreeSet::new();
    for g in &golden {
        golden_keys.insert((g.case.as_str(), g.check.as_str()));
        let entry = by_case.entry((g.case.clone(), g.level)).or_insert(None);
        match observed.get(&(g.case.as_str(), g.check.as_str())) {
            Some((o, _)) if o.value == g.value => summary.agreed += 1,
            Some((o, _)) => {
                summary.differed += 1;
                let class = o.class.ok_or(LabError::UnclassifiedFailure {
                    line: line_of(&observed_lines, o),
                })?;
                *entry = Some(entry.map_or(class, |c| c.min(class)));
            }
            None => {
                summary.differed += 1;
                *entry = Some(entry.map_or(FailureClass::PhoenixError, |c| {
                    c.min(FailureClass::PhoenixError)
                }));
            }
        }
    }
    summary.extra = observed
        .keys()
        .filter(|key| !golden_keys.contains(*key))
        .count();
    let results = by_case
        .into_iter()
        .map(|((case, level), class)| CaseResult {
            case,
            level,
            outcome: class.map_or(Outcome::Pass, Outcome::Fail),
        })
        .collect();
    Ok((results, summary))
}

/// The 1-based position of an observed line among the lines that count (blank and comment lines are not counted, so this is a position in the list, used only to point
/// the person at the line).
fn line_of(lines: &[Line], wanted: &Line) -> usize {
    lines
        .iter()
        .position(|l| l == wanted)
        .map_or(0, |index| index + 1)
}

/// The results as the text that `parse_results` reads back (`case<TAB>level<TAB>pass|fail:<class>`), one line per case and level.
pub fn render_results(results: &[CaseResult]) -> String {
    let mut text = String::new();
    for r in results {
        let outcome = match r.outcome {
            Outcome::Pass => "pass".to_string(),
            Outcome::Fail(FailureClass::PhoenixError) => "fail:phoenix-error".to_string(),
            Outcome::Fail(FailureClass::FormatLimitation) => "fail:format-limitation".to_string(),
            Outcome::Fail(FailureClass::FileError) => "fail:file-error".to_string(),
        };
        text.push_str(&format!("{}\t{}\t{}\n", r.case, r.level, outcome));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::parse_results;

    const GOLDEN: &str = "a\tL0:opens-clean\ttrue\na\tL3:drc-errors\t0\nb\tL0:opens-clean\ttrue\n";

    fn get(results: &[CaseResult], case: &str, level: u8) -> Outcome {
        results
            .iter()
            .find(|r| r.case == case && r.level.number() == level)
            .unwrap()
            .outcome
    }

    #[test]
    fn an_observed_file_equal_to_the_golden_file_passes_every_level() {
        let (r, s) = compare(GOLDEN, GOLDEN, &Config::default()).unwrap();
        assert_eq!(r.len(), 3);
        assert!(r.iter().all(|x| x.outcome == Outcome::Pass));
        assert_eq!((s.agreed, s.differed, s.extra), (3, 0, 0));
    }

    #[test]
    fn a_difference_with_a_class_fails_only_its_level() {
        let observed = "a\tL0:opens-clean\ttrue\na\tL3:drc-errors\t2\tphoenix-error\nb\tL0:opens-clean\ttrue\n";
        let (r, s) = compare(GOLDEN, observed, &Config::default()).unwrap();
        assert_eq!(get(&r, "a", 0), Outcome::Pass);
        assert_eq!(get(&r, "a", 3), Outcome::Fail(FailureClass::PhoenixError));
        assert_eq!(get(&r, "b", 0), Outcome::Pass);
        assert_eq!((s.agreed, s.differed), (2, 1));
    }

    #[test]
    fn a_difference_without_a_class_is_refused() {
        let observed = "a\tL0:opens-clean\tfalse\nb\tL0:opens-clean\ttrue\na\tL3:drc-errors\t0\n";
        assert!(matches!(
            compare(GOLDEN, observed, &Config::default()),
            Err(LabError::UnclassifiedFailure { line: 1 })
        ));
    }

    #[test]
    fn a_missing_check_is_an_error_of_the_tool_unless_the_file_says_otherwise() {
        let observed = "a\tL0:opens-clean\ttrue\nb\tL0:opens-clean\ttrue\n";
        let (r, _) = compare(GOLDEN, observed, &Config::default()).unwrap();
        assert_eq!(get(&r, "a", 3), Outcome::Fail(FailureClass::PhoenixError));
        let observed = "a\tL0:opens-clean\ttrue\na\tL3:drc-errors\t-\tformat-limitation\nb\tL0:opens-clean\ttrue\n";
        let (r, _) = compare(GOLDEN, observed, &Config::default()).unwrap();
        assert_eq!(
            get(&r, "a", 3),
            Outcome::Fail(FailureClass::FormatLimitation)
        );
    }

    #[test]
    fn the_worst_class_of_a_level_wins_and_extra_checks_are_only_counted() {
        let golden = "a\tL1:x\t1\na\tL1:y\t1\n";
        let observed = "a\tL1:x\t2\tfile-error\na\tL1:y\t2\tphoenix-error\na\tL1:z\t9\n";
        let (r, s) = compare(golden, observed, &Config::default()).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].outcome, Outcome::Fail(FailureClass::PhoenixError));
        assert_eq!(s.extra, 1);
    }

    #[test]
    fn bad_lines_are_refused_with_their_line_number() {
        let config = Config::default();
        for text in [
            "onlycase\n",
            "a\tL9:x\t1\n",
            "a\tx\t1\n",
            "a\tL1:\t1\n",
            "\tL1:x\t1\n",
            "a\tL1:x\t1\tphoenix-error\n",
        ] {
            assert!(
                matches!(
                    compare(text, "", &config),
                    Err(LabError::OracleSyntax { line: 1 })
                ),
                "{text:?}"
            );
        }
        assert!(matches!(
            compare("a\tL1:x\t1\na\tL1:x\t2\n", "", &config),
            Err(LabError::DuplicateResult { line: 2 })
        ));
        assert!(matches!(
            compare("", "a\tL1:x\t1\tmy-fault\n", &config),
            Err(LabError::UnclassifiedFailure { line: 1 })
        ));
    }

    #[test]
    fn the_rendered_results_read_back_and_the_size_limit_applies() {
        let observed =
            "a\tL0:opens-clean\ttrue\na\tL3:drc-errors\t2\tfile-error\nb\tL0:opens-clean\ttrue\n";
        let config = Config::default();
        let (r, _) = compare(GOLDEN, observed, &config).unwrap();
        let text = render_results(&r);
        assert_eq!(parse_results(&text, &config).unwrap(), r);
        let small = Config {
            max_results_bytes: 5,
            ..Config::default()
        };
        assert!(matches!(
            compare(GOLDEN, "", &small),
            Err(LabError::TooLarge { .. })
        ));
    }
}

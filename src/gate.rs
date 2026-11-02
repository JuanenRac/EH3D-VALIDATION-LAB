// SPDX-License-Identifier: GPL-3.0-or-later
//! The regression gate: the measured rate of each level must not drop between two runs.
//!
//! Rates are compared as integer fractions (no floating point). The baseline and the report hold
//! only opaque case hashes, levels, classes and counts, so they can be shared or kept in a
//! repository without exposing the names of the cases.

use crate::classify::{CaseResult, FailureClass, Level, Outcome};
use crate::config::Config;
use crate::error::LabError;
use crate::hash::opaque_id;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Version of the baseline file written by this tool.
pub const BASELINE_VERSION: u32 = 1;

/// One case of a baseline.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BaselineCase {
    /// The opaque hash of the case name.
    pub id: String,
    /// The level measured.
    pub level: Level,
    /// Whether the case reached the level.
    pub pass: bool,
}

/// What a past run measured: the point the next run must not fall below.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Baseline {
    /// Format version of this file.
    pub version: u32,
    /// Every case and level, sorted.
    pub cases: Vec<BaselineCase>,
}

impl Baseline {
    /// Make a baseline from the results of a run.
    pub fn from_results(results: &[CaseResult], config: &Config) -> Baseline {
        let mut cases: Vec<BaselineCase> = results
            .iter()
            .map(|r| BaselineCase {
                id: opaque_id(&config.report_salt, &r.case),
                level: r.level,
                pass: r.outcome == Outcome::Pass,
            })
            .collect();
        cases.sort();
        Baseline {
            version: BASELINE_VERSION,
            cases,
        }
    }

    /// Write the baseline as JSON.
    pub fn to_json(&self) -> Result<String, LabError> {
        serde_json::to_string_pretty(self).map_err(|_| LabError::BadBaseline)
    }

    /// Read a baseline, refusing another version, an oversize file and repeated cases.
    pub fn from_json(text: &str, config: &Config) -> Result<Baseline, LabError> {
        if text.len() as u64 > config.max_results_bytes {
            return Err(LabError::TooLarge {
                what: "baseline bytes",
            });
        }
        let baseline: Baseline = serde_json::from_str(text).map_err(|_| LabError::BadBaseline)?;
        if baseline.version != BASELINE_VERSION || baseline.cases.len() > config.max_entries {
            return Err(LabError::BadBaseline);
        }
        let unique: BTreeSet<_> = baseline.cases.iter().map(|c| (&c.id, c.level)).collect();
        if unique.len() != baseline.cases.len() {
            return Err(LabError::BadBaseline);
        }
        Ok(baseline)
    }
}

/// The rate of one level in both runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LevelReport {
    /// The level.
    pub level: Level,
    /// Cases that reached it in the baseline.
    pub baseline_passed: u64,
    /// Cases measured at it in the baseline.
    pub baseline_total: u64,
    /// Cases that reach it now.
    pub passed: u64,
    /// Cases measured at it now.
    pub total: u64,
    /// True when the rate now is lower than in the baseline.
    pub dropped: bool,
}

/// A case, by hash, at a level.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CaseRef {
    /// The opaque hash of the case name.
    pub id: String,
    /// The level.
    pub level: Level,
}

/// A case that passed in the baseline and fails now.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct NewFailure {
    /// The case and level.
    #[serde(flatten)]
    pub case: CaseRef,
    /// The class of the failure.
    pub class: FailureClass,
}

/// The outcome of a gate. It holds only hashes, levels, classes and counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GateReport {
    /// True when the gate lets the run through.
    pub passed: bool,
    /// The rate of every level of the baseline.
    pub levels: Vec<LevelReport>,
    /// Cases that passed before and fail now.
    pub new_failures: Vec<NewFailure>,
    /// Cases of the baseline that are not in this run.
    pub missing: Vec<CaseRef>,
    /// Failures of this run by class.
    pub failures_by_class: BTreeMap<FailureClass, u64>,
}

impl GateReport {
    /// The report as JSON.
    pub fn to_json(&self) -> Result<String, LabError> {
        serde_json::to_string_pretty(self).map_err(|_| LabError::BadBaseline)
    }
}

/// Compare a run with the baseline.
pub fn evaluate(baseline: &Baseline, results: &[CaseResult], config: &Config) -> GateReport {
    let now: BTreeMap<(String, Level), Outcome> = results
        .iter()
        .map(|r| {
            (
                (opaque_id(&config.report_salt, &r.case), r.level),
                r.outcome,
            )
        })
        .collect();

    let mut base_stats: BTreeMap<Level, (u64, u64)> = BTreeMap::new();
    let mut now_stats: BTreeMap<Level, (u64, u64)> = BTreeMap::new();
    for case in &baseline.cases {
        let entry = base_stats.entry(case.level).or_default();
        entry.0 += u64::from(case.pass);
        entry.1 += 1;
    }
    for ((_, level), outcome) in &now {
        let entry = now_stats.entry(*level).or_default();
        entry.0 += u64::from(*outcome == Outcome::Pass);
        entry.1 += 1;
    }

    let levels: Vec<LevelReport> = base_stats
        .iter()
        .map(|(level, &(base_passed, base_total))| {
            let (passed, total) = now_stats.get(level).copied().unwrap_or((0, 0));
            // passed / total < base_passed / base_total, without division
            let dropped = if total == 0 {
                base_total > 0
            } else {
                u128::from(passed) * u128::from(base_total)
                    < u128::from(base_passed) * u128::from(total)
            };
            LevelReport {
                level: *level,
                baseline_passed: base_passed,
                baseline_total: base_total,
                passed,
                total,
                dropped,
            }
        })
        .collect();

    let mut new_failures = Vec::new();
    let mut missing = Vec::new();
    for case in &baseline.cases {
        match now.get(&(case.id.clone(), case.level)) {
            None => missing.push(CaseRef {
                id: case.id.clone(),
                level: case.level,
            }),
            Some(Outcome::Fail(class)) if case.pass => new_failures.push(NewFailure {
                case: CaseRef {
                    id: case.id.clone(),
                    level: case.level,
                },
                class: *class,
            }),
            Some(_) => {}
        }
    }
    new_failures.sort();
    missing.sort();

    let mut failures_by_class = BTreeMap::new();
    for outcome in now.values() {
        if let Outcome::Fail(class) = outcome {
            *failures_by_class.entry(*class).or_insert(0u64) += 1;
        }
    }

    let passed = levels.iter().all(|l| !l.dropped)
        && (!config.fail_on_new_failure || new_failures.is_empty())
        && (!config.fail_on_missing_case || missing.is_empty());
    GateReport {
        passed,
        levels,
        new_failures,
        missing,
        failures_by_class,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::parse_results;

    fn run(text: &str) -> Vec<CaseResult> {
        parse_results(text, &Config::default()).unwrap()
    }

    fn gate(base: &str, now: &str, config: &Config) -> GateReport {
        evaluate(
            &Baseline::from_results(&run(base), config),
            &run(now),
            config,
        )
    }

    const BASE: &str = "a\tL0\tpass\nb\tL0\tpass\nc\tL0\tfail:file-error\nd\tL0\tpass\n";

    #[test]
    fn the_same_run_passes_the_gate() {
        let report = gate(BASE, BASE, &Config::default());
        assert!(report.passed);
        assert!(report.new_failures.is_empty() && report.missing.is_empty());
        assert_eq!(report.levels[0].baseline_passed, 3);
        assert_eq!(report.levels[0].baseline_total, 4);
    }

    #[test]
    fn a_better_run_passes() {
        let better = "a\tL0\tpass\nb\tL0\tpass\nc\tL0\tpass\nd\tL0\tpass\n";
        assert!(gate(BASE, better, &Config::default()).passed);
    }

    #[test]
    fn a_lower_rate_fails_and_names_the_new_failure() {
        let worse = "a\tL0\tpass\nb\tL0\tfail:phoenix-error\nc\tL0\tfail:file-error\nd\tL0\tpass\n";
        let report = gate(BASE, worse, &Config::default());
        assert!(!report.passed);
        assert!(report.levels[0].dropped);
        assert_eq!(report.new_failures.len(), 1);
        assert_eq!(report.new_failures[0].class, FailureClass::PhoenixError);
        assert_eq!(report.failures_by_class[&FailureClass::PhoenixError], 1);
    }

    #[test]
    fn a_swap_keeps_the_rate_but_still_fails_by_default() {
        // c is fixed, b breaks: 3 of 4 before and after
        let swap = "a\tL0\tpass\nb\tL0\tfail:phoenix-error\nc\tL0\tpass\nd\tL0\tpass\n";
        let strict = gate(BASE, swap, &Config::default());
        assert!(!strict.levels[0].dropped);
        assert!(
            !strict.passed,
            "a case that used to pass and now fails is a regression"
        );
        let lenient = Config {
            fail_on_new_failure: false,
            ..Config::default()
        };
        assert!(gate(BASE, swap, &lenient).passed);
    }

    #[test]
    fn dropping_failing_cases_to_raise_the_rate_is_caught() {
        let trimmed = "a\tL0\tpass\nb\tL0\tpass\nd\tL0\tpass\n";
        let report = gate(BASE, trimmed, &Config::default());
        assert!(!report.levels[0].dropped);
        assert_eq!(report.missing.len(), 1);
        assert!(!report.passed);
    }

    #[test]
    fn a_level_with_no_cases_left_counts_as_a_drop() {
        let report = gate("a\tL2\tpass\n", "a\tL0\tpass\n", &Config::default());
        assert!(report.levels[0].dropped);
        assert!(!report.passed);
    }

    #[test]
    fn rates_are_compared_exactly_without_rounding() {
        // 1/3 before, 33/100 now: lower by a hair, which a rounding to whole percent would hide
        let mut base = String::from("x0\tL1\tpass\n");
        base.push_str("x1\tL1\tfail:file-error\nx2\tL1\tfail:file-error\n");
        let baseline = Baseline::from_results(&run(&base), &Config::default());
        let mut now = String::new();
        for i in 0..100u32 {
            now.push_str(&format!(
                "n{i}\tL1\t{}\n",
                if i < 33 { "pass" } else { "fail:file-error" }
            ));
        }
        let config = Config {
            fail_on_missing_case: false,
            fail_on_new_failure: false,
            ..Config::default()
        };
        let report = evaluate(&baseline, &parse_results(&now, &config).unwrap(), &config);
        assert!(report.levels[0].dropped);
    }

    #[test]
    fn the_baseline_round_trips_and_refuses_damage() {
        let config = Config::default();
        let baseline = Baseline::from_results(&run(BASE), &config);
        let text = baseline.to_json().unwrap();
        assert_eq!(Baseline::from_json(&text, &config).unwrap(), baseline);
        assert!(Baseline::from_json("{", &config).is_err());
        assert!(
            Baseline::from_json(&text.replace("\"version\": 1", "\"version\": 2"), &config)
                .is_err()
        );
        assert!(Baseline::from_json(&text.replace("\"L0\"", "\"L9\""), &config).is_err());
        let doubled = r#"{"version":1,"cases":[{"id":"x","level":"L0","pass":true},{"id":"x","level":"L0","pass":false}]}"#;
        assert!(Baseline::from_json(doubled, &config).is_err());
    }

    #[test]
    fn a_different_salt_cannot_pretend_to_match() {
        let baseline = Baseline::from_results(&run(BASE), &Config::default());
        let salted = Config {
            report_salt: "other".into(),
            ..Config::default()
        };
        let report = evaluate(&baseline, &run(BASE), &salted);
        assert!(!report.passed);
        assert_eq!(report.missing.len(), 4);
    }

    #[test]
    fn reports_and_baselines_never_contain_a_name_from_the_corpus() {
        let secret = "My-Private-Board-Name_rev7";
        let text = format!("{secret}\tL0\tpass\n{secret}\tL3\tfail:phoenix-error\nother-secret\tL0\tfail:file-error\n");
        let results = run(&text);
        let config = Config::default();
        let baseline = Baseline::from_results(&results, &config);
        let worse = run(&format!("{secret}\tL0\tfail:phoenix-error\n{secret}\tL3\tfail:phoenix-error\nother-secret\tL0\tfail:file-error\n"));
        let report = evaluate(&baseline, &worse, &config);
        let mut all = baseline.to_json().unwrap();
        all.push_str(&report.to_json().unwrap());
        for needle in [secret, "other-secret", "Private", "secret"] {
            assert!(!all.contains(needle), "{needle:?} leaked");
        }
    }
}

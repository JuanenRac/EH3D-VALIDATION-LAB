// SPDX-License-Identifier: GPL-3.0-or-later
//! EH3D-VALIDATION-LAB: the test environment of the PHOENIX family.
//!
//! This version holds the parts that need no design model: manifests of golden sets with
//! redundant copies, failures that must carry a class, and the gate that a measured rate never
//! drops. Nothing here reads a design file, and nothing runs a reference tool.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod classify;
pub mod cli;
pub mod config;
pub mod corpus;
pub mod error;
pub mod gate;
pub mod hash;
pub mod manifest;
pub mod oracle;
pub mod verify;

pub use error::LabError;

/// The simulation of the laboratory: a golden set made from a seed with redundant copies that fail on purpose (a bit changed, a file gone, an extra file, a file replaced by a
/// folder), a tool under test that fails a chosen share of the cases (the same cases for the same seed, and more of them when the share grows), and results that are damaged
/// (cases missing, text cut). It lets the checks of the copies and the gate be tried on every kind of failure and recovery with no corpus and no reference program.
/// Nothing here has read a design file.
pub mod sim {
    use crate::classify::{CaseResult, FailureClass, Level, Outcome};
    use crate::hash::sha256_hex;
    use std::collections::BTreeMap;
    use std::fs;
    use std::io;
    use std::path::Path;

    /// A small deterministic generator (xorshift64*). It reads no clock.
    #[derive(Debug, Clone)]
    pub struct SimRng(u64);

    impl SimRng {
        /// A generator for the seed (zero is replaced, because xorshift cannot start from zero).
        pub fn new(seed: u64) -> SimRng {
            SimRng(if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            })
        }

        /// A number below `n` (`n` must be above zero).
        pub fn below(&mut self, n: u64) -> u64 {
            let mut x = self.0;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.0 = x;
            x.wrapping_mul(0x2545_F491_4F6C_DD1D) % n
        }
    }

    /// What goes wrong with one file of a copy of the golden set.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Fault {
        /// One bit of the file is flipped.
        BitRot,
        /// The file is deleted.
        Missing,
        /// A file the manifest does not list is added.
        Extra,
        /// The file is replaced by a folder of the same name.
        Unreadable,
    }

    /// Every fault, in a fixed order.
    pub const ALL_FAULTS: [Fault; 4] = [
        Fault::BitRot,
        Fault::Missing,
        Fault::Extra,
        Fault::Unreadable,
    ];

    /// A golden set in memory: relative path and content.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SimSet {
        files: BTreeMap<String, Vec<u8>>,
    }

    impl SimSet {
        /// `count` files made from the seed, spread over libraries, schematics and boards (24 to 87 bytes each).
        pub fn generate(seed: u64, count: usize) -> SimSet {
            let mut rng = SimRng::new(seed);
            let mut files = BTreeMap::new();
            for index in 0..count {
                let (folder, extension) =
                    [("libs", "lbr"), ("schematics", "sch"), ("boards", "brd")][index % 3];
                let len = 24 + rng.below(64) as usize;
                let bytes = (0..len).map(|_| rng.below(256) as u8).collect();
                files.insert(format!("{folder}/case{index:03}.{extension}"), bytes);
            }
            SimSet { files }
        }

        /// The relative paths, in order.
        pub fn paths(&self) -> Vec<String> {
            self.files.keys().cloned().collect()
        }

        /// Write the whole set under `dir`.
        pub fn materialize(&self, dir: &Path) -> io::Result<()> {
            for path in self.files.keys() {
                self.restore(dir, path)?;
            }
            Ok(())
        }

        /// Write one file again as it is in the set; whatever stands in its place (a folder too) is removed first.
        pub fn restore(&self, dir: &Path, path: &str) -> io::Result<()> {
            let bytes = self
                .files
                .get(path)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "not in the set"))?;
            let target = dir.join(path);
            if target.is_dir() {
                fs::remove_dir_all(&target)?;
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(target, bytes)
        }

        /// The manifest text of the set (`<sha256>  <path>`, sorted).
        pub fn manifest_text(&self) -> String {
            self.files
                .iter()
                .map(|(path, bytes)| format!("{}  {path}\n", sha256_hex(bytes)))
                .collect()
        }

        /// Break one file of a copy under `dir`; for `Extra` the path is not used and a file called `notes.txt` is added.
        pub fn inject(
            &self,
            dir: &Path,
            path: &str,
            fault: Fault,
            rng: &mut SimRng,
        ) -> io::Result<()> {
            let target = dir.join(path);
            match fault {
                Fault::BitRot => {
                    let mut bytes = fs::read(&target)?;
                    let at = rng.below(bytes.len() as u64) as usize;
                    bytes[at] ^= 1 << rng.below(8);
                    fs::write(target, bytes)
                }
                Fault::Missing => fs::remove_file(target),
                Fault::Extra => fs::write(dir.join("notes.txt"), b"not in the manifest"),
                Fault::Unreadable => {
                    fs::remove_file(&target)?;
                    fs::create_dir(target)
                }
            }
        }
    }

    /// A tool under test that fails `fail_per_mille` in a thousand of the (case, level) pairs. Which pairs fail depends only on the seed and the pair, so a larger share
    /// fails every pair that a smaller one failed and more.
    #[derive(Debug, Clone, Copy)]
    pub struct SimTool {
        /// The seed that picks the pairs.
        pub seed: u64,
        /// Failures in a thousand pairs.
        pub fail_per_mille: u64,
    }

    impl SimTool {
        fn draw(&self, case: &str, level: u8) -> u64 {
            let mut x = self.seed ^ 0x9E37_79B9_7F4A_7C15;
            for byte in case.bytes().chain([level]) {
                x ^= u64::from(byte);
                x = x.wrapping_mul(0x0100_0000_01B3);
                x ^= x >> 29;
            }
            x ^= x >> 33;
            x = x.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
            x ^ (x >> 33)
        }

        /// What the tool does with a case at a level.
        pub fn outcome(&self, case: &str, level: u8) -> Outcome {
            let h = self.draw(case, level);
            if h % 1000 < self.fail_per_mille {
                Outcome::Fail(match (h / 1000) % 3 {
                    0 => FailureClass::PhoenixError,
                    1 => FailureClass::FormatLimitation,
                    _ => FailureClass::FileError,
                })
            } else {
                Outcome::Pass
            }
        }

        /// The results of a run over the cases at the levels `L0` to `L2`.
        pub fn run(&self, cases: &[String]) -> Vec<CaseResult> {
            let mut out = Vec::new();
            for case in cases {
                for n in 0..=2 {
                    if let Some(level) = Level::new(n) {
                        out.push(CaseResult {
                            case: case.clone(),
                            level,
                            outcome: self.outcome(case, n),
                        });
                    }
                }
            }
            out
        }
    }

    /// The names of `count` cases.
    pub fn case_names(count: usize) -> Vec<String> {
        (0..count).map(|i| format!("case-{i:04}")).collect()
    }

    /// Results as they come back from a bad run: each line is lost with the chance `lose_per_mille` in a thousand.
    pub fn lose_lines(results: &[CaseResult], seed: u64, lose_per_mille: u64) -> Vec<CaseResult> {
        let mut rng = SimRng::new(seed);
        results
            .iter()
            .filter(|_| rng.below(1000) >= lose_per_mille)
            .cloned()
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::classify::parse_results;
        use crate::config::Config;
        use crate::error::LabError;
        use crate::gate::{evaluate, Baseline};
        use crate::manifest::Manifest;
        use crate::oracle::render_results;
        use crate::verify::{verify_copies, EntryStatus};

        fn failures(results: &[CaseResult]) -> usize {
            results
                .iter()
                .filter(|r| matches!(r.outcome, Outcome::Fail(_)))
                .count()
        }

        #[test]
        fn the_same_seed_gives_the_same_set_and_manifest_and_another_seed_a_different_one() {
            assert_eq!(SimSet::generate(4, 9), SimSet::generate(4, 9));
            assert_ne!(SimSet::generate(4, 9), SimSet::generate(5, 9));
            assert_eq!(
                SimSet::generate(4, 9).manifest_text(),
                SimSet::generate(4, 9).manifest_text()
            );
        }

        #[test]
        fn two_clean_copies_are_redundant_and_clean_and_each_fault_in_one_copy_is_found_and_repaired(
        ) {
            let config = Config::default();
            let set = SimSet::generate(1, 9);
            let manifest = Manifest::parse(&set.manifest_text(), &config).unwrap();
            for fault in ALL_FAULTS {
                let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
                set.materialize(a.path()).unwrap();
                set.materialize(b.path()).unwrap();
                let roots = [a.path().to_path_buf(), b.path().to_path_buf()];
                assert!(verify_copies(&manifest, &roots, &config)
                    .unwrap()
                    .is_redundant_and_clean(&config));
                let path = set.paths()[4].clone();
                set.inject(b.path(), &path, fault, &mut SimRng::new(3))
                    .unwrap();
                let report = verify_copies(&manifest, &roots, &config).unwrap();
                assert!(report.copies[0].is_clean(&config), "{fault:?}");
                assert!(!report.copies[1].is_clean(&config), "{fault:?}");
                assert!(!report.is_redundant_and_clean(&config), "{fault:?}");
                let status = report.copies[1]
                    .entries
                    .iter()
                    .find(|(p, _)| *p == path)
                    .map(|(_, s)| s.clone())
                    .unwrap();
                match fault {
                    Fault::BitRot => assert_eq!(status, EntryStatus::Mismatch),
                    Fault::Missing => assert_eq!(status, EntryStatus::Missing),
                    Fault::Unreadable => assert_eq!(status, EntryStatus::Unreadable),
                    Fault::Extra => {
                        assert_eq!(status, EntryStatus::Ok);
                        assert_eq!(report.copies[1].extra, vec!["notes.txt".to_string()]);
                    }
                }
                if fault == Fault::Extra {
                    fs::remove_file(b.path().join("notes.txt")).unwrap();
                } else {
                    set.restore(b.path(), &path).unwrap();
                }
                assert!(
                    verify_copies(&manifest, &roots, &config)
                        .unwrap()
                        .is_redundant_and_clean(&config),
                    "{fault:?}"
                );
            }
        }

        #[test]
        fn the_same_folder_given_twice_is_refused_and_one_copy_is_not_enough() {
            let config = Config::default();
            let set = SimSet::generate(1, 3);
            let manifest = Manifest::parse(&set.manifest_text(), &config).unwrap();
            let dir = tempfile::tempdir().unwrap();
            set.materialize(dir.path()).unwrap();
            let twice = [dir.path().to_path_buf(), dir.path().to_path_buf()];
            assert!(matches!(
                verify_copies(&manifest, &twice, &config),
                Err(LabError::DuplicateCopy)
            ));
            let one = verify_copies(&manifest, &twice[..1], &config).unwrap();
            assert!(one.copies[0].is_clean(&config) && !one.is_redundant_and_clean(&config));
        }

        #[test]
        fn a_tool_that_does_as_well_as_the_baseline_passes_and_one_that_does_worse_fails_the_gate()
        {
            let config = Config::default();
            let cases = case_names(200);
            let base = Baseline::from_results(
                &SimTool {
                    seed: 7,
                    fail_per_mille: 100,
                }
                .run(&cases),
                &config,
            );
            let same = evaluate(
                &base,
                &SimTool {
                    seed: 7,
                    fail_per_mille: 100,
                }
                .run(&cases),
                &config,
            );
            assert!(same.passed && same.new_failures.is_empty() && same.missing.is_empty());
            let better = evaluate(
                &base,
                &SimTool {
                    seed: 7,
                    fail_per_mille: 50,
                }
                .run(&cases),
                &config,
            );
            assert!(better.passed && better.new_failures.is_empty());
            let worse = evaluate(
                &base,
                &SimTool {
                    seed: 7,
                    fail_per_mille: 300,
                }
                .run(&cases),
                &config,
            );
            assert!(!worse.passed && !worse.new_failures.is_empty());
            assert!(worse.levels.iter().any(|l| l.dropped));
            assert!(worse.failures_by_class.values().sum::<u64>() > 0);
        }

        #[test]
        fn over_many_seeds_a_bigger_share_of_failures_always_fails_the_gate_and_a_smaller_one_never_does(
        ) {
            let config = Config::default();
            let cases = case_names(120);
            for seed in 1..=20u64 {
                let low = SimTool {
                    seed,
                    fail_per_mille: 80,
                }
                .run(&cases);
                let high = SimTool {
                    seed,
                    fail_per_mille: 250,
                }
                .run(&cases);
                assert!(failures(&high) > failures(&low), "seed {seed}");
                let from_low = Baseline::from_results(&low, &config);
                let from_high = Baseline::from_results(&high, &config);
                assert!(!evaluate(&from_low, &high, &config).passed, "seed {seed}");
                assert!(evaluate(&from_high, &low, &config).passed, "seed {seed}");
                assert_eq!(
                    evaluate(&from_low, &high, &config),
                    evaluate(&from_low, &high, &config)
                );
            }
        }

        #[test]
        fn cases_that_never_came_back_are_missing_and_fail_the_gate_unless_the_setting_allows_it() {
            let config = Config::default();
            let cases = case_names(100);
            let tool = SimTool {
                seed: 3,
                fail_per_mille: 0,
            };
            let full = tool.run(&cases);
            let base = Baseline::from_results(&full, &config);
            let partial = lose_lines(&full, 9, 150);
            assert!(partial.len() < full.len());
            let report = evaluate(&base, &partial, &config);
            assert_eq!(report.missing.len(), full.len() - partial.len());
            assert!(!report.passed);
            assert!(report.new_failures.is_empty());
            let lenient = Config {
                fail_on_missing_case: false,
                ..Config::default()
            };
            assert!(evaluate(&base, &partial, &lenient).passed);
            assert!(evaluate(&base, &full, &config).passed);
        }

        #[test]
        fn results_cut_at_any_place_never_panic_and_the_full_text_reads_back_as_written() {
            let config = Config::default();
            let results = SimTool {
                seed: 2,
                fail_per_mille: 300,
            }
            .run(&case_names(15));
            let text = render_results(&results);
            assert_eq!(parse_results(&text, &config).unwrap(), results);
            for cut in 0..=text.len() {
                if let Ok(read) = parse_results(&text[..cut], &config) {
                    assert!(read.len() <= results.len(), "cut {cut}");
                    assert_eq!(read[..], results[..read.len()], "cut {cut}");
                }
            }
            assert!(matches!(
                parse_results("a\tL0\tfail\n", &config),
                Err(LabError::UnclassifiedFailure { .. })
            ));
            assert!(matches!(
                parse_results("a\tL0\tpass\na\tL0\tpass\n", &config),
                Err(LabError::DuplicateResult { .. })
            ));
        }

        #[test]
        fn a_baseline_cut_at_any_place_is_refused_and_the_whole_one_reads_back() {
            let config = Config::default();
            let results = SimTool {
                seed: 2,
                fail_per_mille: 300,
            }
            .run(&case_names(10));
            let base = Baseline::from_results(&results, &config);
            let json = base.to_json().unwrap();
            assert_eq!(Baseline::from_json(&json, &config).unwrap(), base);
            for cut in (0..json.trim_end().len()).step_by(5) {
                assert!(
                    Baseline::from_json(&json[..cut], &config).is_err(),
                    "cut {cut}"
                );
            }
        }

        #[test]
        fn the_generator_never_stalls_on_a_zero_seed() {
            let mut rng = SimRng::new(0);
            let drawn: Vec<u64> = (0..8).map(|_| rng.below(1_000_000)).collect();
            assert!(drawn.windows(2).any(|w| w[0] != w[1]));
        }
    }
}

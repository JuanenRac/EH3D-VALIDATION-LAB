// SPDX-License-Identifier: GPL-3.0-or-later
//! The command line. It is a thin layer: parsing the arguments and printing; the checks live in the other modules.
//!
//! Exit codes: 0 everything is as expected, 1 a check failed (a copy differs, the gate dropped), 2 the input or the arguments are unusable.

use crate::classify::parse_results;
use crate::config::Config;
use crate::corpus;
use crate::error::LabError;
use crate::gate::{evaluate, Baseline};
use crate::hash::opaque_id;
use crate::manifest::{Manifest, MANIFEST_NAME};
use crate::oracle;
use crate::verify::{verify_copies, EntryStatus};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const USAGE: &str = "\
eh3d-validation-lab [--config FILE] <command>

  manifest <dir>                          print the SHA256SUMS of a golden set
  verify [--show-paths] <manifest> <copy-dir>...
                                          check redundant copies against a manifest
  baseline <results.tsv>                  print the baseline of a run
  gate <baseline.json> <results.tsv>      compare a run with a baseline
  corpus-inventory <dir>                  count the design files of a folder by level and by family and version
  oracle <golden.tsv> <observed.tsv>      compare what the tool says with the golden file; prints a results file for `baseline` and `gate`
  corpus-copy <dir> <target>              make a checked working copy of the design files, with a SHA256SUMS

Reports show only hashes, levels, classes and counts unless --show-paths is given.
";

fn read_text(path: &Path, limit: u64) -> Result<String, LabError> {
    let io = |source| LabError::Io {
        path: path.to_path_buf(),
        source,
    };
    let size = fs::metadata(path).map_err(io)?.len();
    if size > limit {
        return Err(LabError::TooLarge { what: "input file" });
    }
    fs::read_to_string(path).map_err(io)
}

/// Run the command line with the given arguments (without the program name). Returns the exit code.
pub fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    match dispatch(args, out) {
        Ok(code) => code,
        Err(Failure::Usage) => {
            let _ = err.write_all(USAGE.as_bytes());
            2
        }
        Err(Failure::Lab(error)) => {
            let _ = writeln!(err, "error: {error}");
            2
        }
    }
}

enum Failure {
    Usage,
    Lab(LabError),
}

impl From<LabError> for Failure {
    fn from(error: LabError) -> Failure {
        Failure::Lab(error)
    }
}

fn dispatch(args: &[String], out: &mut dyn Write) -> Result<i32, Failure> {
    let mut args = args.to_vec();
    let mut config = Config::default();
    if args.first().map(String::as_str) == Some("--config") {
        if args.len() < 3 {
            return Err(Failure::Usage);
        }
        let path = PathBuf::from(&args[1]);
        config = Config::parse(&read_text(&path, 1024 * 1024)?)?;
        args.drain(..2);
    }
    let (command, rest) = args.split_first().ok_or(Failure::Usage)?;
    match (command.as_str(), rest) {
        ("manifest", [dir]) => {
            let manifest = Manifest::from_dir(Path::new(dir), &config)?;
            let _ = out.write_all(manifest.render().as_bytes());
            Ok(0)
        }
        ("verify", rest) => verify_command(rest, &config, out),
        ("corpus-inventory", [dir]) => {
            let inventory = corpus::inventory(Path::new(dir), &config)?;
            let _ = writeln!(out, "{}", inventory.to_json()?);
            Ok(0)
        }
        ("corpus-copy", [dir, target]) => {
            let report = corpus::copy(Path::new(dir), Path::new(target), &config)?;
            let value = serde_json::json!({
                "copied": report.copied, "bytes": report.bytes, "skipped_path": report.skipped_path,
                "skipped_duplicate": report.skipped_duplicate, "skipped_unsupported": report.skipped_unsupported,
                "manifest_sha256": report.manifest_sha256, "set_digest": report.set_digest,
            });
            let _ = writeln!(out, "{value:#}");
            Ok(0)
        }
        ("oracle", [golden, observed]) => {
            let golden = read_text(Path::new(golden), config.max_results_bytes)?;
            let observed = read_text(Path::new(observed), config.max_results_bytes)?;
            let (results, summary) = oracle::compare(&golden, &observed, &config)?;
            let _ = write!(out, "{}", oracle::render_results(&results));
            let _ = writeln!(
                out,
                "# agreed {} differed {} extra {}",
                summary.agreed, summary.differed, summary.extra
            );
            Ok(if summary.differed == 0 { 0 } else { 1 })
        }
        ("baseline", [results]) => {
            let results = parse_results(
                &read_text(Path::new(results), config.max_results_bytes)?,
                &config,
            )?;
            let json = Baseline::from_results(&results, &config).to_json()?;
            let _ = writeln!(out, "{json}");
            Ok(0)
        }
        ("gate", [baseline, results]) => {
            let baseline = Baseline::from_json(
                &read_text(Path::new(baseline), config.max_results_bytes)?,
                &config,
            )?;
            let results = parse_results(
                &read_text(Path::new(results), config.max_results_bytes)?,
                &config,
            )?;
            let report = evaluate(&baseline, &results, &config);
            let _ = writeln!(out, "{}", report.to_json()?);
            Ok(if report.passed { 0 } else { 1 })
        }
        _ => Err(Failure::Usage),
    }
}

fn verify_command(rest: &[String], config: &Config, out: &mut dyn Write) -> Result<i32, Failure> {
    let (show_paths, rest) = match rest.first().map(String::as_str) {
        Some("--show-paths") => (true, &rest[1..]),
        _ => (false, rest),
    };
    let (manifest_path, copies) = rest.split_first().ok_or(Failure::Usage)?;
    if copies.is_empty() {
        return Err(Failure::Usage);
    }
    let manifest = Manifest::parse(
        &read_text(Path::new(manifest_path), config.max_manifest_bytes)?,
        config,
    )?;
    let roots: Vec<PathBuf> = copies.iter().map(PathBuf::from).collect();
    let report = verify_copies(&manifest, &roots, config)?;
    let name = |path: &str| {
        if show_paths {
            path.to_string()
        } else {
            opaque_id(&config.report_salt, path)
        }
    };
    for (index, copy) in report.copies.iter().enumerate() {
        let bad = copy
            .entries
            .iter()
            .filter(|(_, s)| *s != EntryStatus::Ok)
            .count();
        let _ = writeln!(
            out,
            "copy {}: {} listed, {} not intact, {} extra",
            index + 1,
            copy.entries.len(),
            bad,
            copy.extra.len()
        );
        for (path, status) in copy.entries.iter().filter(|(_, s)| *s != EntryStatus::Ok) {
            let _ = writeln!(out, "  {:?} {}", status, name(path));
        }
        for path in &copy.extra {
            let _ = writeln!(out, "  Extra {}", name(path));
        }
    }
    let redundant = report.copies.len() >= config.min_copies;
    if !redundant {
        let _ = writeln!(
            out,
            "only {} copies given, {} required",
            report.copies.len(),
            config.min_copies
        );
    }
    let clean = report.is_redundant_and_clean(config);
    let _ = writeln!(
        out,
        "{} ({MANIFEST_NAME})",
        if clean { "PASS" } else { "FAIL" }
    );
    Ok(if clean { 0 } else { 1 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn call(list: &[&str]) -> (i32, String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args(list), &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    #[test]
    fn no_arguments_and_unknown_commands_print_the_usage_and_exit_2() {
        for list in [
            &[][..],
            &["nonsense"],
            &["manifest"],
            &["verify"],
            &["verify", "m"],
            &["--config"],
        ] {
            let (code, out, err) = call(list);
            assert_eq!(code, 2, "{list:?}");
            assert!(out.is_empty());
            assert!(err.contains("manifest <dir>"));
        }
    }

    #[test]
    fn manifest_then_verify_then_tamper() {
        let work = tempfile::tempdir().unwrap();
        let (a, b) = (work.path().join("a"), work.path().join("b"));
        for d in [&a, &b] {
            fs::create_dir_all(d).unwrap();
            fs::write(d.join("case-secret.brd"), b"board bytes").unwrap();
        }
        let (code, text, _) = call(&["manifest", a.to_str().unwrap()]);
        assert_eq!(code, 0);
        let sums = work.path().join("SHA256SUMS");
        fs::write(&sums, text).unwrap();
        let (code, report, _) = call(&[
            "verify",
            sums.to_str().unwrap(),
            a.to_str().unwrap(),
            b.to_str().unwrap(),
        ]);
        assert_eq!(
            (code, report.lines().last()),
            (0, Some("PASS (SHA256SUMS)"))
        );

        fs::write(b.join("case-secret.brd"), b"BOARD bytes").unwrap();
        let (code, report, _) = call(&[
            "verify",
            sums.to_str().unwrap(),
            a.to_str().unwrap(),
            b.to_str().unwrap(),
        ]);
        assert_eq!(code, 1);
        assert!(report.contains("Mismatch"));
        assert!(
            !report.contains("case-secret"),
            "paths are hidden by default"
        );
        let (_, shown, _) = call(&[
            "verify",
            "--show-paths",
            sums.to_str().unwrap(),
            a.to_str().unwrap(),
            b.to_str().unwrap(),
        ]);
        assert!(shown.contains("case-secret.brd"));
    }

    #[test]
    fn a_single_copy_fails_with_the_default_setting() {
        let work = tempfile::tempdir().unwrap();
        fs::write(work.path().join("f"), b"x").unwrap();
        let (_, text, _) = call(&["manifest", work.path().to_str().unwrap()]);
        let sums = work.path().join("SHA256SUMS");
        fs::write(&sums, text).unwrap();
        let (code, report, _) = call(&[
            "verify",
            sums.to_str().unwrap(),
            work.path().to_str().unwrap(),
        ]);
        assert_eq!(code, 1);
        assert!(report.contains("only 1 copies given, 2 required"));
    }

    #[test]
    fn baseline_and_gate_exit_codes() {
        let work = tempfile::tempdir().unwrap();
        let first = work.path().join("first.tsv");
        let second = work.path().join("second.tsv");
        fs::write(&first, "a\tL0\tpass\nb\tL0\tpass\n").unwrap();
        fs::write(&second, "a\tL0\tpass\nb\tL0\tfail:phoenix-error\n").unwrap();
        let (code, json, _) = call(&["baseline", first.to_str().unwrap()]);
        assert_eq!(code, 0);
        let base = work.path().join("base.json");
        fs::write(&base, json).unwrap();
        assert_eq!(
            call(&["gate", base.to_str().unwrap(), first.to_str().unwrap()]).0,
            0
        );
        let (code, report, _) = call(&["gate", base.to_str().unwrap(), second.to_str().unwrap()]);
        assert_eq!(code, 1);
        assert!(report.contains("\"passed\": false"));
    }

    #[test]
    fn an_unclassified_failure_is_an_input_error_not_a_gate_result() {
        let work = tempfile::tempdir().unwrap();
        let bad = work.path().join("bad.tsv");
        fs::write(&bad, "a\tL0\tfail\n").unwrap();
        let (code, _, err) = call(&["baseline", bad.to_str().unwrap()]);
        assert_eq!(code, 2);
        assert!(err.contains("must carry a class"));
    }

    #[test]
    fn a_config_file_changes_the_settings() {
        let work = tempfile::tempdir().unwrap();
        fs::write(work.path().join("f"), b"x").unwrap();
        let cfg = work.path().join("lab.cfg");
        fs::write(&cfg, "min_copies = 1\nallow_extra_files = true\n").unwrap();
        let (_, text, _) = call(&["manifest", work.path().to_str().unwrap()]);
        let sums = work.path().join("SHA256SUMS");
        fs::write(&sums, text).unwrap();
        let (code, _, _) = call(&[
            "--config",
            cfg.to_str().unwrap(),
            "verify",
            sums.to_str().unwrap(),
            work.path().to_str().unwrap(),
        ]);
        assert_eq!(code, 0);
        fs::write(&cfg, "min_copes = 1\n").unwrap();
        assert_eq!(
            call(&[
                "--config",
                cfg.to_str().unwrap(),
                "manifest",
                work.path().to_str().unwrap()
            ])
            .0,
            2
        );
    }

    #[test]
    fn the_corpus_commands_print_counts_and_no_names() {
        let src = tempfile::tempdir().unwrap();
        let out = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(src.path().join("private_folder")).unwrap();
        std::fs::write(
            src.path().join("private_folder/secret_name.brd"),
            b"<?xml version=\"1.0\"?><eagle version=\"9.5.2\"></eagle>",
        )
        .unwrap();
        let dir = src.path().to_str().unwrap();
        let (code, json, err) = call(&["corpus-inventory", dir]);
        assert_eq!((code, err.as_str()), (0, ""), "{err}");
        assert!(json.contains("xml 9.5.2") && json.contains("\"design_files\": 1"));
        assert!(!json.contains("private_folder") && !json.contains("secret_name"));
        let target = out.path().join("copy");
        let (code, json, err) = call(&["corpus-copy", dir, target.to_str().unwrap()]);
        assert_eq!((code, err.as_str()), (0, ""), "{err}");
        assert!(json.contains("\"copied\": 1") && !json.contains("secret_name"));
        assert!(target.join("SHA256SUMS").is_file());
        // a second copy into the same place is refused, and so is a copy inside the original
        let (code, _, err) = call(&["corpus-copy", dir, target.to_str().unwrap()]);
        assert_eq!(code, 2);
        assert!(err.contains("inside the original"), "{err}");
        let inner = src.path().join("inside");
        let (code, _, _) = call(&["corpus-copy", dir, inner.to_str().unwrap()]);
        assert_eq!(code, 2);
        assert!(!inner.exists());
        assert_eq!(call(&["corpus-inventory"]).0, 2);
    }

    #[test]
    fn the_oracle_command_compares_and_its_output_feeds_the_gate() {
        let work = tempfile::tempdir().unwrap();
        let golden = work.path().join("golden.tsv");
        let same = work.path().join("same.tsv");
        let worse = work.path().join("worse.tsv");
        fs::write(&golden, "a\tL0:opens-clean\ttrue\na\tL3:drc-errors\t0\n").unwrap();
        fs::write(&same, "a\tL0:opens-clean\ttrue\na\tL3:drc-errors\t0\n").unwrap();
        fs::write(
            &worse,
            "a\tL0:opens-clean\ttrue\na\tL3:drc-errors\t2\tphoenix-error\n",
        )
        .unwrap();
        let (code, results, err) = call(&["oracle", golden.to_str().unwrap(), same.to_str().unwrap()]);
        assert_eq!((code, err.as_str()), (0, ""), "{err}");
        let first = work.path().join("first.tsv");
        fs::write(&first, &results).unwrap();
        let (_, base, _) = call(&["baseline", first.to_str().unwrap()]);
        let base_path = work.path().join("base.json");
        fs::write(&base_path, base).unwrap();
        let (code, results, _) = call(&["oracle", golden.to_str().unwrap(), worse.to_str().unwrap()]);
        assert_eq!(code, 1);
        assert!(results.contains("a\tL3\tfail:phoenix-error"), "{results}");
        let second = work.path().join("second.tsv");
        fs::write(&second, &results).unwrap();
        let (code, _, _) = call(&["gate", base_path.to_str().unwrap(), second.to_str().unwrap()]);
        assert_eq!(code, 1, "a level that passed before and fails now stops the gate");
        let bad = work.path().join("bad.tsv");
        fs::write(&bad, "a\tL3:drc-errors\t2\n").unwrap();
        assert_eq!(call(&["oracle", golden.to_str().unwrap(), bad.to_str().unwrap()]).0, 2);
    }
}

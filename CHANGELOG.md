# Changelog

All notable changes to this project are documented here.

## [Unreleased]

- **A baseline or an oracle run with no cases is refused:** `baseline` and `oracle` exit with 1 and say so when the results or the golden file hold no cases (a baseline of nothing would let every run pass), `min_copies` and `max_entries` must be at least 1 in the settings file, and `baseline` warns on the error output when `report_salt` is empty. Tests cover each.
- **A simulated laboratory (`sim`):** a golden set made from a seed whose redundant copies fail on purpose (a bit changed, a file gone, an extra file, a folder in place of a file) and are repaired, a tool under test that fails a chosen share of the cases (a larger share fails every case that a smaller one failed, and more), results with cases lost and results cut at every place, and a baseline cut at every place. Nine new tests (69 in all) check that each fault is found in the right copy and that the gate fails for a tool that gets worse, passes for one that does as well or better, reports the cases that never came back and refuses cut results and baselines, over 20 seeds. It has only run on a computer: no corpus and no reference program were used.
- **The register of requirements names a test for each row:** `docs/REQUIREMENTS.md` has one row for each capability with an identifier (`VLAB-001` on), its state (planned, prototype, implemented or validated), how far it is (done, partial, pending or blocked), its evidence (simulator, host-test or none), the test that shows it and its limits; a capability that is not written names the test that will show it. `tests/reqs.rs` reads the table and fails when a test does not exist, when a state does not agree with the progress or the evidence, when `docs/STATUS.md` does not say the same numbers, when a README does not link the documents or when a document holds a date or a private reference. `docs/SCOPE.md`, `docs/STATUS.md` and `docs/NOT_YET.md` are rewritten to the same shape and the eight READMEs link the five documents.

## [0.0.3] - Shared validation checks, split CI workflows and pinned Rust toolchain

- **The shared validation tools are updated:** the validator also runs the workflow and lessons checks of the suite with their latest rules, including a check to run before every push.
- **The CI validation checks the workflows and the lessons of past CI failures against a baseline:** `tools/workflow_hygiene.py` and `tools/lessons_lint.py` (copies of the shared tools) run in the validation, and only findings that are not in `tools/workflow_hygiene_baseline.json` and `tools/lessons_lint_baseline.json` fail it, so what exists today stays as it is and a new one is caught. `python tools/workflow_hygiene.py --root . --detail` and `python tools/lessons_lint.py --repo .` list the findings.
- **The validation of the CI is the reusable one of the documentation hub:** the job `validate` calls `validate.yml` of EH3D-DOCS (pinned by commit) and the check of the CHANGELOG is a job of its own.
- **The CI is split in two workflows:** the validation of the repository (manifest, documentation, pictures, tools) stays in `ci.yml` and the Rust jobs move to `rust.yml`, which does not run for a change that only touches Markdown files or pictures, so a change of the documentation no longer builds the code on two systems, and they keep a cache of the build. The names of the jobs are the same.
- **The Rust toolchain is pinned:** `rust-toolchain.toml` fixes the version of Rust (the same one in every repository of the suite) with `rustfmt` and `clippy`, the workflows install that version and not `stable`, so a new release of Rust cannot turn the CI red by itself.

## [0.0.2] - First release: the library, the command line, the stable error codes and the tests

- **Brand:** Add the application icon and the logos of the program.
- **Shared checks updated:** the validator also compares the badge and the Version row of the README with the manifest, fails when one of its checks cannot run, and the CHANGELOG step of the workflow reads the base branch from the environment.

- Write and read the checksum manifest of a golden set (`SHA256SUMS`), with safe relative paths, no duplicates and bounded sizes.
- Verify redundant copies of a golden set against its manifest and report changed, missing and extra files per copy.
- Read results files in which every failure must carry one of three classes, with levels L0 to L5.
- Add the gate that the measured rate of a level must not drop, compared as exact fractions, with a baseline that holds only hashed case names.
- Show only hashes, levels, classes and counts in reports, with a test that no name of a case leaks.
- Add the `manifest`, `verify`, `baseline` and `gate` commands and a settings file for every tuned value.
- Run format, lints and tests of the Rust code on Linux and Windows in the CI.
- Add `corpus-inventory`: counts the design files of a folder by level (library, schematic, board) and by format family and version, with one digest of the set and no names.
- Add `corpus-copy`: makes a working copy of the design files with a `SHA256SUMS`, reads each copy back, opens the originals only for reading and refuses a target inside the original, around it or not empty.
- Add the `oracle` command: compares what a tool says with a golden file check by check, by level, and writes the results file that the baseline and the gate read; a difference needs a class and a missing check counts as an error of the tool.
- Add the settings `corpus_extensions` and `corpus_head_bytes`.
- **The workflows are hardened:** every action is pinned by its commit with its version in a comment, and the checkouts keep no credentials.

## [0.0.1] - The base of the repository

- **The base that every repository of the suite has:** the manifest, the README files in eight languages, this changelog, the CI workflow, the issue and pull-request templates, and the shared checks of EH3D-DOCS (the repository check and the drawers of the two README pictures).
- **A planned project, described honestly:** the scope, the requirements (all planned), the relations with other repositories, the threat model, the decisions that apply and the rules for writing code are in `docs/`; there is no code yet.

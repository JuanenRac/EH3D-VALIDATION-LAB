# Changelog

All notable changes to this project are documented here.

## [Unreleased]

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

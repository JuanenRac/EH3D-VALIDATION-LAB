# Changelog

All notable changes to this project are documented here.

## [Unreleased]
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

# Changelog

All notable changes to this project are documented here.

## [Unreleased]

- Write and read the checksum manifest of a golden set (`SHA256SUMS`), with safe relative paths, no duplicates and bounded sizes.
- Verify redundant copies of a golden set against its manifest and report changed, missing and extra files per copy.
- Read results files in which every failure must carry one of three classes, with levels L0 to L5.
- Add the gate that the measured rate of a level must not drop, compared as exact fractions, with a baseline that holds only hashed case names.
- Show only hashes, levels, classes and counts in reports, with a test that no name of a case leaks.
- Add the `manifest`, `verify`, `baseline` and `gate` commands and a settings file for every tuned value.
- Run format, lints and tests of the Rust code on Linux and Windows in the CI.

## [0.0.1] - The base of the repository

- **The base that every repository of the suite has:** the manifest, the README files in eight languages, this changelog, the CI workflow, the issue and pull-request templates, and the shared checks of EH3D-DOCS (the repository check and the drawers of the two README pictures).
- **A planned project, described honestly:** the scope, the requirements (all planned), the relations with other repositories, the threat model, the decisions that apply and the rules for writing code are in `docs/`; there is no code yet.

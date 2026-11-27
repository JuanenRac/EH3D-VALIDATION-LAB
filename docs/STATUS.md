# EH3D-VALIDATION-LAB: status

## Current state

EH3D-VALIDATION-LAB is early code on a computer: a Rust library and a command line that write and verify the checksum manifest of a golden set against redundant copies, read results files in which every failure carries a class, compare a run with a baseline through a gate, count the design files of a folder and make a checked working copy of them, and compare what a tool says with a golden file. A simulated laboratory makes golden sets whose copies fail on purpose and a tool that gets worse. All of it has run only on data made by its own tests, never on a real corpus or a real golden set; it reads no design file, runs no reference tool and does not yet compare the output of any design tool.

## Capabilities

The official state of each capability is in `REQUIREMENTS.md`. The register has 49 requirements: 42 implemented, 4 prototype, 3 planned and 0 validated. By progress: 42 done, 4 partial, 2 pending and 1 blocked. This page never states a state that `REQUIREMENTS.md` does not.

## Open questions

* Which versions of the reference tool the first golden sets cover, and whether a semi-automatic way of producing them works.
* Where the folder with the originals of the corpus is.

## What it needs from other repositories

* EH3D-PHOENIX-PCB: the readers and the model to compare.
* EH3D-COMMON: the conformance vectors for the container.

## Known technical debt and omitted tests

* 206 tests of the Rust code (nine of them keep this register true) and the 3 of the base; the Rust tests build their own folders and results and never touch a real project.
* Nine of the tests run the simulated laboratory (`src/lib.rs`).
* Not tested: very large golden sets (millions of files), folders synchronised by a cloud service, copies on removable media that disappear during a run, and links inside a golden set (they are refused, which is tested).
* `cargo deny` is not run yet.

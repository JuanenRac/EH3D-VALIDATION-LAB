# EH3D-VALIDATION-LAB: status

## Current state

EH3D-VALIDATION-LAB is early code. Beside the base that every repository of the suite carries, it has a Rust library and a command line that write and verify the checksum manifest of a golden set against redundant copies, read results files in which every failure carries a class, and compare a run with a baseline through a gate. It also counts the design files of a folder by level and by format version and makes a checked working copy of them (the originals are only read), and compares what a tool says with a golden file. All of it has run only on data made by its own tests, never on a real corpus or a real golden set. It reads no design file, runs no reference tool and does not yet compare the output of any design tool.

## Capabilities

The official state of each capability is in REQUIREMENTS.md: 6 requirements: 4 `prototype` (VLAB-001, 002, 003 and 006) and 2 `planned` (VLAB-004 and 005). This page never states a state that REQUIREMENTS.md does not.

## Open questions

* Which reference tool versions are covered by the first golden sets, and whether a semi-automatic way of producing them works.

## Needs from other repositories

* EH3D-PHOENIX-PCB: readers and the model to compare
* EH3D-COMMON: conformance vectors for the container

## Known technical debt and omitted tests

* 41 tests of the Rust code and 3 of the base; the Rust tests build their own folders and results and never touch a real project.
* Not tested: very large golden sets (millions of files), folders synchronised by a cloud service, copies on removable media that disappear during a run, and links inside a golden set (they are refused, which is tested).
* `cargo deny` is not run yet.

# EH3D-VALIDATION-LAB: scope

Status: **early code**. EH3D-VALIDATION-LAB is a library and a command line for the manifests of golden sets, the results in which every failure carries a class and the gate that a measured rate does not drop. It has been tried on data made by its own tests only. This page says what it is for, what it will not do and where it sits among the other programs of the suite.

## Purpose

The test environment of the PHOENIX family: a corpus of reference cases with expected results, oracles made of stored golden files with checksums and redundant verified copies, and regression gates for the whole family, in which the exit criterion is a measured, classified rate on a named corpus and never a promise of 100 %.

## What it does

* Writes and verifies the SHA256SUMS manifest of a golden set against redundant copies: a changed byte, a missing file, an extra file and a folder in place of a file are each reported, in the copy where they happen.
* Reads results files in which every failure carries a class (an error of this project, a limitation of the format, or an error of the file) and refuses an unclassified failure, a repeated line and an unknown level (levels run from L0 to L5).
* Compares a run with a baseline through a gate: a lower rate fails and names the new failures, rates are compared as exact fractions, dropping failing cases to raise the rate is caught, and a case that used to pass and fails now, or is missing, fails the gate (both can be switched off).
* Compares what a tool says with a golden file check by check (`oracle`), and writes the results file that the gate reads; a difference needs a class given by the person who looked at it.
* Counts the design files of a folder by level and by family and version (`corpus-inventory`) and makes a checked working copy of them (`corpus-copy`); the originals are only read.
* Hides the names of the cases in its reports (an opaque identifier with a salt) and runs a simulated laboratory that fails on purpose.

## What it will not do

* Private designs and their golden files never enter this repository; they stay on the machines of the owner.
* The reference tool is never run headless or decompiled: its results are produced by hand through its normal interface and stored.
* It invents no tolerance number: they are fixed after the bench has been characterised.
* It reads no design file and runs no reference tool; the reader that fills the observed file belongs to the design tool.
* It does not claim that anything was proven on a real corpus: the evidence of every capability is `simulator` or `host-test`.

## Where it sits

* **It reads:** a golden set folder and its redundant copies, a SHA256SUMS manifest, results files, baselines, golden and observed files of the oracle, a folder of design files (for the inventory and the copy) and a settings file.
* **It writes:** a manifest, a baseline, a results file or a checked working copy that the person asks for; it never writes to a golden set or to an original, and a copy never goes inside the original nor over files.
* **It is read by:** nothing reads it yet: the continuous integration of the family will run its gate once the design tool fills the observed file.
* **It depends on:** nothing of the suite at build time; it reads no design file yet.

## What it never decides

Whether a design is right, and the limits, the interlocks and the emergency stop of any machine, belong to the person who designs it and to the machine. A gate that passes says that a measured rate did not drop on a named corpus; it does not say that the tool is correct. Nothing here starts motion.

The requirements of the program, with the state of each and the test that shows it, are in `REQUIREMENTS.md`; the state in a paragraph is in `STATUS.md`.

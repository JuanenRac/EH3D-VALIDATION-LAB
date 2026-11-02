# EH3D-VALIDATION-LAB: scope

Status: **early code**. EH3D-VALIDATION-LAB has a library and a command line for golden-set manifests, classified results and the rate gate; the rest below is planned. This page says what it does, what it is meant to do, what it will not do, and where it sits. Nothing here has been used on a real corpus or on a machine.

## Purpose

EH3D-VALIDATION-LAB is planned as a test environment with reference use cases and regression tests of the system.

Test environment of the PHOENIX family: a corpus of reference cases with expected results, oracles made of stored golden files, and regression gates.

## What it does today and what it is planned to do

Written, and tried only on data made by its own tests: the manifest of a golden set and its verification against redundant copies, results files in which every failure carries a class, the gate that a measured rate does not drop, and reports that hide the names of cases. Everything else in this list is planned.

* A corpus of reference cases with expected results, kept as data and runnable from a command line.
* An oracle made of golden files: results produced by a reference tool while it still works, stored with checksums and redundant verified copies, compared by local scripts.
* Regression gates for the whole family: the measured rate of each level of agreement must not drop, every failure is classified (error of this project, limitation of the format, or error of the file), and the measured figure is reported.
* A corpus panel in redacted mode: only codes, counts and levels, never names, values or paths.
* A case reducer that turns a failure on a real project into a minimal anonymised case that the owner reviews before anyone else sees it.
* Named tests for the invariants of the design model (identities survive save, migration and reopening; references never dangle; a failed operation leaves the model intact; undo and redo restore the intended state; import and export do not duplicate objects).
* Templates for physical validation: instrument, condition, result and comparison with the simulation.

## What it will not do

* Private designs and their golden files never enter this repository; they stay on the owner's machines.
* The reference tool is never run headless or decompiled: its results are produced by hand through its normal interface (a semi-automatic way is to be tested) and stored.
* No tolerance numbers are invented: they are fixed after the bench has been characterised.
* No promise of '100 %': the exit criterion is a measured, classified rate on a named corpus.

## Planned shape

A library without a window, written in Rust, with a command-line interface (the commands `manifest`, `verify`, `baseline` and `gate`) and, where it applies, a kind of task for the server (planned). Programs of the suite exchange work through project files or through the server, never by depending on each other.

It is one Rust package with a library and a program. It reads no design file and depends on no other repository of the suite yet.

## Where it sits

It belongs to the PHOENIX family of the EH3D Fabrication Suite: independent repositories that relate through versioned contracts. The design tool (EH3D-PHOENIX-PCB) does not depend at build time on the repositories that depend on it.
See DEPENDENCIES.md for the relations, REQUIREMENTS.md for the requirements and STATUS.md for the honest state.

# EH3D-VALIDATION-LAB: decisions that apply here

Decisions of the suite that shape this project, written as project documentation. A decision marked proposed is not final: this project must not rely on it silently, and where a number or a limit depends on it, the code will keep it as one named constant.

## Independent repositories related by contracts

State: **accepted**.

Every repository of the suite is independent. Repositories relate through versioned contracts and build next to each other as sibling folders. Programs never depend on each other: they exchange work through project files or through the server. The design tool does not depend at build time on the satellites that depend on it; it reaches them through their command line or through server tasks. A lower layer never depends on a higher one.

## The PHOENIX project keeps its own format

State: **accepted**.

The design tool stores everything in its own project format, with one folder per domain (design, mechanical, simulation, thermal, RF, history, passport, models, exports). The `.eh3dproj` package stays the exchange format for fabrication and is exported from it; the two are not nested. Domains this project does not know are preserved, and domains that change electrical or geometric meaning must be understood or the project is opened read-only.

## Third-party tools as oracles through golden files

State: **accepted**.

Results of a reference tool (opening without warnings, rule checks, manufacturing outputs) are produced by hand through its normal interface while it still works, stored with checksums and redundant verified copies on more than one medium, and compared by local scripts. The files stay private. How a run is made is recorded with each set: tool version and build, rule and output-job files, settings and operating system.

## Clean-room policy

State: **accepted**.

Formats are learned from public specifications, from files of the author's own and from golden-file comparisons. Executables or libraries of third-party software are never decompiled, disassembled or inspected to derive format details. Help texts, libraries, scripts and output jobs of other vendors are never copied into a repository or redistributed.

## Canonical JSON and numbers in new formats

State: **proposed**.

New JSON files use the JSON Canonicalization Scheme (RFC 8785) plus UTF-8 without BOM and a final line feed. Design values are integers with the unit in the name; exchange values in millimetres are quantised before writing; physical results with uncertainty are compared numerically, not byte for byte. The existing `.eh3dproj` 1.0.0 behaviour does not change.

## Stable diagnostic codes

State: **proposed**.

Failures that a reader or a check cannot hide become diagnostics with a stable code of the form `PHOENIX-<AREA>-<NNN>` (the registry is kept in the board tool). A code is never reused or renumbered, its message may be translated but the code and field names are not, and a repository never keeps a private list: it asks for numbers in the registry in the same change that uses them, together with a test.

## Private designs stay private

State: **accepted**.

The real projects of the author are never committed, pasted into issues or sent to a service. Tests use synthetic cases or minimal anonymised reproducers that the owner approved. Scripts that run over a private corpus use redacted mode, which reports only codes, counts, levels and hashes.

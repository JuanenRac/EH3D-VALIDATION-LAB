# EH3D-VALIDATION-LAB: threat model

Basic model. A row is a result only where its state says so; the others are plans. The rows are the threats to design against from the first line of code.

| Threat | Planned mitigation | Planned test | State | Residual risk |
|---|---|---|---|---|
| Corpus content leaks through diagnostics, logs or reports | redacted mode and a leak test | automated pattern search over every output | prototype: the gate and copy-check reports show only hashes, levels, classes and counts, with a leak test; the corpus runner does not exist yet | case names are hashed with a salt that is empty by default, so a guessable name could be recovered: set `report_salt` |
| A golden file is silently replaced or corrupted | checksum manifest and redundant copies | verification run | prototype: a changed byte, a missing file and an extra file are each reported per copy | the copies may sit on the same medium, which the tool cannot tell; the manifest itself is not signed |
| Private projects committed by mistake | ignore rules, a scan for private-data patterns in CI | automated scan | planned | not assessed: nothing is implemented |
| Corrupt or malicious project or input files (oversized entries, path tricks, deep nesting, resource exhaustion) | name rules, size and depth limits, bounded readers | fuzzing and hostile samples | prototype: manifests, results and baselines are bounded, and unsafe paths, links and duplicates are refused, with tests; no fuzzing yet | not assessed beyond those tests |
| Temporary files, logs and reports that reveal private designs | no private data in outputs; redaction | automated pattern search | planned | not assessed: nothing is implemented |
| Compromised dependency or binary | pinned versions, licence and advisory checks, software bill of materials for releases | CI checks | planned | not assessed: nothing is implemented |
| Power loss or forced close during a save | atomic write: temporary file, then rename; old file never deleted first | interruption tests at controlled points | planned | not assessed: nothing is implemented |

Minimum negative tests once there is code: hostile files and resource limits; secrets absent from logs, errors and reports; pinned dependencies and licence review before publishing. Report a vulnerability privately as described in SECURITY.md, without attaching private designs.

# EH3D-VALIDATION-LAB: requirements

This register is the single source of the state of each capability. Status values: planned, prototype, implemented, validated. Every requirement is `planned` today: nothing is implemented or verified. High test coverage is not proof of geometric or electrical correctness, so each requirement says how it will be checked and what result is expected.

| Requirement | Description | Status | How it will be checked | Expected result | Known limits |
|---|---|---|---|---|---|
| VLAB-001 | Run the corpus in redacted mode. | planned | automatic: leak test | No string taken from the corpus appears in any report. | Not started: no limits known yet. |
| VLAB-002 | Golden-set manifest with checksums, verified against redundant copies. | planned | automatic | A copy that differs from the manifest is reported. | Not started: no limits known yet. |
| VLAB-003 | Compare local results with the golden sets and classify each failure. | planned | automatic | Every failure carries one of the three classes. | Not started: no limits known yet. |
| VLAB-004 | Reduce a failing real project to a minimal anonymised case. | planned | automatic plus owner review | The reduced case contains no recognisable names, attributes or geometry. | Not started: no limits known yet. |
| VLAB-005 | Named tests for the model invariants, each linked to a requirement. | planned | automatic: property tests | Each invariant has at least one named test. | Not started: no limits known yet. |
| VLAB-006 | Gate: the measured rate of a level does not drop between runs. | planned | automatic | A drop fails the gate and lists the new failures. | Not started: no limits known yet. |

Rules: a claim of correctness names the corpus, the tolerance and the tool version it holds for. A phase is complete when the measured rate does not drop, every failure is classified and fixed or documented, and the measured figure is reported.

# EH3D-VALIDATION-LAB: requirements

This register is the single source of the state of each capability: `STATUS.md` and the README files cite it and never state a state that it does not. A state is one of `planned`, `prototype`, `implemented` and `validated`. `validated` means that a person has run the capability on a real machine and the evidence of the manifest says `bench` or `field`; nothing here is validated, because nothing of this program has been run on a real corpus, a real golden set or a reference tool.

The register has 49 requirements: 42 implemented, 4 prototype, 3 planned and 0 validated. By progress: 42 done, 4 partial, 2 pending and 1 blocked. High coverage is not proof of correctness, so each row says how it is checked: the test that names it, and what it shows.

## How to read the register

* **ID:** `VLAB-NNN`, never reused and never renumbered; a new capability takes the next number.
* **State:** `implemented` is code that is complete and tested on a computer, `prototype` is part of the capability, `planned` is not written.
* **Progress:** `done`, `partial`, `pending` (can be written without hardware) or `blocked` (waits for hardware or for another program of the suite).
* **Evidence:** `simulator` when the test runs against a simulated golden set whose copies fail on purpose, a tool under test that fails a chosen share of the cases and damaged results, `host-test` when it checks a calculation, a reader or a screen-less piece, `none` when nothing is written yet.
* **Test:** the acceptance criterion: `file::name` is the test `name` of `tests/file.rs` (`src/file::name` is a test inside `src/file.rs`), which must exist. For a capability that is not written the cell says `planned:` and the name of the test that will show it; the day that test exists, the row must change.
* **Limits:** what is not done or not proven; a row that is `partial` or `blocked` always says it.

The check is `tests/reqs.rs`: it reads this table, finds each test and fails when a state, a test or a count does not agree.

## Corpus, golden sets and the gate

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| VLAB-001 | Run the corpus in redacted mode: no string taken from the corpus appears in any report. | prototype | partial | host-test | `src/corpus::the_report_holds_no_name_and_no_content` | The reports of the gate and of the copy check hide case names and paths and the corpus commands print counts and digests only; the `oracle` command prints the names of the cases it is given, because its output is the input of the gate on the same machine, so keep that output out of any repository. |
| VLAB-002 | Golden-set manifest with checksums, verified against redundant copies. | prototype | partial | host-test | `src/verify::one_changed_byte_is_reported_as_a_mismatch_in_that_copy_only` | It compares the bytes of folders; it cannot know that two copies are on different media. |
| VLAB-003 | Compare local results with the golden sets and classify each failure. | prototype | partial | host-test | `src/oracle::a_difference_with_a_class_fails_only_its_level` | The values are compared as text: nothing here understands a design file, and the reader that fills the observed file belongs to the design tool. |
| VLAB-004 | Reduce a failing real project to a minimal anonymised case. | planned | pending | none | `planned:reducer::a_reduced_case_holds_no_recognisable_name_attribute_or_geometry` | Not written: it needs the readers of the design tool and the review of the owner. |
| VLAB-005 | Named tests for the invariants of the design model, each linked to a requirement. | planned | pending | none | `planned:invariants::each_invariant_of_the_model_has_a_named_test` | Not written: it needs the model of the design tool. |
| VLAB-006 | Gate: the measured rate of a level does not drop between runs. | prototype | partial | host-test | `src/gate::a_lower_rate_fails_and_names_the_new_failure` | Rates are compared as exact fractions; the gate also fails on a case that used to pass and fails now, or that is missing (both can be switched off). |

## The results and the gate

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| VLAB-007 | Levels run from L0 to L5 and nothing else. | implemented | done | host-test | `src/classify::levels_run_from_l0_to_l5_and_nothing_else` | - |
| VLAB-008 | A results file parses with every class. | implemented | done | host-test | `src/classify::a_results_file_parses_with_every_class` | - |
| VLAB-009 | A failure without a known class is refused. | implemented | done | host-test | `src/classify::a_failure_without_a_known_class_is_refused` | - |
| VLAB-010 | Malformed and repeated lines are refused. | implemented | done | host-test | `src/classify::malformed_and_repeated_lines_are_refused` | - |
| VLAB-011 | The size limit of the results applies. | implemented | done | host-test | `src/classify::the_size_limit_applies` | - |
| VLAB-012 | The same run passes the gate. | implemented | done | host-test | `src/gate::the_same_run_passes_the_gate` | - |
| VLAB-013 | A swap keeps the rate but still fails by default. | implemented | done | host-test | `src/gate::a_swap_keeps_the_rate_but_still_fails_by_default` | - |
| VLAB-014 | Dropping failing cases to raise the rate is caught. | implemented | done | host-test | `src/gate::dropping_failing_cases_to_raise_the_rate_is_caught` | - |
| VLAB-015 | A level with no cases left counts as a drop. | implemented | done | host-test | `src/gate::a_level_with_no_cases_left_counts_as_a_drop` | - |
| VLAB-016 | Rates are compared exactly without rounding. | implemented | done | host-test | `src/gate::rates_are_compared_exactly_without_rounding` | - |
| VLAB-017 | The baseline round-trips and refuses damage. | implemented | done | host-test | `src/gate::the_baseline_round_trips_and_refuses_damage` | - |
| VLAB-018 | A different salt cannot pretend to match. | implemented | done | host-test | `src/gate::a_different_salt_cannot_pretend_to_match` | - |
| VLAB-019 | Reports and baselines never contain a name from the corpus. | implemented | done | host-test | `src/gate::reports_and_baselines_never_contain_a_name_from_the_corpus` | - |
| VLAB-020 | A difference without a class is refused by the oracle. | implemented | done | host-test | `src/oracle::a_difference_without_a_class_is_refused` | - |
| VLAB-021 | A missing check is an error of the tool unless the file says otherwise. | implemented | done | host-test | `src/oracle::a_missing_check_is_an_error_of_the_tool_unless_the_file_says_otherwise` | - |
| VLAB-022 | The worst class of a level wins and extra checks are only counted. | implemented | done | host-test | `src/oracle::the_worst_class_of_a_level_wins_and_extra_checks_are_only_counted` | - |

## Manifests, copies and the corpus

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| VLAB-023 | The hash agrees with the published test vectors. | implemented | done | host-test | `src/hash::matches_the_published_vectors` | - |
| VLAB-024 | The manifest renders and parses back and is sorted. | implemented | done | host-test | `src/manifest::render_and_parse_round_trip_and_are_sorted` | - |
| VLAB-025 | Unsafe paths are refused in a manifest. | implemented | done | host-test | `src/manifest::unsafe_paths_are_refused` | - |
| VLAB-026 | Identical copies are redundant and clean. | implemented | done | host-test | `src/verify::identical_copies_are_redundant_and_clean` | - |
| VLAB-027 | A missing file and an extra file are both failures. | implemented | done | host-test | `src/verify::a_missing_file_and_an_extra_file_are_both_failures` | - |
| VLAB-028 | A single copy is not redundant with the default setting. | implemented | done | host-test | `src/verify::a_single_copy_is_not_redundant_with_the_default_setting` | - |
| VLAB-029 | The same folder twice does not count as two copies. | implemented | done | host-test | `src/verify::the_same_folder_twice_does_not_count_as_two_copies` | - |
| VLAB-030 | The inventory counts by level and family and ignores the rest. | implemented | done | host-test | `src/corpus::the_inventory_counts_by_level_and_family_and_ignores_the_rest` | - |
| VLAB-031 | A copy is byte for byte, with a manifest that both copies verify against. | implemented | done | host-test | `src/corpus::a_copy_is_byte_for_byte_with_a_manifest_that_both_copies_verify_against` | - |
| VLAB-032 | The originals are never touched. | implemented | done | host-test | `src/corpus::the_originals_are_never_touched` | - |
| VLAB-033 | A copy will not go inside the original nor over files. | implemented | done | host-test | `src/corpus::a_copy_will_not_go_inside_the_original_nor_the_original_inside_the_copy_nor_over_files` | - |
| VLAB-034 | Links are left alone and counted. | implemented | done | host-test | `src/corpus::links_are_left_alone_and_counted` | - |

## The command line and the settings

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| VLAB-035 | No arguments and unknown commands print the usage and exit with 2. | implemented | done | host-test | `src/cli::no_arguments_and_unknown_commands_print_the_usage_and_exit_2` | - |
| VLAB-036 | `manifest`, then `verify`, then a tampered file is found. | implemented | done | host-test | `src/cli::manifest_then_verify_then_tamper` | - |
| VLAB-037 | A single copy fails with the default setting. | implemented | done | host-test | `src/cli::a_single_copy_fails_with_the_default_setting` | - |
| VLAB-038 | `baseline` and `gate` exit with the code that tells the result. | implemented | done | host-test | `src/cli::baseline_and_gate_exit_codes` | - |
| VLAB-039 | An unclassified failure is an input error and not a gate result. | implemented | done | host-test | `src/cli::an_unclassified_failure_is_an_input_error_not_a_gate_result` | - |
| VLAB-040 | The corpus commands print counts and no names. | implemented | done | host-test | `src/cli::the_corpus_commands_print_counts_and_no_names` | - |
| VLAB-041 | The output of `oracle` feeds the gate. | implemented | done | host-test | `src/cli::the_oracle_command_compares_and_its_output_feeds_the_gate` | - |
| VLAB-042 | Unknown keys and bad values of the settings are refused with the line. | implemented | done | host-test | `src/config::unknown_keys_and_bad_values_are_refused_with_the_line` | - |

## Simulated laboratory

| ID | Requirement | State | Progress | Evidence | Test | Limits |
|---|---|---|---|---|---|---|
| VLAB-043 | The same seed gives the same set and manifest and another seed a different one. | implemented | done | simulator | `src/lib::the_same_seed_gives_the_same_set_and_manifest_and_another_seed_a_different_one` | The set is made by the twin, not a real corpus. |
| VLAB-044 | Two clean copies are redundant and each fault in one copy is found and repaired. | implemented | done | simulator | `src/lib::two_clean_copies_are_redundant_and_clean_and_each_fault_in_one_copy_is_found_and_repaired` | - |
| VLAB-045 | A tool that does as well as the baseline passes and one that does worse fails the gate. | implemented | done | simulator | `src/lib::a_tool_that_does_as_well_as_the_baseline_passes_and_one_that_does_worse_fails_the_gate` | - |
| VLAB-046 | Over many seeds a bigger share of failures always fails the gate and a smaller one never does. | implemented | done | simulator | `src/lib::over_many_seeds_a_bigger_share_of_failures_always_fails_the_gate_and_a_smaller_one_never_does` | - |
| VLAB-047 | Cases that never came back are missing and fail the gate unless the setting allows it. | implemented | done | simulator | `src/lib::cases_that_never_came_back_are_missing_and_fail_the_gate_unless_the_setting_allows_it` | - |
| VLAB-048 | Results cut at any place never panic and the whole text reads back as written. | implemented | done | simulator | `src/lib::results_cut_at_any_place_never_panic_and_the_full_text_reads_back_as_written` | - |
| VLAB-049 | Compare the output of the design tool with its golden files on a real corpus. | planned | blocked | none | `planned:real_corpus::the_output_of_the_design_tool_is_compared_with_a_real_golden_set` | Waits for the readers of the design tool and for the owner to name the corpus. |

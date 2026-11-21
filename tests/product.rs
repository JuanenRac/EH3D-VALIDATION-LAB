//! Tests of the behaviour of the laboratory: the levels and classes, the results files, the manifest and its paths, the verification of copies, the gate that a
//! rate never drops, the comparison with golden files, the corpus and the command line. Limits are tested at the exact edge, damaged input must be refused
//! with a typed error, and the properties (round trips, stable order, idempotence) must hold for every seed.

use eh3d_validation_lab::classify::{parse_results, CaseResult, FailureClass, Level, Outcome};
use eh3d_validation_lab::cli;
use eh3d_validation_lab::config::Config;
use eh3d_validation_lab::corpus::{self, family_of, Family, Kind};
use eh3d_validation_lab::error::LabError;
use eh3d_validation_lab::gate::{evaluate, Baseline};
use eh3d_validation_lab::hash::{is_sha256_hex, opaque_id, sha256_hex, sha256_reader};
use eh3d_validation_lab::manifest::{check_path, list_files, Manifest};
use eh3d_validation_lab::oracle::{compare, render_results};
use eh3d_validation_lab::sim::{case_names, lose_lines, SimRng, SimSet, SimTool, ALL_FAULTS};
use eh3d_validation_lab::verify::{verify_copies, verify_copy, EntryStatus};
use std::fs;
use std::path::PathBuf;

const EMPTY_SHA: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
const ABC_SHA: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

fn cfg() -> Config {
    Config::default()
}

fn level(n: u8) -> Level {
    Level::new(n).unwrap()
}

fn result(case: &str, n: u8, outcome: Outcome) -> CaseResult {
    CaseResult {
        case: case.into(),
        level: level(n),
        outcome,
    }
}

fn call(list: &[&str]) -> (i32, String, String) {
    let args: Vec<String> = list.iter().map(|s| s.to_string()).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = cli::run(&args, &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

fn path_str(path: &std::path::Path) -> String {
    path.to_str().unwrap().to_string()
}

// ---------------------------------------------------------------- levels and classes

#[test]
fn levels_go_from_zero_to_five() {
    for n in 0..=5u8 {
        assert_eq!(Level::new(n).unwrap().number(), n);
    }
    assert!(Level::new(6).is_none() && Level::new(255).is_none());
    assert_eq!(Level::MAX, 5);
}

#[test]
fn a_level_is_written_as_l_and_one_digit() {
    assert_eq!(level(0).to_string(), "L0");
    assert_eq!(level(5).to_string(), "L5");
    for n in 0..=5u8 {
        assert_eq!(Level::parse(&level(n).to_string()), Some(level(n)));
    }
}

#[test]
fn a_level_text_with_anything_else_is_refused() {
    for text in [
        "", "L", "l1", "1", "L6", "L10", "L-1", "L1 ", " L1", "LL1", "L+1", "L１",
    ] {
        assert!(Level::parse(text).is_none(), "{text:?}");
    }
}

#[test]
fn levels_are_ordered_by_number() {
    let mut list = vec![level(4), level(0), level(2)];
    list.sort();
    assert_eq!(list, vec![level(0), level(2), level(4)]);
}

#[test]
fn a_level_travels_in_json_as_text_and_a_bad_one_is_refused() {
    assert_eq!(serde_json::to_string(&level(3)).unwrap(), "\"L3\"");
    assert_eq!(serde_json::from_str::<Level>("\"L2\"").unwrap(), level(2));
    assert!(serde_json::from_str::<Level>("\"L9\"").is_err());
    assert!(serde_json::from_str::<Level>("2").is_err());
}

#[test]
fn failure_classes_are_read_by_their_kebab_names_only() {
    assert_eq!(
        FailureClass::parse("phoenix-error"),
        Some(FailureClass::PhoenixError)
    );
    assert_eq!(
        FailureClass::parse("format-limitation"),
        Some(FailureClass::FormatLimitation)
    );
    assert_eq!(
        FailureClass::parse("file-error"),
        Some(FailureClass::FileError)
    );
    for text in ["", "Phoenix-Error", "phoenix_error", "file-error ", "other"] {
        assert!(FailureClass::parse(text).is_none(), "{text:?}");
    }
}

#[test]
fn failure_classes_are_ordered_with_the_phoenix_error_first() {
    assert!(FailureClass::PhoenixError < FailureClass::FormatLimitation);
    assert!(FailureClass::FormatLimitation < FailureClass::FileError);
    assert_eq!(
        serde_json::to_string(&FailureClass::FormatLimitation).unwrap(),
        "\"format-limitation\""
    );
}

// ---------------------------------------------------------------- results files

#[test]
fn a_results_file_is_read_with_comments_and_blank_lines_skipped() {
    let text = "# header\n\nc1\tL0\tpass\nc1\tL1\tfail:file-error\n";
    let got = parse_results(text, &cfg()).unwrap();
    assert_eq!(got.len(), 2);
    assert_eq!(got[1].outcome, Outcome::Fail(FailureClass::FileError));
}

#[test]
fn an_empty_results_file_has_no_cases_and_is_not_an_error() {
    assert!(parse_results("", &cfg()).unwrap().is_empty());
    assert!(parse_results("# only a comment\n\n", &cfg())
        .unwrap()
        .is_empty());
}

#[test]
fn crlf_results_are_read_like_lf_ones() {
    let lf = parse_results("a\tL0\tpass\nb\tL1\tpass\n", &cfg()).unwrap();
    let crlf = parse_results("a\tL0\tpass\r\nb\tL1\tpass\r\n", &cfg()).unwrap();
    assert_eq!(lf, crlf);
}

#[test]
fn a_bare_fail_is_refused_as_unclassified_with_its_line() {
    let e = parse_results("a\tL0\tpass\nb\tL0\tfail\n", &cfg()).unwrap_err();
    assert!(matches!(e, LabError::UnclassifiedFailure { line: 2 }));
    let e = parse_results("a\tL0\tfail:nonsense\n", &cfg()).unwrap_err();
    assert!(matches!(e, LabError::UnclassifiedFailure { line: 1 }));
}

#[test]
fn badly_shaped_results_lines_are_syntax_errors() {
    for text in [
        "a\tL0\n",
        "a\tL0\tpass\textra\n",
        "\tL0\tpass\n",
        "a\tL9\tpass\n",
        "a\tL0\tok\n",
        "a L0 pass\n",
        " # not a comment\n",
    ] {
        let e = parse_results(text, &cfg()).unwrap_err();
        assert!(matches!(e, LabError::ResultsSyntax { line: 1 }), "{text:?}");
    }
}

#[test]
fn line_numbers_count_comments_and_blank_lines() {
    let e = parse_results("# c\n\nbad line\n", &cfg()).unwrap_err();
    assert!(matches!(e, LabError::ResultsSyntax { line: 3 }));
}

#[test]
fn the_same_case_and_level_twice_is_refused_but_another_level_is_not() {
    assert!(parse_results("a\tL0\tpass\na\tL1\tpass\n", &cfg()).is_ok());
    let e = parse_results("a\tL0\tpass\na\tL0\tfail:file-error\n", &cfg()).unwrap_err();
    assert!(matches!(e, LabError::DuplicateResult { line: 2 }));
}

#[test]
fn the_number_of_results_is_limited_at_the_exact_edge() {
    let two = Config {
        max_entries: 2,
        ..cfg()
    };
    assert!(parse_results("a\tL0\tpass\nb\tL0\tpass\n", &two).is_ok());
    let e = parse_results("a\tL0\tpass\nb\tL0\tpass\nc\tL0\tpass\n", &two).unwrap_err();
    assert!(matches!(e, LabError::TooLarge { what: "results" }));
}

#[test]
fn the_size_of_a_results_file_is_limited_at_the_exact_byte() {
    let text = "a\tL0\tpass\n";
    let exact = Config {
        max_results_bytes: text.len() as u64,
        ..cfg()
    };
    let less = Config {
        max_results_bytes: text.len() as u64 - 1,
        ..cfg()
    };
    assert!(parse_results(text, &exact).is_ok());
    assert!(matches!(
        parse_results(text, &less),
        Err(LabError::TooLarge { .. })
    ));
}

#[test]
fn rendered_results_are_read_back_unchanged() {
    let list = vec![
        result("a", 0, Outcome::Pass),
        result("a", 1, Outcome::Fail(FailureClass::PhoenixError)),
        result("b", 2, Outcome::Fail(FailureClass::FormatLimitation)),
        result("c d", 5, Outcome::Fail(FailureClass::FileError)),
    ];
    let text = render_results(&list);
    assert_eq!(parse_results(&text, &cfg()).unwrap(), list);
    assert!(text.ends_with('\n'));
}

#[test]
fn rendering_no_results_gives_no_text() {
    assert_eq!(render_results(&[]), "");
}

// ---------------------------------------------------------------- settings

#[test]
fn the_defaults_are_the_strict_ones() {
    let c = cfg();
    assert_eq!(c.min_copies, 2);
    assert!(!c.allow_extra_files && c.fail_on_new_failure && c.fail_on_missing_case);
    assert_eq!(c.corpus_extensions, vec!["lbr", "sch", "brd"]);
    assert_eq!(Config::parse("").unwrap(), c);
}

#[test]
fn flags_are_only_true_or_false_in_lower_case() {
    assert!(
        Config::parse("allow_extra_files = true\n")
            .unwrap()
            .allow_extra_files
    );
    for word in ["True", "yes", "1", "on", ""] {
        let e = Config::parse(&format!("allow_extra_files = {word}\n")).unwrap_err();
        assert!(
            matches!(e, LabError::ConfigBadValue { line: 1, .. }),
            "{word}"
        );
    }
}

#[test]
fn counts_must_be_at_least_one() {
    assert!(Config::parse("min_copies = 0\n").is_err());
    assert!(Config::parse("max_entries = 0\n").is_err());
    assert_eq!(Config::parse("min_copies = 1\n").unwrap().min_copies, 1);
    assert_eq!(Config::parse("max_entries = 1\n").unwrap().max_entries, 1);
}

#[test]
fn the_head_bytes_are_bounded_on_both_sides() {
    assert!(Config::parse("corpus_head_bytes = 15\n").is_err());
    assert!(Config::parse("corpus_head_bytes = 16\n").is_ok());
    assert!(Config::parse("corpus_head_bytes = 1048576\n").is_ok());
    assert!(Config::parse("corpus_head_bytes = 1048577\n").is_err());
}

#[test]
fn the_corpus_extensions_are_lower_cased_and_limited_to_the_three_kinds() {
    let c = Config::parse("corpus_extensions = LBR, Sch\n").unwrap();
    assert_eq!(c.corpus_extensions, vec!["lbr", "sch"]);
    for bad in [
        "corpus_extensions = png\n",
        "corpus_extensions =\n",
        "corpus_extensions = lbr,,sch\n",
    ] {
        assert!(Config::parse(bad).is_err(), "{bad:?}");
    }
}

#[test]
fn the_salt_is_trimmed_and_may_hold_spaces_inside() {
    let c = Config::parse("report_salt =   a b c  \n").unwrap();
    assert_eq!(c.report_salt, "a b c");
}

#[test]
fn settings_errors_name_the_line_and_the_key() {
    let e = Config::parse("# x\nnonsense\n").unwrap_err();
    assert_eq!(
        e.to_string(),
        "configuration line 2: expected `key = value`"
    );
    let e = Config::parse("\n\nzzz = 1\n").unwrap_err();
    assert_eq!(e.to_string(), "configuration line 3: unknown key `zzz`");
    let e = Config::parse("max_entries = x\n").unwrap_err();
    assert_eq!(
        e.to_string(),
        "configuration line 1: bad value for `max_entries`"
    );
}

// ---------------------------------------------------------------- hashes

#[test]
fn the_hash_of_known_inputs_is_the_standard_one() {
    assert_eq!(sha256_hex(b""), EMPTY_SHA);
    assert_eq!(sha256_hex(b"abc"), ABC_SHA);
}

#[test]
fn the_reader_and_the_slice_give_the_same_hash_across_block_edges() {
    for len in [0usize, 1, 65_535, 65_536, 65_537, 200_000] {
        let bytes: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
        assert_eq!(
            sha256_reader(std::io::Cursor::new(&bytes)).unwrap(),
            sha256_hex(&bytes),
            "{len}"
        );
    }
}

#[test]
fn a_hash_text_must_be_64_lower_case_hex_characters() {
    assert!(is_sha256_hex(EMPTY_SHA));
    assert!(!is_sha256_hex(&EMPTY_SHA[..63]));
    assert!(!is_sha256_hex(&format!("{EMPTY_SHA}0")));
    assert!(!is_sha256_hex(&EMPTY_SHA.to_uppercase()));
    assert!(!is_sha256_hex(&format!("g{}", &EMPTY_SHA[1..])));
    assert!(!is_sha256_hex(""));
}

#[test]
fn an_opaque_id_is_sixteen_hex_characters_and_depends_on_salt_and_name() {
    let id = opaque_id("salt", "case");
    assert_eq!(id.len(), 16);
    assert!(id
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
    assert_eq!(id, opaque_id("salt", "case"));
    assert_ne!(id, opaque_id("other", "case"));
    assert_ne!(id, opaque_id("salt", "case2"));
}

#[test]
fn the_boundary_between_salt_and_name_is_not_ambiguous() {
    assert_ne!(opaque_id("a", "bc"), opaque_id("ab", "c"));
    assert_ne!(opaque_id("", "x"), opaque_id("x", ""));
}

// ---------------------------------------------------------------- manifest paths

#[test]
fn ordinary_relative_paths_are_accepted() {
    for p in ["a", "a/b.sch", "libs/x y.lbr", "a-b_c/d.e", "ü/ñ.brd"] {
        assert!(check_path(p, &cfg(), 1).is_ok(), "{p}");
    }
}

#[test]
fn unsafe_paths_are_refused_with_the_line() {
    for p in [
        "", "/a", "a/", "a//b", "./a", "a/./b", "../a", "a/..", "a\\b", "c:x", "a\tb", "a\nb",
        "a\u{7f}b",
    ] {
        let e = check_path(p, &cfg(), 7).unwrap_err();
        assert!(matches!(e, LabError::BadPath { line: 7 }), "{p:?}");
    }
}

#[test]
fn a_path_is_limited_in_bytes_at_the_exact_edge() {
    let c = Config {
        max_path_bytes: 5,
        ..cfg()
    };
    assert!(check_path("abcde", &c, 1).is_ok());
    assert!(check_path("abcdef", &c, 1).is_err());
    assert!(check_path("üüü", &c, 1).is_err());
}

// ---------------------------------------------------------------- manifest

fn h(n: u8) -> String {
    format!("{n:02x}").repeat(32)
}

#[test]
fn a_manifest_is_read_and_written_in_sorted_order() {
    let text = format!("{}  b.sch\n{}  a.lbr\n", h(2), h(1));
    let m = Manifest::parse(&text, &cfg()).unwrap();
    assert_eq!(m.len(), 2);
    assert_eq!(m.render(), format!("{}  a.lbr\n{}  b.sch\n", h(1), h(2)));
    assert_eq!(Manifest::parse(&m.render(), &cfg()).unwrap(), m);
}

#[test]
fn an_empty_manifest_renders_nothing() {
    let m = Manifest::new();
    assert!(m.is_empty());
    assert_eq!(m.render(), "");
    assert!(Manifest::parse("", &cfg()).unwrap().is_empty());
    assert!(Manifest::parse("\n\n", &cfg()).unwrap().is_empty());
}

#[test]
fn a_manifest_line_needs_two_spaces_between_hash_and_path() {
    for text in [
        format!("{} a\n", h(1)),
        "nonsense\n".to_string(),
        format!("{}\ta\n", h(1)),
    ] {
        let e = Manifest::parse(&text, &cfg()).unwrap_err();
        assert!(
            matches!(e, LabError::ManifestSyntax { line: 1 }),
            "{text:?}"
        );
    }
}

#[test]
fn an_upper_case_or_short_hash_is_refused() {
    let upper = format!("{}  a\n", h(0xab).to_uppercase());
    assert!(matches!(
        Manifest::parse(&upper, &cfg()),
        Err(LabError::ManifestSyntax { line: 1 })
    ));
    assert!(Manifest::parse("abcd  a\n", &cfg()).is_err());
}

#[test]
fn a_path_with_a_space_inside_is_kept_whole() {
    let m = Manifest::parse(&format!("{}  my file.sch\n", h(1)), &cfg()).unwrap();
    assert!(m.contains("my file.sch"));
}

#[test]
fn paths_that_differ_only_in_letter_case_are_duplicates() {
    let text = format!("{}  A.sch\n{}  a.SCH\n", h(1), h(2));
    let e = Manifest::parse(&text, &cfg()).unwrap_err();
    assert!(matches!(e, LabError::DuplicatePath { line: 2 }));
}

#[test]
fn the_entries_of_a_manifest_are_limited_at_the_exact_edge() {
    let c = Config {
        max_entries: 2,
        ..cfg()
    };
    let mut m = Manifest::new();
    m.insert("a", &h(1), &c, 1).unwrap();
    m.insert("b", &h(2), &c, 2).unwrap();
    assert!(matches!(
        m.insert("c", &h(3), &c, 3),
        Err(LabError::TooLarge { what: "entries" })
    ));
    assert_eq!(m.len(), 2);
}

#[test]
fn the_size_of_a_manifest_is_limited_at_the_exact_byte() {
    let text = format!("{}  a\n", h(1));
    let exact = Config {
        max_manifest_bytes: text.len() as u64,
        ..cfg()
    };
    let less = Config {
        max_manifest_bytes: text.len() as u64 - 1,
        ..cfg()
    };
    assert!(Manifest::parse(&text, &exact).is_ok());
    assert!(matches!(
        Manifest::parse(&text, &less),
        Err(LabError::TooLarge {
            what: "manifest bytes"
        })
    ));
}

#[test]
fn the_manifest_of_a_folder_lists_every_file_below_it_and_leaves_out_its_own_file() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("libs/deep")).unwrap();
    fs::write(dir.path().join("a.txt"), b"abc").unwrap();
    fs::write(dir.path().join("libs/deep/b.txt"), b"").unwrap();
    fs::write(dir.path().join("SHA256SUMS"), b"ignored").unwrap();
    let m = Manifest::from_dir(dir.path(), &cfg()).unwrap();
    let list: Vec<(&str, &str)> = m.iter().collect();
    assert_eq!(
        list,
        vec![("a.txt", ABC_SHA), ("libs/deep/b.txt", EMPTY_SHA)]
    );
}

#[test]
fn the_manifest_of_an_empty_folder_is_empty_and_of_a_missing_folder_an_error() {
    let dir = tempfile::tempdir().unwrap();
    assert!(Manifest::from_dir(dir.path(), &cfg()).unwrap().is_empty());
    let missing = dir.path().join("nope");
    assert!(matches!(
        Manifest::from_dir(&missing, &cfg()),
        Err(LabError::Io { .. })
    ));
}

#[test]
fn the_list_of_files_is_sorted_and_limited_at_the_exact_edge() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["b", "a", "c"] {
        fs::write(dir.path().join(name), b"x").unwrap();
    }
    assert_eq!(list_files(dir.path(), &cfg()).unwrap(), vec!["a", "b", "c"]);
    let three = Config {
        max_entries: 3,
        ..cfg()
    };
    let two = Config {
        max_entries: 2,
        ..cfg()
    };
    assert!(list_files(dir.path(), &three).is_ok());
    assert!(matches!(
        list_files(dir.path(), &two),
        Err(LabError::TooLarge { .. })
    ));
}

// ---------------------------------------------------------------- verification of copies

fn two_copies(set: &SimSet) -> (tempfile::TempDir, tempfile::TempDir, Manifest) {
    let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    set.materialize(a.path()).unwrap();
    set.materialize(b.path()).unwrap();
    let m = Manifest::parse(&set.manifest_text(), &cfg()).unwrap();
    (a, b, m)
}

#[test]
fn a_missing_file_is_reported_as_missing() {
    let set = SimSet::generate(11, 6);
    let (a, _b, m) = two_copies(&set);
    let victim = set.paths()[2].clone();
    fs::remove_file(a.path().join(&victim)).unwrap();
    let report = verify_copy(&m, a.path(), &cfg()).unwrap();
    let bad: Vec<_> = report
        .entries
        .iter()
        .filter(|(_, s)| *s != EntryStatus::Ok)
        .collect();
    assert_eq!(bad.len(), 1);
    assert_eq!(
        (bad[0].0.as_str(), &bad[0].1),
        (victim.as_str(), &EntryStatus::Missing)
    );
    assert!(!report.is_clean(&cfg()));
}

#[test]
fn a_changed_file_is_a_mismatch_and_a_folder_in_its_place_is_unreadable() {
    let set = SimSet::generate(12, 6);
    let (a, _b, m) = two_copies(&set);
    let paths = set.paths();
    fs::write(a.path().join(&paths[0]), b"changed").unwrap();
    fs::remove_file(a.path().join(&paths[1])).unwrap();
    fs::create_dir(a.path().join(&paths[1])).unwrap();
    let report = verify_copy(&m, a.path(), &cfg()).unwrap();
    assert_eq!(report.entries[0].1, EntryStatus::Mismatch);
    assert_eq!(report.entries[1].1, EntryStatus::Unreadable);
}

#[test]
fn an_extra_file_makes_a_copy_unclean_unless_the_setting_allows_it() {
    let set = SimSet::generate(13, 4);
    let (a, _b, m) = two_copies(&set);
    fs::write(a.path().join("notes.txt"), b"x").unwrap();
    let report = verify_copy(&m, a.path(), &cfg()).unwrap();
    assert_eq!(report.extra, vec!["notes.txt"]);
    assert!(!report.is_clean(&cfg()));
    let lax = Config {
        allow_extra_files: true,
        ..cfg()
    };
    assert!(report.is_clean(&lax));
}

#[test]
fn the_number_of_copies_must_reach_the_minimum() {
    let set = SimSet::generate(14, 3);
    let (a, b, m) = two_copies(&set);
    let one = verify_copies(&m, &[a.path().to_path_buf()], &cfg()).unwrap();
    assert!(!one.is_redundant_and_clean(&cfg()));
    let single = Config {
        min_copies: 1,
        ..cfg()
    };
    assert!(one.is_redundant_and_clean(&single));
    let both = verify_copies(
        &m,
        &[a.path().to_path_buf(), b.path().to_path_buf()],
        &cfg(),
    )
    .unwrap();
    assert!(both.is_redundant_and_clean(&cfg()));
}

#[test]
fn the_same_copy_given_twice_is_refused_even_by_another_spelling() {
    let set = SimSet::generate(15, 3);
    let (a, _b, m) = two_copies(&set);
    let same = a.path().join(".");
    let e = verify_copies(&m, &[a.path().to_path_buf(), same], &cfg()).unwrap_err();
    assert!(matches!(e, LabError::DuplicateCopy));
}

#[test]
fn a_missing_copy_folder_is_an_io_error() {
    let set = SimSet::generate(16, 3);
    let (a, _b, m) = two_copies(&set);
    let gone: PathBuf = a.path().join("gone");
    assert!(matches!(
        verify_copies(&m, &[gone], &cfg()),
        Err(LabError::Io { .. })
    ));
}

#[test]
fn a_manifest_of_nothing_is_verified_clean_against_an_empty_folder() {
    let dir = tempfile::tempdir().unwrap();
    let r = verify_copy(&Manifest::new(), dir.path(), &cfg()).unwrap();
    assert!(r.entries.is_empty() && r.extra.is_empty() && r.is_clean(&cfg()));
}

// ---------------------------------------------------------------- the gate

fn run_of(passes: &[(&str, u8, bool)]) -> Vec<CaseResult> {
    passes
        .iter()
        .map(|(c, l, ok)| {
            result(
                c,
                *l,
                if *ok {
                    Outcome::Pass
                } else {
                    Outcome::Fail(FailureClass::PhoenixError)
                },
            )
        })
        .collect()
}

#[test]
fn a_baseline_is_sorted_hides_the_names_and_does_not_depend_on_the_input_order() {
    let one = run_of(&[("a", 0, true), ("b", 1, false), ("c", 0, true)]);
    let mut other = one.clone();
    other.reverse();
    let (x, y) = (
        Baseline::from_results(&one, &cfg()),
        Baseline::from_results(&other, &cfg()),
    );
    assert_eq!(x, y);
    assert!(x.cases.windows(2).all(|w| w[0] <= w[1]));
    assert!(!x.to_json().unwrap().contains("\"a\""));
}

#[test]
fn the_salt_changes_every_id_of_the_baseline() {
    let r = run_of(&[("a", 0, true)]);
    let salted = Config {
        report_salt: "s".into(),
        ..cfg()
    };
    assert_ne!(
        Baseline::from_results(&r, &cfg()).cases[0].id,
        Baseline::from_results(&r, &salted).cases[0].id
    );
}

#[test]
fn a_baseline_goes_through_json_and_back() {
    let b = Baseline::from_results(&run_of(&[("a", 0, true), ("b", 2, false)]), &cfg());
    assert_eq!(
        Baseline::from_json(&b.to_json().unwrap(), &cfg()).unwrap(),
        b
    );
    assert_eq!(b.version, 1);
}

#[test]
fn a_damaged_baseline_is_refused() {
    let good = Baseline::from_results(&run_of(&[("a", 0, true)]), &cfg())
        .to_json()
        .unwrap();
    for text in [
        "".to_string(),
        "{}".to_string(),
        "[]".to_string(),
        good.replace("\"version\": 1", "\"version\": 2"),
        good.replace("\"L0\"", "\"L9\""),
    ] {
        assert!(
            matches!(
                Baseline::from_json(&text, &cfg()),
                Err(LabError::BadBaseline)
            ),
            "{text}"
        );
    }
}

#[test]
fn a_baseline_with_the_same_case_and_level_twice_is_refused() {
    let one = Baseline::from_results(&run_of(&[("a", 0, true)]), &cfg());
    let mut twice = one.clone();
    twice.cases.push(one.cases[0].clone());
    let text = twice.to_json().unwrap();
    assert!(matches!(
        Baseline::from_json(&text, &cfg()),
        Err(LabError::BadBaseline)
    ));
}

#[test]
fn the_baseline_is_limited_at_the_exact_edge() {
    let b = Baseline::from_results(&run_of(&[("a", 0, true), ("b", 0, true)]), &cfg());
    let text = b.to_json().unwrap();
    let two = Config {
        max_entries: 2,
        ..cfg()
    };
    let one = Config {
        max_entries: 1,
        ..cfg()
    };
    assert!(Baseline::from_json(&text, &two).is_ok());
    assert!(Baseline::from_json(&text, &one).is_err());
    let bytes = Config {
        max_results_bytes: text.len() as u64 - 1,
        ..cfg()
    };
    assert!(matches!(
        Baseline::from_json(&text, &bytes),
        Err(LabError::TooLarge { .. })
    ));
}

#[test]
fn the_same_run_passes_its_own_baseline() {
    let r = run_of(&[("a", 0, true), ("b", 0, false), ("c", 1, true)]);
    let b = Baseline::from_results(&r, &cfg());
    let report = evaluate(&b, &r, &cfg());
    assert!(report.passed && report.new_failures.is_empty() && report.missing.is_empty());
    assert_eq!(report.levels.len(), 2);
}

#[test]
fn a_case_that_passed_and_now_fails_is_a_new_failure_and_a_dropped_rate() {
    let base = Baseline::from_results(&run_of(&[("a", 0, true), ("b", 0, true)]), &cfg());
    let now = run_of(&[("a", 0, true), ("b", 0, false)]);
    let report = evaluate(&base, &now, &cfg());
    assert!(!report.passed);
    assert_eq!(report.new_failures.len(), 1);
    assert_eq!(report.new_failures[0].class, FailureClass::PhoenixError);
    let l0 = &report.levels[0];
    assert_eq!(
        (
            l0.baseline_passed,
            l0.baseline_total,
            l0.passed,
            l0.total,
            l0.dropped
        ),
        (2, 2, 1, 2, true)
    );
}

#[test]
fn a_failure_that_is_fixed_never_fails_the_gate() {
    let base = Baseline::from_results(&run_of(&[("a", 0, false), ("b", 0, true)]), &cfg());
    let now = run_of(&[("a", 0, true), ("b", 0, true)]);
    let report = evaluate(&base, &now, &cfg());
    assert!(report.passed);
    assert!(report.failures_by_class.is_empty());
}

#[test]
fn a_failure_that_was_already_there_does_not_fail_the_gate() {
    let r = run_of(&[("a", 0, false), ("b", 0, true)]);
    let base = Baseline::from_results(&r, &cfg());
    let report = evaluate(&base, &r, &cfg());
    assert!(report.passed);
    assert_eq!(report.failures_by_class[&FailureClass::PhoenixError], 1);
}

#[test]
fn a_missing_case_fails_unless_the_setting_says_otherwise() {
    let base = Baseline::from_results(&run_of(&[("a", 0, true), ("b", 0, true)]), &cfg());
    let now = run_of(&[("a", 0, true)]);
    let strict = evaluate(&base, &now, &cfg());
    assert!(!strict.passed && strict.missing.len() == 1);
    // the rate of the remaining cases is the same (1 of 1 against 2 of 2), so only the missing case decides
    let lax = Config {
        fail_on_missing_case: false,
        ..cfg()
    };
    assert!(evaluate(&base, &now, &lax).passed);
}

#[test]
fn a_dropped_rate_fails_even_when_new_failures_are_allowed() {
    let base = Baseline::from_results(&run_of(&[("a", 0, true), ("b", 0, true)]), &cfg());
    let now = run_of(&[("a", 0, true), ("b", 0, false)]);
    let lax = Config {
        fail_on_new_failure: false,
        ..cfg()
    };
    assert!(!evaluate(&base, &now, &lax).passed);
}

#[test]
fn a_new_failure_is_allowed_when_the_rate_holds_and_the_setting_says_so() {
    let base = Baseline::from_results(&run_of(&[("a", 0, true), ("b", 0, false)]), &cfg());
    let now = run_of(&[("a", 0, false), ("b", 0, true)]);
    assert!(!evaluate(&base, &now, &cfg()).passed);
    let lax = Config {
        fail_on_new_failure: false,
        ..cfg()
    };
    assert!(evaluate(&base, &now, &lax).passed);
}

#[test]
fn an_empty_run_against_a_baseline_drops_every_level() {
    let base = Baseline::from_results(&run_of(&[("a", 0, true), ("b", 1, true)]), &cfg());
    let report = evaluate(&base, &[], &cfg());
    assert!(!report.passed);
    assert!(report.levels.iter().all(|l| l.dropped && l.total == 0));
    assert_eq!(report.missing.len(), 2);
}

#[test]
fn an_empty_baseline_lets_everything_pass_and_new_cases_are_ignored() {
    let base = Baseline::from_results(&[], &cfg());
    let report = evaluate(&base, &run_of(&[("x", 0, false)]), &cfg());
    assert!(report.passed && report.levels.is_empty());
    assert_eq!(report.failures_by_class[&FailureClass::PhoenixError], 1);
}

#[test]
fn a_case_added_to_the_run_is_not_counted_against_the_baseline() {
    let base = Baseline::from_results(&run_of(&[("a", 0, true)]), &cfg());
    let now = run_of(&[("a", 0, true), ("new", 0, false)]);
    let report = evaluate(&base, &now, &cfg());
    assert!(report.new_failures.is_empty());
    assert_eq!(report.levels[0].total, 2);
    assert!(report.levels[0].dropped);
}

#[test]
fn the_gate_report_is_json_with_stable_keys() {
    let r = run_of(&[("a", 0, true)]);
    let report = evaluate(&Baseline::from_results(&r, &cfg()), &r, &cfg());
    let v: serde_json::Value = serde_json::from_str(&report.to_json().unwrap()).unwrap();
    for key in [
        "passed",
        "levels",
        "new_failures",
        "missing",
        "failures_by_class",
    ] {
        assert!(v.get(key).is_some(), "{key}");
    }
    assert_eq!(v["passed"], true);
    assert_eq!(v["levels"][0]["level"], "L0");
}

#[test]
fn the_gate_does_not_depend_on_the_order_of_the_results() {
    let r = run_of(&[
        ("a", 0, true),
        ("b", 1, false),
        ("c", 0, true),
        ("d", 1, true),
    ]);
    let base = Baseline::from_results(&r, &cfg());
    let mut shuffled = r.clone();
    shuffled.reverse();
    assert_eq!(
        evaluate(&base, &r, &cfg()),
        evaluate(&base, &shuffled, &cfg())
    );
}

// ---------------------------------------------------------------- the comparison with the golden files

#[test]
fn equal_values_agree_and_give_a_pass() {
    let (r, s) = compare("c\tL0:a\t1\n", "c\tL0:a\t1\n", &cfg()).unwrap();
    assert_eq!(r, vec![result("c", 0, Outcome::Pass)]);
    assert_eq!((s.agreed, s.differed, s.extra), (1, 0, 0));
}

#[test]
fn a_different_value_takes_the_class_of_the_observation() {
    let (r, s) = compare("c\tL0:a\t1\n", "c\tL0:a\t2\tfile-error\n", &cfg()).unwrap();
    assert_eq!(r[0].outcome, Outcome::Fail(FailureClass::FileError));
    assert_eq!((s.agreed, s.differed), (0, 1));
}

#[test]
fn a_different_value_without_a_class_is_refused_with_its_line() {
    let e = compare("c\tL0:a\t1\n", "# c\nc\tL0:a\t2\n", &cfg()).unwrap_err();
    assert!(matches!(e, LabError::UnclassifiedFailure { line: 1 }));
}

#[test]
fn an_unknown_class_is_refused() {
    let e = compare("c\tL0:a\t1\n", "c\tL0:a\t2\tweird\n", &cfg()).unwrap_err();
    assert!(matches!(e, LabError::UnclassifiedFailure { .. }));
}

#[test]
fn a_check_the_tool_did_not_report_is_a_phoenix_error() {
    let (r, s) = compare("c\tL0:a\t1\n", "", &cfg()).unwrap();
    assert_eq!(r[0].outcome, Outcome::Fail(FailureClass::PhoenixError));
    assert_eq!(s.differed, 1);
}

#[test]
fn the_worst_class_of_a_case_and_level_wins() {
    let golden = "c\tL0:a\t1\nc\tL0:b\t1\n";
    let observed = "c\tL0:a\t2\tfile-error\nc\tL0:b\t2\tphoenix-error\n";
    let (r, _) = compare(golden, observed, &cfg()).unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].outcome, Outcome::Fail(FailureClass::PhoenixError));
}

#[test]
fn checks_of_one_case_are_grouped_by_level_and_the_results_are_sorted() {
    let golden = "b\tL1:x\t1\na\tL1:y\t1\na\tL0:z\t1\n";
    let (r, s) = compare(golden, golden, &cfg()).unwrap();
    let keys: Vec<(String, u8)> = r
        .iter()
        .map(|x| (x.case.clone(), x.level.number()))
        .collect();
    assert_eq!(
        keys,
        vec![
            ("a".to_string(), 0),
            ("a".to_string(), 1),
            ("b".to_string(), 1)
        ]
    );
    assert_eq!(s.agreed, 3);
}

#[test]
fn what_the_golden_file_does_not_know_is_counted_as_extra() {
    let (_, s) = compare(
        "c\tL0:a\t1\n",
        "c\tL0:a\t1\nc\tL0:more\t5\nd\tL1:x\t1\n",
        &cfg(),
    )
    .unwrap();
    assert_eq!((s.agreed, s.differed, s.extra), (1, 0, 2));
}

#[test]
fn badly_shaped_oracle_lines_are_syntax_errors() {
    for (golden, observed) in [
        ("c\tL0:a\n", ""),
        ("c\tL0:a\t1\textra\n", ""),
        ("\tL0:a\t1\n", ""),
        ("c\tL9:a\t1\n", ""),
        ("c\tL0a\t1\n", ""),
        ("c\tL0:\t1\n", ""),
        ("c\tL0:a\t1\n", "c\tL0:a\t1\tfile-error\textra\n"),
    ] {
        let e = compare(golden, observed, &cfg()).unwrap_err();
        assert!(
            matches!(e, LabError::OracleSyntax { line: 1 }),
            "{golden:?} {observed:?}"
        );
    }
}

#[test]
fn a_check_reported_twice_is_refused() {
    let e = compare("c\tL0:a\t1\nc\tL0:a\t2\n", "", &cfg()).unwrap_err();
    assert!(matches!(e, LabError::DuplicateResult { line: 2 }));
}

#[test]
fn the_oracle_size_and_lines_are_limited_at_the_exact_edge() {
    let golden = "c\tL0:a\t1\n";
    let exact = Config {
        max_results_bytes: golden.len() as u64,
        ..cfg()
    };
    let less = Config {
        max_results_bytes: golden.len() as u64 - 1,
        ..cfg()
    };
    assert!(compare(golden, "", &exact).is_ok());
    assert!(matches!(
        compare(golden, "", &less),
        Err(LabError::TooLarge {
            what: "oracle bytes"
        })
    ));
    let one = Config {
        max_entries: 1,
        ..cfg()
    };
    assert!(compare("c\tL0:a\t1\n", "", &one).is_ok());
    assert!(matches!(
        compare("c\tL0:a\t1\nc\tL0:b\t1\n", "", &one),
        Err(LabError::TooLarge {
            what: "oracle lines"
        })
    ));
}

#[test]
fn comments_and_blank_lines_are_skipped_in_the_oracle_files() {
    let (r, _) = compare("# g\n\nc\tL0:a\t1\n", "# o\n\nc\tL0:a\t1\n", &cfg()).unwrap();
    assert_eq!(r.len(), 1);
}

// ---------------------------------------------------------------- the simulation

#[test]
fn the_generator_has_a_fixed_stream_and_the_zero_seed_is_valid() {
    let (mut a, mut b) = (SimRng::new(5), SimRng::new(5));
    let one: Vec<u64> = (0..20).map(|_| a.below(1000)).collect();
    let two: Vec<u64> = (0..20).map(|_| b.below(1000)).collect();
    assert_eq!(one, two);
    let mut z = SimRng::new(0);
    let zeros: Vec<u64> = (0..20).map(|_| z.below(1000)).collect();
    assert!(zeros.iter().any(|n| *n != 0));
    assert!(one.iter().all(|n| *n < 1000));
}

#[test]
fn a_generated_set_has_the_asked_number_of_files_in_three_folders() {
    let set = SimSet::generate(3, 7);
    let paths = set.paths();
    assert_eq!(paths.len(), 7);
    assert!(paths.windows(2).all(|w| w[0] < w[1]));
    for p in &paths {
        assert!(
            p.starts_with("libs/") || p.starts_with("schematics/") || p.starts_with("boards/"),
            "{p}"
        );
    }
    assert!(SimSet::generate(3, 0).paths().is_empty());
    assert_eq!(SimSet::generate(3, 0).manifest_text(), "");
}

#[test]
fn the_manifest_text_of_a_set_is_the_manifest_of_its_files_on_disk() {
    for seed in 0..6 {
        let set = SimSet::generate(seed, 8);
        let dir = tempfile::tempdir().unwrap();
        set.materialize(dir.path()).unwrap();
        let from_text = Manifest::parse(&set.manifest_text(), &cfg()).unwrap();
        let from_disk = Manifest::from_dir(dir.path(), &cfg()).unwrap();
        assert_eq!(from_text, from_disk, "{seed}");
    }
}

#[test]
fn every_bit_flip_is_found_as_a_mismatch() {
    for seed in 1..=15 {
        let set = SimSet::generate(seed, 5);
        let (a, _b, m) = two_copies(&set);
        let path = set.paths()[seed as usize % 5].clone();
        let mut rng = SimRng::new(seed);
        set.inject(
            a.path(),
            &path,
            eh3d_validation_lab::sim::Fault::BitRot,
            &mut rng,
        )
        .unwrap();
        let report = verify_copy(&m, a.path(), &cfg()).unwrap();
        let bad = report
            .entries
            .iter()
            .filter(|(_, s)| *s == EntryStatus::Mismatch)
            .count();
        assert_eq!(bad, 1, "{seed}");
    }
}

#[test]
fn each_kind_of_fault_is_found_and_the_restore_repairs_it() {
    assert_eq!(ALL_FAULTS.len(), 4);
    for (n, fault) in ALL_FAULTS.iter().enumerate() {
        let set = SimSet::generate(40 + n as u64, 6);
        let (a, _b, m) = two_copies(&set);
        let path = set.paths()[3].clone();
        let mut rng = SimRng::new(9);
        set.inject(a.path(), &path, *fault, &mut rng).unwrap();
        assert!(
            !verify_copy(&m, a.path(), &cfg()).unwrap().is_clean(&cfg()),
            "{fault:?}"
        );
        set.restore(a.path(), &path).unwrap();
        if *fault == eh3d_validation_lab::sim::Fault::Extra {
            fs::remove_file(a.path().join("notes.txt")).unwrap();
        }
        assert!(
            verify_copy(&m, a.path(), &cfg()).unwrap().is_clean(&cfg()),
            "{fault:?}"
        );
    }
}

#[test]
fn restoring_a_path_that_is_not_in_the_set_is_an_error() {
    let set = SimSet::generate(1, 3);
    let dir = tempfile::tempdir().unwrap();
    assert!(set.restore(dir.path(), "not/there.sch").is_err());
}

#[test]
fn the_simulated_tool_is_deterministic_and_its_rate_is_honoured_at_the_extremes() {
    let names = case_names(40);
    assert_eq!(names[0], "case-0000");
    assert_eq!(names[39], "case-0039");
    let sure_pass = SimTool {
        seed: 1,
        fail_per_mille: 0,
    };
    let sure_fail = SimTool {
        seed: 1,
        fail_per_mille: 1000,
    };
    let pass = sure_pass.run(&names);
    let fail = sure_fail.run(&names);
    assert_eq!(pass.len(), 120);
    assert!(pass.iter().all(|r| r.outcome == Outcome::Pass));
    assert!(fail.iter().all(|r| matches!(r.outcome, Outcome::Fail(_))));
    assert_eq!(pass, sure_pass.run(&names));
}

#[test]
fn the_simulated_failures_use_all_three_classes() {
    let names = case_names(100);
    let run = SimTool {
        seed: 9,
        fail_per_mille: 1000,
    }
    .run(&names);
    for class in [
        FailureClass::PhoenixError,
        FailureClass::FormatLimitation,
        FailureClass::FileError,
    ] {
        assert!(
            run.iter().any(|r| r.outcome == Outcome::Fail(class)),
            "{class:?}"
        );
    }
}

#[test]
fn losing_lines_is_exact_at_the_extremes_and_repeatable() {
    let run = SimTool {
        seed: 2,
        fail_per_mille: 100,
    }
    .run(&case_names(30));
    assert_eq!(lose_lines(&run, 5, 0), run);
    assert!(lose_lines(&run, 5, 1000).is_empty());
    assert_eq!(lose_lines(&run, 5, 300), lose_lines(&run, 5, 300));
    assert!(lose_lines(&run, 5, 300).len() < run.len());
}

#[test]
fn a_run_that_loses_lines_fails_the_gate_for_the_missing_cases() {
    let run = SimTool {
        seed: 4,
        fail_per_mille: 50,
    }
    .run(&case_names(30));
    let base = Baseline::from_results(&run, &cfg());
    let lost = lose_lines(&run, 8, 200);
    let report = evaluate(&base, &lost, &cfg());
    assert!(!report.passed);
    assert_eq!(report.missing.len(), run.len() - lost.len());
}

#[test]
fn a_simulated_run_survives_the_text_round_trip_for_many_seeds() {
    for seed in 0..25 {
        let run = SimTool {
            seed,
            fail_per_mille: 300,
        }
        .run(&case_names(20));
        let text = render_results(&run);
        assert_eq!(parse_results(&text, &cfg()).unwrap(), run, "{seed}");
    }
}

// ---------------------------------------------------------------- the corpus

fn xml(version: &str) -> Vec<u8> {
    format!("<?xml version=\"1.0\"?>\n<!DOCTYPE eagle SYSTEM \"eagle.dtd\">\n<eagle version=\"{version}\">\n</eagle>\n").into_bytes()
}

#[test]
fn the_family_of_an_xml_file_is_its_version() {
    assert_eq!(family_of(&xml("9.6.2")), Family::Xml("9.6.2".into()));
    assert_eq!(family_of(&xml("6")), Family::Xml("6".into()));
}

#[test]
fn a_byte_order_mark_and_leading_blanks_do_not_hide_the_family() {
    let mut bom = vec![0xEF, 0xBB, 0xBF];
    bom.extend(xml("7.0.0"));
    assert_eq!(family_of(&bom), Family::Xml("7.0.0".into()));
    let mut blanks = b"  \n\t".to_vec();
    blanks.extend(xml("7.0.0"));
    assert_eq!(family_of(&blanks), Family::Xml("7.0.0".into()));
}

#[test]
fn a_file_that_starts_with_byte_sixteen_is_the_legacy_binary_family() {
    assert_eq!(family_of(&[0x10, 0x80, 0, 0]), Family::LegacyBinary);
    assert_eq!(family_of(&[0xEF, 0xBB, 0xBF, 0x10]), Family::LegacyBinary);
}

#[test]
fn anything_else_is_the_other_family_without_panicking() {
    for head in [
        &b""[..],
        b"hello",
        b"<?xml version=\"1.0\"?><other/>",
        b"<?xml?><eagle version=\"\">",
        b"<?xml?><eagle version=\"abc\">",
        b"<?xml?><eagle version=\".5\">",
        b"<?xml?><eagle version=\"9.6.2",
        b"<?xml?><eagle version=\"1234567890.1234567\">",
        &[0xFF, 0xFE, 0x00],
    ] {
        assert_eq!(family_of(head), Family::Other, "{head:?}");
    }
}

#[test]
fn a_version_of_sixteen_characters_is_the_longest_accepted() {
    let long = "1".repeat(16);
    assert_eq!(family_of(&xml(&long)), Family::Xml(long));
    assert_eq!(family_of(&xml(&"1".repeat(17))), Family::Other);
}

#[test]
fn the_family_labels_and_their_order_are_fixed() {
    assert_eq!(Family::Xml("9.6.2".into()).label(), "xml 9.6.2");
    assert_eq!(Family::LegacyBinary.label(), "legacy-binary");
    assert_eq!(Family::Other.label(), "other");
    assert!(Family::Xml("z".into()) < Family::LegacyBinary);
    assert!(Family::LegacyBinary < Family::Other);
}

#[test]
fn the_kind_comes_from_the_extension_in_any_letter_case_when_it_is_enabled() {
    assert_eq!(Kind::of_extension("LBR", &cfg()), Some(Kind::Library));
    assert_eq!(Kind::of_extension("Sch", &cfg()), Some(Kind::Schematic));
    assert_eq!(Kind::of_extension("brd", &cfg()), Some(Kind::Board));
    assert_eq!(Kind::of_extension("png", &cfg()), None);
    let only = Config {
        corpus_extensions: vec!["brd".into()],
        ..cfg()
    };
    assert_eq!(Kind::of_extension("sch", &only), None);
    assert_eq!(Kind::Library.name(), "library");
    assert_eq!(Kind::Schematic.name(), "schematic");
    assert_eq!(Kind::Board.name(), "board");
}

fn corpus_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("deep/er")).unwrap();
    fs::write(dir.path().join("a.sch"), xml("9.6.2")).unwrap();
    fs::write(dir.path().join("deep/b.brd"), xml("9.6.2")).unwrap();
    fs::write(dir.path().join("deep/er/c.lbr"), [0x10u8, 1, 2, 3]).unwrap();
    fs::write(dir.path().join("deep/er/d.SCH"), b"not xml").unwrap();
    fs::write(dir.path().join("readme.txt"), b"text").unwrap();
    dir
}

#[test]
fn the_inventory_counts_design_files_by_kind_and_family() {
    let dir = corpus_dir();
    let inv = corpus::inventory(dir.path(), &cfg()).unwrap();
    assert_eq!((inv.total_files, inv.design_files), (5, 4));
    assert_eq!(inv.cells.len(), 4);
    let cell = &inv.cells[&(Kind::Schematic, Family::Xml("9.6.2".into()))];
    assert_eq!(cell.files, 1);
    assert_eq!(inv.cells[&(Kind::Library, Family::LegacyBinary)].bytes, 4);
    assert_eq!(inv.cells[&(Kind::Schematic, Family::Other)].files, 1);
    assert_eq!(inv.unreadable, 0);
}

#[test]
fn the_inventory_bytes_add_up_to_the_size_of_the_design_files() {
    let dir = corpus_dir();
    let inv = corpus::inventory(dir.path(), &cfg()).unwrap();
    let total: u64 = inv.cells.values().map(|c| c.bytes).sum();
    assert_eq!(total, inv.design_bytes);
    let expect = (xml("9.6.2").len() * 2 + 4 + 7) as u64;
    assert_eq!(inv.design_bytes, expect);
}

#[test]
fn the_digest_of_a_set_does_not_depend_on_the_names_or_the_folders() {
    let one = tempfile::tempdir().unwrap();
    let two = tempfile::tempdir().unwrap();
    fs::write(one.path().join("a.sch"), b"one").unwrap();
    fs::write(one.path().join("b.sch"), b"two").unwrap();
    fs::create_dir(two.path().join("x")).unwrap();
    fs::write(two.path().join("x/q.sch"), b"two").unwrap();
    fs::write(two.path().join("p.sch"), b"one").unwrap();
    let a = corpus::inventory(one.path(), &cfg()).unwrap();
    let b = corpus::inventory(two.path(), &cfg()).unwrap();
    assert_eq!(a.set_digest, b.set_digest);
    fs::write(two.path().join("p.sch"), b"changed").unwrap();
    assert_ne!(
        a.set_digest,
        corpus::inventory(two.path(), &cfg()).unwrap().set_digest
    );
}

#[test]
fn the_inventory_of_an_empty_folder_is_empty_and_has_the_digest_of_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let inv = corpus::inventory(dir.path(), &cfg()).unwrap();
    assert_eq!(
        (inv.total_files, inv.design_files, inv.design_bytes),
        (0, 0, 0)
    );
    assert!(inv.cells.is_empty());
    assert_eq!(inv.set_digest, EMPTY_SHA);
}

#[test]
fn a_missing_folder_is_counted_as_not_readable_and_is_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let inv = corpus::inventory(&dir.path().join("nope"), &cfg()).unwrap();
    assert_eq!(inv.design_files, 0);
    assert_eq!(inv.skipped_unsupported, 1);
}

#[test]
fn the_inventory_is_limited_in_files_at_the_exact_edge() {
    let dir = corpus_dir();
    let four = Config {
        max_entries: 4,
        ..cfg()
    };
    let three = Config {
        max_entries: 3,
        ..cfg()
    };
    assert!(corpus::inventory(dir.path(), &four).is_ok());
    assert!(matches!(
        corpus::inventory(dir.path(), &three),
        Err(LabError::TooLarge { .. })
    ));
}

#[test]
fn a_short_head_still_gives_the_right_hash_and_size() {
    let dir = corpus_dir();
    let short = Config {
        corpus_head_bytes: 16,
        ..cfg()
    };
    let a = corpus::inventory(dir.path(), &short).unwrap();
    let b = corpus::inventory(dir.path(), &cfg()).unwrap();
    assert_eq!(a.set_digest, b.set_digest);
    assert_eq!(a.design_bytes, b.design_bytes);
}

#[test]
fn the_inventory_json_has_the_stable_keys() {
    let dir = corpus_dir();
    let text = corpus::inventory(dir.path(), &cfg())
        .unwrap()
        .to_json()
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    for key in [
        "total_files",
        "design_files",
        "design_bytes",
        "skipped_unsupported",
        "skipped_path",
        "unreadable",
        "cells",
        "set_digest",
    ] {
        assert!(v.get(key).is_some(), "{key}");
    }
    assert!(v["cells"][0].get("level").is_some() && v["cells"][0].get("family").is_some());
}

#[test]
fn a_copy_of_the_corpus_has_the_same_digest_and_a_manifest_that_verifies() {
    let src = corpus_dir();
    let work = tempfile::tempdir().unwrap();
    let dst = work.path().join("copy");
    let report = corpus::copy(src.path(), &dst, &cfg()).unwrap();
    assert_eq!(report.copied, 4);
    let inv = corpus::inventory(src.path(), &cfg()).unwrap();
    assert_eq!(report.set_digest, inv.set_digest);
    assert_eq!(report.bytes, inv.design_bytes);
    let text = fs::read_to_string(dst.join("SHA256SUMS")).unwrap();
    assert_eq!(report.manifest_sha256, sha256_hex(text.as_bytes()));
    let m = Manifest::parse(&text, &cfg()).unwrap();
    let r = verify_copy(&m, &dst, &cfg()).unwrap();
    assert!(r.is_clean(&cfg()));
    assert!(!dst.join("readme.txt").exists());
}

#[test]
fn a_copy_into_the_original_around_it_or_into_a_full_folder_is_refused() {
    let src = corpus_dir();
    let inside = src.path().join("inside");
    assert!(matches!(
        corpus::copy(src.path(), &inside, &cfg()),
        Err(LabError::UnsafeTarget)
    ));
    assert!(!inside.exists());
    let parent = src.path().parent().unwrap().to_path_buf();
    assert!(matches!(
        corpus::copy(src.path(), &parent, &cfg()),
        Err(LabError::UnsafeTarget)
    ));
    let full = tempfile::tempdir().unwrap();
    fs::write(full.path().join("x"), b"x").unwrap();
    assert!(matches!(
        corpus::copy(src.path(), full.path(), &cfg()),
        Err(LabError::UnsafeTarget)
    ));
    assert!(matches!(
        corpus::copy(src.path(), src.path(), &cfg()),
        Err(LabError::UnsafeTarget)
    ));
}

#[test]
fn a_copy_of_a_missing_folder_is_an_io_error_and_leaves_nothing() {
    let work = tempfile::tempdir().unwrap();
    let dst = work.path().join("copy");
    let e = corpus::copy(&work.path().join("nope"), &dst, &cfg()).unwrap_err();
    assert!(matches!(e, LabError::Io { .. }));
    assert!(!dst.exists());
}

#[test]
fn a_copy_into_an_empty_existing_folder_is_allowed() {
    let src = corpus_dir();
    let dst = tempfile::tempdir().unwrap();
    assert_eq!(
        corpus::copy(src.path(), dst.path(), &cfg()).unwrap().copied,
        4
    );
}

// ---------------------------------------------------------------- the command line

#[test]
fn the_manifest_command_prints_the_rendered_manifest() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), b"abc").unwrap();
    let (code, out, _) = call(&["manifest", &path_str(dir.path())]);
    assert_eq!(code, 0);
    assert_eq!(out, format!("{ABC_SHA}  a.txt\n"));
}

#[test]
fn the_manifest_command_on_a_missing_folder_exits_2_with_an_error_line() {
    let dir = tempfile::tempdir().unwrap();
    let (code, out, err) = call(&["manifest", &path_str(&dir.path().join("nope"))]);
    assert_eq!(code, 2);
    assert!(out.is_empty() && err.starts_with("error: cannot read "));
}

#[test]
fn verify_says_pass_for_clean_copies_and_hides_the_paths_of_failures_unless_asked() {
    let set = SimSet::generate(21, 5);
    let (a, b, m) = two_copies(&set);
    let manifest = a.path().parent().unwrap().join("dd_verify_manifest.txt");
    let manifest_file = tempfile::NamedTempFile::new_in(a.path().parent().unwrap()).unwrap();
    drop(manifest);
    fs::write(manifest_file.path(), m.render()).unwrap();
    let mf = path_str(manifest_file.path());
    let (code, out, _) = call(&["verify", &mf, &path_str(a.path()), &path_str(b.path())]);
    assert_eq!(code, 0);
    assert!(out.contains("copy 1: 5 listed, 0 not intact, 0 extra"));
    assert!(out.trim_end().ends_with("PASS (SHA256SUMS)"));
    let victim = set.paths()[1].clone();
    fs::remove_file(b.path().join(&victim)).unwrap();
    let (code, out, _) = call(&["verify", &mf, &path_str(a.path()), &path_str(b.path())]);
    assert_eq!(code, 1);
    assert!(out.contains("Missing ") && !out.contains(&victim));
    assert!(out.contains(&opaque_id("", &victim)));
    let (_, out, _) = call(&[
        "verify",
        "--show-paths",
        &mf,
        &path_str(a.path()),
        &path_str(b.path()),
    ]);
    assert!(out.contains(&format!("Missing {victim}")));
    assert!(out.trim_end().ends_with("FAIL (SHA256SUMS)"));
}

#[test]
fn verify_with_one_copy_says_how_many_are_required() {
    let set = SimSet::generate(22, 3);
    let (a, _b, m) = two_copies(&set);
    let file = tempfile::NamedTempFile::new().unwrap();
    fs::write(file.path(), m.render()).unwrap();
    let (code, out, _) = call(&["verify", &path_str(file.path()), &path_str(a.path())]);
    assert_eq!(code, 1);
    assert!(out.contains("only 1 copies given, 2 required"));
}

#[test]
fn verify_with_the_same_copy_twice_exits_2() {
    let set = SimSet::generate(23, 3);
    let (a, _b, m) = two_copies(&set);
    let file = tempfile::NamedTempFile::new().unwrap();
    fs::write(file.path(), m.render()).unwrap();
    let (code, _, err) = call(&[
        "verify",
        &path_str(file.path()),
        &path_str(a.path()),
        &path_str(a.path()),
    ]);
    assert_eq!(code, 2);
    assert!(err.contains("the same copy was given more than once"));
}

#[test]
fn baseline_refuses_a_results_file_without_cases_and_exits_1() {
    let file = tempfile::NamedTempFile::new().unwrap();
    fs::write(file.path(), "# nothing\n").unwrap();
    let (code, out, _) = call(&["baseline", &path_str(file.path())]);
    assert_eq!(code, 1);
    assert!(out.starts_with("# refused:"));
}

#[test]
fn baseline_and_gate_work_together_through_files() {
    let dir = tempfile::tempdir().unwrap();
    let results = dir.path().join("r.tsv");
    fs::write(&results, "a\tL0\tpass\nb\tL0\tpass\n").unwrap();
    let (code, json, _) = call(&["baseline", &path_str(&results)]);
    assert_eq!(code, 0);
    let base = dir.path().join("b.json");
    fs::write(&base, json).unwrap();
    assert_eq!(call(&["gate", &path_str(&base), &path_str(&results)]).0, 0);
    let worse = dir.path().join("w.tsv");
    fs::write(&worse, "a\tL0\tpass\nb\tL0\tfail:file-error\n").unwrap();
    let (code, out, _) = call(&["gate", &path_str(&base), &path_str(&worse)]);
    assert_eq!(code, 1);
    assert!(out.contains("\"passed\": false"));
}

#[test]
fn gate_with_a_damaged_baseline_or_results_exits_2() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("b.json");
    fs::write(&bad, "{").unwrap();
    let r = dir.path().join("r.tsv");
    fs::write(&r, "a\tL0\tpass\n").unwrap();
    let (code, _, err) = call(&["gate", &path_str(&bad), &path_str(&r)]);
    assert_eq!(code, 2);
    assert!(err.contains("the baseline cannot be read"));
    let bad_results = dir.path().join("x.tsv");
    fs::write(&bad_results, "a\tL0\tfail\n").unwrap();
    let ok_base = dir.path().join("ok.json");
    fs::write(
        &ok_base,
        Baseline::from_results(&[], &cfg()).to_json().unwrap(),
    )
    .unwrap();
    let (code, _, err) = call(&["gate", &path_str(&ok_base), &path_str(&bad_results)]);
    assert_eq!(code, 2);
    assert!(err.contains("a failure must carry a class"));
}

#[test]
fn oracle_prints_results_then_the_summary_and_exits_by_the_differences() {
    let dir = tempfile::tempdir().unwrap();
    let g = dir.path().join("g.tsv");
    let o = dir.path().join("o.tsv");
    fs::write(&g, "c\tL0:a\t1\n").unwrap();
    fs::write(&o, "c\tL0:a\t1\n").unwrap();
    let (code, out, _) = call(&["oracle", &path_str(&g), &path_str(&o)]);
    assert_eq!(code, 0);
    assert_eq!(out, "c\tL0\tpass\n# agreed 1 differed 0 extra 0\n");
    fs::write(&o, "c\tL0:a\t2\tformat-limitation\n").unwrap();
    let (code, out, _) = call(&["oracle", &path_str(&g), &path_str(&o)]);
    assert_eq!(code, 1);
    assert_eq!(
        out,
        "c\tL0\tfail:format-limitation\n# agreed 0 differed 1 extra 0\n"
    );
}

#[test]
fn oracle_refuses_an_empty_golden_file() {
    let dir = tempfile::tempdir().unwrap();
    let g = dir.path().join("g.tsv");
    fs::write(&g, "\n").unwrap();
    let (code, out, _) = call(&["oracle", &path_str(&g), &path_str(&g)]);
    assert_eq!(code, 1);
    assert!(out.starts_with("# refused: the golden file has no cases"));
}

#[test]
fn corpus_inventory_and_copy_print_json_that_can_be_read() {
    let src = corpus_dir();
    let (code, out, _) = call(&["corpus-inventory", &path_str(src.path())]);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["design_files"], 4);
    let work = tempfile::tempdir().unwrap();
    let dst = path_str(&work.path().join("c"));
    let (code, out, _) = call(&["corpus-copy", &path_str(src.path()), &dst]);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["copied"], 4);
    for key in [
        "bytes",
        "skipped_path",
        "skipped_duplicate",
        "skipped_unsupported",
        "manifest_sha256",
        "set_digest",
    ] {
        assert!(v.get(key).is_some(), "{key}");
    }
    let (code, _, err) = call(&["corpus-copy", &path_str(src.path()), &dst]);
    assert_eq!(code, 2);
    assert!(err.contains("the target of the copy is inside the original"));
}

#[test]
fn a_settings_file_changes_the_commands_and_a_bad_one_exits_2() {
    let dir = tempfile::tempdir().unwrap();
    let r = dir.path().join("r.tsv");
    fs::write(&r, "a\tL0\tpass\n").unwrap();
    let c = dir.path().join("s.cfg");
    fs::write(&c, "max_results_bytes = 3\n").unwrap();
    let (code, _, err) = call(&["--config", &path_str(&c), "baseline", &path_str(&r)]);
    assert_eq!(code, 2);
    assert!(err.contains("limit exceeded: input file"));
    fs::write(&c, "nonsense\n").unwrap();
    let (code, _, err) = call(&["--config", &path_str(&c), "baseline", &path_str(&r)]);
    assert_eq!(code, 2);
    assert!(err.contains("configuration line 1"));
    assert_eq!(call(&["--config", "x"]).0, 2);
}

#[test]
fn a_command_with_extra_arguments_prints_the_usage_and_exits_2() {
    for list in [
        &["manifest", "a", "b"][..],
        &["baseline"],
        &["gate", "x"],
        &["oracle", "x"],
        &["corpus-copy", "x"],
        &["corpus-inventory"],
    ] {
        let (code, out, err) = call(list);
        assert_eq!(code, 2, "{list:?}");
        assert!(out.is_empty() && err.contains("Reports show only hashes"));
    }
}

// Copyright (c) 2024, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Source-derived boundary cases; no Java runtime/oracle outputs are claimed.
use syson_domain::names::*;

const JAVA_STRIP: &[char] = &[
    '\u{9}', '\u{a}', '\u{b}', '\u{c}', '\u{d}', '\u{1c}', '\u{1d}', '\u{1e}', '\u{1f}', '\u{20}',
    '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}',
    '\u{2008}', '\u{2009}', '\u{200a}', '\u{2028}', '\u{2029}', '\u{205f}', '\u{3000}',
];

#[test]
fn java_strip_accepts_its_25_whitespace_scalars_on_both_operands() {
    assert_eq!(JAVA_STRIP.len(), 25);
    for &space in JAVA_STRIP {
        let padded = format!("{space}λ😀{space}");
        assert!(
            name_matches(Some(&padded), Some("λ😀")),
            "U+{:04X}",
            space as u32
        );
        assert!(
            name_matches(Some("λ😀"), Some(&padded)),
            "U+{:04X}",
            space as u32
        );
        assert!(name_matches(Some(&space.to_string()), Some("")));
    }
}

#[test]
fn java_strip_excludes_nonbreaking_spaces_nel_bom_and_neighbors() {
    for space in [
        '\u{0}',
        '\u{8}',
        '\u{e}',
        '\u{1b}',
        '\u{7f}',
        '\u{85}',
        '\u{a0}',
        '\u{167f}',
        '\u{1681}',
        '\u{180e}',
        '\u{2007}',
        '\u{200b}',
        '\u{2027}',
        '\u{202a}',
        '\u{202f}',
        '\u{205e}',
        '\u{2060}',
        '\u{2fff}',
        '\u{3001}',
        '\u{feff}',
        '\u{1f600}',
    ] {
        let padded = format!("{space}λ😀{space}");
        assert!(
            !name_matches(Some(&padded), Some("λ😀")),
            "U+{:04X}",
            space as u32
        );
        assert!(
            !name_matches(Some("λ😀"), Some(&padded)),
            "U+{:04X}",
            space as u32
        );
    }
}

#[test]
fn quote_detection_precedes_stripping_and_is_not_symmetric() {
    assert!(name_matches(Some("λ😀"), Some("'λ😀'")));
    assert!(!name_matches(Some("'λ😀'"), Some("λ😀")));
    assert!(name_matches(Some("'λ😀'"), Some("'λ😀'")));
    for &space in JAVA_STRIP {
        let padded_query = format!("{space}'λ😀'{space}");
        assert!(!name_matches(Some("λ😀"), Some(&padded_query)));
        let padded_candidate = format!("{space}λ😀{space}");
        assert!(!name_matches(Some(&padded_candidate), Some("'λ😀'")));
        let same_inside = format!("'{padded_candidate}'");
        assert!(name_matches(Some(&padded_candidate), Some(&same_inside)));
    }
}

#[test]
fn empty_null_and_single_quote_combinations_are_distinct() {
    for (candidate, query, expected) in [
        (None, None, false),
        (None, Some(""), false),
        (Some(""), None, false),
        (Some(""), Some(""), true),
        (Some(""), Some("''"), true),
        (Some("''"), Some(""), false),
        (Some(""), Some("'"), false),
        (Some("'"), Some("'"), true),
        (Some("'"), Some("'''"), true),
        (Some("''"), Some("''''"), true),
    ] {
        assert_eq!(
            name_matches(candidate, query),
            expected,
            "{candidate:?} / {query:?}"
        );
    }
}

#[test]
fn names_remain_case_and_normalization_sensitive() {
    for (a, b) in [
        ("é", "e\u{301}"),
        ("λ", "Λ"),
        ("Vehicle", "vehicle"),
        ("A", "Α"),
        ("😀", "😁"),
    ] {
        assert!(!name_matches(Some(a), Some(b)));
        assert!(!name_matches(Some(a), Some(&format!("'{b}'"))));
        let mut filter = NameConflictingFilter::default();
        assert!(filter.check_name(Some(a)));
        assert!(filter.check_name(Some(b)));
        assert!(!filter.check_name(Some(a)));
    }
}

#[test]
fn literal_qualified_split_preserves_unicode_and_splits_inside_quotes() {
    for (name, expected) in [
        ("😀::'é::λ'::", vec!["😀", "'é", "λ'"]),
        ("::😀::::λ::", vec!["", "😀", "", "λ"]),
        (":::😀", vec!["", ":😀"]),
        ("::::", vec![]),
        ("😀::\u{3000}", vec!["😀", "\u{3000}"]),
        ("'::'", vec!["'", "'"]),
        ("", vec![""]),
    ] {
        assert_eq!(parse_qualified_name(name), expected);
    }
}

#[test]
fn unescape_removes_only_a_pair_of_outer_ascii_quotes() {
    for (raw, expected) in [
        ("'😀'", "😀"),
        ("'e\u{301}'", "e\u{301}"),
        ("'a\\'b'", "a\\'b"),
        ("'a''b'", "a''b"),
        ("'\\u00E9'", "\\u00E9"),
        ("‘é’", "‘é’"),
        ("'é", "'é"),
        ("é'", "é'"),
        ("'", "'"),
        ("''", ""),
        ("", ""),
    ] {
        let actual = unescape_string(Some(raw)).unwrap();
        assert_eq!(actual, expected);
        assert!(
            actual.as_ptr() >= raw.as_ptr()
                && actual.as_ptr() <= raw.as_ptr().wrapping_add(raw.len())
        );
    }
    assert_eq!(unescape_string(None), None);
}

#[test]
fn printable_names_use_ascii_identifier_rules_not_unicode_categories() {
    for raw in [
        "é",
        "λ",
        "😀",
        "e\u{301}",
        "１",
        "1x",
        "a b",
        "a::b",
        "'x'",
        "a-b",
        "a\u{2007}b",
    ] {
        assert_eq!(printable_name(Some(raw), false), format!("'{raw}'"));
    }
    for raw in ["A", "z", "_", "_9", "A09_z", "Package", "PART"] {
        assert_eq!(printable_name(Some(raw), false), raw);
    }
    assert_eq!(printable_name(None, false), "");
    assert_eq!(printable_name(Some(""), true), "");
    assert_eq!(printable_name(Some("part"), false), "'part'");
}

#[test]
fn quote_escaping_does_not_claim_printable_match_roundtrip() {
    let declared = "a'😀";
    let escaped = printable_name(Some(declared), true);
    assert_eq!(escaped, "'a\\'😀'");
    assert!(!name_matches(Some(declared), Some(&escaped)));
    assert_eq!(unescape_string(Some(&escaped)), Some("a\\'😀"));
    let unescaped = printable_name(Some(declared), false);
    assert!(name_matches(Some(declared), Some(&unescaped)));
    assert_eq!(printable_name(Some("a\\'b"), true), "'a\\\\'b'");
}

#[test]
fn java_escaping_preserves_del_and_uses_uppercase_utf16_escapes() {
    for (raw, expected) in [
        (
            "\u{0}\u{1}\u{7}\u{b}\u{1f}",
            "\\u0000\\u0001\\u0007\\u000B\\u001F",
        ),
        ("\u{8}\t\n\u{c}\r", "\\b\\t\\n\\f\\r"),
        (" \u{7e}\u{7f}", " \u{7e}\u{7f}"),
        (
            "\u{80}\u{85}\u{a0}\u{2007}\u{202f}",
            "\\u0080\\u0085\\u00A0\\u2007\\u202F",
        ),
        ("\"'\\/", "\\\"'\\\\/"),
        (
            "éλ😀\u{1d11e}\u{10ffff}",
            "\\u00E9\\u03BB\\uD83D\\uDE00\\uD834\\uDD1E\\uDBFF\\uDFFF",
        ),
    ] {
        assert_eq!(escape_string(Some(raw)).as_deref(), Some(expected));
    }
    assert_eq!(escape_string(None), None);
    assert_eq!(escape_string(Some("")), Some(String::new()));
}

#[test]
fn escaped_ascii_text_is_literal_input_not_a_decoder_request() {
    let escaped = escape_string(Some("😀\n")).unwrap();
    assert_eq!(escaped, "\\uD83D\\uDE00\\n");
    assert_eq!(
        escape_string(Some(&escaped)).unwrap(),
        "\\\\uD83D\\\\uDE00\\\\n"
    );
    assert_eq!(unescape_string(Some(&escaped)), Some(escaped.as_str()));
}

#[test]
fn alias_reference_identity_controls_short_name_reservation() {
    let alias = String::from("😀");
    let member = String::from("😀");
    assert_eq!(alias, member);
    assert_ne!(alias.as_ptr(), member.as_ptr());
    let mut distinct = NameConflictingFilter::default();
    assert!(distinct.check_membership(Some(&alias), true, Some((Some(&member), Some("λ")))));
    assert!(distinct.check_name(Some("λ")));
    assert!(!distinct.check_name(Some("😀")));
    let mut same = NameConflictingFilter::default();
    assert!(same.check_membership(Some(&member), false, Some((Some(&member), Some("λ")))));
    assert!(!same.check_name(Some("λ")));
    assert!(!same.check_name(Some("😀")));
}

#[test]
fn orphan_alias_and_null_names_reserve_nothing_but_empty_name_does() {
    let mut filter = NameConflictingFilter::default();
    assert!(filter.check_membership(Some("孤"), true, None));
    assert!(filter.check_name(Some("孤")));
    assert!(filter.check_element(None, None));
    assert!(filter.check_name(None));
    assert!(filter.check_name(None));
    assert!(filter.check_name(Some("")));
    assert!(!filter.check_name(Some("")));
}

#[test]
fn conflict_state_is_exact_unicode_text_with_no_strip_or_quote_normalization() {
    let mut filter = NameConflictingFilter::default();
    for name in ["λ", "'λ'", " λ ", "\u{1c}λ", "λ\u{a0}", "Λ"] {
        assert!(filter.check_name(Some(name)));
        assert!(!filter.check_name(Some(name)));
    }
}

#[test]
fn failed_short_name_keeps_long_reservation_and_failed_long_skips_short() {
    let mut filter = NameConflictingFilter::default();
    assert!(filter.check_name(Some("短")));
    assert!(!filter.check_element(Some("長😀"), Some("短")));
    assert!(!filter.check_name(Some("長😀")));
    assert!(!filter.check_element(Some("長😀"), Some("unusedλ")));
    assert!(filter.check_name(Some("unusedλ")));
    assert!(!filter.check_element(Some("same"), Some("same")));
    assert!(!filter.check_name(Some("same")));
}

#[test]
fn prefill_reserves_exact_effective_names_without_alias_processing() {
    let mut filter = NameConflictingFilter::default();
    filter.fill_used_names([
        (Some("é"), Some("e\u{301}")),
        (None, Some("😀")),
        (Some("é"), None),
    ]);
    for name in ["é", "e\u{301}", "😀"] {
        assert!(!filter.check_name(Some(name)));
    }
    assert!(filter.check_name(Some("'é'")));
    assert!(filter.check_name(Some("alias")));
}

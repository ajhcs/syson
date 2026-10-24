// Copyright (c) 2024, 2026 Obeo.
// This program and accompanying materials are made available under the terms
// of the Eclipse Public License v2.0, https://www.eclipse.org/legal/epl-2.0/.
// SPDX-License-Identifier: EPL-2.0
// Contributors: Obeo - initial API and implementation
// Rust translation and compatibility tests: SysON Rust port contributors.

use syson_domain::{model::*, names::*, uuid::*};

// Four exact examples transcribed from upstream NameHelperTest.testEscapeSimpleQuotes.
#[test]
fn upstream_quote_regression() {
    for (name, escape, expected) in [
        ("Hello what's up!", true, "'Hello what\\'s up!'"),
        ("Hello what's up!", false, "'Hello what's up!'"),
        ("Hel'lo", true, "'Hel\\'lo'"),
        ("Hel'lo", false, "'Hel'lo'"),
    ] {
        assert_eq!(printable_name(Some(name), escape), expected);
    }
}
#[test]
fn every_pinned_keyword_is_quoted() {
    assert_eq!(KEYWORDS.len(), 117);
    for word in KEYWORDS {
        assert_eq!(printable_name(Some(word), true), format!("'{word}'"));
    }
}
#[test]
fn identifier_edges() {
    for (input, expected) in [
        (None, ""),
        (Some(""), ""),
        (Some("9name"), "'9name'"),
        (Some("_9"), "_9"),
        (Some("Part"), "Part"),
        (Some("é"), "'é'"),
        (Some("name::x"), "'name::x'"),
        (Some(" "), "' '"),
    ] {
        assert_eq!(printable_name(input, true), expected);
    }
}
#[test]
fn java_split_edges_are_preserved() {
    for (input, expected) in [
        ("", vec![""]),
        ("::", vec![]),
        ("a::", vec!["a"]),
        ("::a", vec!["", "a"]),
        ("a::::b", vec!["a", "", "b"]),
        ("'a::b'", vec!["'a", "b'"]),
    ] {
        assert_eq!(parse_qualified_name(input), expected);
    }
}
#[test]
fn unescape_is_outer_quote_removal_only() {
    assert_eq!(unescape_string(None), None);
    assert_eq!(unescape_string(Some("'")), Some("'"));
    assert_eq!(unescape_string(Some("''")), Some(""));
    assert_eq!(unescape_string(Some("'a\\'b'")), Some("a\\'b"));
}
#[test]
fn java_utf16_escape_compatibility() {
    assert_eq!(escape_string(None), None);
    assert_eq!(
        escape_string(Some("\n\t\r\u{8}\u{c}\\\"'é😀\u{7f}")),
        Some("\\n\\t\\r\\b\\f\\\\\\\"'\\u00E9\\uD83D\\uDE00\u{7f}".to_owned())
    );
}
#[test]
fn names_use_java_strip_and_optional_quote_rules() {
    assert!(name_matches(Some(" name "), Some("name")));
    assert!(name_matches(Some("name"), Some("'name'")));
    assert!(!name_matches(Some("name"), Some(" 'name' ")));
    assert!(!name_matches(None, Some("name")));
    for whitespace in ['\u{a0}', '\u{2007}', '\u{202f}', '\u{85}'] {
        assert!(!name_matches(
            Some(&format!("{whitespace}name")),
            Some("name")
        ));
    }
    assert!(name_matches(Some("\u{1c}name"), Some("name")));
}
#[test]
fn conflict_short_circuits_and_keeps_successful_name_reservation() {
    let mut filter = NameConflictingFilter::default();
    assert!(filter.check_name(Some("short")));
    assert!(!filter.check_element(Some("long"), Some("short")));
    assert!(!filter.check_name(Some("long")));
    assert!(!filter.check_element(Some("long"), Some("unused")));
    assert!(filter.check_name(Some("unused")));
    assert!(filter.check_element(None, None));
    assert!(filter.check_name(Some("")));
    assert!(!filter.check_name(Some("")));
}
#[test]
fn conflict_uses_alias_identity_as_upstream_does() {
    let mut filter = NameConflictingFilter::default();
    assert!(filter.check_membership(Some("alias"), true, Some((Some("long"), Some("short")))));
    assert!(filter.check_name(Some("long")));
    assert!(!filter.check_name(Some("alias")));
    assert!(filter.check_membership(Some("orphan"), true, None));
    assert!(filter.check_name(Some("orphan")));
}
#[test]
fn existing_member_fill_reserves_both_names_without_aliases() {
    let mut filter = NameConflictingFilter::default();
    filter.fill_used_names([(Some("a"), Some("a")), (None, Some("s"))]);
    assert!(!filter.check_name(Some("a")));
    assert!(!filter.check_name(Some("s")));
}
// Seven exact IDs transcribed from ElementUtilTest, including nested library elements.
#[test]
fn upstream_library_uuid_fixtures() {
    let cases = [
        (
            KERML_LIBRARY_BASE_URI,
            "Base",
            "cdd5d1e3-fe4b-52bd-8a01-51a53f22ba47",
        ),
        (
            KERML_LIBRARY_BASE_URI,
            "ScalarValues",
            "40bb440c-5036-58e1-8675-5afccb8b8f1d",
        ),
        (
            SYSML_LIBRARY_BASE_URI,
            "ISQ",
            "3fcc2ef4-a31c-522b-b69d-717d74bccfa6",
        ),
        (
            SYSML_LIBRARY_BASE_URI,
            "ISQBase",
            "c42ede81-a2db-50ca-9727-40dc60310326",
        ),
    ];
    let mut ids = Vec::new();
    for (base, name, expected) in cases {
        let id = uuid_v5(NAMESPACE_URL, &format!("{base}{name}"));
        assert_eq!(id.to_string(), expected);
        ids.push(id);
    }
    assert_eq!(
        uuid_v5(ids[1], "ScalarValues::Boolean").to_string(),
        "d1e9242d-b2e3-5270-bf69-4f4fb0447193"
    );
    assert_eq!(
        uuid_v5(ids[3], "ISQBase::mass").to_string(),
        "9af08108-292b-534a-a527-068c17423dfd"
    );
    assert_eq!(
        uuid_v5(ids[0], "Base::Anything").to_string(),
        "d5b4e7df-e644-5f2f-b95e-cf6f1f6c076d"
    );
}
#[test]
fn long_and_unicode_uuid_name_sha1_blocks() {
    // Additional deterministic fixtures generated independently with Python uuid.uuid5.
    assert_eq!(
        uuid_v5(NAMESPACE_URL, &"x".repeat(200)).to_string(),
        "cd3580b9-d244-586a-8077-74317319b48d"
    );
    assert_eq!(
        uuid_v5(NAMESPACE_URL, "é😀").to_string(),
        "570461e2-0c94-561f-9e1e-8646068fd096"
    );
}
#[test]
fn standard_library_uri_prefix_quirk() {
    assert!(is_standard_library_resource(Some("sysmllibrary-other")));
    assert!(!is_standard_library_resource(Some("SYSMLLIBRARY:foo")));
    assert!(!is_standard_library_resource(None));
    assert_eq!(
        standard_library_uuid("kermllibrary:/base", "Base").to_string(),
        "cdd5d1e3-fe4b-52bd-8a01-51a53f22ba47"
    );
}
fn namespace(name: Option<&str>) -> Element {
    let mut e = Element::new("Namespace", name);
    e.is_namespace = true;
    e
}
fn any(_: &Element) -> bool {
    true
}
#[test]
fn qualified_name_uses_actual_owner_behavior() {
    let mut m = Model::default();
    let root = m.insert(namespace(None), None).unwrap();
    let pkg = m.insert(namespace(Some("Package")), Some(root)).unwrap();
    let mut member = Element::new("OwningMembership", Some("ignored"));
    member.is_membership = true;
    let member = m.insert(member, Some(pkg)).unwrap();
    let part = m
        .insert(Element::new("PartUsage", Some("part")), Some(member))
        .unwrap();
    assert_eq!(
        m.qualified_name(part).unwrap(),
        Some("Package::'part'".to_owned())
    );
    assert_eq!(m.qualified_name(root).unwrap(), None);
}
#[test]
fn qualified_name_does_not_fall_back_to_short_name() {
    let mut m = Model::default();
    let mut e = Element::new("Element", None);
    e.declared_short_name = Some("short".to_owned());
    assert_eq!(e.escaped_name(), "short");
    let id = m.insert(e, None).unwrap();
    assert_eq!(m.qualified_name(id).unwrap(), None);
}
#[test]
fn qualified_lookup_ignores_unnamed_root_and_accepts_short_names() {
    let mut m = Model::default();
    let root = m.insert(namespace(None), None).unwrap();
    let pkg = m.insert(namespace(Some("Pkg")), Some(root)).unwrap();
    let mut part = Element::new("PartUsage", Some("A long name"));
    part.declared_short_name = Some("s".to_owned());
    let part = m.insert(part, Some(pkg)).unwrap();
    assert_eq!(
        m.find_by_name_and_type("Pkg::'A long name'", any).unwrap(),
        Some(part)
    );
    assert_eq!(m.find_by_name_and_type("Pkg::s", any).unwrap(), Some(part));
    assert_eq!(
        m.find_by_name_and_type("Pkg::s", |e| e.class == "PortUsage")
            .unwrap(),
        None
    );
}
#[test]
fn qualified_lookup_first_name_wins_even_if_type_does_not() {
    let mut m = Model::default();
    let root = m.insert(namespace(Some("Pkg")), None).unwrap();
    m.insert(Element::new("PortUsage", Some("same")), Some(root))
        .unwrap();
    let part = m
        .insert(Element::new("PartUsage", Some("same")), Some(root))
        .unwrap();
    assert_eq!(
        m.find_by_name_and_type("Pkg::same", |e| e.class == "PartUsage")
            .unwrap(),
        None
    );
    assert_eq!(
        m.find_by_name_and_type("same", |e| e.class == "PartUsage")
            .unwrap(),
        Some(part)
    );
}
#[test]
fn public_import_is_fallback_only_for_namespaces() {
    let mut m = Model::default();
    let remote = m
        .insert(Element::new("PartUsage", Some("remote")), None)
        .unwrap();
    let mut pkg = namespace(Some("Pkg"));
    pkg.imports.push(ImportedMembership {
        visibility: Visibility::Private,
        related: vec![remote],
    });
    let private = m.insert(pkg, None).unwrap();
    assert_eq!(m.find_by_name_and_type("Pkg::remote", any).unwrap(), None);
    let mut pkg = namespace(Some("Public"));
    pkg.imports.push(ImportedMembership {
        visibility: Visibility::Public,
        related: vec![remote],
    });
    let public = m.insert(pkg, None).unwrap();
    assert_eq!(
        m.find_by_name_and_type("Public::remote", any).unwrap(),
        Some(remote)
    );
    assert_ne!(private, public);
}
#[test]
fn absent_owner_is_explicit_and_non_mutating() {
    let mut m = Model::default();
    assert_eq!(
        m.insert(Element::new("Element", None), Some(ElementId(99))),
        Err(ModelError::MissingElement(ElementId(99)))
    );
    assert_eq!(
        m.insert(Element::new("Element", None), None).unwrap(),
        ElementId(0)
    );
}

#[test]
fn membership_projection_lookup_and_owned_precedence() {
    let mut m = Model::default();
    let remote = m.insert(Element::new("Remote", Some("x")), None).unwrap();
    let mut pkg = namespace(Some("Pkg"));
    pkg.imports.push(ImportedMembership {
        visibility: Visibility::Public,
        related: vec![remote],
    });
    let pkg = m.insert(pkg, None).unwrap();
    let mut membership = Element::new("OwningMembership", None);
    membership.is_membership = true;
    let membership = m.insert(membership, Some(pkg)).unwrap();
    let owned = m
        .insert(Element::new("Owned", Some("x")), Some(membership))
        .unwrap();
    assert_eq!(m.find_by_name_and_type("Pkg::x", any).unwrap(), Some(owned));
    assert_eq!(
        m.find_by_name_and_type("Pkg::x", |e| e.class == "Remote")
            .unwrap(),
        None
    );
}

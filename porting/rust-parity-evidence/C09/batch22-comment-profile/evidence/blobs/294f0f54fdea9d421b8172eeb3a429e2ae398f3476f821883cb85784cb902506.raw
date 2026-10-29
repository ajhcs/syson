// SPDX-License-Identifier: EPL-2.0
use std::sync::{Arc, OnceLock, atomic::AtomicBool};
use syson_ast::{Arena, Lexeme, Node, Slot, Value};
use syson_ast_adapter::{Adapter, Error, Limits};

fn construct(source: &str, limits: Limits) -> Result<Arena, Error> {
    static ADAPTER: OnceLock<Adapter> = OnceLock::new();
    ADAPTER
        .get_or_init(|| Adapter::pinned().unwrap())
        .construct(source, limits, Arc::new(AtomicBool::new(false)))
}
fn build(source: &str) -> Arena {
    construct(source, Limits::default()).unwrap()
}
fn declarations(arena: &Arena) -> Vec<&Node> {
    arena
        .nodes()
        .values()
        .filter(|node| {
            matches!(
                node.kind(),
                Some("Package" | "PartDefinition" | "PartUsage")
            )
        })
        .collect()
}
fn name(node: &Node) -> &syson_ast::Datum {
    match node.slot("declaredName") {
        Slot::Present(value) => value,
        Slot::Absent => panic!("missing name"),
    }
}
fn assert_span(source: &str, lexeme: &Lexeme) {
    assert_eq!(&source[lexeme.span.bytes.clone()], lexeme.text);
    assert_eq!(
        lexeme.span.utf16.start,
        source[..lexeme.span.bytes.start].encode_utf16().count()
    );
    assert_eq!(
        lexeme.span.utf16.end,
        source[..lexeme.span.bytes.end].encode_utf16().count()
    );
}

#[test]
fn each_line_ending_terminates_hidden_notes_at_real_token_boundaries() {
    for ending in ["\n", "\r", "\r\n"] {
        let source = format!(
            "// 🚗 }}; @ 'fake'{ending}package// name boundary{ending}P {{\
             part// definition boundary{ending}def T;\
             part x:// type boundary{ending}T; }}// EOF {{ package Ghost;"
        );
        let arena = build(&source);
        let actual: Vec<_> = declarations(&arena)
            .into_iter()
            .map(|node| (node.kind().unwrap(), name(node).value.clone()))
            .collect();
        assert_eq!(
            actual,
            vec![
                ("Package", Value::String("P".into())),
                ("PartDefinition", Value::String("T".into())),
                ("PartUsage", Value::String("x".into())),
            ]
        );
        for node in arena.nodes().values() {
            assert_span(&source, node.origin().unwrap());
            if let Slot::Present(value) = node.slot("declaredName") {
                assert_span(&source, value.lexeme.as_ref().unwrap());
            }
        }
        let package = declarations(&arena)[0];
        assert_eq!(
            package.origin().unwrap().span.bytes.start,
            source.find("package").unwrap()
        );
        assert!(package.origin().unwrap().text.ends_with('}'));
        // Current construction keeps Namespace's visible-token extent, as
        // qualified in #147; hidden leading/trailing notes are not root CST.
        assert_eq!(
            arena.node(arena.root()).unwrap().origin().unwrap().text,
            package.origin().unwrap().text
        );
    }
}

#[test]
fn quoted_slashes_and_escaped_quotes_stay_in_names_and_references() {
    let source = "// 😀 prefix\npackage 'http://example' { part def 'a\\'//b'; \
                  part 'x//y': 'http://example' // reference segment\r\n :: 'a\\'//b'; }";
    let arena = build(source);
    let actual: Vec<_> = declarations(&arena)
        .into_iter()
        .map(|node| name(node).value.clone())
        .collect();
    assert_eq!(
        actual,
        vec![
            Value::String("'http://example'".into()),
            Value::String("'a\\'//b'".into()),
            Value::String("'x//y'".into()),
        ]
    );
    let reference = arena
        .nodes()
        .values()
        .find(|node| node.kind() == Some("TypeReference"))
        .unwrap();
    let Slot::Present(parts) = reference.slot("parts") else {
        panic!()
    };
    let Value::Array(parts) = &parts.value else {
        panic!()
    };
    assert_eq!(parts.len(), 2);
    for (part, text, target_type) in [
        (&parts[0], "'http://example'", "Element"),
        (&parts[1], "'a\\'//b'", "Type"),
    ] {
        assert_eq!(
            part.value,
            Value::UnresolvedReference {
                text: text.into(),
                target_type: target_type.into()
            }
        );
        assert_eq!(part.lexeme.as_ref().unwrap().text, text);
        assert_span(source, part.lexeme.as_ref().unwrap());
    }
    for node in declarations(&arena) {
        assert_span(source, name(node).lexeme.as_ref().unwrap());
    }
}

#[test]
fn comment_only_eof_has_no_visible_tokens_and_keeps_original_root_span() {
    let source = "// 🚗 @ { } 'never closed /* :;";
    let arena = construct(
        source,
        Limits {
            tokens: 0,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(arena.nodes().len(), 1);
    let root = arena.node(arena.root()).unwrap();
    assert_eq!(root.kind(), Some("Namespace"));
    assert_eq!(root.origin().unwrap().text, source);
    assert_span(source, root.origin().unwrap());
}

#[test]
fn notes_do_not_spend_visible_tokens_but_do_spend_original_source_bytes() {
    let source = "// leading\npackage// mid\rP;// trailing";
    assert!(
        construct(
            source,
            Limits {
                tokens: 3,
                ..Limits::default()
            }
        )
        .is_ok()
    );
    assert!(matches!(
        construct(
            source,
            Limits {
                tokens: 2,
                ..Limits::default()
            }
        ),
        Err(Error::Limit)
    ));
    assert!(matches!(
        construct(
            source,
            Limits {
                source_bytes: source.len() - 1,
                ..Limits::default()
            }
        ),
        Err(Error::Limit)
    ));
    assert!(
        construct(
            source,
            Limits {
                source_bytes: source.len(),
                ..Limits::default()
            }
        )
        .is_ok()
    );
}

#[test]
fn ignored_punctuation_cannot_close_a_body_or_change_original_error_offsets() {
    let missing = "package P { // 😀 }";
    assert!(
        matches!(construct(missing, Limits::default()), Err(Error::Malformed { offset, expected: "}" }) if offset == missing.len())
    );
    let unsupported = "// 🚗 @ ignored\r\npackage P; @";
    assert!(
        matches!(construct(unsupported, Limits::default()), Err(Error::Unsupported { offset }) if offset == unsupported.rfind('@').unwrap())
    );
    let split = "pack// no lexical merging\nage P;";
    assert!(matches!(
        construct(split, Limits::default()),
        Err(Error::Unsupported { offset: 0 })
    ));
}

#[test]
fn semantic_block_comments_and_overlapping_multiline_notes_remain_explicit() {
    for source in [
        "/* semantic */ package P;",
        "/** semantic */",
        "//* note\n*/ package P;",
    ] {
        assert!(matches!(
            construct(source, Limits::default()),
            Err(Error::Unsupported { offset: 0 })
        ));
    }
    // A fourth slash is ordinary line-note content; only exact //* is fenced.
    assert!(declarations(&build("////* ordinary\npackage P;")).len() == 1);
}

#[test]
fn unicode_line_separators_stay_note_content_until_lf_or_cr() {
    let source =
        "// hidden\u{0085}package Hidden;\u{2028}part Gone;\u{2029}package AlsoHidden;\npackage P;";
    let arena = build(source);
    let actual = declarations(&arena);
    assert_eq!(actual.len(), 1);
    assert_eq!(name(actual[0]).value, Value::String("P".into()));
    let origin = actual[0].origin().unwrap();
    assert_eq!(origin.span.bytes.start, source.rfind("package P;").unwrap());
    assert_span(source, origin);
}

#[test]
fn quoted_note_prefix_does_not_hide_an_unterminated_name_error() {
    let source = "// 雪🚗\r\npackage 'still // quoted";
    assert!(source.len() > source.encode_utf16().count());
    assert_eq!(
        construct(source, Limits::default()).unwrap_err(),
        Error::Malformed {
            offset: source.len(),
            expected: "closing name quote"
        }
    );
}

#[test]
fn overlapping_notes_are_refused_at_their_original_offset_after_visible_limit() {
    let source = "package P;//* hidden until P02";
    assert!(matches!(
        construct(
            source,
            Limits {
                tokens: 3,
                ..Limits::default()
            }
        ),
        Err(Error::Unsupported { offset }) if offset == source.find("//*").unwrap()
    ));
    let prefixed = "// 雪🚗\r\npackage P; /* semantic Comment */";
    assert!(matches!(
        construct(prefixed, Limits::default()),
        Err(Error::Unsupported { offset }) if offset == prefixed.find("/*").unwrap()
    ));
}

#[test]
fn empty_and_hidden_only_root_extents_match_the_original_cst_fallback() {
    for (source, expected) in [
        ("", 0..0),
        ("   ", 0..0),
        (" \r\n", 0..0),
        ("//x", 0..3),
        ("  //x", 2..5),
        ("//x  ", 0..5),
        ("//x\n", 0..3),
        (" \n//x\r\n  ", 2..5),
        ("//x\n//y", 0..7),
        ("\n//雪🧭\n\t", 1..10),
    ] {
        let arena = construct(
            source,
            Limits {
                tokens: 0,
                ..Limits::default()
            },
        )
        .unwrap();
        assert_eq!(arena.nodes().len(), 1);
        let origin = arena.node(arena.root()).unwrap().origin().unwrap();
        assert_eq!(origin.span.bytes, expected, "{source:?}");
        assert_span(source, origin);
    }
}

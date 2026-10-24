// Copyright (c) 2026 SysON Rust port contributors.
// SPDX-License-Identifier: EPL-2.0
use std::sync::atomic::AtomicBool;
use syson_syntax::*;
fn grammar(language: Language) -> Grammar {
    Grammar::load(language).expect("complete pinned grammar must compile")
}
#[test]
fn both_full_rule_sets_compile_and_keywords_are_derived() {
    let k = grammar(Language::KerML);
    let s = grammar(Language::SysML);
    assert_eq!(k.rule_names().count(), 270);
    assert_eq!(s.rule_names().count(), 441);
    assert_eq!(
        (k.all_declared_keywords().count(), k.keywords().count()),
        (146, 145)
    );
    assert_eq!(
        (s.all_declared_keywords().count(), s.keywords().count()),
        (174, 173)
    );
    assert!(k.keywords().any(|k| k == "namespace"));
    assert!(s.keywords().any(|k| k == "satisfy"));
}
#[test]
fn original_upstream_namespace_and_camera_fixtures_are_recognized() {
    for (lang, input) in [
        (Language::KerML, include_str!("fixtures/namespace.kerml")),
        (Language::SysML, include_str!("fixtures/camera.sysml")),
    ] {
        let result = grammar(lang)
            .recognize(input, Options::default())
            .unwrap_or_else(|e| panic!("{lang:?}: {e:?}"));
        assert!(!result.cst.is_empty());
        assert!(matches!(result.cst[result.root].kind, CstKind::Rule(_)));
        let recovered: String = result
            .tokens
            .iter()
            .map(|t| t.text(input).unwrap())
            .collect();
        assert_eq!(recovered, input);
    }
}
#[test]
fn original_incomplete_completion_fixture_returns_expected_tokens() {
    let input = include_str!("fixtures/incomplete.sysml");
    let error = grammar(Language::SysML)
        .recognize(input, Options::default())
        .unwrap_err();
    match error {
        Error::Unexpected {
            expected, found, ..
        } => {
            assert!(!expected.is_empty());
            assert_eq!(found.as_deref(), Some("}"));
        }
        e => panic!("{e:?}"),
    }
}
#[test]
fn all_construct_markers_are_visible_without_claiming_semantic_ast() {
    let g = grammar(Language::KerML);
    let result = g
        .recognize("class A; feature x : A;", Options::default())
        .unwrap();
    assert!(
        result
            .cst
            .iter()
            .any(|n| matches!(n.kind, CstKind::Assignment { .. }))
    );
    assert!(
        result
            .cst
            .iter()
            .any(|n| matches!(n.kind, CstKind::CrossReference { .. }))
    );
    let expression = g
        .recognize_rule("OwnedExpression", "1 + 2 * 3", Options::default())
        .unwrap();
    assert!(
        expression
            .cst
            .iter()
            .any(|n| matches!(n.kind, CstKind::Action { .. }))
    );
    for n in expression.cst {
        assert!(n.span.start.byte <= n.span.end.byte);
    }
}
#[test]
fn semantic_invalid_upstream_input_is_only_syntax_recognized() {
    // Syside sysml-validator.test.ts: AttributeUsageTyping is an error for P.
    // This syntax crate has no validation API and does not resolve the type.
    grammar(Language::SysML)
        .recognize("part def P; attribute a : P;", Options::default())
        .unwrap();
}
#[test]
fn keyword_prefixes_and_longest_punctuation_do_not_split_identifiers() {
    let g = grammar(Language::SysML);
    let input = "part partThing :> Other::Thing 0..*";
    let tokens = g.lex(input, Options::default()).unwrap();
    let visible: Vec<_> = tokens.iter().filter(|t| !t.hidden).collect();
    assert_eq!(visible[0].kind, TokenKind::Keyword("part".into()));
    assert_eq!(visible[1].kind, TokenKind::Terminal("ID".into()));
    assert!(visible.iter().any(|t| t.text(input) == Some(":>")));
    assert!(visible.iter().any(|t| t.text(input) == Some("..")));
}
#[test]
fn comments_keep_visible_annotation_tokens_and_hidden_note_priority() {
    let g = grammar(Language::KerML);
    let input = "/* visible */ //* hidden\n block */ // line\nclass A;";
    let tokens = g.lex(input, Options::default()).unwrap();
    assert_eq!(
        tokens[0].kind,
        TokenKind::Terminal("REGULAR_COMMENT".into())
    );
    assert!(!tokens[0].hidden);
    assert!(
        tokens
            .iter()
            .any(|t| t.kind == TokenKind::Terminal("ML_NOTE".into()) && t.hidden)
    );
    assert!(
        tokens
            .iter()
            .any(|t| t.kind == TokenKind::Terminal("SL_NOTE".into()) && t.hidden)
    );
    // A failed ML_NOTE regex falls back to the valid SL_NOTE prefix.
    g.recognize("//* unfinished block note\nclass A;", Options::default())
        .unwrap();
}
#[test]
fn ecmascript_whitespace_and_ascii_identifier_distinctions() {
    let g = grammar(Language::SysML);
    for c in ['\u{feff}', '\u{a0}', '\u{2007}', '\u{202f}', '\u{2028}'] {
        assert!(g.lex(&format!("part{c}a;"), Options::default()).is_ok());
    }
    for c in ['\u{85}', '\u{1c}', 'é'] {
        assert!(matches!(
            g.lex(&format!("part {c};"), Options::default()),
            Err(Error::Lexical { .. })
        ));
    }
}
#[test]
fn utf8_utf16_and_crlf_positions_are_separate() {
    let g = grammar(Language::KerML);
    let input = "class '😀';\r\n";
    let tokens = g.lex(input, Options::default()).unwrap();
    let name = tokens
        .iter()
        .find(|t| t.kind == TokenKind::Terminal("UNRESTRICTED_NAME".into()))
        .unwrap();
    assert_eq!((name.span.start.byte, name.span.end.byte), (6, 12));
    assert_eq!((name.span.start.utf16, name.span.end.utf16), (6, 10));
    let eof = tokens.last().unwrap().span.end;
    assert_eq!(
        (eof.byte, eof.utf16, eof.line, eof.column_utf16),
        (15, 13, 1, 0)
    );
}
#[test]
fn source_number_quirk_and_quote_rules_are_not_normalized() {
    let g = grammar(Language::KerML);
    let input = "1e++-2 .5 1.0 'a\\\'b' \"raw\\n\"";
    let tokens = g.lex(input, Options::default()).unwrap();
    let text: Vec<_> = tokens
        .iter()
        .filter(|t| !t.hidden)
        .map(|t| t.text(input).unwrap())
        .collect();
    assert_eq!(text, vec!["1e++-2", ".5", "1.0", "'a\\\'b'", "\"raw\\n\""]);
    assert!(matches!(
        g.lex("'bad\\q'", Options::default()),
        Err(Error::Lexical { .. })
    ));
    assert!(matches!(
        g.lex("'unfinished", Options::default()),
        Err(Error::Lexical { .. })
    ));
}
#[test]
fn cancellation_and_every_budget_are_explicit_errors() {
    let g = grammar(Language::KerML);
    let cancelled = AtomicBool::new(true);
    assert_eq!(
        g.recognize(
            "",
            Options {
                cancellation: Some(&cancelled),
                ..Default::default()
            }
        ),
        Err(Error::Cancelled)
    );
    for options in [
        Options {
            max_input_bytes: 0,
            ..Default::default()
        },
        Options {
            max_tokens: 0,
            ..Default::default()
        },
        Options {
            max_steps: 0,
            ..Default::default()
        },
        Options {
            max_chart_states: 0,
            ..Default::default()
        },
        Options {
            max_cst_nodes: 0,
            ..Default::default()
        },
    ] {
        assert!(matches!(
            g.recognize("class A;", options),
            Err(Error::BudgetExceeded { .. })
        ));
    }
}
#[test]
fn unknown_constructs_and_terminal_mutations_fail_closed() {
    let original = include_str!("../data/kerml.json");
    let mut alien: serde_json::Value = serde_json::from_str(original).unwrap();
    alien["$type"] = "AlienGrammar".into();
    assert!(
        Grammar::from_json(Language::KerML, &alien.to_string())
            .unwrap_err()
            .unsupported
            .contains("root type")
    );
    let mut value: serde_json::Value = serde_json::from_str(original).unwrap();
    value["rules"][0]["definition"]["$type"] = "UnorderedGroup".into();
    assert!(
        Grammar::from_json(Language::KerML, &value.to_string())
            .unwrap_err()
            .unsupported
            .contains("UnsupportedConstruct")
    );
    let mut value: serde_json::Value = serde_json::from_str(original).unwrap();
    let rules = value["rules"].as_array_mut().unwrap();
    let rule = rules
        .iter_mut()
        .find(|r| r["name"] == "ID" && r["$type"] == "TerminalRule")
        .unwrap();
    rule["definition"]["regex"] = "changed".into();
    assert!(
        Grammar::from_json(Language::KerML, &value.to_string())
            .unwrap_err()
            .unsupported
            .contains("terminal")
    );
}
#[test]
fn no_trailing_garbage_or_missing_delimiters_are_silently_accepted() {
    let g = grammar(Language::KerML);
    for input in [
        "class A",
        "class A; garbage",
        "namespace N { class A;",
        "feature x : ;",
    ] {
        assert!(
            matches!(
                g.recognize(input, Options::default()),
                Err(Error::Unexpected { .. })
            ),
            "{input}"
        );
    }
    assert!(matches!(
        g.recognize_rule("does_not_exist", "", Options::default()),
        Err(Error::UnknownRule(_))
    ));
}

#[test]
fn unreachable_import_kind_does_not_change_runtime_lexer_priority() {
    let g = grammar(Language::KerML);
    let input = "namespace N { import A::*::**; }";
    let tokens = g.lex(input, Options::default()).unwrap();
    assert!(
        tokens
            .iter()
            .any(|t| t.kind == TokenKind::Keyword("::*".into()))
    );
    assert!(
        tokens
            .iter()
            .any(|t| t.kind == TokenKind::Keyword("::**".into()))
    );
    assert!(!tokens.iter().any(|t| t.text(input) == Some("::*::**")));
    g.recognize(input, Options::default()).unwrap();
    assert!(matches!(
        g.recognize_rule("ImportKind", "", Options::default()),
        Err(Error::UnsupportedEntry(_))
    ));
}

#[test]
fn empty_documents_and_repeated_recognition_are_stable_with_contained_spans() {
    for lang in [Language::KerML, Language::SysML] {
        let g = grammar(lang);
        g.recognize("", Options::default()).unwrap();
        let input = "/* doc */  ";
        let a = g.recognize(input, Options::default()).unwrap();
        let b = g.recognize(input, Options::default()).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.semantics, SemanticStatus::NotPerformed);
        for parent in &a.cst {
            for child in &parent.children {
                assert!(parent.span.start.byte <= a.cst[*child].span.start.byte);
                assert!(a.cst[*child].span.end.byte <= parent.span.end.byte);
            }
        }
    }
}

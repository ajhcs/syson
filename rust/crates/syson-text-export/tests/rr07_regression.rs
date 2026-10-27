// SPDX-License-Identifier: EPL-2.0
//! Export tokens follow the actual importer while native NameHelper stays stable.
use std::sync::{Arc, atomic::AtomicBool};
use syson_ast_adapter::is_reserved_word;
use syson_domain::names::{KEYWORDS, printable_name};
use syson_model::{Document, Editor, NodeIds};
use syson_text_export::{Error, Newline, Options, export};
use syson_text_import::Importer;

// Frozen target-source expectation, not a second production keyword authority.
// 170 lexer words minus 117 NameHelper words is a net difference of 53, but
// `constant` belongs only to NameHelper: the actual lexer-only set has 54 words.
const LEXER_ONLY: &[&str] = &[
    "and",
    "as",
    "assoc",
    "behavior",
    "bool",
    "chains",
    "class",
    "classifier",
    "composite",
    "conjugate",
    "conjugates",
    "conjugation",
    "connector",
    "datatype",
    "differences",
    "disjoining",
    "disjoint",
    "expr",
    "false",
    "feature",
    "featured",
    "featuring",
    "function",
    "hastype",
    "implies",
    "interaction",
    "intersects",
    "inv",
    "inverse",
    "inverting",
    "istype",
    "member",
    "meta",
    "metaclass",
    "multiplicity",
    "namespace",
    "null",
    "or",
    "portion",
    "predicate",
    "readonly",
    "redefinition",
    "specialization",
    "step",
    "struct",
    "subclassifier",
    "subset",
    "subtype",
    "true",
    "type",
    "typed",
    "typing",
    "unions",
    "xor",
];
fn import(source: &str) -> Document {
    let mut editor = Editor::new(Document::empty());
    let ids = (1..=1024).map(|n| {
        NodeIds::new(
            &format!("30000000-0000-4000-8000-{n:012x}"),
            &format!("40000000-0000-4000-8000-{n:012x}"),
        )
        .unwrap()
    });
    Importer::pinned()
        .unwrap()
        .stage(
            editor.begin(),
            source,
            None,
            ids,
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .commit(&mut editor)
        .unwrap();
    editor.document().clone()
}
fn roundtrip(source: &str, options: Options) -> String {
    let document = import(source);
    let before = document.clone();
    let text = export(&document, options).unwrap();
    assert_eq!(document, before);
    let reloaded = import(&text);
    assert_eq!(export(&reloaded, options).unwrap(), text);
    text
}
fn parser_words() -> Vec<&'static str> {
    // The shared generated file is the lexer authority. Its frozen shape/count
    // and the explicit fixture above prevent silent omission of a target word.
    include_str!("../../syson-ast-adapter/src/reserved.rs")
        .lines()
        .filter_map(|line| line.trim().strip_prefix('"')?.strip_suffix("\","))
        .collect()
}
#[test]
fn quoted_class_reimports_and_native_name_helper_remains_unchanged() {
    assert_eq!(
        roundtrip("package 'class';", Options::default()),
        "package 'class';"
    );
    assert_eq!(printable_name(Some("class"), true), "class");
    assert_eq!(printable_name(Some("constant"), true), "'constant'");
    assert_eq!(
        roundtrip("package 'constant';", Options::default()),
        "package 'constant';"
    );
}
#[test]
fn the_target_has_54_lexer_only_words_and_one_native_only_word() {
    let parser = parser_words();
    assert_eq!(parser.len(), 170);
    assert_eq!(KEYWORDS.len(), 117);
    let extra: Vec<_> = parser
        .iter()
        .copied()
        .filter(|word| !KEYWORDS.contains(word))
        .collect();
    assert_eq!(extra, LEXER_ONLY);
    let native_only: Vec<_> = KEYWORDS
        .iter()
        .copied()
        .filter(|word| !parser.contains(word))
        .collect();
    assert_eq!(native_only, ["constant"]);
    for word in &parser {
        assert!(is_reserved_word(word), "{word}");
    }
    assert!(!is_reserved_word("constant"));
}
#[test]
fn every_lexer_only_word_is_quoted_in_declarations_and_each_qualified_segment() {
    for word in LEXER_ONLY {
        let source = format!(
            "package '{word}' {{package '{word}' {{part def '{word}';}}}} \
             package Consumer {{part wheel:'{word}'::'{word}'::'{word}';}}"
        );
        let text = roundtrip(&source, Options::default());
        assert!(
            text.contains(&format!("package '{word}' {{")),
            "{word}: {text}"
        );
        assert!(
            text.contains(&format!("part def '{word}';")),
            "{word}: {text}"
        );
        assert!(
            text.contains(&format!("part wheel : '{word}'::'{word}'::'{word}';")),
            "{word}: {text}"
        );
        // A keyword-named usage must be tested separately: giving it the
        // root package's name above would deliberately shadow that package.
        let usage_source = format!("package P {{part def T;part '{word}':T;}}");
        let usage_text = roundtrip(&usage_source, Options::default());
        assert!(
            usage_text.contains(&format!("part '{word}' : T;")),
            "{word}: {usage_text}"
        );
        // The additional quoting is intentionally local to text export.
        assert_eq!(printable_name(Some(word), true), *word);
    }
}
#[test]
fn every_parser_or_native_keyword_keeps_its_name_in_one_actual_importer_roundtrip() {
    let mut words = parser_words();
    words.extend(KEYWORDS.iter().copied());
    words.sort_unstable();
    words.dedup();
    assert_eq!(words.len(), 171);
    let mut source = String::from("package P {");
    let mut expected = String::from("package P {");
    for word in words {
        source.push_str(&format!("part def '{word}';"));
        expected.push_str(&format!("\n    part def '{word}';"));
    }
    source.push('}');
    expected.push_str("\n}");
    assert_eq!(roundtrip(&source, Options::default()), expected);
}
#[test]
fn ordinary_case_sensitive_unicode_and_punctuation_controls_preserve_quoting_policy() {
    for (name, expected) in [
        ("Class", "Class"),
        ("classifierName", "classifierName"),
        ("_class", "_class"),
        ("falsehood", "falsehood"),
        ("wheel2", "wheel2"),
        ("9wheel", "'9wheel'"),
        ("类", "'类'"),
        ("Road parts", "'Road parts'"),
        ("wheel-set", "'wheel-set'"),
    ] {
        let source = format!("package '{name}';");
        assert_eq!(
            roundtrip(&source, Options::default()),
            format!("package {expected};")
        );
    }
}
#[test]
fn shortest_reference_stays_short_and_crlf_and_indentation_remain_explicit() {
    let source = "package 'class' {part def 'type';part wheel:'class'::'type';}";
    assert_eq!(
        roundtrip(
            source,
            Options {
                newline: Newline::CrLf,
                indentation: "\t"
            }
        ),
        "package 'class' {\r\n\tpart def 'type';\r\n\tpart wheel : 'type';\r\n}"
    );
    let ordinary = "package Class {part def Type;part wheel:Class::Type;}";
    assert!(roundtrip(ordinary, Options::default()).contains("part wheel : Type;"));
}
fn exact_boundary_source(bytes: usize) -> String {
    let mut source = String::from("package 'class';");
    let count = 32;
    let names = bytes - source.len() - count * 10;
    for index in 0..count {
        let prefix = format!("P{index:02}_");
        let length = names / count + usize::from(index < names % count);
        assert!(length <= 2048);
        source.push_str("\npackage ");
        source.push_str(&prefix);
        source.push_str(&"x".repeat(length - prefix.len()));
        source.push(';');
    }
    assert_eq!(source.len(), bytes);
    source
}
#[test]
fn added_keyword_quotes_count_toward_the_existing_exact_output_limit() {
    let source = exact_boundary_source(65_536);
    assert_eq!(roundtrip(&source, Options::default()), source);
    let document = import(&source);
    let root = &document.value()["content"][32]["data"];
    let id = root["elementId"].as_str().unwrap().to_owned();
    let name = root["declaredName"].as_str().unwrap().to_owned() + "x";
    let mut editor = Editor::new(document);
    let mut tx = editor.begin();
    tx.rename(&id, &name).unwrap();
    editor.commit(tx).unwrap();
    let before = editor.document().clone();
    assert_eq!(
        export(editor.document(), Options::default()),
        Err(Error::Limit)
    );
    assert_eq!(editor.document(), &before);
}

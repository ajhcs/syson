// SPDX-License-Identifier: EPL-2.0
//! Independent language oracles exercise the CFG engine, not upstream parser parity.
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, Ordering};
use syson_syntax::{Error, Grammar, Language, Options, SemanticStatus, TokenKind};

fn kw(s: &str) -> Value {
    json!({"$type":"Keyword","value":s})
}
fn group(xs: Vec<Value>) -> Value {
    json!({"$type":"Group","elements":xs})
}
fn alt(xs: Vec<Value>) -> Value {
    json!({"$type":"Alternatives","elements":xs})
}
fn call(i: usize) -> Value {
    json!({"$type":"RuleCall","rule":{"$ref":format!("#/rules@{}",12+i)},"arguments":[]})
}
fn custom(definitions: Vec<Value>) -> Grammar {
    let mut v: Value = serde_json::from_str(include_str!(
        "../../../../porting/validation/fixtures/syntax-terminal-base.json"
    ))
    .unwrap();
    for (i, definition) in definitions.into_iter().enumerate() {
        v["rules"].as_array_mut().unwrap().push(json!({"$type":"ParserRule","name":format!("R{i}"),"entry":i==0,"fragment":false,"dataType":false,"definesHiddenTokens":false,"hiddenTokens":[],"parameters":[],"wildcard":false,"definition":definition}));
    }
    Grammar::from_json(Language::KerML, &v.to_string()).unwrap()
}
fn words(n: usize, mask: usize, a: char, b: char, spaced: bool) -> String {
    (0..n)
        .map(|i| {
            if mask & (1 << i) == 0 {
                a.to_string()
            } else {
                b.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(if spaced { " " } else { "" })
}
fn valid_tree(input: &str, r: &syson_syntax::Recognition) {
    assert_eq!(r.semantics, SemanticStatus::NotPerformed);
    assert!(r.root < r.cst.len());
    let mut parent_counts = vec![0; r.cst.len()];
    for n in &r.cst {
        assert!(n.span.start.byte <= n.span.end.byte);
        assert!(input.is_char_boundary(n.span.start.byte));
        assert!(input.is_char_boundary(n.span.end.byte));
        for &child in &n.children {
            assert!(child < r.cst.len());
            parent_counts[child] += 1;
            assert!(n.span.start.byte <= r.cst[child].span.start.byte);
            assert!(r.cst[child].span.end.byte <= n.span.end.byte);
        }
    }
    assert_eq!(parent_counts[r.root], 0);
    // Nodes may be shared in the selected derivation, but none is orphaned.
    assert!(
        parent_counts
            .iter()
            .enumerate()
            .all(|(i, &n)| i == r.root || n > 0)
    );
    assert_eq!(
        r.tokens
            .iter()
            .map(|t| t.text(input).unwrap())
            .collect::<String>(),
        input
    );
}
#[test]
fn exhaustive_balanced_parentheses_match_independent_counter_oracle() {
    let g = custom(vec![alt(vec![
        group(vec![kw("("), call(0), kw(")"), call(0)]),
        group(vec![]),
    ])]);
    for n in 0..=10 {
        for bits in 0..1 << n {
            let input = words(n, bits, '(', ')', false);
            let mut depth = 0i32;
            let mut valid = true;
            for c in input.chars() {
                depth += if c == '(' { 1 } else { -1 };
                if depth < 0 {
                    valid = false;
                }
            }
            valid &= depth == 0;
            let got = g.recognize(&input, Options::default());
            assert_eq!(got.is_ok(), valid, "{input:?}: {got:?}");
            if let Ok(r) = got {
                valid_tree(&input, &r);
            }
        }
    }
}
#[test]
fn nullable_left_recursion_and_late_completions_match_small_language_oracles() {
    // R0=R1 R2; R1=R1 a|epsilon; R2=R2 b|R1 => a* b*.
    let g = custom(vec![
        group(vec![call(1), call(2)]),
        alt(vec![group(vec![call(1), kw("a")]), group(vec![])]),
        alt(vec![group(vec![call(2), kw("b")]), call(1)]),
    ]);
    for n in 0..=8 {
        for bits in 0..1 << n {
            let input = words(n, bits, 'a', 'b', true);
            let valid = !input.contains("b a");
            let got = g.recognize(&input, Options::default());
            assert_eq!(got.is_ok(), valid, "{input:?}: {got:?}");
            if let Ok(r) = got {
                valid_tree(&input, &r);
            }
        }
    }
    let late = custom(vec![
        group(vec![call(1), call(1)]),
        alt(vec![group(vec![]), kw("a")]),
    ]);
    for n in 0..=4 {
        assert_eq!(
            late.recognize(&vec!["a"; n].join(" "), Options::default())
                .is_ok(),
            n <= 2
        );
    }
}
#[test]
fn ambiguous_nullable_recursion_terminates_and_returns_only_a_complete_derivation() {
    let g = custom(vec![alt(vec![
        group(vec![call(0), call(0)]),
        kw("a"),
        group(vec![]),
    ])]);
    for n in 0..=9 {
        let input = vec!["a"; n].join(" ");
        let a = g.recognize(&input, Options::default()).unwrap();
        valid_tree(&input, &a);
        assert_eq!(a, g.recognize(&input, Options::default()).unwrap());
    }
    assert!(g.recognize("a b", Options::default()).is_err());
}
#[test]
fn source_terminal_boundaries_preserve_raw_spelling_and_unicode_offsets() {
    let g = Grammar::load(Language::KerML).unwrap();
    let input = "1. 1..2 1e 1e+-2 .25 2E--3 // last\u{2028}line\r\n'😀\\n' \"x\\\" tail";
    let ts = g.lex(input, Options::default()).unwrap();
    let visible: Vec<_> = ts
        .iter()
        .filter(|t| !t.hidden)
        .map(|t| t.text(input).unwrap())
        .collect();
    assert_eq!(
        visible,
        vec![
            "1",
            ".",
            "1",
            "..",
            "2",
            "1",
            "e",
            "1e+-2",
            ".25",
            "2E--3",
            "'😀\\n'",
            "\"x\\\"",
            "tail"
        ]
    );
    let name = ts
        .iter()
        .find(|t| t.text(input) == Some("'😀\\n'"))
        .unwrap();
    assert_eq!((name.span.start.line, name.span.start.column_utf16), (1, 0));
    assert_eq!(name.span.end.utf16 - name.span.start.utf16, 6);
    assert_eq!(name.span.end.byte - name.span.start.byte, 8);
    assert!(
        ts.iter()
            .any(|t| t.kind == TokenKind::Terminal("SL_NOTE".into())
                && t.text(input).unwrap().contains('\u{2028}'))
    );
    assert_eq!(
        ts.iter()
            .map(|t| t.text(input).unwrap())
            .collect::<String>(),
        input
    );
    for input in ["'a\\q'", "\"unterminated", "'unterminated", "\u{85}", "\0"] {
        assert!(
            matches!(g.lex(input, Options::default()), Err(Error::Lexical { .. })),
            "{input:?}"
        );
    }
}
#[test]
fn resource_boundaries_and_cancellation_fail_without_successful_partial_results() {
    let g = Grammar::load(Language::SysML).unwrap();
    let input = include_str!("../../../../porting/validation/fixtures/parser-oracle.sysml");
    let tokens = g.lex(input, Options::default()).unwrap();
    assert!(
        g.lex(
            input,
            Options {
                max_input_bytes: input.len(),
                max_tokens: tokens.len(),
                ..Default::default()
            }
        )
        .is_ok()
    );
    for options in [
        Options {
            max_input_bytes: input.len() - 1,
            ..Default::default()
        },
        Options {
            max_tokens: tokens.len() - 1,
            ..Default::default()
        },
        Options {
            max_steps: 100,
            ..Default::default()
        },
        Options {
            max_chart_states: 10,
            ..Default::default()
        },
        Options {
            max_cst_nodes: 10,
            ..Default::default()
        },
    ] {
        assert!(matches!(
            g.recognize(input, options),
            Err(Error::BudgetExceeded { .. })
        ));
    }
    let cancel = AtomicBool::new(false);
    let huge = format!("//{}", "x".repeat(1_000_000));
    std::thread::scope(|scope| {
        scope.spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(1));
            cancel.store(true, Ordering::Relaxed);
        });
        assert_eq!(
            g.lex(
                &huge,
                Options {
                    cancellation: Some(&cancel),
                    ..Default::default()
                }
            ),
            Err(Error::Cancelled)
        );
    });
}
#[test]
fn same_executed_reference_fixture_is_recognized_with_semantics_not_performed() {
    let input = include_str!("../../../../porting/validation/fixtures/parser-oracle.sysml");
    assert_eq!(input.len(), 37);
    let result = Grammar::load(Language::SysML)
        .unwrap()
        .recognize(input, Options::default())
        .unwrap();
    valid_tree(input, &result);
}
#[test]
fn actual_grammar_rejects_truncation_and_extra_tokens_at_bounded_entry_points() {
    for language in [Language::KerML, Language::SysML] {
        let g = Grammar::load(language).unwrap();
        for input in [
            "namespace N {",
            "class A; }",
            "/* unfinished",
            "'bad\\z'",
            "@",
        ] {
            assert!(
                g.recognize(input, Options::default()).is_err(),
                "{language:?} {input:?}"
            );
        }
    }
    let s = Grammar::load(Language::SysML).unwrap();
    for input in [
        "package Oracle { part def Vehicle; }",
        "package P { part def A; part x : A; }",
        "package P { attribute x = 1e+-2; }",
    ] {
        let r = s.recognize(input, Options::default()).unwrap();
        valid_tree(input, &r);
    }
}
#[test]
fn malformed_or_unsupported_grammar_changes_fail_closed() {
    fn base() -> Value {
        let mut v: Value = serde_json::from_str(include_str!(
            "../../../../porting/validation/fixtures/syntax-terminal-base.json"
        ))
        .unwrap();
        v["rules"].as_array_mut().unwrap().push(json!({"$type":"ParserRule","name":"R0","entry":true,"fragment":false,"definesHiddenTokens":false,"hiddenTokens":[],"parameters":[],"wildcard":false,"definition":{"$type":"Keyword","value":"a"}}));
        v
    }
    let mut cases = vec![];
    let mut v = base();
    v["rules"][12]["definition"] = json!({"$type":"UnorderedGroup","elements":[]});
    cases.push(v);
    let mut v = base();
    v["rules"][12]["definition"]["cardinality"] = "!".into();
    cases.push(v);
    let mut v = base();
    v["rules"][12]["parameters"] = json!([{"name":"conditional"}]);
    cases.push(v);
    let mut v = base();
    v["definesHiddenTokens"] = true.into();
    cases.push(v);
    let mut v = base();
    v["imports"] = json!([{"path":"external"}]);
    cases.push(v);
    let mut v = base();
    v["rules"][12]["definition"] =
        json!({"$type":"RuleCall","rule":{"$ref":"#/rules@999"},"arguments":[]});
    cases.push(v);
    let mut v = base();
    v["rules"][12]["definition"] =
        json!({"$type":"RuleCall","rule":{"$ref":"http://external/rule"},"arguments":[]});
    cases.push(v);
    let mut v = base();
    let i = v["rules"]
        .as_array()
        .unwrap()
        .iter()
        .position(|r| r["name"] == "DECIMAL")
        .unwrap();
    v["rules"][12]["definition"] =
        json!({"$type":"RuleCall","rule":{"$ref":format!("#/rules@{i}")},"arguments":[]});
    cases.push(v);
    let mut v = base();
    v["rules"][0]["hidden"] = true.into();
    cases.push(v);
    let mut v = base();
    v["rules"][0]["definition"] = json!({"$type":"RegexToken","regex":".*"});
    cases.push(v);
    let mut v = base();
    v["rules"][12]["definition"] =
        json!({"$type":"Action","type":{"$ref":"#/rules@12"},"feature":"x","operator":"?="});
    cases.push(v);
    let mut v = base();
    let duplicate = v["rules"][12].clone();
    v["rules"].as_array_mut().unwrap().push(duplicate);
    cases.push(v);
    for (i, v) in cases.into_iter().enumerate() {
        assert!(
            Grammar::from_json(Language::KerML, &v.to_string()).is_err(),
            "mutation {i}"
        );
    }
}

use syson_ast_values::*;
fn control() -> Control<'static> {
    Control {
        max_utf16_units: 4096,
        is_cancelled: &|| false,
    }
}
fn lexeme(s: &str) -> Lexeme<'_> {
    Lexeme {
        text: s,
        span: Span {
            start: 7,
            end: 7 + s.len(),
        },
    }
}
fn convert_rule<'a>(name: &str, ty: Option<&str>, text: &'a str) -> Converted<'a> {
    match convert(
        Dispatch::Rule(Rule {
            name,
            declared_type: ty,
        }),
        Presence::Value(lexeme(text)),
        &control(),
    )
    .unwrap()
    {
        Presence::Value(v) => v,
        _ => panic!("present lexeme lost"),
    }
}
fn text(name: &str, ty: Option<&str>, input: &str) -> JsString {
    match convert_rule(name, ty, input).value {
        Primitive::String(v) => v,
        _ => panic!("expected string"),
    }
}
fn number(input: &str) -> f64 {
    match convert_rule("EXPONENTIAL_VALUE", Some("number"), input).value {
        Primitive::Number(v) => v,
        _ => panic!("expected number"),
    }
}
#[test]
fn absence_null_and_empty_string_remain_distinct() {
    assert_eq!(
        convert(Dispatch::Raw, Presence::Undefined, &control()).unwrap(),
        Presence::Undefined
    );
    assert_eq!(
        convert(Dispatch::Raw, Presence::Null, &control()).unwrap(),
        Presence::Null
    );
    assert_eq!(text("STRING_VALUE", Some("string"), ""), JsString(vec![]));
}
#[test]
fn reachable_string_terminals_retain_quotes_escapes_and_unicode() {
    for (rule, s) in [
        ("STRING_VALUE", "\"a\\n😀\""),
        ("UNRESTRICTED_NAME", "'a\\nλ'"),
        ("TypedByToken", "typedby"),
    ] {
        let value = convert_rule(rule, Some("string"), s);
        assert_eq!(value.lexeme, lexeme(s));
        assert_eq!(
            value.value,
            Primitive::String(JsString(s.encode_utf16().collect()))
        );
    }
}
#[test]
fn named_converter_precedes_declared_type_and_is_ascii_case_insensitive() {
    assert_eq!(
        text("sTrInG", Some("number"), "\"\\n\""),
        JsString(vec![10])
    );
    assert_eq!(
        text("Id", Some("number"), "^^Vehicle").to_utf8().unwrap(),
        "^Vehicle"
    );
    assert_eq!(text("id", None, "Vehicle").to_utf8().unwrap(), "Vehicle");
    assert_eq!(text("ID", None, "😀").to_utf8().unwrap(), "😀");
}
#[test]
fn string_escape_scan_is_not_json_decoding() {
    let converted = text(
        "STRING",
        None,
        "\"\\b\\f\\n\\r\\t\\v\\0\\x41\\u0041\\q\\\\\"\" ",
    );
    assert_eq!(
        converted.to_utf8().unwrap(),
        "\u{8}\u{c}\n\r\t\u{b}\0x41u0041q\\\"\""
    );
    assert_eq!(text("STRING", None, "\"a\\\"").to_utf8().unwrap(), "a\"");
    assert_eq!(text("STRING", None, "x"), JsString(vec![]));
}
#[test]
fn regex_substring_and_string_slicing_follow_utf16_units() {
    assert_eq!(
        text("REGEXLITERAL", None, "/a+/ ").to_utf8().unwrap(),
        "a+/"
    );
    assert_eq!(text("REGEXLITERAL", None, "x").to_utf8().unwrap(), "x");
    assert_eq!(text("STRING", None, "a😀").0, vec![0xd83d]);
    assert!(text("STRING", None, "a😀").to_utf8().is_err());
}
#[test]
fn name_sanitation_is_a_later_distinct_layer() {
    assert_eq!(sanitize_name(None, &control()).unwrap(), None);
    assert_eq!(sanitize_name(Some(""), &control()).unwrap(), None);
    assert_eq!(
        sanitize_name(Some("''"), &control()).unwrap(),
        Some(JsString(vec![]))
    );
    assert_eq!(
        sanitize_name(Some("'A\\n\\q\\v😀'"), &control())
            .unwrap()
            .unwrap()
            .to_utf8()
            .unwrap(),
        "A\n\\q\\v😀"
    );
    assert_eq!(
        sanitize_name(Some("'"), &control()).unwrap(),
        Some(JsString(vec![]))
    );
    assert_eq!(
        sanitize_name(Some("'😀"), &control()).unwrap().unwrap().0,
        vec![0xd83d]
    );
    assert_eq!(
        sanitize_name(Some("unquoted"), &control())
            .unwrap()
            .unwrap()
            .to_utf8()
            .unwrap(),
        "unquoted"
    );
}
#[test]
fn name_escaping_preserves_ascii_rule_and_source_escape_set() {
    for (input, expected) in [
        ("Vehicle_1", "Vehicle_1"),
        ("λ", "'λ'"),
        ("", "''"),
        ("1x", "'1x'"),
        ("a\nb'\\", "'a\\nb\\'\\\\'"),
    ] {
        assert_eq!(
            unsanitize_name(&JsString(input.encode_utf16().collect()), &control())
                .unwrap()
                .to_utf8()
                .unwrap(),
            expected
        );
    }
}
#[test]
fn decimal_values_round_binary64_but_keep_source_spelling() {
    assert_eq!(number("9007199254740993"), 9007199254740992.0);
    assert_eq!(number("9007199254740995"), 9007199254740996.0);
    for raw in ["1", "1.0", "1e0"] {
        let converted = convert_rule("NUMBER", Some("number"), raw);
        assert_eq!(converted.lexeme.text, raw);
        assert_eq!(converted.value, Primitive::Number(1.0));
    }
    assert_eq!(syson_literal_kind("1"), SysOnLiteralKind::Integer);
    assert_eq!(syson_literal_kind("1.0"), SysOnLiteralKind::Rational);
    assert_eq!(syson_literal_kind("1e0"), SysOnLiteralKind::Rational);
}
#[test]
fn signs_subnormal_overflow_and_nonfinite_are_explicit_numbers() {
    for raw in ["-0", "-0.0", "-0e999", "-1e-9999"] {
        assert_eq!(number(raw).to_bits(), (-0.0f64).to_bits(), "{raw}");
    }
    assert_eq!(number("5e-324").to_bits(), 1);
    assert_eq!(number("1e309"), f64::INFINITY);
    assert_eq!(number("-Infinity"), f64::NEG_INFINITY);
    assert_eq!(number("+Infinity"), f64::INFINITY);
    assert!(number("NaN").is_nan());
}
#[test]
fn number_syntax_is_not_rust_parse_syntax() {
    for raw in [
        "1e", "1e+", ".", "+", "inf", "infinity", "-NaN", "1_000", "1 2", "-0x1", "+0b1", "0x",
        "0b2", "0o8", "１２",
    ] {
        assert!(number(raw).is_nan(), "{raw}");
    }
    for (raw, expected) in [
        (".5", 0.5),
        ("1.", 1.0),
        ("+2e-1", 0.2),
        ("0x10", 16.0),
        ("0O10", 8.0),
        ("0b10", 2.0),
        ("", 0.0),
    ] {
        assert_eq!(number(raw), expected, "{raw}");
    }
}
#[test]
fn js_whitespace_is_not_rust_whitespace() {
    assert_eq!(number("\u{feff}\u{a0}2\u{2028}"), 2.0);
    assert_eq!(number("\t \n"), 0.0);
    assert!(number("\u{85}2").is_nan());
    assert!(number("\u{180e}2").is_nan());
}
#[test]
fn radix_rounding_uses_guard_and_sticky_bits() {
    assert_eq!(number("0x20000000000001"), 9007199254740992.0);
    assert_eq!(number("0x20000000000003"), 9007199254740996.0);
    assert_eq!(number("0xffffffffffffffff"), 18446744073709551616.0);
    assert_eq!(
        number(&format!("0b1{}", "0".repeat(1023))),
        f64::from_bits(2046u64 << 52)
    );
    assert_eq!(number(&format!("0b1{}", "0".repeat(1024))), f64::INFINITY);
    assert!(number(&format!("0b1{}x", "0".repeat(1024))).is_nan());
}
#[test]
fn inherited_int_converter_uses_prefix_and_default_hex_detection() {
    for (raw, expected) in [
        ("12tail", 12.0),
        ("1e3", 1.0),
        ("0b11", 0.0),
        ("0x10rest", 16.0),
        (" -0Xf!", -15.0),
    ] {
        assert_eq!(
            convert_rule("INT", None, raw).value,
            Primitive::Number(expected)
        );
    }
    assert!(matches!(convert_rule("INT",None,"word").value,Primitive::Number(v) if v.is_nan()));
    assert!(
        matches!(convert_rule("INT",None,"-0x0").value,Primitive::Number(v) if v.to_bits()==(-0.0f64).to_bits())
    );
}
#[test]
fn boolean_helper_has_no_whitespace_trim_and_bigint_date_are_unsupported() {
    assert_eq!(
        convert_rule("x", Some("BOOLEAN"), "TrUe").value,
        Primitive::Boolean(true)
    );
    assert_eq!(
        convert_rule("x", Some("boolean"), " true ").value,
        Primitive::Boolean(false)
    );
    for (ty, kind) in [
        ("bigint", ErrorKind::UnsupportedBigInt),
        ("date", ErrorKind::UnsupportedDate),
    ] {
        assert_eq!(
            convert(
                Dispatch::Rule(Rule {
                    name: "x",
                    declared_type: Some(ty)
                }),
                Presence::Value(lexeme("1")),
                &control()
            )
            .unwrap_err(),
            ConversionError {
                kind,
                span: Some(lexeme("1").span)
            }
        );
    }
}
#[test]
fn budgets_cancellation_and_invalid_dispatch_retain_failure_span() {
    let value = lexeme("😀");
    let tiny = Control {
        max_utf16_units: 1,
        is_cancelled: &|| false,
    };
    assert_eq!(
        convert(Dispatch::Raw, Presence::Value(value), &tiny)
            .unwrap_err()
            .kind,
        ErrorKind::BudgetExceeded
    );
    assert_eq!(
        convert(
            Dispatch::Raw,
            Presence::Value(value),
            &Control {
                max_utf16_units: 10,
                is_cancelled: &|| true
            }
        )
        .unwrap_err(),
        ConversionError {
            kind: ErrorKind::Cancelled,
            span: Some(value.span)
        }
    );
    assert_eq!(
        convert(
            Dispatch::UnresolvedRuleCall,
            Presence::Value(value),
            &control()
        )
        .unwrap_err()
        .kind,
        ErrorKind::UnresolvedRule
    );
    assert_eq!(
        convert(
            Dispatch::Rule(Rule {
                name: "ſtring",
                declared_type: None
            }),
            Presence::Value(value),
            &control()
        )
        .unwrap_err()
        .kind,
        ErrorKind::NonAsciiDispatch
    );
    let invalid = Lexeme {
        text: "x",
        span: Span { start: 9, end: 1 },
    };
    assert_eq!(
        convert(Dispatch::Raw, Presence::Value(invalid), &control())
            .unwrap_err()
            .kind,
        ErrorKind::InvalidSpan
    );
}

#[test]
fn empty_quoted_name_obeys_output_budget_and_cancellation() {
    for maximum in [0, 1] {
        assert_eq!(
            unsanitize_name(
                &JsString(vec![]),
                &Control {
                    max_utf16_units: maximum,
                    is_cancelled: &|| false
                }
            )
            .unwrap_err(),
            ErrorKind::BudgetExceeded
        );
    }
    assert_eq!(
        unsanitize_name(
            &JsString(vec![]),
            &Control {
                max_utf16_units: 2,
                is_cancelled: &|| false
            }
        )
        .unwrap(),
        JsString(vec![39, 39])
    );
    assert_eq!(
        unsanitize_name(
            &JsString(vec![]),
            &Control {
                max_utf16_units: 2,
                is_cancelled: &|| true
            }
        )
        .unwrap_err(),
        ErrorKind::Cancelled
    );
}

#[test]
fn unicode_spans_are_utf8_bytes_and_failures_keep_the_supplied_span() {
    let raw = "λ😀";
    let valid = lexeme(raw);
    let converted = convert(Dispatch::Raw, Presence::Value(valid), &control()).unwrap();
    assert_eq!(
        converted,
        Presence::Value(Converted {
            lexeme: valid,
            value: Primitive::String(JsString(raw.encode_utf16().collect())),
        })
    );
    let utf16_width = Lexeme {
        text: raw,
        span: Span {
            start: valid.span.start,
            end: valid.span.start + raw.encode_utf16().count(),
        },
    };
    assert_eq!(
        convert(Dispatch::Raw, Presence::Value(utf16_width), &control()).unwrap_err(),
        ConversionError {
            kind: ErrorKind::InvalidSpan,
            span: Some(utf16_width.span),
        }
    );
}

#[test]
fn int_no_digit_failures_do_not_apply_a_negative_sign_to_nan() {
    // Original pinned converter observations distinguish these bits from -NaN.
    for raw in ["-word", "-0xnot", "-", "-0x", "- 1"] {
        let Primitive::Number(value) = convert_rule("INT", None, raw).value else {
            panic!("INT must retain a numeric value");
        };
        assert_eq!(value.to_bits(), f64::NAN.to_bits(), "{raw}");
    }
    for raw in ["-0", "-0x0"] {
        let Primitive::Number(value) = convert_rule("INT", None, raw).value else {
            panic!("INT must retain a numeric value");
        };
        assert_eq!(value.to_bits(), (-0.0f64).to_bits(), "{raw}");
    }
}

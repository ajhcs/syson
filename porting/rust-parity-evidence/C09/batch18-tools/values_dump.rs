// Development evidence only: direct public F10 API observations, no oracle data.
use serde_json::{Value, json};
use syson_ast_values::{
    Control, Dispatch, JsString, Lexeme, Presence, Primitive, Rule, Span, convert, sanitize_name,
    unsanitize_name,
};

fn string(value: JsString) -> Value {
    json!({"kind": "string", "units": value.0})
}
fn error(value: impl std::fmt::Debug) -> Value {
    json!({"kind": "rust_error", "name": format!("{value:?}")})
}
fn main() {
    let path = std::env::args().nth(1).expect("explicit corpus path");
    let corpus: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(corpus["schema"], 1);
    let control = Control {
        max_utf16_units: 262_144,
        is_cancelled: &|| false,
    };
    let mut observations = Vec::new();
    let mut lexemes = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let text = case.get("text").and_then(Value::as_str);
        let value = match case["op"].as_str().unwrap() {
            "sanitize" => match sanitize_name(text, &control) {
                Ok(Some(value)) => string(value),
                Ok(None) => json!({"kind": "undefined"}),
                Err(err) => error(err),
            },
            "unsanitize" => {
                let units = match case.get("input_units") {
                    Some(units) => units
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|u| u16::try_from(u.as_u64().unwrap()).unwrap())
                        .collect(),
                    None => text.unwrap().encode_utf16().collect(),
                };
                match unsanitize_name(&JsString(units), &control) {
                    Ok(value) => string(value),
                    Err(err) => error(err),
                }
            }
            "convert" => {
                let text = text.unwrap();
                let dispatch = match case["dispatch"].as_str().unwrap_or("rule") {
                    "raw" => Dispatch::Raw,
                    "unresolved" => Dispatch::UnresolvedRuleCall,
                    "rule" => Dispatch::Rule(Rule {
                        name: case["rule"].as_str().unwrap(),
                        declared_type: case.get("declared_type").and_then(Value::as_str),
                    }),
                    _ => panic!("unknown dispatch"),
                };
                let input = Lexeme {
                    text,
                    span: Span {
                        start: 7,
                        end: 7 + text.len(),
                    },
                };
                match convert(dispatch, Presence::Value(input), &control) {
                    Ok(Presence::Value(value)) => {
                        assert_eq!(value.lexeme, input);
                        lexemes.push(json!({"id": case["id"], "text": value.lexeme.text,
                            "start": value.lexeme.span.start, "end": value.lexeme.span.end}));
                        match value.value {
                            Primitive::String(value) => string(value),
                            Primitive::Number(value) => {
                                json!({"kind": "number", "bits": format!("{:016x}", value.to_bits())})
                            }
                            Primitive::Boolean(value) => json!({"kind": "boolean", "value": value}),
                        }
                    }
                    Ok(Presence::Undefined) => json!({"kind": "undefined"}),
                    Ok(Presence::Null) => json!({"kind": "null"}),
                    Err(err) => {
                        assert_eq!(err.span, Some(input.span));
                        error(err.kind)
                    }
                }
            }
            _ => panic!("unknown operation"),
        };
        observations.push(json!({"id": case["id"], "value": value}));
    }
    println!(
        "{}",
        json!({"schema": 1, "observations": observations, "retained_lexemes": lexemes})
    );
}

// SPDX-License-Identifier: EPL-2.0
//! Actual default ABI import/selection; no model or rendered-result replacement.
use serde_json::{Value, json};
use syson_model::Document;
use syson_web::{dispatch, initialize, input_byte, input_reset, output_len, output_ptr};
fn html() -> String {
    // SAFETY: copy synchronous public output before any later dispatch.
    let bytes = unsafe { std::slice::from_raw_parts(output_ptr() as *const u8, output_len()) };
    String::from_utf8(bytes.to_vec()).unwrap()
}
fn send(action: &str, fields: Value) -> String {
    input_reset();
    for byte in serde_json::to_vec(&json!({"action":action,"fields":fields})).unwrap() {
        input_byte(byte.into());
    }
    dispatch();
    html()
}
fn decode(text: &str) -> String {
    text.replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}
fn textarea(html: &str, id: &str) -> String {
    let rest = html
        .split_once(&format!("<textarea id=\"{id}\""))
        .unwrap()
        .1;
    decode(
        rest.split_once('>')
            .unwrap()
            .1
            .split_once("</textarea>")
            .unwrap()
            .0
            .trim_start_matches('\n'),
    )
}
#[test]
fn long_saved_values_are_disclosed_without_changing_saved_json_or_small_controls() {
    initialize();
    let mut value = Document::empty().value().clone();
    let body = "\n雪🧭\r\n\0<>&\"'".repeat(300);
    let aliases = vec!["read-only\r\n<&>雪".repeat(300)];
    value["content"] = json!([{"id":"comment-resource","eClass":"sysml:Comment",
        "data":{"elementId":"comment0","declaredName":"Ordinary", "body":body,"aliasIds":aliases}}]);
    let input = value.to_string();
    let expected = Document::from_json(&input).unwrap().to_json().unwrap();
    send("import", json!({"json":input}));
    let rendered = send("select", json!({"value":"comment0"}));
    assert_eq!(textarea(&rendered, "export-json"), expected);
    assert!(!rendered.contains("id=\"saved-value-declaredName\""));
    assert!(rendered.contains("class=\"property-state\">Saved value: &quot;Ordinary&quot;</p>"));
    for (name, expected_value) in [("body", json!(body)), ("aliasIds", json!(aliases))] {
        let disclosed = textarea(&rendered, &format!("saved-json-{name}"));
        assert_eq!(
            serde_json::from_str::<Value>(&disclosed).unwrap(),
            expected_value
        );
        assert_eq!(disclosed, expected_value.to_string());
        assert!(rendered.contains(&format!(
            "<details id=\"saved-value-{name}\"><summary>Full saved value for "
        )));
        assert!(rendered.contains(&format!(
            "<textarea id=\"saved-json-{name}\" readonly rows=\"6\" wrap=\"off\""
        )));
    }
    assert!(!rendered.contains("data-attribute=\"aliasIds\""));
    assert_eq!(textarea(&rendered, "export-json"), expected);
    // A later action must not publish normalized textarea text into saved data.
    let rejected = send("import", json!({"json":"invalid"}));
    assert_eq!(textarea(&rejected, "export-json"), expected);
}

#[test]
fn saved_summary_threshold_counts_compact_json_bytes_not_characters() {
    initialize();
    for (body, long) in [
        ("W".repeat(78), false),
        ("W".repeat(79), true),
        ("\0".repeat(13), false),
        ("\0".repeat(14), true),
        ("雪".repeat(26), false),
        ("雪".repeat(27), true),
    ] {
        let mut value = Document::empty().value().clone();
        value["content"] = json!([{"id":"comment-resource","eClass":"sysml:Comment",
            "data":{"elementId":"comment0","body":body}}]);
        send("import", json!({"json":value.to_string()}));
        let rendered = send("select", json!({"value":"comment0"}));
        assert_eq!(rendered.contains("id=\"saved-value-body\""), long);
        if long {
            assert_eq!(
                textarea(&rendered, "saved-json-body"),
                json!(body).to_string()
            );
        } else {
            assert!(rendered.contains("class=\"property-state\">Saved value: "));
        }
        assert_eq!(
            serde_json::from_str::<Value>(&textarea(&rendered, "export-json")).unwrap(),
            value
        );
    }
}

#[test]
fn readonly_wide_array_uses_full_value_disclosure_without_losing_json() {
    initialize();
    let aliases = json!(["W".repeat(120)]);
    assert_eq!(aliases.to_string().len(), 124);
    let mut value = Document::empty().value().clone();
    value["content"] = json!([{"id":"comment-resource","eClass":"sysml:Comment",
        "data":{"elementId":"comment0","aliasIds":aliases}}]);
    let raw = value.to_string();
    let expected = Document::from_json(&raw).unwrap().to_json().unwrap();
    send("import", json!({"json":raw}));
    let rendered = send("select", json!({"value":"comment0"}));
    assert!(rendered.contains(
        "<details id=\"saved-value-aliasIds\"><summary>Full saved value for Alias ids</summary>"
    ));
    assert!(
        rendered.contains("<textarea id=\"saved-json-aliasIds\" readonly rows=\"6\" wrap=\"off\"")
    );
    assert_eq!(
        textarea(&rendered, "saved-json-aliasIds"),
        aliases.to_string()
    );
    assert!(!rendered.contains("data-attribute=\"aliasIds\""));
    assert_eq!(textarea(&rendered, "export-json"), expected);
}

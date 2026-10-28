// SPDX-License-Identifier: EPL-2.0
//! Public ABI navigation keeps complete model authority across visual abbreviation.
use serde_json::{Value, json};
use syson_model::Document;
use syson_web::{dispatch, initialize, input_byte, input_reset, output_len, output_ptr};
fn html() -> String {
    // SAFETY: copy synchronous public ABI output before the next dispatch.
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
fn exported(html: &str) -> String {
    html.split_once("<textarea id=\"export-json\"")
        .unwrap()
        .1
        .split_once('>')
        .unwrap()
        .1
        .split_once("</textarea>")
        .unwrap()
        .0
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}
#[test]
fn full_long_navigation_name_and_history_remain_exact() {
    let name = format!("Long <&> 雪🧭 {} END", "x".repeat(4096));
    let mut raw = Document::empty().value().clone();
    raw["content"] = json!([{"id":"nav-resource","eClass":"sysml:Package","data":{"elementId":"nav0","declaredName":name}}]);
    let expected = Document::from_json(&raw.to_string())
        .unwrap()
        .to_json()
        .unwrap();
    initialize();
    let imported = send("import", json!({"json":raw.to_string()}));
    assert_eq!(exported(&imported), expected);
    let escaped = name
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    assert!(imported.contains(&format!(
        "<span class=\"model-name\">{escaped}</span><span class=\"kind\">Package</span>"
    )));
    assert!(imported.contains("aria-describedby=\"model-name-help\""));
    let selected = send("select", json!({"value":"nav0"}));
    assert!(selected.contains("aria-pressed=\"true\""));
    assert!(selected.contains(&format!("rows=\"2\" required>\n{escaped}</textarea>")));
    assert_eq!(exported(&selected), expected);
    send("rename", json!({"name":"Changed exact name"}));
    let changed = exported(&send("commit", json!({})));
    assert_ne!(changed, expected);
    assert_eq!(exported(&send("undo", json!({}))), expected);
    assert_eq!(exported(&send("redo", json!({}))), changed);
}

#[test]
fn admitted_unknown_class_keeps_full_readonly_name_without_claiming_native_support() {
    let name = format!("Readonly unknown <&> 雪🧭\r\n{}", "r".repeat(4096));
    let mut raw = Document::empty().value().clone();
    raw["content"] = json!([{"id":"unknown-resource","eClass":"sysml:UnknownClass","data":{"elementId":"unknown0","declaredName":name}}]);
    let document = Document::from_json(&raw.to_string()).unwrap();
    assert!(document.inspect_node("unknown0").is_err());
    let expected = document.to_json().unwrap();
    initialize();
    assert_eq!(
        exported(&send("import", json!({"json":raw.to_string()}))),
        expected
    );
    let selected = send("select", json!({"value":"unknown0"}));
    assert!(!selected.contains("<form data-action=\"rename\""));
    let escaped = name
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\r', "&#13;");
    assert!(selected.contains(&format!("<label for=\"full-element-name\">Full element name (read only)</label><textarea id=\"full-element-name\" rows=\"2\" readonly>\n{escaped}</textarea>")));
    assert!(selected.contains("Properties are unavailable for this element type."));
    assert_eq!(exported(&selected), expected);
    assert_eq!(exported(&send("canvas_cancel", json!({}))), expected);
}

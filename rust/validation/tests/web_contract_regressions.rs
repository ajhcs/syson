// SPDX-License-Identifier: EPL-2.0
//! Independent native ABI checks of advertised UI actions; no browser claim.
use serde_json::{Value, json};
use syson_web::{dispatch, initialize, input_byte, input_reset, output_len, output_ptr};
fn html() -> String {
    let pointer = output_ptr() as *const u8;
    let length = output_len();
    // Public ABI guarantees validity until initialize/dispatch. Copy immediately
    // on this thread before any state-changing call; no pointer is retained.
    let bytes = unsafe { std::slice::from_raw_parts(pointer, length) };
    String::from_utf8(bytes.to_vec()).unwrap()
}
fn action(value: Value) -> String {
    input_reset();
    for byte in serde_json::to_vec(&value).unwrap() {
        input_byte(u32::from(byte));
    }
    dispatch();
    html()
}
fn import_and_select(class: &str) -> String {
    initialize();
    let document = json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"id":"r","eClass":class,"data":{"elementId":"selected","declaredName":"Subject"}}]});
    let imported = action(json!({"action":"import","fields":{"json":document.to_string()}}));
    assert!(!imported.contains("role=\"alert\""));
    action(json!({"action":"select","fields":{"value":"selected"}}))
}
#[test]
fn advertised_rename_succeeds_for_known_imported_literal_or_is_unavailable() {
    let rendered = import_and_select("sysml:LiteralInteger");
    if rendered.contains("data-action=\"rename\"") {
        let staged = action(json!({"action":"rename","fields":{"name":"Renamed"}}));
        assert!(
            !staged.contains("role=\"alert\""),
            "advertised rename failed"
        );
        let committed = action(json!({"action":"commit"}));
        assert!(!committed.contains("role=\"alert\""));
        assert!(committed.contains("Renamed"));
    }
}
#[test]
fn advertised_action_usage_parent_now_accepts_source_supported_creation() {
    let rendered = import_and_select("sysml:ActionUsage");
    assert!(
        rendered.contains("<option value=\"selected\">"),
        "source-supported parent missing from choices"
    );
    let staged = action(
        json!({"action":"stage","fields":{"kind":"package","name":"Child","parent":"selected"},"entropy":(0..96u32).collect::<Vec<_>>()}),
    );
    assert!(
        !staged.contains("role=\"alert\""),
        "advertised parent rejected bounded creation"
    );
    let committed = action(json!({"action":"commit"}));
    assert!(!committed.contains("role=\"alert\""));
    assert!(committed.contains("Child"));
}
fn exported_document(rendered: &str) -> Value {
    let start = rendered.find("<textarea id=\"export-json\"").unwrap();
    let content = &rendered[start..];
    let content = &content[content.find('>').unwrap() + 1..];
    let content = &content[..content.find("</textarea>").unwrap()];
    let decoded = content
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&");
    serde_json::from_str(&decoded).unwrap()
}
fn import_text(class: &str, attribute: &str, text: &str) -> String {
    initialize();
    let mut document = json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"id":"r","eClass":class,"data":{"elementId":"selected","declaredName":"Subject"}}]});
    document["content"][0]["data"][attribute] = json!(text);
    let rendered = action(json!({"action":"import","fields":{"json":document.to_string()}}));
    assert!(!rendered.contains("role=\"alert\""));
    action(json!({"action":"select","fields":{"value":"selected"}}))
}
#[test]
fn unchanged_normalized_textarea_submission_preserves_saved_crlf_cr_and_nul() {
    let original = "\r\nfirst\0second\rlast\r\n";
    let rendered = import_text("sysml:LiteralString", "value", original);
    assert!(rendered.contains("<textarea id=\"scalar-value\""));
    assert!(rendered.contains("&#13;\nfirst\u{fffd}second&#13;last&#13;\n</textarea>"));
    let staged = action(
        json!({"action":"attribute","fields":{"attribute":"value","mode":"value","value":"\nfirst\u{fffd}second\nlast\n"}}),
    );
    assert!(!staged.contains("role=\"alert\""));
    let committed = action(json!({"action":"commit"}));
    assert_eq!(
        exported_document(&committed)["content"][0]["data"]["value"],
        original
    );
}
#[test]
fn multiline_edits_and_unchanged_name_line_endings_survive_native_abi_roundtrip() {
    let rendered = import_text("sysml:Comment", "body", "\nfirst\nsecond\n");
    assert!(rendered.contains("scalar-state-body\">\n\nfirst\nsecond\n</textarea>"));
    let changed = "\nfirst\nchanged & <tag>\n";
    let staged = action(
        json!({"action":"attribute","fields":{"attribute":"body","mode":"value","value":changed}}),
    );
    assert!(!staged.contains("role=\"alert\""));
    let committed = action(json!({"action":"commit"}));
    assert_eq!(
        exported_document(&committed)["content"][0]["data"]["body"],
        changed
    );
    let name = "\nfirst\r\nsecond";
    import_text("sysml:ActionUsage", "declaredName", name);
    let staged = action(json!({"action":"rename","fields":{"name":"\nfirst\nsecond"}}));
    assert!(!staged.contains("role=\"alert\""));
    let committed = action(json!({"action":"commit"}));
    assert_eq!(
        exported_document(&committed)["content"][0]["data"]["declaredName"],
        name
    );
}

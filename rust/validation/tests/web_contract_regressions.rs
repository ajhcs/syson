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
fn advertised_creation_parent_accepts_bounded_create_or_is_unavailable() {
    let rendered = import_and_select("sysml:ActionUsage");
    if rendered.contains("<option value=\"selected\">") {
        let staged = action(
            json!({"action":"stage","fields":{"kind":"package","name":"Child","parent":"selected"},"entropy":(0..96u32).collect::<Vec<_>>()}),
        );
        assert!(
            !staged.contains("role=\"alert\""),
            "advertised parent rejected bounded creation"
        );
    }
}

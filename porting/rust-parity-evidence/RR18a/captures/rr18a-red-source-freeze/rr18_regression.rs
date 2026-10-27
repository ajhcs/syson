// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use syson_web::{dispatch, initialize, input_byte, input_reset, output_len, output_ptr};
const ESCAPED: &str = include_str!("../../../../porting/rust-parity-evidence/RR18a/fixtures/escaped-import.json");
fn html() -> String {
    // SAFETY: these public pointer/length exports are read synchronously on the
    // same thread; no initialize/dispatch occurs until after copying the bytes.
    let bytes = unsafe { std::slice::from_raw_parts(output_ptr() as *const u8, output_len()) };
    String::from_utf8(bytes.to_vec()).unwrap()
}
fn send(request: Value) -> String {
    let input = serde_json::to_vec(&request).unwrap();
    assert!(input.len() <= 1_048_576);
    input_reset(); for byte in input { input_byte(byte.into()); } dispatch(); html()
}
fn export(html: &str) -> &str {
    let start = html.find("<textarea id=\"export-json\"").expect("committed export remains reachable");
    let rest = &html[start..]; let value = &rest[rest.find('>').unwrap()+1..];
    &value[..value.find("</textarea>").unwrap()]
}
#[test]
fn expanding_import_preserves_the_prior_editor_export_and_undo_history() {
    initialize();
    send(json!({"action":"stage","fields":{"kind":"package","name":"Prior editor","parent":""},"entropy":(0..96).collect::<Vec<u8>>()}));
    let before = send(json!({"action":"commit"}));
    let expected = export(&before).to_owned();
    assert!(expected.contains("Prior editor"));
    let after = send(json!({"action":"import","fields":{"json":ESCAPED}}));
    assert_eq!(export(&after), expected, "preview refusal replaced the prior editor");
    let undone = send(json!({"action":"undo"}));
    assert!(!export(&undone).contains("Prior editor"), "prior undo history was lost");
}

// SPDX-License-Identifier: EPL-2.0
//! Actual accepted ABI import; fixture construction and transfer are unmeasured.
#[path = "support/rr18b3c_alloc.rs"]
mod allocation;
use allocation::observed;
use syson_model::Document;
use syson_web::{dispatch, initialize, input_byte, input_reset, output_len, output_ptr};
fn html() -> String {
    // SAFETY: copy synchronous public output before any subsequent ABI dispatch.
    let bytes = unsafe { std::slice::from_raw_parts(output_ptr() as *const u8, output_len()) };
    String::from_utf8(bytes.to_vec()).unwrap()
}
#[test]
fn accepted_opaque_escaped_export_avoids_repeated_full_escaped_copies() {
    let mut raw = Document::empty().value().clone();
    raw["opaque"] = serde_json::json!("&".repeat(700_000));
    let fixture = raw.to_string();
    assert!(fixture.len() < 1_048_576);
    let expected = Document::from_json(&fixture).unwrap().to_json().unwrap();
    let input =
        serde_json::to_vec(&serde_json::json!({"action":"import","fields":{"json":fixture}}))
            .unwrap();
    assert!(input.len() < 1_048_576);
    initialize();
    input_reset();
    for byte in input {
        input_byte(byte.into());
    }
    let (_, stats) = observed(|| dispatch());
    let result = html();
    eprintln!(
        "RR18B3C accepted opaque export: {stats:?}; html_bytes={}",
        result.len()
    );
    assert!(result.len() > 3_500_000 && result.len() < 16 * 1_048_576);
    assert!(result.contains("Imported a new document."));
    assert!(!result.contains("id=\"recovery-export\""));
    let exported = result
        .split_once("<textarea id=\"export-json\"")
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
        .replace("&amp;", "&");
    assert_eq!(exported, expected);
    assert!(
        stats.requested <= 32 * 1_048_576,
        "repeated escaped copies: requested {} bytes",
        stats.requested
    );
}

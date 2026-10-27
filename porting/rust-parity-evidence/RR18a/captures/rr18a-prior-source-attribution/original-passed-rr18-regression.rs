// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use syson_web::{dispatch, initialize, input_byte, input_reset, output_len, output_ptr};
const ESCAPED: &str =
    include_str!("../../../../porting/rust-parity-evidence/RR18a/fixtures/escaped-import.json");
fn html() -> String {
    // SAFETY: these public pointer/length exports are read synchronously on the
    // same thread; no initialize/dispatch occurs until after copying the bytes.
    let bytes = unsafe { std::slice::from_raw_parts(output_ptr() as *const u8, output_len()) };
    String::from_utf8(bytes.to_vec()).unwrap()
}
fn send(request: Value) -> String {
    let input = serde_json::to_vec(&request).unwrap();
    assert!(input.len() <= 1_048_576);
    input_reset();
    for byte in input {
        input_byte(byte.into());
    }
    dispatch();
    html()
}
fn export(html: &str) -> &str {
    let start = html
        .find("<textarea id=\"export-json\"")
        .expect("committed export remains reachable");
    let rest = &html[start..];
    let value = &rest[rest.find('>').unwrap() + 1..];
    &value[..value.find("</textarea>").unwrap()]
}
#[test]
fn expanding_import_preserves_the_prior_editor_export_and_undo_history() {
    initialize();
    send(
        json!({"action":"stage","fields":{"kind":"package","name":"Prior editor","parent":""},"entropy":(0..96).collect::<Vec<u8>>()}),
    );
    let before = send(json!({"action":"commit"}));
    let expected = export(&before).to_owned();
    assert!(expected.contains("Prior editor"));
    let after = send(json!({"action":"import","fields":{"json":ESCAPED}}));
    assert_eq!(
        export(&after),
        expected,
        "preview refusal replaced the prior editor"
    );
    let undone = send(json!({"action":"undo"}));
    assert!(
        !export(&undone).contains("Prior editor"),
        "prior undo history was lost"
    );
}

fn committed() -> String {
    initialize();
    send(
        json!({"action":"stage","fields":{"kind":"package","name":"Prior editor","parent":""},"entropy":(0..96).collect::<Vec<u8>>()}),
    );
    send(json!({"action":"commit"}))
}
fn selected(html: &str) -> String {
    let key = "data-action=\"select\" data-value=\"";
    let value = &html[html.find(key).unwrap() + key.len()..];
    value[..value.find('"').unwrap()].into()
}
fn stage_expansion(before: &str) -> String {
    send(json!({"action":"select","fields":{"value":selected(before)}}));
    send(json!({"action":"rename","fields":{"name":"&".repeat(700_000)}}))
}
fn download(html: &str) -> String {
    let key = "href=\"data:application/json;charset=utf-8,";
    let value = &html[html.find(key).expect("safe download remains reachable") + key.len()..];
    let value = &value[..value.find('"').unwrap()];
    assert_eq!(value.len() % 3, 0);
    let bytes: Vec<_> = value
        .as_bytes()
        .chunks_exact(3)
        .map(|chunk| {
            assert_eq!(chunk[0], b'%');
            u8::from_str_radix(std::str::from_utf8(&chunk[1..]).unwrap(), 16).unwrap()
        })
        .collect();
    String::from_utf8(bytes).unwrap()
}
fn unescape(text: &str) -> String {
    text.replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}
#[test]
fn staged_preview_fallback_exports_committed_bytes_and_recovers_without_losing_history() {
    let before = committed();
    let fallback = stage_expansion(&before);
    assert!(fallback.len() < 16 * 1_048_576);
    assert_eq!(download(&fallback), unescape(export(&before)));
    let recovered = send(json!({"action":"recover_preview"}));
    assert_eq!(export(&recovered), export(&before));
    assert!(recovered.contains("data-inspector-focus=\"name\""));
    assert!(!export(&send(json!({"action":"undo"}))).contains("Prior editor"));
}
#[test]
fn discard_requires_confirmation_and_keep_document_preserves_committed_bytes() {
    let before = committed();
    stage_expansion(&before);
    assert_eq!(
        download(&send(json!({"action":"confirm_discard"}))),
        unescape(export(&before))
    );
    let armed = send(json!({"action":"discard_document"}));
    assert!(armed.contains("data-action=\"confirm_discard\""));
    let kept = send(json!({"action":"cancel_discard"}));
    assert_eq!(download(&kept), unescape(export(&before)));
    assert!(!kept.contains("data-action=\"confirm_discard\""));
    send(json!({"action":"discard_document"}));
    let discarded = send(json!({"action":"confirm_discard"}));
    assert!(!export(&discarded).contains("Prior editor"));
    assert!(send(json!({"action":"undo"})).contains("No history available."));
}
#[test]
fn accepted_import_still_clears_history_and_preserves_its_document() {
    committed();
    let raw = include_str!("../../../../porting/rust-parity-evidence/RR18a/fixtures/before.json");
    let imported = send(json!({"action":"import","fields":{"json":raw}}));
    let expected = syson_model::Document::from_json(raw)
        .unwrap()
        .to_json()
        .unwrap();
    assert_eq!(unescape(export(&imported)), expected);
    assert!(send(json!({"action":"undo"})).contains("No history available."));
}
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
thread_local! { static TRACK: Cell<bool> = const { Cell::new(false) }; static ALLOCS: Cell<usize> = const { Cell::new(0) }; }
struct Observed;
fn record() {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = ALLOCS.try_with(|n| n.set(n.get() + 1));
    }
}
// SAFETY: every pointer/layout passes unchanged to System; constant TLS does not allocate.
unsafe impl GlobalAlloc for Observed {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            record();
        }
        p
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            record();
        }
        p
    }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let p = unsafe { System.realloc(p, layout, size) };
        if !p.is_null() {
            record();
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        unsafe { System.dealloc(p, layout) };
    }
}
#[global_allocator]
static ALLOCATOR: Observed = Observed;
#[test]
fn absent_canvas_frames_do_not_serialize_or_allocate_a_model_copy() {
    let doc = syson_model::Document::from_json(ESCAPED).unwrap();
    let canvas = syson_web::layout_controller::Canvas::default();
    ALLOCS.set(0);
    TRACK.set(true);
    let scene = canvas.scene(&doc);
    let export = canvas.export(&doc);
    TRACK.set(false);
    assert!(scene.is_none() && export.is_empty());
    assert_eq!(ALLOCS.get(), 0, "absent canvas serialized or allocated");
}

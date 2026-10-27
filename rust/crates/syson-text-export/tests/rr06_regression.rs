// SPDX-License-Identifier: EPL-2.0
//! Allocation bookkeeping is confined to this development-only test executable.
use serde_json::Value;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::{Arc, atomic::AtomicBool};
use syson_model::{Document, Editor, NodeIds};
use syson_text_export::{Error, Newline, Options, export};
use syson_text_import::Importer;

const LIMIT: usize = 65_536;
#[derive(Clone, Copy, Debug, Default)]
struct Allocations {
    live: isize,
    peak: isize,
    largest: usize,
}
thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static STATS: Cell<Allocations> = const { Cell::new(Allocations {
        live: 0, peak: 0, largest: 0,
    }) };
}
fn record(change: isize, requested: usize) {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = STATS.try_with(|cell| {
            let mut stats = cell.get();
            stats.live += change;
            stats.peak = stats.peak.max(stats.live);
            stats.largest = stats.largest.max(requested);
            cell.set(stats);
        });
    }
}
struct TrackingAllocator;
// SAFETY: Every operation delegates the original pointer/layout to System.
// Constant TLS bookkeeping allocates nothing and never changes the allocation.
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record(layout.size() as isize, layout.size());
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            record(layout.size() as isize, layout.size());
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        record(-(layout.size() as isize), 0);
        unsafe { System.dealloc(pointer, layout) };
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(pointer, layout, size) };
        if !next.is_null() {
            record(size as isize - layout.size() as isize, size);
        }
        next
    }
}
#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;
fn observed_export(document: &Document, options: Options) -> (Result<String, Error>, Allocations) {
    STATS.set(Allocations::default());
    TRACK.set(true);
    let output = export(document, options);
    TRACK.set(false);
    (output, STATS.get())
}
fn import(source: &str) -> Document {
    let mut editor = Editor::new(Document::empty());
    let ids = (1..=1024).map(|n| {
        NodeIds::new(
            &format!("30000000-0000-4000-8000-{n:012x}"),
            &format!("40000000-0000-4000-8000-{n:012x}"),
        )
        .unwrap()
    });
    Importer::pinned()
        .unwrap()
        .stage(
            editor.begin(),
            source,
            None,
            ids,
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap()
        .commit(&mut editor)
        .unwrap();
    editor.document().clone()
}
fn replace_name(document: &Document, old: &str, name: &str) -> Document {
    fn edit(value: &mut Value, old: &str, name: &str) {
        match value {
            Value::Object(fields) => {
                if fields.get("declaredName").and_then(Value::as_str) == Some(old) {
                    fields.insert("declaredName".into(), Value::String(name.into()));
                }
                for child in fields.values_mut() {
                    edit(child, old, name);
                }
            }
            Value::Array(values) => {
                for child in values {
                    edit(child, old, name);
                }
            }
            _ => {}
        }
    }
    let mut value = document.value().clone();
    edit(&mut value, old, name);
    Document::from_json(&value.to_string()).unwrap()
}
fn root_source(bytes: usize, newline: &str, quoted: bool) -> String {
    // Each individual raw name stays below the real F13 2048-byte limit.
    let count = 33;
    let quotes = usize::from(quoted) * 2;
    let names = bytes - count * (9 + quotes) - (count - 1) * newline.len();
    let mut source = String::new();
    for index in 0..count {
        if index != 0 {
            source.push_str(newline);
        }
        let length = names / count + usize::from(index < names % count);
        let prefix = if quoted {
            format!("轮{index:02}_")
        } else {
            format!("P{index:02}_")
        };
        let name = prefix.clone() + &"x".repeat(length - prefix.len());
        assert!(name.len() + quotes <= 2048);
        source.push_str("package ");
        if quoted {
            source.push('\'');
        }
        source.push_str(&name);
        if quoted {
            source.push('\'');
        }
        source.push(';');
    }
    assert_eq!(source.len(), bytes);
    source
}
fn assert_roundtrip(source: &str, options: Options) {
    let document = import(source);
    let before = document.clone();
    let output = export(&document, options).unwrap();
    assert_eq!(output, source);
    assert_eq!(output.len(), source.len());
    assert_eq!(export(&document, options).unwrap(), output);
    assert_eq!(document, before);
    let reloaded = import(&output);
    assert_eq!(export(&reloaded, options).unwrap(), output);
}
#[test]
fn oversized_name_refuses_before_name_or_output_allocation() {
    let small = import("package P;");
    let document = replace_name(&small, "P", &"A".repeat(2 * 1024 * 1024));
    let before = document.clone();
    // Input construction/cloning is outside tracking. A refused export owns no
    // retained result; this fixture creates/frees its scratch on this thread.
    for _ in 0..2 {
        let (result, stats) = observed_export(&document, Options::default());
        assert_eq!(result, Err(Error::Limit));
        assert!(stats.largest <= LIMIT, "{stats:?}");
        assert!(stats.peak <= LIMIT as isize, "{stats:?}");
        assert_eq!(stats.live, 0, "{stats:?}");
    }
    assert_eq!(document, before);
}
#[test]
fn exact_and_near_limit_root_packages_pass_actual_f14_roundtrip() {
    for bytes in [LIMIT - 1, LIMIT] {
        assert_roundtrip(&root_source(bytes, "\n", false), Options::default());
    }
}
#[test]
fn quoted_unicode_and_crlf_are_counted_as_bytes_at_the_exact_limit() {
    for quoted in [false, true] {
        assert_roundtrip(
            &root_source(LIMIT, "\r\n", quoted),
            Options {
                newline: Newline::CrLf,
                indentation: "\t",
            },
        );
    }
}
#[test]
fn one_extra_name_byte_and_crlf_growth_refuse_before_output_allocation() {
    let document = import(&root_source(LIMIT, "\n", false));
    let last = document.value()["content"][32]["data"]["declaredName"]
        .as_str()
        .unwrap();
    let changed = replace_name(&document, last, &(last.to_owned() + "x"));
    for (candidate, newline) in [(&changed, Newline::Lf), (&document, Newline::CrLf)] {
        let before = candidate.clone();
        let (result, stats) = observed_export(
            candidate,
            Options {
                newline,
                indentation: "    ",
            },
        );
        assert_eq!(result, Err(Error::Limit));
        // The aggregate count fails before allocating even a 64KiB output.
        assert!(stats.largest < LIMIT / 2, "{stats:?}");
        assert_eq!(stats.live, 0, "{stats:?}");
        assert_eq!(*candidate, before);
    }
}
#[test]
fn indentation_and_nested_closing_lines_fit_exactly_before_reimport() {
    let count = 32;
    let fixed = "package P {".len() + 1 + (count + 1) * 2 + count * (8 + 9 + 1);
    let names = LIMIT - fixed;
    let mut source = String::from("package P {");
    for index in 0..count {
        let prefix = format!("D{index:02}_");
        let length = names / count + usize::from(index < names % count);
        source.push_str("\r\n        part def ");
        source.push_str(&prefix);
        source.push_str(&"x".repeat(length - prefix.len()));
        source.push(';');
    }
    source.push_str("\r\n}");
    assert_eq!(source.len(), LIMIT);
    assert_roundtrip(
        &source,
        Options {
            newline: Newline::CrLf,
            indentation: "        ",
        },
    );
}
#[test]
fn repeated_qualified_segments_overflow_without_owned_candidates_or_output() {
    let mut source = String::from("package A {package B {part def C;}} package Consumer {");
    for index in 0..16 {
        source.push_str(&format!("part u{index}:A::B::C;"));
    }
    source.push('}');
    let document = import(&source);
    let document = replace_name(&document, "A", &"A".repeat(2040));
    let document = replace_name(&document, "B", &"B".repeat(2040));
    let before = document.clone();
    let (result, stats) = observed_export(&document, Options::default());
    assert_eq!(result, Err(Error::Limit));
    assert!(stats.largest < LIMIT / 2, "{stats:?}");
    assert_eq!(stats.live, 0, "{stats:?}");
    assert_eq!(document, before);
}
#[test]
fn borrowed_qualified_names_keep_shortest_selection_and_unicode() {
    let document = import(
        "package '供应库' {package 'Road parts' {part def '车轮';}} \
         package Consumer {part wheel:'供应库'::'Road parts'::'车轮';}",
    );
    let output = export(&document, Options::default()).unwrap();
    assert!(output.contains("part wheel : '供应库'::'Road parts'::'车轮';"));
    assert_eq!(
        export(&import(&output), Options::default()).unwrap(),
        output
    );
    let document = import("package P {part def T;part wheel:P::T;}");
    assert!(
        export(&document, Options::default())
            .unwrap()
            .contains("part wheel : T;")
    );
}
#[test]
fn rr07_reserved_word_and_lossy_escape_policy_remain_unchanged() {
    let document = import("package P;");
    for name in ["class", "true"] {
        assert!(matches!(
            export(&replace_name(&document, "P", name), Options::default()),
            Err(Error::NotImportable(_))
        ));
    }
    for name in ["a'b", "a\\b", "a\nb"] {
        assert_eq!(
            export(&replace_name(&document, "P", name), Options::default()),
            Err(Error::NonRoundTripName)
        );
    }
}

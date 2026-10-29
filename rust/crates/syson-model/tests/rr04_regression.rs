// SPDX-License-Identifier: EPL-2.0
//! Allocation bookkeeping belongs only to this development test executable.
use serde_json::{Value, json};
use std::io::{self, Write};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};
use syson_json_value::{Error as Admission, Limit};
use syson_model::{ContainmentFeature, Document, Editor, ModelError};

#[derive(Clone, Copy, Default, Debug)]
struct Allocations {
    live: isize,
    peak: isize,
    largest: usize,
}
thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static STATS: Cell<Allocations> = const { Cell::new(Allocations { live: 0, peak: 0, largest: 0 }) };
}
fn record(delta: isize, requested: usize) {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = STATS.try_with(|cell| {
            let mut stats = cell.get();
            stats.live += delta;
            stats.peak = stats.peak.max(stats.live);
            stats.largest = stats.largest.max(requested);
            cell.set(stats);
        });
    }
}
struct ObservedAllocator;
// SAFETY: Identical pointers/layouts are delegated to System. Constant TLS
// bookkeeping allocates nothing and does not change memory ownership.
unsafe impl GlobalAlloc for ObservedAllocator {
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
static ALLOCATOR: ObservedAllocator = ObservedAllocator;
fn id(n: u32) -> String {
    format!("a0000000-0000-4000-8000-{n:012x}")
}
fn node(n: u32, class: &str, extra: Value) -> Value {
    let mut data = extra.as_object().unwrap().clone();
    data.insert("elementId".into(), Value::String(id(n)));
    json!({"id":id(n+100),"eClass":format!("sysml:{class}"),"data":data})
}
fn base() -> Document {
    Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[node(1,"Package",json!({"declaredName":"base"}))]}).to_string()).unwrap()
}
#[test]
fn dense_provisional_export_refuses_before_the_old_16mib_output_allocation() {
    let editor = Editor::new(base());
    let original = editor.document().clone();
    let mut opaque = Value::Array(vec![Value::Null; 100_000]);
    for _ in 0..60 {
        opaque = Value::Array(vec![opaque]);
    }
    let fragment = node(2, "OwningMembership", json!({"opaque":opaque}));
    let mut tx = editor.begin();
    tx.attach_preallocated_subtree(&id(1), ContainmentFeature::OwnedRelationship, fragment)
        .unwrap();
    assert!(serde_json::to_vec(tx.document().value()).unwrap().len() < 1_048_576);
    // Constructing the actual provisional transaction and all input values is
    // outside tracking. Old to_json grows a roughly16MiB pretty String here.
    STATS.set(Allocations::default());
    TRACK.set(true);
    let result = tx.document().to_json();
    TRACK.set(false);
    let stats = STATS.get();
    eprintln!(
        "dense provisional export: {stats:?}; success_bytes={:?}",
        result.as_ref().ok().map(String::len)
    );
    assert!(
        result.is_err(),
        "provisional output must obey the normal4MiB bound"
    );
    assert!(stats.largest <= 1_048_576, "{stats:?}");
    assert!(stats.peak <= 2_097_152, "{stats:?}");
    assert_eq!(stats.live, 0, "{stats:?}");
    assert_eq!(editor.document(), &original);
    tx.cancel();
}

fn observed<T>(action: impl FnOnce() -> T) -> (T, Allocations) {
    STATS.set(Allocations::default());
    TRACK.set(true);
    let output = action();
    TRACK.set(false);
    (output, STATS.get())
}
fn admission<T>(actual: Result<T, ModelError>, expected: Admission) {
    assert!(matches!(actual, Err(ModelError::JsonAdmission(error)) if error == expected));
}
const WIRE: &str = r#"{
  "content": [],
  "json": {
    "encoding": "utf-8",
    "version": "1.0"
  },
  "ns": {
    "sysml": "http://www.eclipse.org/syson/sysml"
  }
}"#;
#[test]
fn exact_count_owned_and_external_output_share_the_legacy_canonical_bytes() {
    let document = Document::empty();
    assert_eq!(
        document.json_size(WIRE.len(), || false).unwrap(),
        WIRE.len()
    );
    assert_eq!(
        document.to_json_bounded(WIRE.len(), || false).unwrap(),
        WIRE
    );
    assert_eq!(document.to_json().unwrap(), WIRE);
    let mut output = Vec::new();
    assert_eq!(
        document
            .write_json(&mut output, WIRE.len(), || false)
            .unwrap(),
        WIRE.len()
    );
    assert_eq!(output, WIRE.as_bytes());
    let mut untouched = b"existing prefix".to_vec();
    admission(
        document.write_json(&mut untouched, WIRE.len() - 1, || false),
        Admission::Limit(Limit::Bytes),
    );
    assert_eq!(untouched, b"existing prefix");
    admission(
        document.json_size(WIRE.len() - 1, || false),
        Admission::Limit(Limit::Bytes),
    );
    admission(
        document.to_json_bounded(WIRE.len() - 1, || false),
        Admission::Limit(Limit::Bytes),
    );
}
#[test]
fn utf8_escape_expansion_marker_objects_and_numeric_lexemes_remain_authoritative_values() {
    let raw = r#"{"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[],"vendor":"\u0000\n\"\\é🚗","n":-0,"exp":1E+000,"marker":{"$serde_json::private::Number":"123"}}"#;
    let document = Document::from_json(raw).unwrap();
    let before = document.clone();
    let expected = serde_json::to_vec_pretty(document.value()).unwrap();
    assert!(expected.len() > raw.len());
    let size = document.json_size(expected.len(), || false).unwrap();
    assert_eq!(size, expected.len());
    let owned = document.to_json_bounded(size, || false).unwrap();
    assert_eq!(owned.as_bytes(), expected);
    assert!(owned.contains(r#""n": -0"#));
    assert!(owned.contains(r#""exp": 1E+000"#));
    assert!(owned.contains(r#""vendor": "\u0000\n\"\\é🚗""#));
    assert_eq!(Document::from_json(&owned).unwrap(), document);
    admission(
        document.to_json_bounded(size - 1, || false),
        Admission::Limit(Limit::Bytes),
    );
    assert_eq!(document, before);
}
#[test]
fn count_allocates_no_output_and_owned_reservation_is_exact_at_the_publication_cap() {
    let mut value = Document::empty().value().clone();
    value["padding"] = Value::String(String::new());
    let overhead = serde_json::to_vec_pretty(&value).unwrap().len();
    let limit = 4 * 1024 * 1024;
    value["padding"] = Value::String("x".repeat(limit - overhead));
    let document = Document::from_json(&value.to_string()).unwrap();
    let (count, counted) = observed(|| document.json_size(limit, || false));
    assert_eq!(count.unwrap(), limit);
    assert!(counted.largest < 1024, "{counted:?}");
    assert_eq!(counted.live, 0, "{counted:?}");
    let (owned, allocated) = observed(|| document.to_json_bounded(limit, || false));
    let owned = owned.unwrap();
    assert_eq!(owned.len(), limit);
    assert_eq!(allocated.largest, limit, "{allocated:?}");
    assert_eq!(allocated.live, limit as isize, "{allocated:?}");
    assert!(allocated.peak <= limit as isize + 1024, "{allocated:?}");
    assert_eq!(Document::from_json(&owned).unwrap(), document);
}
#[test]
fn invalid_budgets_and_initial_cancellation_touch_no_external_writer() {
    let document = base();
    for limit in [4 * 1024 * 1024 + 1, usize::MAX] {
        let mut output = b"untouched".to_vec();
        admission(
            document.write_json(&mut output, limit, || false),
            Admission::InvalidLimits,
        );
        admission(
            document.to_json_bounded(limit, || false),
            Admission::InvalidLimits,
        );
        assert_eq!(output, b"untouched");
    }
    let mut output = b"untouched".to_vec();
    admission(
        document.write_json(&mut output, 4 * 1024 * 1024, || true),
        Admission::Cancelled,
    );
    assert_eq!(output, b"untouched");
}
struct PartialSink {
    bytes: Vec<u8>,
    chunk: usize,
    fail_after: usize,
    interrupts: usize,
    flushes: usize,
}
impl Write for PartialSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.interrupts != 0 {
            self.interrupts -= 1;
            return Err(io::ErrorKind::Interrupted.into());
        }
        if self.bytes.len() == self.fail_after {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        let count = bytes
            .len()
            .min(self.chunk)
            .min(self.fail_after - self.bytes.len());
        self.bytes.extend_from_slice(&bytes[..count]);
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.flushes += 1;
        Err(io::Error::other("unexpected flush"))
    }
}
#[test]
fn external_short_writes_and_interrupted_retries_preserve_exact_output_without_flush() {
    let mut sink = PartialSink {
        bytes: vec![],
        chunk: 1,
        fail_after: usize::MAX,
        interrupts: 2,
        flushes: 0,
    };
    let count = Document::empty()
        .write_json(&mut sink, WIRE.len(), || false)
        .unwrap();
    assert_eq!(count, WIRE.len());
    assert_eq!(sink.bytes, WIRE.as_bytes());
    assert_eq!(sink.interrupts, 0);
    assert_eq!(sink.flushes, 0);
}
#[test]
fn external_failure_reports_acknowledged_prefix_and_does_not_mutate_document() {
    let document = Document::empty();
    let before = document.clone();
    let mut sink = PartialSink {
        bytes: vec![],
        chunk: 2,
        fail_after: 17,
        interrupts: 0,
        flushes: 0,
    };
    assert!(matches!(
        document.write_json(&mut sink, WIRE.len(), || false),
        Err(ModelError::JsonOutputIo {
            kind: io::ErrorKind::BrokenPipe,
            written: 17
        })
    ));
    assert_eq!(sink.bytes, WIRE.as_bytes()[..17]);
    assert_eq!(sink.flushes, 0);
    assert_eq!(document, before);
}
#[test]
fn cancellation_precedes_owned_reservation_and_discards_any_late_private_output() {
    let mut value = Document::empty().value().clone();
    value["padding"] = Value::String("x".repeat(65_536));
    let document = Document::from_json(&value.to_string()).unwrap();
    let mut count_polls = 0;
    let size = document
        .json_size(4 * 1024 * 1024, || {
            count_polls += 1;
            false
        })
        .unwrap();
    let mut polls = 0;
    let (result, stats) = observed(|| {
        document.to_json_bounded(size, || {
            polls += 1;
            polls > count_polls
        })
    });
    admission(result, Admission::Cancelled);
    assert!(stats.largest < 1024, "{stats:?}");
    assert_eq!(stats.live, 0, "{stats:?}");
    let mut polls = 0;
    let (result, stats) = observed(|| {
        document.to_json_bounded(size, || {
            polls += 1;
            polls > count_polls + 4
        })
    });
    admission(result, Admission::Cancelled);
    assert_eq!(stats.live, 0, "{stats:?}");
    assert_eq!(document.value()["padding"].as_str().unwrap().len(), 65_536);
}
#[test]
fn interrupted_io_retries_poll_cancellation_instead_of_hiding_a_retry_loop() {
    struct Interrupted {
        calls: usize,
    }
    impl Write for Interrupted {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            // A regression in cancellation fails finitely instead of hanging.
            Err(if self.calls < 100 {
                io::ErrorKind::Interrupted
            } else {
                io::ErrorKind::BrokenPipe
            }
            .into())
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("no flush")
        }
    }
    let document = Document::empty();
    let mut preflight = 0;
    document
        .json_size(WIRE.len(), || {
            preflight += 1;
            false
        })
        .unwrap();
    let mut polls = 0;
    let mut sink = Interrupted { calls: 0 };
    admission(
        document.write_json(&mut sink, WIRE.len(), || {
            polls += 1;
            polls > preflight + 5
        }),
        Admission::Cancelled,
    );
    assert!(sink.calls > 0 && sink.calls < 100);
}
#[test]
fn late_external_cancellation_can_leave_a_prefix_and_broken_sinks_have_typed_errors() {
    struct Cancels<'a> {
        bytes: Vec<u8>,
        flag: &'a Cell<bool>,
    }
    impl Write for Cancels<'_> {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(bytes);
            self.flag.set(true);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("no flush")
        }
    }
    let flag = Cell::new(false);
    let mut sink = Cancels {
        bytes: vec![],
        flag: &flag,
    };
    admission(
        Document::empty().write_json(&mut sink, WIRE.len(), || flag.get()),
        Admission::Cancelled,
    );
    assert!(!sink.bytes.is_empty() && sink.bytes.len() < WIRE.len());
    assert_eq!(sink.bytes, WIRE.as_bytes()[..sink.bytes.len()]);
    struct Invalid(bool);
    impl Write for Invalid {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            Ok(if self.0 { 0 } else { bytes.len() + 1 })
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("no flush")
        }
    }
    for (zero, expected) in [
        (true, io::ErrorKind::WriteZero),
        (false, io::ErrorKind::InvalidData),
    ] {
        assert!(
            matches!(Document::empty().write_json(&mut Invalid(zero), WIRE.len(), || false), Err(ModelError::JsonOutputIo { kind, written: 0 }) if kind == expected)
        );
    }
    struct MutatesThenErr(Vec<u8>);
    impl Write for MutatesThenErr {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            self.0.extend_from_slice(b"side effect");
            Err(io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> io::Result<()> {
            panic!("no flush")
        }
    }
    let mut dishonest = MutatesThenErr(vec![]);
    assert!(matches!(
        Document::empty().write_json(&mut dishonest, WIRE.len(), || false),
        Err(ModelError::JsonOutputIo {
            kind: io::ErrorKind::BrokenPipe,
            written: 0
        })
    ));
    assert_eq!(dishonest.0, b"side effect"); // Acknowledged bytes are not physical side-effect proof.
}

#[test]
fn provisional_depth_is_refused_before_external_output_without_semantic_bypass() {
    let editor = Editor::new(base());
    let mut opaque = Value::Null;
    for _ in 0..128 {
        opaque = Value::Array(vec![opaque]);
    }
    let mut tx = editor.begin();
    tx.attach_preallocated_subtree(
        &id(1),
        ContainmentFeature::OwnedRelationship,
        node(2, "OwningMembership", json!({"opaque":opaque})),
    )
    .unwrap();
    let mut output = b"untouched".to_vec();
    admission(
        tx.document().json_size(4 * 1024 * 1024, || false),
        Admission::Limit(Limit::Depth),
    );
    admission(
        tx.document().to_json_bounded(4 * 1024 * 1024, || false),
        Admission::Limit(Limit::Depth),
    );
    admission(
        tx.document()
            .write_json(&mut output, 4 * 1024 * 1024, || false),
        Admission::Limit(Limit::Depth),
    );
    assert_eq!(output, b"untouched");
    assert_eq!(editor.document(), &base());
    tx.cancel();
}

#[test]
fn cancellation_after_the_last_acknowledged_write_can_leave_complete_external_output() {
    struct CompleteThenCancels<'a> {
        bytes: Vec<u8>,
        acknowledged: usize,
        flag: &'a Cell<bool>,
        flushes: usize,
    }
    impl Write for CompleteThenCancels<'_> {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(bytes);
            self.acknowledged += bytes.len();
            assert!(self.acknowledged <= WIRE.len());
            if self.acknowledged == WIRE.len() {
                self.flag.set(true);
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }
    let document = Document::empty();
    let before = document.clone();
    let flag = Cell::new(false);
    let mut sink = CompleteThenCancels {
        bytes: vec![],
        acknowledged: 0,
        flag: &flag,
        flushes: 0,
    };
    admission(
        document.write_json(&mut sink, WIRE.len(), || flag.get()),
        Admission::Cancelled,
    );
    assert!(flag.get());
    assert_eq!(sink.acknowledged, WIRE.len());
    assert_eq!(sink.bytes, WIRE.as_bytes());
    assert_eq!(sink.flushes, 0);
    assert_eq!(document, before);
}

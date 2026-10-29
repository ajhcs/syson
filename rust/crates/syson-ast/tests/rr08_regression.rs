// SPDX-License-Identifier: EPL-2.0
//! Development-only requested-allocation oracle, not application instrumentation.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::{Cell, RefCell},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use syson_ast::{Assignment, Builder, Datum, Error, Lexeme, Limits, Reflection, Slot, Value};
#[derive(Clone, Copy, Debug, Default)]
struct Requests {
    total: usize,
    largest: usize,
}
thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static CANCEL_AT: Cell<usize> = const { Cell::new(0) };
    static CANCEL: RefCell<Option<Arc<AtomicBool>>> = const { RefCell::new(None) };
    static REQUESTS: Cell<Requests> = const { Cell::new(Requests { total: 0, largest: 0 }) };
}
fn record(size: usize) {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = REQUESTS.try_with(|cell| {
            let mut requests = cell.get();
            requests.total = requests.total.saturating_add(size);
            requests.largest = requests.largest.max(size);
            cell.set(requests);
        });
        if CANCEL_AT
            .try_with(Cell::get)
            .is_ok_and(|threshold| threshold != 0 && size >= threshold)
        {
            let _ = CANCEL.try_with(|target| {
                if let Ok(target) = target.try_borrow() {
                    if let Some(flag) = target.as_ref() {
                        flag.store(true, Ordering::Relaxed);
                    }
                }
            });
        }
    }
}
struct Observed;
// SAFETY: System receives each original pointer/layout unchanged. Constant TLS
// bookkeeping allocates nothing. Full successful realloc requests count again;
// this is allocation traffic, not net live memory, peak memory, RSS or latency.
unsafe impl GlobalAlloc for Observed {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record(layout.size());
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            record(layout.size());
        }
        pointer
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(pointer, layout, size) };
        if !next.is_null() {
            record(size);
        }
        next
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }
}
#[global_allocator]
static ALLOCATOR: Observed = Observed;
fn builder(limits: Limits) -> Builder {
    let mut reflection = Reflection::default();
    reflection.declare("Element".into(), vec![]).unwrap();
    Builder::new(reflection, limits, Arc::new(AtomicBool::new(false))).unwrap()
}
fn string(text: &str) -> Datum {
    Datum::new(Value::String(text.into()))
}
fn assign(builder: &mut Builder, node: syson_ast::NodeId, key: &str, value: Datum) {
    builder.assign(node, key, Assignment::Set, value).unwrap();
}
fn cancellation_window(
    flag: &Arc<AtomicBool>,
    call: impl FnOnce() -> Result<(), Error>,
) -> (Result<(), Error>, Requests) {
    // The outside owner and installed clone exist before the callback runs.
    // No Arc allocation/clone/drop or TLS mutation occurs in that callback.
    CANCEL.with(|target| *target.borrow_mut() = Some(flag.clone()));
    CANCEL_AT.set(65_536);
    REQUESTS.set(Requests::default());
    TRACK.set(true);
    let result = call();
    TRACK.set(false);
    CANCEL_AT.set(0);
    CANCEL.with(|target| *target.borrow_mut() = None);
    (result, REQUESTS.get())
}
#[test]
fn merge_keeps_source_first_entries_target_lexeme_and_rolls_back_foreign_failure() {
    let mut b = builder(Limits::default());
    let target = b.allocate(Some("Element"), None).unwrap();
    let source = b.allocate(Some("Element"), None).unwrap();
    let lexeme = Lexeme::from_source("'😀'", 0, 6).unwrap();
    let target_data = Datum {
        value: Value::Array(vec![string("target")]),
        lexeme: Some(lexeme.clone()),
    };
    assign(&mut b, target, "items", target_data);
    assign(
        &mut b,
        source,
        "items",
        Datum::new(Value::Array(vec![string("source")])),
    );
    b.merge(target, source).unwrap();
    assert_eq!(
        b.node(target).unwrap().slot("items"),
        Slot::Present(&Datum {
            value: Value::Array(vec![string("source"), string("target")]),
            lexeme: Some(lexeme),
        })
    );
    let mut foreign = builder(Limits::default());
    let foreign_id = foreign.allocate(Some("Element"), None).unwrap();
    let result: Result<(), Error> = b.transaction(|b| {
        b.assign(
            target,
            "items",
            Assignment::Append,
            string("must disappear"),
        )?;
        b.merge(target, foreign_id)
    });
    assert_eq!(result, Err(Error::ForeignOwner(foreign_id)));
    let Slot::Present(datum) = b.node(target).unwrap().slot("items") else {
        panic!()
    };
    assert_eq!(
        datum.value,
        Value::Array(vec![string("source"), string("target")])
    );
}
#[test]
fn kind_and_unknown_type_copies_refuse_before_allocation_or_id_reservation() {
    let kind = "K".repeat(65_536);
    let mut reflection = Reflection::default();
    reflection.declare(kind.clone(), vec![]).unwrap();
    reflection.declare("Element".into(), vec![]).unwrap();
    let mut bounded = Builder::new(
        reflection.clone(),
        Limits {
            max_work: 32,
            ..Limits::default()
        },
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    REQUESTS.set(Requests::default());
    TRACK.set(true);
    let result = bounded.allocate(Some(&kind), None);
    TRACK.set(false);
    assert_eq!(result, Err(Error::Budget));
    assert_eq!(REQUESTS.get().largest, 0);
    // Even the UnknownType payload copy must fit before constructing its error.
    let mut unknown = builder(Limits {
        max_work: 32,
        ..Limits::default()
    });
    REQUESTS.set(Requests::default());
    TRACK.set(true);
    let result = unknown.allocate(Some(&kind), None);
    TRACK.set(false);
    assert_eq!(result, Err(Error::Budget));
    assert_eq!(REQUESTS.get().largest, 0);
    let flag = Arc::new(AtomicBool::new(false));
    let mut b = Builder::new(reflection, Limits::default(), flag.clone()).unwrap();
    let (result, stats) = cancellation_window(&flag, || b.allocate(Some(&kind), None).map(|_| ()));
    assert_eq!(result, Err(Error::Cancelled));
    assert_eq!(stats.largest, kind.len());
    flag.store(false, Ordering::Relaxed);
    assert_eq!(b.allocate(Some("Element"), None).unwrap().value(), 0);
    assert_eq!(
        b.allocate(Some("Unknown"), None),
        Err(Error::UnknownType("Unknown".into()))
    );
}

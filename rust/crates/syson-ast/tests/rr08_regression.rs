// SPDX-License-Identifier: EPL-2.0
//! Development-only requested-allocation oracle, not application instrumentation.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::{Cell, RefCell},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use syson_ast::{
    Assignment, Builder, Constructed, Datum, Error, Lexeme, Limits, Reflection, Slot, Value,
};
use syson_ast_adapter::{
    operator_expression::{OperatorPostprocessor, Report},
    Adapter,
};
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
fn construction_requests(count: usize) -> Requests {
    let mut builder = builder(Limits::default());
    REQUESTS.set(Requests::default());
    TRACK.set(true);
    for _ in 0..count {
        let node = builder.begin_node(Some("Element"), None).unwrap();
        assert_eq!(builder.end_frame().unwrap(), Constructed::Node(node));
    }
    TRACK.set(false);
    REQUESTS.get()
}
#[test]
fn growing_construction_avoids_repeated_whole_state_allocation() {
    // Run all sizes before asserting so a baseline failure retains every point.
    let small = construction_requests(100);
    let medium = construction_requests(1000);
    let large = construction_requests(5000);
    eprintln!("RR08 construction100={small:?};1000={medium:?};5000={large:?}");
    // Broad bounds admit map/journal growth and allocator layout differences;
    // whole-State snapshots grow approximately 100x and 25x at these steps.
    assert!(
        medium.total <= small.total.saturating_mul(20),
        "{small:?} {medium:?}"
    );
    assert!(
        large.total <= medium.total.saturating_mul(10),
        "{medium:?} {large:?}"
    );
}

fn string(text: &str) -> Datum {
    Datum::new(Value::String(text.into()))
}
fn assign(builder: &mut Builder, node: syson_ast::NodeId, key: &str, value: Datum) {
    builder.assign(node, key, Assignment::Set, value).unwrap();
}
#[test]
fn nested_success_and_failure_restore_fields_frames_cst_and_spent_ids() {
    let mut b = builder(Limits::default());
    let root = b.begin_node(Some("Element"), None).unwrap();
    assign(&mut b, root, "name", string("original"));
    b.associate_cst(7, root).unwrap();
    let mut spent = None;
    let result: Result<(), Error> = b.transaction(|b| {
        assign(b, root, "name", string("outer"));
        b.begin_datatype()?;
        b.append_datatype("outer")?;
        let inner: Result<(), Error> = b.transaction(|b| {
            b.append_datatype("-failed")?;
            let node = b.allocate(Some("Element"), None)?;
            spent = Some(node);
            b.associate_cst(7, node)?;
            b.unsupported("inner")
        });
        assert_eq!(inner, Err(Error::UnsupportedConstruct("inner".into())));
        assert!(matches!(b.node(spent.unwrap()), Err(Error::UnknownNode(_))));
        b.transaction(|b| {
            b.append_datatype("-success")?;
            b.assign(
                root,
                "items",
                Assignment::Append,
                string("retained until outer rollback"),
            )
        })?;
        assert_eq!(b.end_frame()?, Constructed::Text("outer-success".into()));
        b.action("Element", Some(("child", Assignment::Set)), None)?;
        b.unsupported("outer")
    });
    assert_eq!(result, Err(Error::UnsupportedConstruct("outer".into())));
    assert_eq!(b.current_node().unwrap(), root);
    assert_eq!(
        b.node(root).unwrap().slot("name"),
        Slot::Present(&string("original"))
    );
    assert_eq!(b.node(root).unwrap().slot("items"), Slot::Absent);
    let next = b.allocate(Some("Element"), None).unwrap();
    assert!(next.value() > spent.unwrap().value());
    b.end_frame().unwrap();
    let arena = b.finish(root).unwrap();
    assert_eq!(arena.nodes().len(), 1);
    assert_eq!(arena.cst_element(7), Some(root));
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
fn scalar_copy_budget_refuses_before_large_reservation_and_work_stays_spent() {
    let mut b = builder(Limits {
        max_work: 100_000,
        ..Limits::default()
    });
    let target = b.allocate(Some("Element"), None).unwrap();
    let source = b.allocate(Some("Element"), None).unwrap();
    assign(&mut b, source, "payload", string(&"x".repeat(60_000)));
    REQUESTS.set(Requests::default());
    TRACK.set(true);
    let result = b.merge(target, source);
    TRACK.set(false);
    let stats = REQUESTS.get();
    assert_eq!(result, Err(Error::Budget));
    assert!(stats.largest < 4096, "{stats:?}");
    assert_eq!(b.node(target).unwrap().slot("payload"), Slot::Absent);
    assert_eq!(b.allocate(Some("Element"), None), Err(Error::Budget));
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
fn cancellation_at_scalar_reservation_restores_outer_changes_and_datatype_frames() {
    let flag = Arc::new(AtomicBool::new(false));
    let mut reflection = Reflection::default();
    reflection.declare("Element".into(), vec![]).unwrap();
    let mut b = Builder::new(reflection, Limits::default(), flag.clone()).unwrap();
    let root = b.allocate(Some("Element"), None).unwrap();
    let source = b.allocate(Some("Element"), None).unwrap();
    let payload = "雪😀".repeat(16_384);
    assign(&mut b, source, "payload", string(&payload));
    b.associate_cst(1, root).unwrap();
    let mut spent = None;
    let (result, stats) = cancellation_window(&flag, || {
        b.transaction(|b| {
            b.assign(root, "temporary", Assignment::Set, string("outer"))?;
            spent = Some(b.allocate(Some("Element"), None)?);
            b.associate_cst(1, source)?;
            b.merge(root, source)
        })
    });
    assert_eq!(result, Err(Error::Cancelled));
    assert!(flag.load(Ordering::Relaxed));
    // The only large reservation in this call is the source payload copy.
    assert_eq!(stats.largest, payload.len());
    assert_eq!(b.node(root).unwrap().slot("temporary"), Slot::Absent);
    assert_eq!(b.node(root).unwrap().slot("payload"), Slot::Absent);
    flag.store(false, Ordering::Relaxed);
    assert!(b.allocate(Some("Element"), None).unwrap().value() > spent.unwrap().value());
    b.begin_datatype().unwrap();
    b.append_datatype("prefix").unwrap();
    let (result, stats) = cancellation_window(&flag, || b.append_datatype(&payload));
    assert_eq!(result, Err(Error::Cancelled));
    assert!(stats.largest >= payload.len());
    flag.store(false, Ordering::Relaxed);
    assert_eq!(b.end_frame().unwrap(), Constructed::Text("prefix".into()));
    let arena = b.finish(root).unwrap();
    assert_eq!(arena.cst_element(1), Some(root));
}
fn postprocess_requests(p: &OperatorPostprocessor, count: usize) -> Requests {
    // An explicit stress-fixture work allowance, not a production default change.
    let mut b = p
        .builder(
            Limits {
                max_work: 100_000_000,
                ..Limits::default()
            },
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let mut last = None;
    REQUESTS.set(Requests::default());
    TRACK.set(true);
    for _ in 0..count {
        let root = b.allocate(Some("OperatorExpression"), None).unwrap();
        b.assign(root, "operator", Assignment::Set, string("as"))
            .unwrap();
        assert!(matches!(
            p.apply(&mut b, root).unwrap(),
            Report::Inserted { .. }
        ));
        last = Some(root);
    }
    TRACK.set(false);
    let stats = REQUESTS.get();
    let root = last.unwrap();
    let arena = b.finish(root).unwrap();
    assert_eq!(arena.nodes().len(), 4);
    let Slot::Present(Datum {
        value: Value::Array(operands),
        ..
    }) = arena.node(root).unwrap().slot("operands")
    else {
        panic!()
    };
    let Value::Node(child) = operands[0].value else {
        panic!()
    };
    assert_eq!(arena.node(child).unwrap().container().unwrap().node, root);
    stats
}
#[test]
fn actual_f12_growth_is_measured_separately_from_actual_parser_construction() {
    let p = OperatorPostprocessor::pinned().unwrap();
    let small = postprocess_requests(&p, 100);
    let medium = postprocess_requests(&p, 1000);
    let large = postprocess_requests(&p, 5000);
    eprintln!("RR08 actualF12_100={small:?};1000={medium:?};5000={large:?}");
    assert!(
        medium.total <= small.total.saturating_mul(20),
        "{small:?} {medium:?}"
    );
    assert!(
        large.total <= medium.total.saturating_mul(10),
        "{medium:?} {large:?}"
    );
    // The finite parser does not invoke this postprocessor; test its real route
    // independently, without describing synthetic operators as parsed source.
    let source = "package '🚗' {part def Wheel; part wheel:Wheel;}";
    let arena = Adapter::pinned()
        .unwrap()
        .construct(source, Default::default(), Arc::new(AtomicBool::new(false)))
        .unwrap();
    assert!(arena
        .nodes()
        .values()
        .any(|node| node.kind() == Some("PartUsage")));
    let root = arena.node(arena.root()).unwrap();
    assert_eq!(root.origin().unwrap().span.bytes, 0..source.len());
    let limited = p
        .builder(
            Limits {
                max_nodes: 2,
                ..Limits::default()
            },
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
    let mut limited = limited;
    let root = limited.allocate(Some("OperatorExpression"), None).unwrap();
    assign(&mut limited, root, "operator", string("as"));
    assert!(matches!(
        p.apply(&mut limited, root),
        Err(syson_ast_adapter::operator_expression::Error::Kernel(
            Error::Budget
        ))
    ));
    assert_eq!(limited.node(root).unwrap().slot("operands"), Slot::Absent);
}

#[test]
fn copied_work_failure_restores_partial_defaults_and_adopted_kind() {
    let mut reflection = Reflection::default();
    reflection
        .declare(
            "Element".into(),
            vec![
                syson_ast::DefaultValue::Boolean("a".into()),
                syson_ast::DefaultValue::Boolean("z".repeat(200)),
            ],
        )
        .unwrap();
    let mut b = Builder::new(
        reflection,
        Limits {
            max_work: 220,
            ..Limits::default()
        },
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    let root = b.allocate(Some("Element"), None).unwrap();
    assert_eq!(b.defaults(root), Err(Error::Budget));
    assert_eq!(b.node(root).unwrap().slot("a"), Slot::Absent);
    let mut b = builder(Limits {
        max_work: 15_000,
        ..Limits::default()
    });
    let target = b.allocate(None, None).unwrap();
    let source = b.allocate(Some("Element"), None).unwrap();
    assign(&mut b, source, "payload", string(&"x".repeat(6000)));
    assert_eq!(b.merge(target, source), Err(Error::Budget));
    assert_eq!(b.node(target).unwrap().kind(), None);
    assert_eq!(b.node(target).unwrap().slot("payload"), Slot::Absent);
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

#[test]
fn finish_charges_each_containment_label_before_repeated_copying() {
    let mut b = builder(Limits {
        max_work: 400_000,
        ..Limits::default()
    });
    let root = b.allocate(Some("Element"), None).unwrap();
    let children: Vec<_> = (0..32)
        .map(|_| Datum::node(b.allocate(Some("Element"), None).unwrap()))
        .collect();
    let key = "k".repeat(65_536);
    assign(&mut b, root, &key, Datum::new(Value::Array(children)));
    REQUESTS.set(Requests::default());
    TRACK.set(true);
    let result = b.finish(root);
    TRACK.set(false);
    let stats = REQUESTS.get();
    eprintln!(
        "RR08 finish32x65536 allocation={stats:?}; refused={}",
        matches!(result, Err(Error::Budget))
    );
    assert!(matches!(result, Err(Error::Budget)));
    // Setup spends about 197k units; <=3 full label reservations fit the
    // remaining work. The fourth must refuse before another 64KiB allocation.
    assert!(stats.total <= 262_144, "{stats:?}");
}

#[test]
fn reference_text_and_target_type_refusal_leave_the_node_unchanged() {
    let large = "R".repeat(65_536);
    for large_text in [true, false] {
        let mut b = builder(Limits {
            max_work: 128,
            ..Limits::default()
        });
        let id = b.allocate(Some("Element"), None).unwrap();
        let text = if large_text { large.as_str() } else { "r" };
        let target = if large_text {
            "Element"
        } else {
            large.as_str()
        };
        let lexeme = Lexeme::from_source(text, 0, text.len()).unwrap();
        REQUESTS.set(Requests::default());
        TRACK.set(true);
        let result = b.reference(id, "reference", Assignment::Set, lexeme, target);
        TRACK.set(false);
        assert_eq!(result, Err(Error::Budget));
        assert!(REQUESTS.get().largest < large.len());
        assert_eq!(b.node(id).unwrap().slot("reference"), Slot::Absent);
    }
}
#[test]
fn merge_charges_discarded_source_scalars_and_target_array_data() {
    let large = "M".repeat(65_536);
    for target_array in [false, true] {
        let mut b = builder(Limits {
            max_work: large.len() * (if target_array { 1 } else { 2 }) + 512,
            ..Limits::default()
        });
        let target = b.allocate(Some("Element"), None).unwrap();
        let source = b.allocate(Some("Element"), None).unwrap();
        if target_array {
            assign(&mut b, source, "items", Datum::new(Value::Array(vec![])));
            assign(
                &mut b,
                target,
                "items",
                Datum::new(Value::Array(vec![string(&large)])),
            );
        } else {
            assign(&mut b, source, "items", string(&large));
            assign(&mut b, target, "items", string("retained"));
        }
        let source_before = b.node(source).unwrap().fields().clone();
        let target_before = b.node(target).unwrap().fields().clone();
        assert_eq!(b.merge(target, source), Err(Error::Budget));
        assert_eq!(b.node(target).unwrap().fields(), &target_before);
        assert_eq!(b.node(source).unwrap().fields(), &source_before);
        // Budget/refusal atomicity must survive replacing A's State snapshots
        // with the suffix journal; allocation admission is checked separately.
    }
}

#[test]
fn large_flat_validation_refuses_before_auxiliary_allocation() {
    let mut b = builder(Limits {
        max_work: 128,
        ..Limits::default()
    });
    let id = b.allocate(Some("Element"), None).unwrap();
    let data = Datum::new(Value::Array(vec![Datum::new(Value::Null); 65_536]));
    REQUESTS.set(Requests::default());
    TRACK.set(true);
    let result = b.assign(id, "items", Assignment::Set, data);
    TRACK.set(false);
    assert_eq!(result, Err(Error::Budget));
    assert!(
        REQUESTS.get().largest <= 4096,
        "auxiliary validation allocation: {:?}",
        REQUESTS.get()
    );
    assert_eq!(b.node(id).unwrap().slot("items"), Slot::Absent);
}
#[test]
fn cancellation_before_successful_callback_return_rolls_back_publication() {
    let mut reflection = Reflection::default();
    reflection.declare("Element".into(), vec![]).unwrap();
    let flag = Arc::new(AtomicBool::new(false));
    let mut b = Builder::new(reflection, Limits::default(), flag.clone()).unwrap();
    let root = b.allocate(Some("Element"), None).unwrap();
    assign(&mut b, root, "value", string("original"));
    b.associate_cst(77, root).unwrap();
    let result: Result<(), Error> = b.transaction(|b| {
        b.assign(root, "value", Assignment::Set, string("temporary"))?;
        let spent = b.allocate(Some("Element"), None)?;
        b.associate_cst(77, spent)?;
        flag.store(true, Ordering::Relaxed);
        Ok(())
    });
    assert_eq!(result, Err(Error::Cancelled));
    flag.store(false, Ordering::Relaxed);
    assert_eq!(
        b.node(root).unwrap().slot("value"),
        Slot::Present(&string("original"))
    );
    assert_eq!(b.allocate(Some("Element"), None).unwrap().value(), 2);
    assert_eq!(b.finish(root).unwrap().cst_element(77), Some(root));
}
#[test]
fn swallowed_budget_error_cannot_publish_partial_state_or_recycle_work() {
    let mut b = builder(Limits {
        max_work: 1024,
        ..Limits::default()
    });
    let root = b.allocate(Some("Element"), None).unwrap();
    assign(&mut b, root, "value", string("original"));
    let large = "B".repeat(65_536);
    let result: Result<(), Error> = b.transaction(|b| {
        b.assign(root, "value", Assignment::Set, string("temporary"))?;
        assert_eq!(b.allocate(Some(&large), None), Err(Error::Budget));
        Ok(())
    });
    assert_eq!(result, Err(Error::Budget));
    assert_eq!(
        b.node(root).unwrap().slot("value"),
        Slot::Present(&string("original"))
    );
    assert_eq!(b.allocate(Some("Element"), None), Err(Error::Budget));
}

// SPDX-License-Identifier: EPL-2.0
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use syson_ast::*;
fn reflection() -> Reflection {
    let mut r = Reflection::default();
    r.declare(
        "T".into(),
        vec![
            DefaultValue::Boolean("flag".into()),
            DefaultValue::Array("items".into()),
            DefaultValue::Array("flag".into()),
        ],
    )
    .unwrap();
    r.declare("W".into(), vec![]).unwrap();
    r
}
fn builder() -> Builder {
    Builder::new(
        reflection(),
        Limits::default(),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap()
}
fn text(s: &str) -> Datum {
    Datum::new(Value::String(s.into()))
}
fn field<'a>(b: &'a Builder, id: NodeId, key: &str) -> &'a Value {
    match b.node(id).unwrap().slot(key) {
        Slot::Present(d) => &d.value,
        Slot::Absent => panic!("missing {key}"),
    }
}
#[test]
fn every_foreign_node_entry_point_rejects_other_builder_ids_without_aliasing() {
    let mut a = builder();
    let foreign = a.allocate(Some("T"), None).unwrap();
    let mut b = builder();
    let local = b.begin_node(Some("T"), None).unwrap();
    assert_ne!(
        foreign, local,
        "IDs must carry arena identity, even when both local counters start at zero"
    );
    assert!(b.node(foreign).is_err());
    assert!(b
        .assign(foreign, "x", Assignment::Set, text("bad"))
        .is_err());
    assert!(b
        .assign(local, "x", Assignment::Set, Datum::node(foreign))
        .is_err());
    assert!(b
        .assign(
            local,
            "x",
            Assignment::Set,
            Datum::new(Value::Array(vec![Datum::node(foreign)]))
        )
        .is_err());
    assert!(b.defaults(foreign).is_err());
    assert!(b.merge(local, foreign).is_err());
    assert!(b.merge(foreign, local).is_err());
    assert!(b.adopt_subrule(foreign).is_err());
    assert!(b.assign_subrule("x", Assignment::Set, foreign, 1).is_err());
    assert!(b.associate_cst(1, foreign).is_err());
    assert!(b
        .reference(
            foreign,
            "ref",
            Assignment::Set,
            Lexeme::from_source("X", 0, 1).unwrap(),
            "T"
        )
        .is_err());
    assert_eq!(b.current_node().unwrap(), local);
    assert_eq!(b.node(local).unwrap().slot("x"), Slot::Absent);
    b.end_frame().unwrap();
    let arena = b.finish(local).unwrap();
    assert!(arena.node(foreign).is_none());
    assert!(arena.clone().node(local).is_some());
    let mut c = builder();
    let _ = c.allocate(Some("T"), None).unwrap();
    assert!(c.finish(foreign).is_err());
}
#[test]
fn appending_an_array_is_rejected_before_it_can_create_nested_property_state() {
    let mut b = builder();
    let r = b.allocate(Some("T"), None).unwrap();
    b.assign(
        r,
        "items",
        Assignment::Set,
        Datum::new(Value::Array(vec![text("existing")])),
    )
    .unwrap();
    for xs in [vec![], vec![text("nested")]] {
        assert_eq!(
            b.assign(r, "items", Assignment::Append, Datum::new(Value::Array(xs))),
            Err(Error::NestedContainmentArray)
        );
        assert_eq!(field(&b, r, "items"), &Value::Array(vec![text("existing")]));
    }
    b.assign(r, "scalar", Assignment::Set, text("preserve"))
        .unwrap();
    assert_eq!(
        b.assign(
            r,
            "scalar",
            Assignment::Append,
            Datum::new(Value::Array(vec![]))
        ),
        Err(Error::NestedContainmentArray)
    );
    assert_eq!(field(&b, r, "scalar"), &Value::String("preserve".into()));
    assert!(b.finish(r).is_ok());
}
#[test]
fn nested_transaction_success_and_failure_preserve_frames_fields_and_cst_ownership() {
    let mut b = builder();
    let r = b.begin_node(Some("T"), None).unwrap();
    b.associate_cst(7, r).unwrap();
    let mut escaped = None;
    b.transaction(|b| {
        b.assign(r, "keep", Assignment::Set, text("outer"))?;
        let bad: Result<(), Error> = b.transaction(|b| {
            let x = b.action("W", Some(("operand", Assignment::Set)), None)?;
            escaped = Some(x);
            b.associate_cst(7, x)?;
            b.assign(r, "keep", Assignment::Set, text("inner"))?;
            b.unsupported("inner")
        });
        assert!(bad.is_err());
        assert_eq!(b.current_node()?, r);
        assert_eq!(field(b, r, "keep"), &Value::String("outer".into()));
        Ok(())
    })
    .unwrap();
    let mut inner_success = None;
    let outer: Result<(), Error> = b.transaction(|b| {
        b.transaction(|b| {
            let x = b.allocate(Some("T"), None)?;
            inner_success = Some(x);
            b.assign(r, "leak", Assignment::Set, Datum::node(x))
        })?;
        b.unsupported("outer")
    });
    assert!(outer.is_err());
    assert_eq!(b.node(r).unwrap().slot("leak"), Slot::Absent);
    assert!(b.node(escaped.unwrap()).is_err());
    assert!(b.node(inner_success.unwrap()).is_err());
    let next = b.allocate(Some("T"), None).unwrap();
    assert_ne!(next, escaped.unwrap());
    assert_ne!(next, inner_success.unwrap());
    b.end_frame().unwrap();
    let arena = b.finish(r).unwrap();
    assert_eq!(arena.cst_element(7), Some(r));
    assert_eq!(arena.nodes().len(), 1);
}
#[test]
fn action_failure_rolls_back_added_defaults_and_nested_wrappers_derive_containers() {
    let mut b = builder();
    let leaf = b.begin_node(Some("T"), None).unwrap();
    assert!(b
        .action("Unknown", Some(("operand", Assignment::Set)), None)
        .is_err());
    assert_eq!(b.current_node().unwrap(), leaf);
    assert_eq!(b.node(leaf).unwrap().slot("flag"), Slot::Absent);
    let w = b
        .action("W", Some(("operand", Assignment::Set)), None)
        .unwrap();
    let top = b
        .action("W", Some(("operands", Assignment::Append)), None)
        .unwrap();
    b.end_frame().unwrap();
    let arena = b.finish(top).unwrap();
    assert_eq!(arena.node(leaf).unwrap().container().unwrap().node, w);
    assert_eq!(
        arena.node(w).unwrap().container(),
        Some(&Container {
            node: top,
            property: "operands".into(),
            index: Some(0)
        })
    );
}
#[test]
fn adoption_source_first_arrays_defaults_and_explicit_cst_retarget_remain_separate() {
    let mut b = builder();
    let target = b.begin_node(None, None).unwrap();
    b.assign(target, "items", Assignment::Append, text("outer"))
        .unwrap();
    b.assign(target, "name", Assignment::Set, text("chosen"))
        .unwrap();
    let source = b.allocate(Some("T"), None).unwrap();
    let child = b.allocate(Some("T"), None).unwrap();
    b.assign(source, "items", Assignment::Append, text("inner"))
        .unwrap();
    b.assign(source, "items", Assignment::Append, Datum::node(child))
        .unwrap();
    b.assign(source, "name", Assignment::Set, text("discarded"))
        .unwrap();
    b.assign(source, "flag", Assignment::Set, Datum::new(Value::Null))
        .unwrap();
    b.associate_cst(1, source).unwrap();
    b.associate_cst(2, source).unwrap();
    b.merge(target, source).unwrap();
    assert_eq!(b.node(target).unwrap().kind(), Some("T"));
    assert_eq!(field(&b, target, "flag"), &Value::Null);
    assert_eq!(field(&b, target, "name"), &Value::String("chosen".into()));
    assert_eq!(
        field(&b, target, "items"),
        &Value::Array(vec![text("inner"), Datum::node(child), text("outer")])
    );
    b.associate_cst(1, target).unwrap();
    b.end_frame().unwrap();
    let arena = b.finish(target).unwrap();
    assert!(arena.node(source).is_none());
    assert_eq!(arena.cst_element(1), Some(target));
    assert_eq!(arena.cst_element(2), None);
    assert_eq!(
        arena.node(child).unwrap().container(),
        Some(&Container {
            node: target,
            property: "items".into(),
            index: Some(1)
        })
    );
}
#[test]
fn cancellation_inside_transaction_rolls_back_and_failed_work_remains_consumed() {
    let flag = Arc::new(AtomicBool::new(false));
    let mut b = Builder::new(reflection(), Limits::default(), flag.clone()).unwrap();
    let r = b.begin_node(Some("T"), None).unwrap();
    let mut escaped = None;
    let result: Result<(), Error> = b.transaction(|b| {
        escaped = Some(b.allocate(Some("T"), None)?);
        flag.store(true, Ordering::Relaxed);
        b.assign(r, "x", Assignment::Set, text("bad"))
    });
    assert_eq!(result, Err(Error::Cancelled));
    flag.store(false, Ordering::Relaxed);
    assert!(b.node(escaped.unwrap()).is_err());
    assert_eq!(b.current_node().unwrap(), r);
    assert_eq!(b.node(r).unwrap().slot("x"), Slot::Absent);
    b.end_frame().unwrap();
    assert!(b.finish(r).is_ok());
    fn remaining(failed: bool) -> usize {
        let mut b = Builder::new(
            reflection(),
            Limits {
                max_work: 30,
                ..Limits::default()
            },
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        if failed {
            let _: Result<(), Error> = b.transaction(|b| {
                b.allocate(Some("T"), None)?;
                b.unsupported("branch")
            });
        }
        let mut n = 0;
        while b.allocate(Some("T"), None).is_ok() {
            n += 1;
        }
        n
    }
    assert!(remaining(true) < remaining(false));
}
#[test]
fn datatype_and_zero_length_subrules_preserve_frame_and_absence_contracts() {
    let mut b = builder();
    assert_eq!(b.current_node(), Err(Error::NoFrame));
    b.begin_datatype().unwrap();
    b.append_datatype("typed").unwrap();
    b.append_datatype("by").unwrap();
    assert_eq!(b.current_node(), Err(Error::WrongFrame));
    assert_eq!(b.end_frame().unwrap(), Constructed::Text("typedby".into()));
    let r = b.begin_node(Some("T"), None).unwrap();
    assert_eq!(b.append_datatype("bad"), Err(Error::WrongFrame));
    let child = b.allocate(Some("T"), None).unwrap();
    assert!(!b
        .assign_subrule("child", Assignment::Set, child, 0)
        .unwrap());
    assert_eq!(b.node(r).unwrap().slot("child"), Slot::Absent);
    assert!(b
        .assign_subrule("child", Assignment::Set, child, 1)
        .unwrap());
    b.end_frame().unwrap();
    assert_eq!(b.finish(r).unwrap().nodes().len(), 2);
}
#[test]
fn node_frame_limits_and_source_spans_fail_explicitly_without_reusing_ids() {
    let source = "A😀'x\\n'";
    let lex = Lexeme::from_source(source, 5, 10).unwrap();
    assert_eq!(lex.span.bytes, 5..10);
    assert_eq!(lex.span.utf16, 3..8);
    assert_eq!(Datum::raw(lex).value, Value::String("'x\\n'".into()));
    for (a, z) in [(2, 3), (6, 99), (9, 4)] {
        assert_eq!(Lexeme::from_source(source, a, z), Err(Error::InvalidSpan));
    }
    let mut b = Builder::new(
        reflection(),
        Limits {
            max_nodes: 2,
            max_frames: 1,
            ..Limits::default()
        },
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    let r = b.begin_node(Some("T"), None).unwrap();
    assert_eq!(b.begin_node(Some("T"), None), Err(Error::Budget));
    let spare = b.allocate(Some("T"), None).unwrap();
    assert_ne!(r, spare);
    assert_eq!(b.allocate(Some("T"), None), Err(Error::Budget));
    b.end_frame().unwrap();
    assert_eq!(b.finish(r).unwrap().nodes().len(), 1);
}
#[test]
fn presence_defaults_raw_references_and_number_input_slots_keep_distinct_contracts() {
    let mut b = builder();
    let r = b.allocate(Some("T"), None).unwrap();
    b.assign(r, "flag", Assignment::Set, Datum::new(Value::Null))
        .unwrap();
    b.assign(r, "items", Assignment::Set, text("reset at defaults"))
        .unwrap();
    let lex = Lexeme::from_source("false", 0, 5).unwrap();
    b.assign(r, "seen", Assignment::Presence, Datum::raw(lex.clone()))
        .unwrap();
    b.reference(
        r,
        "ref",
        Assignment::Set,
        Lexeme::from_source("Pkg::Thing", 0, 10).unwrap(),
        "Target",
    )
    .unwrap();
    b.assign(
        r,
        "number",
        Assignment::Set,
        Datum::new(Value::Number(f64::NAN)),
    )
    .unwrap();
    let arena = b.finish(r).unwrap();
    let n = arena.node(r).unwrap();
    assert_eq!(n.slot("missing"), Slot::Absent);
    assert_eq!(n.slot("flag"), Slot::Present(&Datum::new(Value::Null)));
    assert_eq!(
        n.slot("items"),
        Slot::Present(&Datum::new(Value::Array(vec![])))
    );
    assert_eq!(
        n.slot("seen"),
        Slot::Present(&Datum {
            value: Value::Boolean(true),
            lexeme: Some(lex)
        })
    );
    assert!(
        matches!(n.slot("number"),Slot::Present(Datum{value:Value::Number(x),..}) if x.is_nan())
    );
    assert!(
        matches!(n.slot("ref"),Slot::Present(Datum{value:Value::UnresolvedReference{text,target_type},..}) if text=="Pkg::Thing"&&target_type=="Target")
    );
    assert_eq!(arena.nodes().len(), 1); // An unresolved reference is not containment.
}
#[test]
fn unassigned_subrule_adoption_switches_frame_and_preserves_source_first_order() {
    let mut b = builder();
    let frame = b.begin_node(None, None).unwrap();
    b.assign(frame, "items", Assignment::Append, text("source frame"))
        .unwrap();
    let result = b.allocate(Some("T"), None).unwrap();
    b.assign(result, "items", Assignment::Append, text("target result"))
        .unwrap();
    b.adopt_subrule(result).unwrap();
    assert_eq!(b.current_node().unwrap(), result);
    assert_eq!(b.adopt_subrule(result), Err(Error::SelfMerge));
    assert_eq!(b.current_node().unwrap(), result);
    assert_eq!(b.end_frame().unwrap(), Constructed::Node(result));
    let arena = b.finish(result).unwrap();
    assert!(arena.node(frame).is_none());
    assert_eq!(
        arena.node(result).unwrap().slot("items"),
        Slot::Present(&Datum::new(Value::Array(vec![
            text("source frame"),
            text("target result")
        ])))
    );
}

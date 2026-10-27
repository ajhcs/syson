use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use syson_ast::*;
fn reflection() -> Reflection {
    let mut r = Reflection::default();
    r.declare(
        "Element".into(),
        vec![
            DefaultValue::Array("members".into()),
            DefaultValue::Boolean("flag".into()),
        ],
    )
    .unwrap();
    r.declare("Wrapper".into(), vec![]).unwrap();
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
fn string(s: &str) -> Datum {
    Datum::new(Value::String(s.into()))
}
fn val<'a>(n: &'a Node, k: &str) -> &'a Value {
    match n.slot(k) {
        Slot::Present(d) => &d.value,
        _ => panic!("missing {k}"),
    }
}
#[test]
fn missing_null_false_and_array_defaults_remain_distinct() {
    let mut b = builder();
    let root = b.allocate(Some("Element"), None).unwrap();
    b.assign(root, "null", Assignment::Set, Datum::new(Value::Null))
        .unwrap();
    b.assign(root, "members", Assignment::Set, string("not array"))
        .unwrap();
    let a = b.finish(root).unwrap();
    let n = a.node(root).unwrap();
    assert_eq!(n.slot("missing"), Slot::Absent);
    assert_eq!(val(n, "null"), &Value::Null);
    assert_eq!(val(n, "flag"), &Value::Boolean(false));
    assert_eq!(val(n, "members"), &Value::Array(vec![]));
}
#[test]
fn boolean_default_does_not_overwrite_explicit_null() {
    let mut b = builder();
    let r = b.allocate(Some("Element"), None).unwrap();
    b.assign(r, "flag", Assignment::Set, Datum::new(Value::Null))
        .unwrap();
    assert_eq!(
        val(b.finish(r).unwrap().node(r).unwrap(), "flag"),
        &Value::Null
    );
}
#[test]
fn presence_uses_match_presence_and_retains_raw_lexeme() {
    let mut b = builder();
    let r = b.allocate(Some("Element"), None).unwrap();
    let lex = Lexeme::from_source("abstract", 0, 8).unwrap();
    b.assign(r, "flag", Assignment::Presence, Datum::raw(lex.clone()))
        .unwrap();
    let a = b.finish(r).unwrap();
    assert_eq!(
        a.node(r).unwrap().slot("flag"),
        Slot::Present(&Datum {
            value: Value::Boolean(true),
            lexeme: Some(lex)
        })
    );
}
#[test]
fn set_replaces_and_append_resets_non_array() {
    let mut b = builder();
    let r = b.allocate(Some("Element"), None).unwrap();
    b.assign(r, "x", Assignment::Set, string("a")).unwrap();
    b.assign(r, "x", Assignment::Set, string("b")).unwrap();
    b.assign(r, "x", Assignment::Append, string("c")).unwrap();
    b.assign(r, "x", Assignment::Append, string("d")).unwrap();
    assert_eq!(
        val(b.finish(r).unwrap().node(r).unwrap(), "x"),
        &Value::Array(vec![string("c"), string("d")])
    );
}
#[test]
fn merge_source_arrays_precede_target_and_target_scalars_win() {
    let mut b = builder();
    let t = b.allocate(None, None).unwrap();
    let s = b.allocate(Some("Element"), None).unwrap();
    b.assign(t, "members", Assignment::Append, string("target"))
        .unwrap();
    b.assign(s, "members", Assignment::Append, string("source"))
        .unwrap();
    b.assign(t, "name", Assignment::Set, string("chosen"))
        .unwrap();
    b.assign(s, "name", Assignment::Set, string("old")).unwrap();
    b.merge(t, s).unwrap();
    assert_eq!(b.node(t).unwrap().kind(), Some("Element"));
    let a = b.finish(t).unwrap();
    assert_eq!(a.nodes().len(), 1);
    assert_eq!(
        val(a.node(t).unwrap(), "members"),
        &Value::Array(vec![string("source"), string("target")])
    );
    assert_eq!(
        val(a.node(t).unwrap(), "name"),
        &Value::String("chosen".into())
    );
}
#[test]
fn action_wraps_previous_frame_and_sets_final_container() {
    let mut b = builder();
    let child = b.begin_node(Some("Element"), None).unwrap();
    let parent = b
        .action("Wrapper", Some(("operand", Assignment::Set)), None)
        .unwrap();
    assert_eq!(b.current_node().unwrap(), parent);
    assert_eq!(b.end_frame().unwrap(), Constructed::Node(parent));
    let a = b.finish(parent).unwrap();
    assert_eq!(
        a.node(child).unwrap().container(),
        Some(&Container {
            node: parent,
            property: "operand".into(),
            index: None
        })
    );
}
#[test]
fn fragment_datatype_concatenation_inserts_no_spaces() {
    let mut b = builder();
    b.begin_datatype().unwrap();
    b.append_datatype("typed").unwrap();
    b.append_datatype("by").unwrap();
    assert_eq!(b.end_frame().unwrap(), Constructed::Text("typedby".into()));
}
#[test]
fn quote_and_astral_text_retain_bytes_and_utf16_offsets() {
    let source = "a😀 'x\\n'";
    let lex = Lexeme::from_source(source, 6, 11).unwrap();
    assert_eq!(lex.text, "'x\\n'");
    assert_eq!(lex.span.bytes, 6..11);
    assert_eq!(lex.span.utf16, 4..9);
    // STRING_VALUE/UNRESTRICTED_NAME are raw typed strings, not JSON-decoded STRING.
    assert_eq!(Datum::raw(lex.clone()).value, Value::String(lex.text));
    assert_eq!(Lexeme::from_source(source, 2, 3), Err(Error::InvalidSpan));
}
#[test]
fn references_are_not_containment_or_resolved_targets() {
    let mut b = builder();
    let r = b.allocate(Some("Element"), None).unwrap();
    let lex = Lexeme::from_source("A::B", 0, 4).unwrap();
    b.reference(r, "target", Assignment::Set, lex, "Type")
        .unwrap();
    let a = b.finish(r).unwrap();
    assert_eq!(a.nodes().len(), 1);
    assert_eq!(
        val(a.node(r).unwrap(), "target"),
        &Value::UnresolvedReference {
            text: "A::B".into(),
            target_type: "Type".into()
        }
    );
}
#[test]
fn failed_branch_rolls_back_nodes_fields_frames_and_cst_without_reusing_ids() {
    let mut b = builder();
    let r = b.begin_node(Some("Element"), None).unwrap();
    let mut failed = None;
    let result: Result<(), Error> = b.transaction(|b| {
        let id = b.action("Wrapper", Some(("operand", Assignment::Set)), None)?;
        failed = Some(id);
        b.associate_cst(7, id)?;
        b.assign(r, "name", Assignment::Set, string("leaked"))?;
        b.unsupported("synthetic hook")
    });
    assert_eq!(
        result,
        Err(Error::UnsupportedConstruct("synthetic hook".into()))
    );
    assert_eq!(b.current_node().unwrap(), r);
    assert_eq!(b.node(r).unwrap().slot("name"), Slot::Absent);
    assert!(matches!(
        b.node(failed.unwrap()),
        Err(Error::UnknownNode(_))
    ));
    let next = b.allocate(Some("Element"), None).unwrap();
    assert!(next.value() > failed.unwrap().value());
    b.end_frame().unwrap();
    let a = b.finish(r).unwrap();
    assert_eq!(a.cst_element(7), None);
}
#[test]
fn arrays_produce_property_indexes_and_cst_retarget_is_explicit() {
    let mut b = builder();
    let r = b.allocate(Some("Element"), None).unwrap();
    let x = b.allocate(Some("Element"), None).unwrap();
    let y = b.allocate(Some("Element"), None).unwrap();
    b.assign(r, "members", Assignment::Append, Datum::node(x))
        .unwrap();
    b.assign(r, "members", Assignment::Append, Datum::node(y))
        .unwrap();
    b.associate_cst(4, x).unwrap();
    b.associate_cst(4, y).unwrap();
    let a = b.finish(r).unwrap();
    assert_eq!(a.cst_element(4), Some(y));
    assert_eq!(
        a.node(y).unwrap().container(),
        Some(&Container {
            node: r,
            property: "members".into(),
            index: Some(1)
        })
    );
}
#[test]
fn cycles_and_shared_containment_rejected_independently() {
    let mut b = builder();
    let r = b.allocate(Some("Element"), None).unwrap();
    b.assign(r, "self", Assignment::Set, Datum::node(r))
        .unwrap();
    assert!(matches!(b.finish(r), Err(Error::Cycle(_))));
    let mut b = builder();
    let r = b.allocate(Some("Element"), None).unwrap();
    let c = b.allocate(Some("Element"), None).unwrap();
    b.assign(r, "a", Assignment::Set, Datum::node(c)).unwrap();
    b.assign(r, "b", Assignment::Set, Datum::node(c)).unwrap();
    assert!(matches!(b.finish(r), Err(Error::RepeatedContainment(_))));
}
#[test]
fn cancellation_and_work_budget_are_explicit_and_not_rolled_back() {
    let cancel = Arc::new(AtomicBool::new(false));
    let mut b = Builder::new(reflection(), Limits::default(), cancel.clone()).unwrap();
    cancel.store(true, Ordering::Relaxed);
    assert_eq!(b.allocate(Some("Element"), None), Err(Error::Cancelled));
    let mut b = Builder::new(
        reflection(),
        Limits {
            // Transaction 1 plus two allocations of 1 + seven copied kind bytes.
            max_work: 17,
            ..Limits::default()
        },
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    let _: Result<(), Error> = b.transaction(|b| {
        b.allocate(Some("Element"), None)?;
        b.unsupported("branch")
    });
    assert_eq!(b.allocate(Some("Element"), None).unwrap().value(), 1);
    assert_eq!(b.allocate(Some("Element"), None), Err(Error::Budget));
}
#[test]
fn node_and_frame_limits_do_not_leak_failed_begin() {
    let mut b = Builder::new(
        reflection(),
        Limits {
            max_frames: 0,
            ..Limits::default()
        },
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert_eq!(b.begin_node(Some("Element"), None), Err(Error::Budget));
    let r = b.allocate(Some("Element"), None).unwrap();
    assert_eq!(r.value(), 1);
    assert_eq!(b.finish(r).unwrap().nodes().len(), 1);
}
#[test]
fn unassigned_subrule_adoption_replaces_frame_after_source_first_merge() {
    let mut b = builder();
    let frame = b.begin_node(None, None).unwrap();
    b.assign(frame, "items", Assignment::Append, string("outer"))
        .unwrap();
    let result = b.allocate(Some("Element"), None).unwrap();
    b.assign(result, "items", Assignment::Append, string("inner"))
        .unwrap();
    b.adopt_subrule(result).unwrap();
    assert_eq!(b.current_node().unwrap(), result);
    b.end_frame().unwrap();
    let a = b.finish(result).unwrap();
    assert_eq!(
        val(a.node(result).unwrap(), "items"),
        &Value::Array(vec![string("outer"), string("inner")])
    );
}
#[test]
fn empty_subrule_cst_suppresses_assignment_and_unreachable_node_is_pruned() {
    let mut b = builder();
    let root = b.begin_node(Some("Element"), None).unwrap();
    let child = b.allocate(Some("Element"), None).unwrap();
    assert!(!b.assign_subrule("item", Assignment::Set, child, 0).unwrap());
    b.end_frame().unwrap();
    let a = b.finish(root).unwrap();
    assert_eq!(a.node(root).unwrap().slot("item"), Slot::Absent);
    assert!(a.node(child).is_none());
}
#[test]
fn unknown_types_nested_arrays_and_open_frames_are_errors() {
    let mut b = builder();
    assert_eq!(
        b.allocate(Some("Invented"), None),
        Err(Error::UnknownType("Invented".into()))
    );
    let root = b.begin_node(Some("Element"), None).unwrap();
    assert_eq!(
        b.assign(
            root,
            "nested",
            Assignment::Set,
            Datum::new(Value::Array(vec![Datum::new(Value::Array(vec![]))]))
        ),
        Err(Error::NestedContainmentArray)
    );
    assert!(matches!(b.finish(root), Err(Error::OpenFrames)));
}
#[test]
fn reflection_defaults_keep_first_declared_property_kind() {
    let mut r = Reflection::default();
    r.declare(
        "T".into(),
        vec![
            DefaultValue::Boolean("x".into()),
            DefaultValue::Array("x".into()),
        ],
    )
    .unwrap();
    let mut b = Builder::new(r, Limits::default(), Arc::new(AtomicBool::new(false))).unwrap();
    let root = b.allocate(Some("T"), None).unwrap();
    assert_eq!(
        val(b.finish(root).unwrap().node(root).unwrap(), "x"),
        &Value::Boolean(false)
    );
}
#[test]
fn action_without_feature_replaces_untyped_frame_without_constructing_it() {
    let mut b = builder();
    let old = b.begin_node(None, None).unwrap();
    let root = b.action("Element", None, None).unwrap();
    b.end_frame().unwrap();
    let a = b.finish(root).unwrap();
    assert!(a.node(old).is_none());
}

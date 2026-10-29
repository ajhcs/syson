use std::sync::{atomic::AtomicBool, Arc};
use syson_ast::*;
fn builder() -> Builder {
    let mut reflection = Reflection::default();
    reflection.declare("Element".into(), vec![]).unwrap();
    Builder::new(
        reflection,
        Limits::default(),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap()
}
#[test]
fn foreign_assignment_cannot_mutate_same_index_node() {
    let mut a = builder();
    let foreign = a.allocate(Some("Element"), None).unwrap();
    let mut b = builder();
    let local = b.allocate(Some("Element"), None).unwrap();
    assert_eq!(foreign.value(), local.value());
    assert!(b
        .assign(
            foreign,
            "injected",
            Assignment::Set,
            Datum::new(Value::Null)
        )
        .is_err());
    assert_eq!(b.node(local).unwrap().slot("injected"), Slot::Absent);
}
#[test]
fn foreign_merge_and_cst_association_are_rejected() {
    let mut a = builder();
    let foreign = a.allocate(Some("Element"), None).unwrap();
    let mut b = builder();
    let local = b.allocate(Some("Element"), None).unwrap();
    assert!(b.merge(local, foreign).is_err());
    assert!(b.associate_cst(1, foreign).is_err());
}
#[test]
fn foreign_root_cannot_finish_an_unrelated_builder() {
    let mut a = builder();
    let foreign = a.allocate(Some("Element"), None).unwrap();
    let mut b = builder();
    b.allocate(Some("Element"), None).unwrap();
    assert!(b.finish(foreign).is_err());
}
#[test]
fn arena_lookup_rejects_foreign_and_dropped_builder_handles() {
    let foreign = {
        let mut a = builder();
        a.allocate(Some("Element"), None).unwrap()
    };
    let mut b = builder();
    let local = b.allocate(Some("Element"), None).unwrap();
    let arena = b.finish(local).unwrap();
    assert!(arena.node(foreign).is_none());
    assert!(arena.clone().node(local).is_some());
}
#[test]
fn foreign_values_and_subrule_adoption_cannot_enter_local_state() {
    let mut a = builder();
    let foreign = a.allocate(Some("Element"), None).unwrap();
    let mut b = builder();
    let root = b.begin_node(Some("Element"), None).unwrap();
    let second = b.allocate(Some("Element"), None).unwrap();
    assert!(b.merge(second, foreign).is_err());
    assert!(b.merge(foreign, second).is_err());
    assert!(b
        .assign(root, "foreign", Assignment::Set, Datum::node(foreign))
        .is_err());
    assert!(b
        .assign(root, "foreign", Assignment::Append, Datum::node(foreign))
        .is_err());
    assert!(b.adopt_subrule(foreign).is_err());
    assert_eq!(b.current_node().unwrap(), root);
    assert_eq!(b.node(root).unwrap().slot("foreign"), Slot::Absent);
}
#[test]
fn concurrent_builders_have_distinct_handles_with_matching_local_indexes() {
    let handles: Vec<_> = (0..16)
        .map(|_| {
            std::thread::spawn(|| {
                let mut b = builder();
                b.allocate(Some("Element"), None).unwrap()
            })
        })
        .collect();
    let ids: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert!(ids.iter().all(|id| id.value() == 0));
    assert_eq!(
        ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
        ids.len()
    );
}

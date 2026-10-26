// SPDX-License-Identifier: EPL-2.0
use syson_imports::{Observation::Known, *};
fn n(id: &str) -> NamespaceId {
    NamespaceId(id.into())
}
fn m(id: &str) -> MemberId {
    MemberId(id.into())
}
fn namespace(owned: Vec<(MemberId, Visibility)>, imports: Vec<(Import, Visibility)>) -> Namespace {
    Namespace {
        is_type: false,
        owned,
        imports,
    }
}
fn member(target: Target, name: Option<&str>, short: Option<&str>) -> Member {
    Member {
        target: Known(target),
        alias: Known(Alias::SameReference),
        effective_name: Known(name.map(str::to_owned)),
        effective_short_name: Known(short.map(str::to_owned)),
    }
}
fn ns_import(id: &str) -> Import {
    Import::Namespace {
        target: Known(Some(n(id))),
        recursive: false,
        include_all: false,
    }
}
fn visible(id: &str) -> Query {
    Query::Visible {
        namespace: n(id),
        recursive: false,
        include_all: false,
    }
}
fn eval(snapshot: &Snapshot, query: Query, excluded: &[NamespaceId]) -> Result<Outcome, Error> {
    snapshot.evaluate(
        query,
        excluded,
        Limits {
            steps: 4096,
            depth: 64,
        },
    )
}
fn output(ids: &[&str]) -> Result<Outcome, Error> {
    Ok(Outcome::Memberships(ids.iter().map(|s| m(s)).collect()))
}
struct Case {
    label: &'static str,
    snapshot: Snapshot,
    query: Query,
    excluded: Vec<NamespaceId>,
    expected: Result<Outcome, Error>,
}
include!("data/f02_cases.rs");

#[test]
fn replay_exact_f02_cases_with_one_explicit_unsupported_family() {
    let cases = f02_cases();
    assert_eq!(cases.len(), 19);
    for case in cases {
        assert_eq!(
            eval(&case.snapshot, case.query, &case.excluded),
            case.expected,
            "{}",
            case.label
        );
    }
}
#[test]
fn rejected_primary_name_does_not_touch_unknown_short_name() {
    let mut value = member(Target::Element, Some("occupied"), None);
    value.effective_short_name = Observation::Unresolved;
    let snapshot = Snapshot::new(vec![], vec![(m("one"), Known(value))]).unwrap();
    assert_eq!(
        eval(
            &snapshot,
            Query::Filter {
                seed: vec!["occupied".into()],
                members: vec![m("one")]
            },
            &[]
        ),
        output(&[])
    );
    assert_eq!(
        eval(
            &snapshot,
            Query::Filter {
                seed: vec![],
                members: vec![m("one")]
            },
            &[]
        ),
        Err(Error::Unresolved)
    );
}
#[test]
fn distinct_alias_does_not_read_unknown_effective_names() {
    let value = Member {
        target: Known(Target::Element),
        alias: Known(Alias::Distinct("alias".into())),
        effective_name: Observation::Unresolved,
        effective_short_name: Observation::Unresolved,
    };
    let snapshot = Snapshot::new(vec![], vec![(m("one"), Known(value))]).unwrap();
    assert_eq!(
        eval(
            &snapshot,
            Query::Filter {
                seed: vec![],
                members: vec![m("one")]
            },
            &[]
        ),
        output(&["one"])
    );
}
#[test]
fn short_conflict_retains_new_primary_for_later_candidate() {
    let snapshot = Snapshot::new(
        vec![],
        vec![
            (
                m("one"),
                Known(member(Target::Element, Some("new"), Some("occupied"))),
            ),
            (m("two"), Known(member(Target::Element, Some("new"), None))),
        ],
    )
    .unwrap();
    assert_eq!(
        eval(
            &snapshot,
            Query::Filter {
                seed: vec!["occupied".into()],
                members: vec![m("one"), m("two")]
            },
            &[]
        ),
        output(&[])
    );
}
#[test]
fn owned_seed_ignores_alias_but_reads_both_effective_names() {
    let mut owned = member(Target::Element, Some("taken"), Some("short"));
    owned.alias = Observation::Unresolved;
    let candidate = member(Target::Element, Some("short"), None);
    let snapshot = Snapshot::new(
        vec![],
        vec![
            (m("owned"), Known(owned.clone())),
            (m("candidate"), Known(candidate)),
        ],
    )
    .unwrap();
    assert_eq!(
        eval(
            &snapshot,
            Query::Supplied {
                owned: vec![m("owned")],
                calls: vec![Known(Some(vec![m("candidate")]))]
            },
            &[]
        ),
        output(&[])
    );
    owned.effective_short_name = Observation::Unresolved;
    let unknown = Snapshot::new(vec![], vec![(m("owned"), Known(owned))]).unwrap();
    assert_eq!(
        eval(
            &unknown,
            Query::Supplied {
                owned: vec![m("owned")],
                calls: vec![]
            },
            &[]
        ),
        Err(Error::Unresolved)
    );
}
#[test]
fn hidden_members_and_imports_do_not_read_their_targets() {
    let snapshot = Snapshot::new(
        vec![(
            n("A"),
            Known(namespace(
                vec![(m("missing"), Visibility::Private)],
                vec![(
                    Import::Namespace {
                        target: Observation::Unresolved,
                        recursive: true,
                        include_all: false,
                    },
                    Visibility::Protected,
                )],
            )),
        )],
        vec![],
    )
    .unwrap();
    assert_eq!(eval(&snapshot, visible("A"), &[]), output(&[]));
}
#[test]
fn exclusions_precede_target_kind_and_namespace_lookup() {
    let snapshot = Snapshot::new(vec![(n("unknown"), Observation::Unresolved)], vec![]).unwrap();
    assert_eq!(
        eval(&snapshot, visible("unknown"), &[n("unknown")]),
        output(&[])
    );
    assert_eq!(
        eval(&snapshot, visible("missing"), &[n("missing")]),
        output(&[])
    );
    assert_eq!(
        eval(&snapshot, visible("unknown"), &[]),
        Err(Error::Unresolved)
    );
    assert_eq!(
        eval(&snapshot, visible("missing"), &[]),
        Err(Error::MissingNamespace(n("missing")))
    );
}
#[test]
fn membership_nonrecursive_and_excluded_recursive_do_not_lookup_namespace() {
    let snapshot = Snapshot::new(
        vec![],
        vec![(
            m("child"),
            Known(member(Target::Namespace(n("missing")), None, None)),
        )],
    )
    .unwrap();
    for recursive in [false, true] {
        let result = eval(
            &snapshot,
            Query::Import(Import::Membership {
                target: Known(Some(m("child"))),
                recursive,
                include_all: false,
            }),
            &[n("missing")],
        );
        assert_eq!(
            result,
            if recursive {
                output(&[])
            } else {
                output(&["child"])
            }
        );
    }
}
#[test]
fn base_import_differs_between_direct_and_helper_dispatch() {
    let snapshot = Snapshot::new(
        vec![(
            n("A"),
            Known(namespace(
                vec![],
                vec![(Import::BaseNull, Visibility::Public)],
            )),
        )],
        vec![],
    )
    .unwrap();
    assert_eq!(eval(&snapshot, visible("A"), &[]), output(&[]));
    assert_eq!(
        eval(&snapshot, Query::Imported(n("A")), &[]),
        Err(Error::NullImport)
    );
    assert_eq!(
        eval(&snapshot, Query::Import(Import::BaseNull), &[]),
        Err(Error::NullImport)
    );
}
#[test]
fn top_level_imports_use_fresh_helpers_nested_imports_share_visited() {
    let snapshot = Snapshot::new(
        vec![
            (
                n("A"),
                Known(namespace(
                    vec![],
                    vec![
                        (ns_import("B"), Visibility::Public),
                        (ns_import("B"), Visibility::Public),
                    ],
                )),
            ),
            (
                n("B"),
                Known(namespace(vec![(m("nameless"), Visibility::Public)], vec![])),
            ),
        ],
        vec![(m("nameless"), Known(member(Target::Element, None, None)))],
    )
    .unwrap();
    assert_eq!(
        eval(&snapshot, Query::Imported(n("A")), &[]),
        output(&["nameless", "nameless"])
    );
    assert_eq!(eval(&snapshot, visible("A"), &[]), output(&["nameless"]));
}
#[test]
fn outer_import_pipeline_does_not_apply_helper_public_visibility_filter() {
    let snapshot = Snapshot::new(
        vec![
            (
                n("A"),
                Known(namespace(
                    vec![],
                    vec![(ns_import("B"), Visibility::Private)],
                )),
            ),
            (
                n("B"),
                Known(namespace(vec![(m("one"), Visibility::Public)], vec![])),
            ),
        ],
        vec![(m("one"), Known(member(Target::Element, Some("one"), None)))],
    )
    .unwrap();
    assert_eq!(
        eval(&snapshot, Query::Imported(n("A")), &[]),
        output(&["one"])
    );
    assert_eq!(eval(&snapshot, visible("A"), &[]), output(&[]));
}
#[test]
fn unsupported_flags_and_type_are_errors_only_when_reached() {
    let mut typed = namespace(vec![], vec![]);
    typed.is_type = true;
    let snapshot = Snapshot::new(vec![(n("type"), Known(typed))], vec![]).unwrap();
    assert_eq!(
        eval(&snapshot, visible("type"), &[]),
        Err(Error::UnsupportedType)
    );
    assert_eq!(eval(&snapshot, visible("type"), &[n("type")]), output(&[]));
    assert_eq!(
        eval(
            &snapshot,
            Query::Import(Import::Namespace {
                target: Known(None),
                recursive: true,
                include_all: true
            }),
            &[]
        ),
        output(&[])
    );
}
#[test]
fn immutable_snapshot_survives_independent_replacement() {
    let mut members = vec![(m("one"), Known(member(Target::Element, Some("old"), None)))];
    let old = Snapshot::new(vec![], members.clone()).unwrap();
    members[0].1 = Observation::Unresolved;
    let next = Snapshot::new(vec![], members).unwrap();
    let query = || Query::Filter {
        seed: vec![],
        members: vec![m("one")],
    };
    assert_eq!(eval(&old.clone(), query(), &[]), output(&["one"]));
    assert_eq!(eval(&next, query(), &[]), Err(Error::Unresolved));
}
#[test]
fn duplicate_observation_identity_is_rejected() {
    assert!(matches!(
        Snapshot::new(
            vec![
                (n("A"), Observation::Unresolved),
                (n("A"), Observation::Unresolved)
            ],
            vec![]
        ),
        Err(Error::DuplicateIdentity)
    ));
    assert!(matches!(
        Snapshot::new(
            vec![],
            vec![
                (m("one"), Observation::Unresolved),
                (m("one"), Observation::Unresolved)
            ]
        ),
        Err(Error::DuplicateIdentity)
    ));
}
#[test]
fn depth_and_work_budgets_fail_without_partial_success() {
    let nodes = (0..200)
        .map(|i| {
            (
                n(&i.to_string()),
                Known(namespace(
                    vec![],
                    if i == 199 {
                        vec![]
                    } else {
                        vec![(ns_import(&(i + 1).to_string()), Visibility::Public)]
                    },
                )),
            )
        })
        .collect();
    let snapshot = Snapshot::new(nodes, vec![]).unwrap();
    assert_eq!(eval(&snapshot, visible("0"), &[]), Err(Error::Budget));
    assert_eq!(
        snapshot.evaluate(
            visible("0"),
            &[],
            Limits {
                steps: 0,
                depth: 128
            }
        ),
        Err(Error::Budget)
    );
    assert_eq!(
        snapshot.evaluate(
            visible("0"),
            &[],
            Limits {
                steps: 100,
                depth: 129
            }
        ),
        Err(Error::InvalidLimits)
    );
    assert_eq!(
        snapshot.evaluate(
            visible("0"),
            &[],
            Limits {
                steps: 100,
                depth: 0
            }
        ),
        Err(Error::InvalidLimits)
    );
}

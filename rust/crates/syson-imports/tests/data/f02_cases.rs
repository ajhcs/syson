// Generated from the immutable F02 JSON by generate_fixtures.py.
// Fixture SHA256: 9105f54c8e560dd8b31f5a020d58c6eea3b54a51a387439f3bc1fc89c44636c1
fn f02_members() -> Vec<(MemberId, Observation<Member>)> {
    vec![
        (
            m("owned"),
            Known(Member {
                target: Known(Target::Element),
                alias: Known(Alias::Distinct("owned-alias".into())),
                effective_name: Known(Some("real".into())),
                effective_short_name: Known(Some("own-short".into())),
            }),
        ),
        (
            m("a"),
            Known(Member {
                target: Known(Target::Element),
                alias: Known(Alias::SameReference),
                effective_name: Known(Some("a".into())),
                effective_short_name: Known(Some("s".into())),
            }),
        ),
        (
            m("b"),
            Known(Member {
                target: Known(Target::Element),
                alias: Known(Alias::SameReference),
                effective_name: Known(Some("b".into())),
                effective_short_name: Known(None),
            }),
        ),
        (
            m("a2"),
            Known(Member {
                target: Known(Target::Element),
                alias: Known(Alias::SameReference),
                effective_name: Known(Some("a".into())),
                effective_short_name: Known(None),
            }),
        ),
        (
            m("short-collision"),
            Known(Member {
                target: Known(Target::Element),
                alias: Known(Alias::SameReference),
                effective_name: Known(Some("new".into())),
                effective_short_name: Known(Some("a".into())),
            }),
        ),
        (
            m("pair"),
            Known(Member {
                target: Known(Target::Element),
                alias: Known(Alias::SameReference),
                effective_name: Known(Some("pair".into())),
                effective_short_name: Known(Some("pair".into())),
            }),
        ),
        (
            m("alias-distinct"),
            Known(Member {
                target: Known(Target::Element),
                alias: Known(Alias::Distinct("a".into())),
                effective_name: Known(Some("a".into())),
                effective_short_name: Known(Some("short".into())),
            }),
        ),
        (
            m("alias-same"),
            Known(Member {
                target: Known(Target::Element),
                alias: Known(Alias::SameReference),
                effective_name: Known(Some("a".into())),
                effective_short_name: Known(Some("short".into())),
            }),
        ),
        (
            m("nameless"),
            Known(Member {
                target: Known(Target::Element),
                alias: Known(Alias::None),
                effective_name: Known(None),
                effective_short_name: Known(None),
            }),
        ),
        (
            m("null-element"),
            Known(Member {
                target: Known(Target::Absent),
                alias: Known(Alias::None),
                effective_name: Known(None),
                effective_short_name: Known(None),
            }),
        ),
        (
            m("root"),
            Known(Member {
                target: Known(Target::Namespace(n("A"))),
                alias: Known(Alias::SameReference),
                effective_name: Known(Some("root".into())),
                effective_short_name: Known(None),
            }),
        ),
        (
            m("child"),
            Known(Member {
                target: Known(Target::Namespace(n("B"))),
                alias: Known(Alias::SameReference),
                effective_name: Known(Some("child".into())),
                effective_short_name: Known(None),
            }),
        ),
        (
            m("private"),
            Known(Member {
                target: Known(Target::Element),
                alias: Known(Alias::SameReference),
                effective_name: Known(Some("private".into())),
                effective_short_name: Known(None),
            }),
        ),
        (
            m("c-namespace"),
            Known(Member {
                target: Known(Target::Namespace(n("C"))),
                alias: Known(Alias::SameReference),
                effective_name: Known(Some("nestedC".into())),
                effective_short_name: Known(None),
            }),
        ),
    ]
}
fn f02_cases() -> Vec<Case> {
    vec![
        Case {
            label: "ordered-imports",
            snapshot: Snapshot::new(vec![], f02_members()).unwrap(),
            query: Query::Supplied {
                owned: vec![],
                calls: vec![
                    Known(Some(vec![m("b"), m("a")])),
                    Known(Some(vec![m("a2")])),
                ],
            },
            excluded: vec![n("X")],
            expected: Ok(Outcome::Memberships(vec![m("b"), m("a")])),
        },
        Case {
            label: "owned-seed-uses-effective-names",
            snapshot: Snapshot::new(vec![], f02_members()).unwrap(),
            query: Query::Supplied {
                owned: vec![m("owned")],
                calls: vec![Known(Some(vec![m("b")]))],
            },
            excluded: vec![],
            expected: Ok(Outcome::Memberships(vec![m("b")])),
        },
        Case {
            label: "nameless-duplicates-survive",
            snapshot: Snapshot::new(vec![], f02_members()).unwrap(),
            query: Query::Supplied {
                owned: vec![],
                calls: vec![Known(Some(vec![
                    m("nameless"),
                    m("nameless"),
                    m("null-element"),
                ]))],
            },
            excluded: vec![],
            expected: Ok(Outcome::Memberships(vec![
                m("nameless"),
                m("nameless"),
                m("null-element"),
            ])),
        },
        Case {
            label: "rejected-short-name-retains-primary",
            snapshot: Snapshot::new(vec![], f02_members()).unwrap(),
            query: Query::Filter {
                seed: vec!["a".into()],
                members: vec![m("short-collision")],
            },
            excluded: vec![],
            expected: Ok(Outcome::Memberships(vec![])),
        },
        Case {
            label: "primary-conflict-short-circuits",
            snapshot: Snapshot::new(vec![], f02_members()).unwrap(),
            query: Query::Filter {
                seed: vec!["a".into()],
                members: vec![m("a")],
            },
            excluded: vec![],
            expected: Ok(Outcome::Memberships(vec![])),
        },
        Case {
            label: "equal-alias-text-distinct-reference",
            snapshot: Snapshot::new(vec![], f02_members()).unwrap(),
            query: Query::Filter {
                seed: vec!["short".into()],
                members: vec![m("alias-distinct")],
            },
            excluded: vec![],
            expected: Ok(Outcome::Memberships(vec![m("alias-distinct")])),
        },
        Case {
            label: "same-alias-reference-checks-short",
            snapshot: Snapshot::new(vec![], f02_members()).unwrap(),
            query: Query::Filter {
                seed: vec!["short".into()],
                members: vec![m("alias-same")],
            },
            excluded: vec![],
            expected: Ok(Outcome::Memberships(vec![])),
        },
        Case {
            label: "same-name-and-short-name-self-conflict",
            snapshot: Snapshot::new(vec![], f02_members()).unwrap(),
            query: Query::Filter {
                seed: vec![],
                members: vec![m("pair")],
            },
            excluded: vec![],
            expected: Ok(Outcome::Memberships(vec![])),
        },
        Case {
            label: "cycle-A-B-A",
            snapshot: Snapshot::new(
                vec![
                    (
                        n("A"),
                        Known(Namespace {
                            is_type: false,
                            owned: vec![],
                            imports: vec![(
                                Import::Namespace {
                                    target: Known(Some(n("B"))),
                                    recursive: false,
                                    include_all: false,
                                },
                                Visibility::Public,
                            )],
                        }),
                    ),
                    (
                        n("B"),
                        Known(Namespace {
                            is_type: false,
                            owned: vec![(m("a"), Visibility::Public)],
                            imports: vec![(
                                Import::Namespace {
                                    target: Known(Some(n("A"))),
                                    recursive: false,
                                    include_all: false,
                                },
                                Visibility::Public,
                            )],
                        }),
                    ),
                ],
                f02_members(),
            )
            .unwrap(),
            query: Query::Visible {
                namespace: n("A"),
                recursive: false,
                include_all: false,
            },
            excluded: vec![],
            expected: Ok(Outcome::Memberships(vec![m("a")])),
        },
        Case {
            label: "excluded-target",
            snapshot: Snapshot::new(
                vec![
                    (
                        n("A"),
                        Known(Namespace {
                            is_type: false,
                            owned: vec![],
                            imports: vec![(
                                Import::Namespace {
                                    target: Known(Some(n("B"))),
                                    recursive: false,
                                    include_all: false,
                                },
                                Visibility::Public,
                            )],
                        }),
                    ),
                    (
                        n("B"),
                        Known(Namespace {
                            is_type: false,
                            owned: vec![(m("a"), Visibility::Public)],
                            imports: vec![],
                        }),
                    ),
                ],
                f02_members(),
            )
            .unwrap(),
            query: Query::Import(Import::Namespace {
                target: Known(Some(n("B"))),
                recursive: false,
                include_all: false,
            }),
            excluded: vec![n("B")],
            expected: Ok(Outcome::Memberships(vec![])),
        },
        Case {
            label: "diamond-shared-visited",
            snapshot: Snapshot::new(
                vec![
                    (
                        n("A"),
                        Known(Namespace {
                            is_type: false,
                            owned: vec![],
                            imports: vec![
                                (
                                    Import::Namespace {
                                        target: Known(Some(n("B"))),
                                        recursive: false,
                                        include_all: false,
                                    },
                                    Visibility::Public,
                                ),
                                (
                                    Import::Namespace {
                                        target: Known(Some(n("C"))),
                                        recursive: false,
                                        include_all: false,
                                    },
                                    Visibility::Public,
                                ),
                            ],
                        }),
                    ),
                    (
                        n("B"),
                        Known(Namespace {
                            is_type: false,
                            owned: vec![(m("c-namespace"), Visibility::Public)],
                            imports: vec![(
                                Import::Namespace {
                                    target: Known(Some(n("C"))),
                                    recursive: false,
                                    include_all: false,
                                },
                                Visibility::Public,
                            )],
                        }),
                    ),
                    (
                        n("C"),
                        Known(Namespace {
                            is_type: false,
                            owned: vec![(m("b"), Visibility::Public)],
                            imports: vec![],
                        }),
                    ),
                ],
                f02_members(),
            )
            .unwrap(),
            query: Query::Visible {
                namespace: n("A"),
                recursive: false,
                include_all: false,
            },
            excluded: vec![],
            expected: Ok(Outcome::Memberships(vec![m("c-namespace"), m("b")])),
        },
        Case {
            label: "visibility-public-only",
            snapshot: Snapshot::new(
                vec![(
                    n("A"),
                    Known(Namespace {
                        is_type: false,
                        owned: vec![
                            (m("a"), Visibility::Public),
                            (m("private"), Visibility::Private),
                        ],
                        imports: vec![],
                    }),
                )],
                f02_members(),
            )
            .unwrap(),
            query: Query::Visible {
                namespace: n("A"),
                recursive: false,
                include_all: false,
            },
            excluded: vec![],
            expected: Ok(Outcome::Memberships(vec![m("a")])),
        },
        Case {
            label: "visibility-include-all",
            snapshot: Snapshot::new(
                vec![(
                    n("A"),
                    Known(Namespace {
                        is_type: false,
                        owned: vec![
                            (m("a"), Visibility::Public),
                            (m("private"), Visibility::Private),
                        ],
                        imports: vec![],
                    }),
                )],
                f02_members(),
            )
            .unwrap(),
            query: Query::Visible {
                namespace: n("A"),
                recursive: false,
                include_all: true,
            },
            excluded: vec![],
            expected: Err(Error::UnsupportedIncludeAll),
        },
        Case {
            label: "recursive-membership-excluded-target",
            snapshot: Snapshot::new(
                vec![
                    (
                        n("A"),
                        Known(Namespace {
                            is_type: false,
                            owned: vec![(m("child"), Visibility::Public)],
                            imports: vec![],
                        }),
                    ),
                    (
                        n("B"),
                        Known(Namespace {
                            is_type: false,
                            owned: vec![(m("b"), Visibility::Public)],
                            imports: vec![],
                        }),
                    ),
                ],
                f02_members(),
            )
            .unwrap(),
            query: Query::Import(Import::Membership {
                target: Known(Some(m("child"))),
                recursive: true,
                include_all: false,
            }),
            excluded: vec![n("B")],
            expected: Ok(Outcome::Memberships(vec![])),
        },
        Case {
            label: "nonrecursive-membership-ignores-namespace-exclusion",
            snapshot: Snapshot::new(
                vec![
                    (
                        n("A"),
                        Known(Namespace {
                            is_type: false,
                            owned: vec![(m("child"), Visibility::Public)],
                            imports: vec![],
                        }),
                    ),
                    (
                        n("B"),
                        Known(Namespace {
                            is_type: false,
                            owned: vec![(m("b"), Visibility::Public)],
                            imports: vec![],
                        }),
                    ),
                ],
                f02_members(),
            )
            .unwrap(),
            query: Query::Import(Import::Membership {
                target: Known(Some(m("child"))),
                recursive: false,
                include_all: false,
            }),
            excluded: vec![n("B")],
            expected: Ok(Outcome::Memberships(vec![m("child")])),
        },
        Case {
            label: "recursive-namespace-descendants-appended",
            snapshot: Snapshot::new(
                vec![
                    (
                        n("A"),
                        Known(Namespace {
                            is_type: false,
                            owned: vec![(m("child"), Visibility::Public)],
                            imports: vec![],
                        }),
                    ),
                    (
                        n("B"),
                        Known(Namespace {
                            is_type: false,
                            owned: vec![(m("b"), Visibility::Public)],
                            imports: vec![],
                        }),
                    ),
                ],
                f02_members(),
            )
            .unwrap(),
            query: Query::Visible {
                namespace: n("A"),
                recursive: true,
                include_all: false,
            },
            excluded: vec![],
            expected: Ok(Outcome::Memberships(vec![m("child"), m("b")])),
        },
        Case {
            label: "memberships-of-visibility-is-null",
            snapshot: Snapshot::new(vec![], f02_members()).unwrap(),
            query: Query::MembershipsOfVisibility,
            excluded: vec![],
            expected: Ok(Outcome::NullUnimplemented),
        },
        Case {
            label: "unresolved-proxy-not-absence",
            snapshot: Snapshot::new(vec![(n("A"), Observation::Unresolved)], f02_members())
                .unwrap(),
            query: Query::Visible {
                namespace: n("A"),
                recursive: false,
                include_all: false,
            },
            excluded: vec![],
            expected: Err(Error::Unresolved),
        },
        Case {
            label: "base-import-null-then-flatten",
            snapshot: Snapshot::new(
                vec![(
                    n("A"),
                    Known(Namespace {
                        is_type: false,
                        owned: vec![],
                        imports: vec![(Import::BaseNull, Visibility::Public)],
                    }),
                )],
                f02_members(),
            )
            .unwrap(),
            query: Query::Imported(n("A")),
            excluded: vec![],
            expected: Err(Error::NullImport),
        },
    ]
}

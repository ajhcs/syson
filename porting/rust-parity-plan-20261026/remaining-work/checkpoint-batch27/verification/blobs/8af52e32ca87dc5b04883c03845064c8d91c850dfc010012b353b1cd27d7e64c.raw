# Seven teams, one integration coordinator

Every one of the 790 current units has exactly one primary team in [teams.json](teams.json). These are responsibility groups, not seven simultaneously running agents. The existing tasks, paths, prerequisites, evidence requirements, holds and acceptance criteria remain in force.

| Primary team | Responsibility | Units |
| --- | --- | ---: |
| reference | Native/source/artifact closure, observed contracts and reference handoffs | 14 |
| semantic-model | Graph, identity, library, construction, query, validation and model repairs | 407 |
| language-interchange | Parsing, linking, text/export and model interchange | 198 |
| services-persistence | Hosts, API, collaboration, migrations, persistence and recovery | 47 |
| geometry-diagrams | Layout, forests, routing, diagram providers and graphical semantics | 62 |
| interaction-accessibility | Editing, navigation, forms, connected journeys and accessibility | 32 |
| verification-release | Differential evidence, integration checks, resources and release acceptance | 30 |

The integration coordinator is a separate role, not an eighth team. At most three source authors and one coordinator may run concurrently. The coordinator serializes builds, native processes, databases and browser qualification, and owns shared manifests, locks, exports, routes, registries and application assembly. Reviews use available actors or follow author freeze; they do not create extra active-agent slots.

A team assignment grants no write lease. Before dispatch, the coordinator checks actual prerequisites and current owners, records exact nonoverlapping paths, and obtains explicit owner handoffs for any held overlap. Existing raw/factory/initializer/validation holds and #130's 12 held registrations remain intact. The independent reviewer cannot be the author; no team or coordinator may replace independent feature acceptance with its own summary PASS.

Different teams can still touch the same source. RR06 and RR07 share text-export code; RR01 and RR24a share JSON infrastructure; interaction repairs meet geometry in the renderer and layout caller. Refine or order their leases before writing. Keep each change in its own PR and count all helpers, tests and coordinator wiring against the existing budgets.

The generated team-dispatch.json projects every cross-team edge from the complete typed readiness graph, including expanded joint acceptance inputs. An edge points from a dependent implementation/acceptance node to its prerequisite. Teams can therefore have dependencies in both directions without permitting a cycle in the underlying unit graph. A team-level display never replaces the original readiness graph.

1. Merge this versioned plan amendment before starting the next implementation batch.
2. Select units whose original dependency and ownership gates permit work; team labels do not make blocked units ready.
3. Record the coordinator, author, independent reviewer, primary team, exact paths and live ownership/handoff disposition for each lease.
4. Freeze source, obtain independent review, execute the applicable native/Rust/UI/failure/resource checks, preserve failures, then publish and merge eligible PRs.
5. Update execution status truthfully. Implementation, bounded support, provisional reference evidence and feature acceptance remain distinct.

The static API validate_teams(policy, plan) returns assignment, cross_team_edges, team_counts and policy. validate_projection rejects a changed generated view. validate_dispatch additionally checks declared author/reviewer roles, the three-author/four-agent cap, path overlap, manifest/lock ancestry, unit-specific coordinator wiring and declared held handoffs. It requires ownership_admitted_by to name the coordinator. Known factory/validation scopes cannot be declared free. This checks declarations; it does not inspect live agents, authenticate a handoff, discover every held path or establish readiness. The coordinator still verifies those facts against the current ownership record.

The preservation proof covers all 790 normalized unit records and the full typed readiness graph. Only completion_contract and source_fragment_sha256 are omitted from the unit hash because regeneration changes those bindings; tasks, dependencies, paths, gates and unknown extra fields stay protected. Independent review and all common-candidate C99/native/UI/release gates remain unchanged.

A later reference-work split may hand off separate artifact inventories, model captures and API observations through immutable input/output packets. That is a proposal for a future reviewed amendment, not new child IDs or an implementation authorization. Until then, C01 and its declared units retain their complete obligations.

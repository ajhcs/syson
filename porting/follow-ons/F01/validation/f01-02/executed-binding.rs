// SPDX-License-Identifier: EPL-2.0
// Synthetic selected-JSON/prefix observations; no native runtime oracle.
use serde_json::{Value, json};
use syson_domain::model::Visibility;
use syson_general_view::*;
use syson_initializer::{
    Context, ContextGap, Effect, ExternalObject, Problem, Resolution, Scalar, ScalarFeature, Step,
    Target,
};
use syson_initializer_graph::{ReferenceEncoding, native_query::*};
use syson_metamodel::ClassifierId as C;
use syson_model::Document;
const LOCAL: ReferenceEncoding = ReferenceEncoding::SemanticElementIds;
#[derive(Default)]
struct Local {
    namespace: Option<ExternalObject>,
    cancel: bool,
}
impl Context for Local {
    fn cancelled(&mut self) -> bool {
        self.cancel
    }
    fn owning_namespace(&mut self, _: &[Step]) -> Resolution<Option<ExternalObject>> {
        Resolution::Known(self.namespace.clone())
    }
    fn exact_class_owned_member_count(
        &mut self,
        _: &ExternalObject,
        _: C,
        _: &[Step],
    ) -> Resolution<u64> {
        Resolution::Known(0)
    }
    fn owning_membership(&mut self, _: &[Step]) -> Resolution<Option<ExternalObject>> {
        panic!("unexpected local query")
    }
    fn owner(&mut self, _: &[Step]) -> Resolution<Option<ExternalObject>> {
        panic!("unexpected local query")
    }
    fn is_instance_of(&mut self, _: &ExternalObject, _: C, _: &[Step]) -> Resolution<bool> {
        panic!("unexpected local query")
    }
    fn effective_name(&mut self, _: &ExternalObject, _: &[Step]) -> Resolution<Option<String>> {
        panic!("unexpected local query")
    }
    fn general_view_definition(&mut self, _: &[Step]) -> Resolution<Option<ExternalObject>> {
        panic!("legacy unbound lookup must not execute")
    }
}
fn node(id: &str, class: &str, extra: Value) -> Value {
    let mut data = extra.as_object().unwrap().clone();
    data.insert("elementId".into(), json!(id));
    json!({"id":format!("wire-{id}"),"eClass":format!("sysml:{class}"),"data":data})
}
fn document(roots: Vec<Value>) -> Document {
    Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":roots}).to_string()).unwrap()
}
fn resource(key: &str, roots: Vec<Value>) -> ResourceSnapshot {
    ResourceSnapshot::new(key, document(roots))
}
fn app() -> ResourceSnapshot {
    resource(
        "app",
        vec![node(
            "root",
            "ViewUsage",
            json!({"declaredName":"view0","isComposite":true,"ownedRelationship":[node("query-typing","FeatureTyping",json!({}))]}),
        )],
    )
}
fn library(key: &str, class: &str) -> ResourceSnapshot {
    resource(
        key,
        vec![node(
            "lib",
            "Package",
            json!({"declaredName":"StandardViewDefinitions","ownedRelationship":[node("member","OwningMembership",json!({"ownedRelatedElement":[node("view","ViewDefinition",json!({"declaredName":"unused"})),node("target",class,json!({"declaredName":"GeneralView"}))]}))]}),
        )],
    )
}
fn context() -> ResourceContext {
    ResourceContext::Member(ObjectId::new("app", "root"))
}
fn planned(outcome: Outcome) -> Plan {
    match outcome {
        Outcome::Planned(p) => p,
        Outcome::Incomplete { problem, .. } => panic!("{problem:?}"),
    }
}
fn bind(set: ResourceSet, ctx: ResourceContext) -> Outcome {
    let observation = ctx.clone();
    let mut set = Some(set);
    plan(ctx, LOCAL, &mut Local::default(), move |prefix| {
        Ok(PrefixSnapshot::new(set.take().unwrap(), observation.clone(), prefix.to_vec()).unwrap())
    })
}
fn found(plan: &Plan) -> &ScopedTarget {
    match plan.lookup() {
        Lookup::Found(t) => t,
        s => panic!("{s:?}"),
    }
}
fn type_value(plan: &Plan) -> Option<&ScopedTarget> {
    plan.steps()
        .iter()
        .find_map(|s| match s {
            BoundStep::GeneralViewType { definition, .. } => Some(definition.as_ref()),
            _ => None,
        })
        .expect("actual source Type assignment")
}
fn no_external(step: &BoundStep) -> bool {
    match step {
        BoundStep::GeneralViewType { typing, .. } => !matches!(typing, Target::External(_)),
        BoundStep::Local(s) => match &s.effect {
            Effect::AllocateRaw { .. } => true,
            Effect::SetScalar { target, .. } => !matches!(target, Target::External(_)),
            Effect::Attach { owner, child, .. } => {
                !matches!(owner, Target::External(_)) && !matches!(child, Target::External(_))
            }
            Effect::SetReference { target, value, .. } => {
                !matches!(target, Target::External(_))
                    && !matches!(value, Some(Target::External(_)))
            }
        },
    }
}

#[test]
fn actual_planner_prefix_and_type_assignment_keep_resource_and_snapshot_identity() {
    let mut set = Some(ResourceSet::new(vec![app(), library("stdlib", "ViewDefinition")]).unwrap());
    let p = planned(plan(context(), LOCAL, &mut Local::default(), |prefix| {
        assert_eq!(prefix.len(), 4);
        assert!(
            matches!(&prefix[0].effect,Effect::SetScalar {target:Target::Root,feature:ScalarFeature::DeclaredName,value:Scalar::Text(n)} if n=="view0")
        );
        assert!(matches!(
            &prefix[3].effect,
            Effect::Attach {
                owner: Target::Root,
                ..
            }
        ));
        Ok(PrefixSnapshot::new(set.take().unwrap(), context(), prefix.to_vec()).unwrap())
    }));
    let snapshot = p.snapshot().unwrap();
    snapshot.validate(&p).unwrap();
    let target = found(&p);
    assert_eq!(type_value(&p), Some(target));
    assert_eq!(
        snapshot.resolve(target).unwrap(),
        &ObjectId::new("stdlib", "target")
    );
    assert!(p.steps().iter().all(no_external));
    assert!(matches!(
        p.steps().last(),
        Some(BoundStep::Local(Step {
            effect: Effect::SetReference {
                feature: syson_initializer::ReferenceFeature::TypedFeature,
                value: Some(Target::Root),
                ..
            },
            ..
        }))
    ));
    assert!(p.trace().iter().any(|i| i.case_name == "ViewUsage"));
}
#[test]
fn equal_semantic_ids_in_two_resources_follow_first_resource_order() {
    for (a, b) in [("one", "two"), ("two", "one")] {
        let p = planned(bind(
            ResourceSet::new(vec![
                app(),
                library(a, "ViewDefinition"),
                library(b, "ViewDefinition"),
            ])
            .unwrap(),
            context(),
        ));
        assert_eq!(
            p.snapshot().unwrap().resolve(found(&p)).unwrap(),
            &ObjectId::new(a, "target")
        );
    }
}
#[test]
fn wrong_type_in_an_earlier_resource_does_not_mask_a_later_valid_root() {
    let p = planned(bind(
        ResourceSet::new(vec![
            app(),
            library("wrong", "PartDefinition"),
            library("right", "ViewDefinition"),
        ])
        .unwrap(),
        context(),
    ));
    assert_eq!(
        p.snapshot().unwrap().resolve(found(&p)).unwrap().resource,
        "right"
    );
}
#[test]
fn known_absence_preserves_detached_missing_set_and_no_match_distinctions() {
    for (ctx, reason) in [
        (ResourceContext::Detached, Absence::Detached),
        (
            ResourceContext::ResourceWithoutSet,
            Absence::MissingResourceSet,
        ),
        (context(), Absence::NoMatch),
    ] {
        let p = planned(bind(ResourceSet::new(vec![app()]).unwrap(), ctx));
        assert_eq!(p.lookup(), &Lookup::Absent(reason));
        assert_eq!(type_value(&p), None);
    }
}
#[test]
fn unresolved_member_is_incomplete_and_never_becomes_a_null_type_assignment() {
    for ctx in [
        ResourceContext::Member(ObjectId::new("absent", "root")),
        ResourceContext::Member(ObjectId::new("app", "absent")),
    ] {
        match bind(ResourceSet::new(vec![app()]).unwrap(), ctx) {
            Outcome::Incomplete {
                prefix,
                problem: Problem::Context(_),
            } => {
                assert_eq!(prefix.lookup(), &Lookup::Unresolved);
                assert!(
                    !prefix
                        .steps()
                        .iter()
                        .any(|s| matches!(s, BoundStep::GeneralViewType { .. }))
                );
            }
            _ => panic!("unresolved query must remain incomplete"),
        }
    }
}
#[test]
fn only_public_import_observations_can_supply_the_scoped_target_and_later_gaps_are_lazy() {
    let facade = resource(
        "facade",
        vec![node(
            "lib",
            "Package",
            json!({"declaredName":"StandardViewDefinitions","ownedRelationship":[node("import","NamespaceImport",json!({}))]}),
        )],
    );
    let set = ResourceSet::new(vec![
        app(),
        facade,
        resource(
            "external",
            vec![node(
                "target",
                "ViewDefinition",
                json!({"declaredName":"GeneralView"}),
            )],
        ),
    ])
    .unwrap()
    .with_imported_memberships(
        ObjectId::new("facade", "lib"),
        vec![
            ImportedMembershipObservation {
                visibility: Visibility::Private,
                related: vec![ObjectId::new("absent", "hidden")],
            },
            ImportedMembershipObservation {
                visibility: Visibility::Public,
                related: vec![
                    ObjectId::new("external", "target"),
                    ObjectId::new("absent", "late"),
                ],
            },
        ],
    )
    .unwrap();
    let p = planned(bind(set, context()));
    assert_eq!(
        p.snapshot().unwrap().resolve(found(&p)).unwrap(),
        &ObjectId::new("external", "target")
    );
}
#[test]
fn missing_import_observation_remains_unresolved() {
    let facade = resource(
        "facade",
        vec![node(
            "lib",
            "Package",
            json!({"declaredName":"StandardViewDefinitions","ownedRelationship":[node("import","NamespaceImport",json!({}))]}),
        )],
    );
    match bind(ResourceSet::new(vec![app(), facade]).unwrap(), context()) {
        Outcome::Incomplete {
            prefix,
            problem: Problem::Context(_),
        } => assert_eq!(prefix.lookup(), &Lookup::Unresolved),
        _ => panic!("no invented empty imported set"),
    }
}
#[test]
fn prefix_and_context_mismatches_refuse_before_any_target_is_bound() {
    for wrong_context in [false, true] {
        let mut set =
            Some(ResourceSet::new(vec![app(), library("stdlib", "ViewDefinition")]).unwrap());
        let outcome = plan(context(), LOCAL, &mut Local::default(), |prefix| {
            Ok(PrefixSnapshot::new(
                set.take().unwrap(),
                if wrong_context {
                    ResourceContext::Detached
                } else {
                    context()
                },
                if wrong_context {
                    prefix.to_vec()
                } else {
                    vec![]
                },
            )
            .unwrap())
        });
        match outcome {
            Outcome::Incomplete {
                prefix,
                problem: Problem::Context(_),
            } => assert_eq!(prefix.lookup(), &Lookup::StaleObservation),
            _ => panic!("stale observation must not become absence"),
        }
    }
}
#[test]
fn a_pre_rename_observation_is_stale_and_the_correct_query_point_changes_the_result() {
    let before = resource(
        "app",
        vec![node(
            "root",
            "ViewUsage",
            json!({"declaredName":"StandardViewDefinitions","ownedRelationship":[node("member","OwningMembership",json!({"ownedRelatedElement":[node("target","ViewDefinition",json!({"declaredName":"GeneralView"}))]}))]}),
        )],
    );
    let before_set = ResourceSet::new(vec![before.clone()]).unwrap();
    assert_eq!(
        before_set.find_general_view(&context(), LOCAL).unwrap(),
        Some(ObjectId::new("app", "target"))
    );
    let mut stale = Some(PrefixSnapshot::new(before_set, context(), vec![]).unwrap());
    assert!(matches!(
        plan(context(), LOCAL, &mut Local::default(), |_| Ok(stale
            .take()
            .unwrap())),
        Outcome::Incomplete {
            problem: Problem::Context(_),
            ..
        }
    ));
    let mut value = before.document.value().clone();
    value["content"][0]["data"]["declaredName"] = json!("view0");
    value["content"][0]["data"]["isComposite"] = json!(true);
    value["content"][0]["data"]["ownedRelationship"]
        .as_array_mut()
        .unwrap()
        .push(node("query-typing", "FeatureTyping", json!({})));
    let after = Document::from_json(&value.to_string()).unwrap();
    let p = planned(bind(
        ResourceSet::new(vec![ResourceSnapshot::new("app", after)]).unwrap(),
        context(),
    ));
    assert_eq!(p.lookup(), &Lookup::Absent(Absence::NoMatch));
    assert_eq!(type_value(&p), None);
}
#[test]
fn identical_bytes_in_a_new_snapshot_do_not_authorize_old_targets_or_plans() {
    let a = planned(bind(
        ResourceSet::new(vec![app(), library("stdlib", "ViewDefinition")]).unwrap(),
        context(),
    ));
    let b = planned(bind(
        ResourceSet::new(vec![app(), library("stdlib", "ViewDefinition")]).unwrap(),
        context(),
    ));
    let (sa, sb) = (a.snapshot().unwrap(), b.snapshot().unwrap());
    assert_ne!(sa.identity(), sb.identity());
    assert_eq!(sb.resolve(found(&a)), Err(BindingError::StaleSnapshot));
    assert_eq!(sb.validate(&a), Err(BindingError::StaleSnapshot));
    sa.validate(&a).unwrap();
    assert_eq!(sa.resolve(found(&a)).unwrap().element, "target");
}
#[test]
fn cancellation_and_provider_failure_keep_incomplete_prefixes_without_mutation() {
    let mut cancelled = Local {
        namespace: None,
        cancel: true,
    };
    assert!(matches!(
        plan(context(), LOCAL, &mut cancelled, |_| panic!(
            "provider must not run"
        )),
        Outcome::Incomplete {
            problem: Problem::Cancelled,
            ..
        }
    ));
    let outcome = plan(context(), LOCAL, &mut Local::default(), |_| {
        Err(ContextGap {
            contract: "prefix observation",
            reason: "unavailable".into(),
        })
    });
    match outcome {
        Outcome::Incomplete {
            prefix,
            problem: Problem::Context(e),
        } => {
            assert_eq!(e.contract, "prefix observation");
            assert_eq!(prefix.lookup(), &Lookup::Unresolved);
        }
        _ => panic!("provider refusal must not become absence"),
    }
}

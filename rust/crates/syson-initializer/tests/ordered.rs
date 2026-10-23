// SPDX-License-Identifier: EPL-2.0
// Source-contract regressions derived from pinned Java; ALL NOT_RUN.
use syson_initializer::*;
use syson_metamodel::ClassifierId as C;
#[derive(Clone)]
struct Fixture {
    namespace: Option<ExternalObject>, count: u64,
    membership: Option<ExternalObject>, owner: Option<ExternalObject>,
    instance_of: Vec<(String, C)>, name: Option<String>,
    library: Resolution<Option<ExternalObject>>, cancel: bool,
    calls: Vec<(&'static str, usize)>, counts: Vec<(C, usize)>,
}
impl Default for Fixture {
    fn default() -> Self {
        Self { namespace: None, count: 0, membership: None, owner: None,
            instance_of: Vec::new(), name: None, library: Resolution::Known(None),
            cancel: false, calls: Vec::new(), counts: Vec::new() }
    }
}
impl Context for Fixture {
    fn cancelled(&mut self) -> bool { self.cancel }
    fn owning_namespace(&mut self, p: &[Step]) -> Resolution<Option<ExternalObject>> {
        self.calls.push(("namespace", p.len())); Resolution::Known(self.namespace.clone())
    }
    fn exact_class_owned_member_count(&mut self, _: &ExternalObject, c: C, p: &[Step]) -> Resolution<u64> {
        self.calls.push(("count", p.len())); self.counts.push((c, p.len())); Resolution::Known(self.count)
    }
    fn owning_membership(&mut self, p: &[Step]) -> Resolution<Option<ExternalObject>> {
        self.calls.push(("membership", p.len())); Resolution::Known(self.membership.clone())
    }
    fn owner(&mut self, p: &[Step]) -> Resolution<Option<ExternalObject>> {
        self.calls.push(("owner", p.len())); Resolution::Known(self.owner.clone())
    }
    fn is_instance_of(&mut self, o: &ExternalObject, c: C, p: &[Step]) -> Resolution<bool> {
        self.calls.push(("instanceof", p.len()));
        Resolution::Known(self.instance_of.iter().any(|(id, class)| id == &o.0 && *class == c))
    }
    fn effective_name(&mut self, _: &ExternalObject, p: &[Step]) -> Resolution<Option<String>> {
        self.calls.push(("name", p.len())); Resolution::Known(self.name.clone())
    }
    fn general_view_definition(&mut self, p: &[Step]) -> Resolution<Option<ExternalObject>> {
        self.calls.push(("general_view", p.len())); Resolution::Known(match &self.library {
            Resolution::Known(v) => v.clone(),
            _ => return self.library.clone(),
        })
    }
}
fn planned(c: C, f: &mut Fixture) -> Plan {
    match plan(c, f) { Outcome::Planned(p) => p, other => panic!("unexpected incomplete plan: {other:?}") }
}
fn named(p: &Plan, t: &Target) -> Vec<String> {
    p.steps.iter().filter_map(|s| match &s.effect {
        Effect::SetScalar { target, feature: ScalarFeature::DeclaredName, value: Scalar::Text(n) } if target == t => Some(n.clone()),
        _ => None,
    }).collect()
}
fn allocations(p: &Plan) -> Vec<(PlannedObject, C)> {
    p.steps.iter().filter_map(|s| match &s.effect {
        Effect::AllocateRaw { object, classifier, identity: IdentityRequirement::FreshSemanticUuidV4ResourceIdUnassigned } => Some((*object, *classifier)),
        _ => None,
    }).collect()
}
#[test]
fn missing_namespace_is_known_zero_without_plus_one() {
    let mut f = Fixture::default();
    let p = planned(C::Package, &mut f);
    assert_eq!(named(&p, &Target::Root), ["Package0"]);
    assert!(f.counts.is_empty());
    assert_eq!(p.application, ApplicationStatus::Unported);
}
#[test]
fn exact_class_count_is_already_after_root_attachment() {
    let mut f = Fixture { namespace: Some(ExternalObject("ns".into())), count: 7, ..Fixture::default() };
    let p = planned(C::AttributeUsage, &mut f);
    assert_eq!(named(&p, &Target::Root), ["attribute7"]);
    assert_eq!(f.counts, [(C::AttributeUsage, 0)]);
    let composite: Vec<_> = p.steps.iter().filter_map(|s| match &s.effect {
        Effect::SetScalar { feature: ScalarFeature::IsComposite, value: Scalar::Boolean(v), .. } => Some(*v), _ => None
    }).collect();
    assert_eq!(composite, [true, false]);
}
#[test]
fn nonnull_no_op_overrides_do_not_initialize_usage() {
    for c in [C::BindingConnectorAsUsage, C::FlowUsage, C::PerformActionUsage, C::SuccessionAsUsage, C::TransitionUsage] {
        let mut f = Fixture::default(); let p = planned(c, &mut f);
        assert!(p.steps.is_empty(), "{c:?}");
        assert!(!p.dispatch_trace.iter().any(|call| call.case_name == "Usage"), "{c:?}");
        assert!(f.calls.is_empty(), "{c:?}");
    }
}
#[test]
fn flow_end_preserves_direct_generated_feature_call() {
    let p = planned(C::FlowEnd, &mut Fixture::default());
    assert_eq!(p.steps.len(), 1);
    assert!(matches!(&p.steps[0].effect, Effect::SetScalar { feature: ScalarFeature::IsEnd, value: Scalar::Boolean(true), .. }));
    assert!(p.dispatch_trace.iter().any(|c| c.case_name == "Feature" && c.kind == CallKind::DirectCase));
    assert!(!p.dispatch_trace.iter().any(|c| c.case_name == "Usage"));
}
#[test]
fn interface_creates_ports_then_connection_references_then_name() {
    let mut f = Fixture { namespace: Some(ExternalObject("ns".into())), count: 7, ..Fixture::default() };
    let p = planned(C::InterfaceDefinition, &mut f);
    let classes: Vec<_> = allocations(&p).into_iter().map(|(_, c)| c).collect();
    assert_eq!(classes, [C::PortUsage,C::FeatureMembership,C::PortUsage,C::FeatureMembership,C::ReferenceUsage,C::FeatureMembership,C::ReferenceUsage,C::FeatureMembership]);
    let generated: Vec<_> = p.dispatch_trace.iter().filter(|c| c.kind == CallKind::GeneratedDispatch).map(|c| c.case_name).collect();
    let pos = |name| generated.iter().position(|n| *n == name).unwrap();
    assert!(pos("InterfaceDefinition") < pos("ConnectionDefinition"));
    assert!(pos("ConnectionDefinition") < pos("Definition"));
    assert!(p.dispatch_trace.iter().any(|c| c.case_name == "InterfaceDefinition" && c.kind == CallKind::SuperCase));
    assert_eq!(named(&p, &Target::Root), ["InterfaceDefinition7"]);
    let sufficient = p.steps.iter().filter(|s| matches!(&s.effect, Effect::SetScalar { feature: ScalarFeature::IsSufficient, value: Scalar::Boolean(true), .. })).count();
    assert_eq!(sufficient, 2); // preserve duplicate source setter requests
    assert_eq!(f.counts, [(C::InterfaceDefinition, p.steps.len()-1)]);
}
#[test]
fn direct_port_helper_stays_raw_but_root_port_runs_usage_fallback() {
    let interface = planned(C::InterfaceDefinition, &mut Fixture::default());
    for (id, class) in allocations(&interface) {
        if class == C::PortUsage {
            assert!(!interface.steps.iter().any(|s| matches!(&s.effect, Effect::SetScalar { target: Target::Planned(t), feature: ScalarFeature::IsComposite, .. } if *t == id)));
        }
    }
    let root = planned(C::PortUsage, &mut Fixture::default());
    assert!(root.steps.iter().any(|s| matches!(&s.effect, Effect::SetScalar { target: Target::Root, feature: ScalarFeature::IsComposite, value: Scalar::Boolean(true) })));
}
#[test]
fn accept_parameters_preserve_rawness_directions_and_attachment_order() {
    let p = planned(C::AcceptActionUsage, &mut Fixture::default());
    let allocations = allocations(&p);
    assert_eq!(allocations.iter().map(|(_, c)| *c).collect::<Vec<_>>(), [C::ReferenceUsage,C::ParameterMembership,C::ReferenceUsage,C::ParameterMembership]);
    for (id, class) in &allocations {
        if *class == C::ReferenceUsage {
            assert!(!p.steps.iter().any(|s| matches!(&s.effect, Effect::SetScalar { target: Target::Planned(t), feature: ScalarFeature::IsComposite, .. } if t == id)));
        }
    }
    let directions: Vec<_> = p.steps.iter().filter_map(|s| match &s.effect {
        Effect::SetScalar { value: Scalar::Direction(d), .. } => Some(*d), _ => None
    }).collect();
    assert_eq!(directions, [Direction::InOut,Direction::In]);
    for (reference, membership) in [(allocations[0].0,allocations[1].0),(allocations[2].0,allocations[3].0)] {
        let inner = p.steps.iter().position(|s| matches!(&s.effect, Effect::Attach { owner: Target::Planned(m), child: Target::Planned(r), feature: Containment::OwnedRelatedElement } if *m==membership && *r==reference)).unwrap();
        let outer = p.steps.iter().position(|s| matches!(&s.effect, Effect::Attach { owner: Target::Root, child: Target::Planned(m), feature: Containment::OwnedRelationship } if *m==membership)).unwrap();
        assert!(inner < outer);
    }
}
#[test]
fn connection_helpers_keep_intermediate_reference_initialization() {
    let p = planned(C::ConnectionDefinition, &mut Fixture::default());
    let refs: Vec<_> = allocations(&p).into_iter().filter(|(_, c)| *c==C::ReferenceUsage).collect();
    assert_eq!(named(&p,&Target::Planned(refs[0].0)), ["reference0","source"]);
    assert_eq!(named(&p,&Target::Planned(refs[1].0)), ["reference0","target"]);
}
#[test]
fn port_definition_conjugation_is_ordered_and_not_recursive() {
    let p = planned(C::PortDefinition, &mut Fixture::default());
    let a = allocations(&p);
    assert_eq!(a.iter().map(|(_, c)| *c).collect::<Vec<_>>(), [C::OwningMembership,C::ConjugatedPortDefinition,C::PortConjugation]);
    assert_eq!(named(&p, &Target::Planned(a[1].0)), Vec::<String>::new());
    assert!(p.steps.iter().any(|s| matches!(&s.effect, Effect::SetReference { target: Target::Planned(t), feature: ReferenceFeature::ConjugatedType, value: Some(Target::Planned(c)) } if *t==a[2].0 && *c==a[1].0)));
    assert!(p.steps.iter().any(|s| matches!(&s.effect, Effect::SetReference { target: Target::Planned(t), feature: ReferenceFeature::OriginalPortDefinition, value: Some(Target::Root) } if *t==a[2].0)));
}
#[test]
fn actor_precedes_stakeholder_and_count_is_not_role_filtered() {
    let mut f=Fixture { namespace:Some(ExternalObject("ns".into())),count:7,membership:Some(ExternalObject("m".into())),
        instance_of:vec![("m".into(),C::ActorMembership),("m".into(),C::StakeholderMembership)],..Fixture::default() };
    let p=planned(C::PartUsage,&mut f);
    assert_eq!(named(&p,&Target::Root), ["part7","actor7"]);
    assert_eq!(f.counts, [(C::PartUsage,0),(C::PartUsage,2)]);
}
#[test]
fn subject_priority_does_not_skip_source_owner_query() {
    let mut f=Fixture { membership:Some(ExternalObject("m".into())),owner:Some(ExternalObject("o".into())),
        instance_of:vec![("m".into(),C::SubjectMembership),("o".into(),C::ActionUsage)],..Fixture::default() };
    let p=planned(C::ReferenceUsage,&mut f);
    assert_eq!(named(&p,&Target::Root), ["reference0","subject"]);
    assert!(f.calls.iter().position(|(n,_)| *n=="owner").unwrap() < f.calls.iter().position(|(n,_)| *n=="membership").unwrap());
}
#[test]
fn action_owner_names_reference_as_parameter() {
    let mut f=Fixture { namespace:Some(ExternalObject("ns".into())),count:2,owner:Some(ExternalObject("o".into())),instance_of:vec![("o".into(),C::ActionDefinition)],..Fixture::default() };
    let p=planned(C::ReferenceUsage,&mut f);
    assert_eq!(named(&p,&Target::Root), ["reference2","parameter2"]);
}
#[test]
fn objective_preserves_java_null_name_concatenation() {
    let mut f=Fixture { membership:Some(ExternalObject("m".into())),owner:Some(ExternalObject("o".into())),
        instance_of:vec![("m".into(),C::ObjectiveMembership),("o".into(),C::UseCaseDefinition)],name:None,..Fixture::default() };
    let p=planned(C::RequirementUsage,&mut f);
    assert_eq!(named(&p,&Target::Root), ["requirement0","null's objective"]);
}
#[test]
fn view_known_missing_library_assigns_null_then_typed_feature() {
    let mut f=Fixture::default(); let p=planned(C::ViewUsage,&mut f);
    assert_eq!(f.calls.iter().find(|(n,_)| *n=="general_view").unwrap().1,4);
    let ty=p.steps.iter().position(|s| matches!(&s.effect,Effect::SetReference {feature:ReferenceFeature::Type,value:None,..})).unwrap();
    let feature=p.steps.iter().position(|s| matches!(&s.effect,Effect::SetReference {feature:ReferenceFeature::TypedFeature,value:Some(Target::Root),..})).unwrap();
    assert!(ty < feature);
}
#[test]
fn unresolved_library_is_not_fake_absence_or_graph_rollback() {
    let gap=ContextGap {contract:"GeneralView lookup",reason:"native library resolution unavailable".into()};
    let mut f=Fixture {library:Resolution::Unresolved(gap.clone()),..Fixture::default()};
    match plan(C::ViewUsage,&mut f) {
        Outcome::Incomplete(p) => {
            assert_eq!(p.problem,Problem::Context(gap)); assert_eq!(p.prefix.len(),4);
            assert!(!p.prefix.iter().any(|s| matches!(&s.effect,Effect::SetReference {..})));
            assert_eq!(p.application,ApplicationStatus::Unported);
        }
        other=>panic!("unresolved lookup produced full plan: {other:?}")
    }
}
#[test]
fn unsupported_context_stays_distinct() {
    let gap=ContextGap {contract:"GeneralView lookup",reason:"unsupported proxy state".into()};
    let mut f=Fixture {library:Resolution::Unsupported(gap.clone()),..Fixture::default()};
    assert!(matches!(plan(C::ViewUsage,&mut f),Outcome::Incomplete(IncompletePlan {problem:Problem::UnsupportedContext(g),..}) if g==gap));
}
#[test]
fn cancellation_never_publishes_an_applicable_plan() {
    let mut f=Fixture {cancel:true,..Fixture::default()};
    match plan(C::PortDefinition,&mut f) {
        Outcome::Incomplete(p)=>{ assert_eq!(p.problem,Problem::Cancelled);assert!(p.prefix.is_empty());assert!(f.calls.is_empty()); }
        other=>panic!("cancelled plan was published: {other:?}")
    }
}
#[test]
fn planned_helper_ids_are_distinct_and_not_semantic_uuid_values() {
    let a=planned(C::PortDefinition,&mut Fixture::default());
    let b=planned(C::PortDefinition,&mut Fixture::default());
    let x=allocations(&a);let y=allocations(&b);
    assert_ne!(x[0].0.owner(),y[0].0.owner());
    assert_eq!(x.iter().map(|(id,_)| id.ordinal()).collect::<Vec<_>>(),[0,1,2]);
    assert!((0..x.len()).all(|i| (i+1..x.len()).all(|j| x[i].0 != x[j].0)));
}
#[test]
fn all_concrete_source_dispatches_have_planner_paths_under_known_absence() {
    for row in DISPATCH {
        let outcome=plan(row.classifier,&mut Fixture::default());
        if row.raw_constructible {
            assert!(matches!(&outcome,Outcome::Planned(_)),"{:?}: {outcome:?}",row.classifier);
        } else {
            assert!(matches!(&outcome,Outcome::Incomplete(IncompletePlan {problem:Problem::AbstractClass,..})),"{:?}: {outcome:?}",row.classifier);
        }
    }
}

#[test]
fn known_general_view_type_is_retained_as_external_reference() {
    let mut f=Fixture {library:Resolution::Known(Some(ExternalObject("general-view".into()))),..Fixture::default()};
    let p=planned(C::ViewUsage,&mut f);
    assert!(p.steps.iter().any(|s| matches!(&s.effect,Effect::SetReference {feature:ReferenceFeature::Type,value:Some(Target::External(o)),..} if o.0=="general-view")));
}
#[test]
fn stakeholder_is_counted_after_actor_predicate_fails() {
    let mut f=Fixture {namespace:Some(ExternalObject("ns".into())),count:3,membership:Some(ExternalObject("m".into())),instance_of:vec![("m".into(),C::StakeholderMembership)],..Fixture::default()};
    assert_eq!(named(&planned(C::PartUsage,&mut f),&Target::Root),["part3","stakeholder3"]);
}
#[test]
fn textual_bodies_and_expose_setters_preserve_source_constants() {
    for (c,expected) in [(C::Comment,"add comment here"),(C::Documentation,"add doc here"),(C::TextualRepresentation,"add textual representation here")] {
        let p=planned(c,&mut Fixture::default());
        assert!(matches!(&p.steps[0].effect,Effect::SetScalar {feature:ScalarFeature::Body,value:Scalar::Text(v),..} if v==expected));
    }
    // Expose itself is abstract; both concrete descendants reach caseExpose.
    assert!(matches!(plan(C::Expose,&mut Fixture::default()),Outcome::Incomplete(IncompletePlan {problem:Problem::AbstractClass,..})));
    for class in [C::MembershipExpose,C::NamespaceExpose] {
        let p=planned(class,&mut Fixture::default());
        assert!(matches!(&p.steps[0].effect,Effect::SetScalar {feature:ScalarFeature::Visibility,value:Scalar::ProtectedVisibility,..}));
        assert!(matches!(&p.steps[1].effect,Effect::SetScalar {feature:ScalarFeature::IsImportAll,value:Scalar::Boolean(true),..}));
    }
}
#[test]
fn enumeration_sets_variation_before_counting_and_naming() {
    let mut f=Fixture {namespace:Some(ExternalObject("ns".into())),count:3,..Fixture::default()};
    let p=planned(C::EnumerationDefinition,&mut f);
    assert!(matches!(&p.steps[0].effect,Effect::SetScalar {feature:ScalarFeature::IsVariation,value:Scalar::Boolean(true),..}));
    assert_eq!(f.counts,[(C::EnumerationDefinition,1)]);
    assert_eq!(named(&p,&Target::Root),["EnumerationDefinition3"]);
}
#[test]
fn enum_identity_is_not_fabricated_as_domain_element_default_case() {
    assert!(matches!(plan(C::FeatureDirectionKind,&mut Fixture::default()),Outcome::Incomplete(IncompletePlan {problem:Problem::NotDomainClass,..})));
}

// Copyright (c) 2023, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Source implementation in progress; NOT_COMPILED / tests NOT_RUN.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT_PLAN: AtomicU64 = AtomicU64::new(1);
type R<T> = Result<T, Problem>;
fn resolved<T>(answer: Resolution<T>) -> R<T> {
    match answer {
        Resolution::Known(v) => Ok(v),
        Resolution::Unresolved(gap) => Err(Problem::Context(gap)),
        Resolution::Unsupported(gap) => Err(Problem::UnsupportedContext(gap)),
    }
}
pub fn plan<C: Context>(classifier: ClassifierId, context: &mut C) -> Outcome {
    let mut engine = Engine { classifier, context, steps: Vec::new(), trace: Vec::new(),
        owner: 0, next_object: 0, objects: Vec::new() };
    let result = (|| {
        let row = DISPATCH.iter().find(|row| row.classifier == classifier).ok_or(Problem::NotDomainClass)?;
        if !row.raw_constructible { return Err(Problem::AbstractClass); }
        engine.owner = NEXT_PLAN.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| Problem::IdentitySpaceExhausted)?;
        for &case in row.sequence {
            if engine.invoke(case, &Target::Root, CallKind::GeneratedDispatch)? { return Ok(()); }
        }
        engine.check()?;
        engine.trace.push(Invocation { case_name: "defaultCase", kind: CallKind::Default });
        // Input is a domain class, hence Element. Foreign EObject default branch
        // is deliberately not accepted by this planner.
        Ok(())
    })();
    match result {
        Ok(()) => Outcome::Planned(Plan { classifier, steps: engine.steps, dispatch_trace: engine.trace, application: ApplicationStatus::Unported }),
        Err(problem) => Outcome::Incomplete(IncompletePlan { classifier, prefix: engine.steps, dispatch_trace: engine.trace, problem, application: ApplicationStatus::Unported }),
    }
}
struct Engine<'a, C> {
    classifier: ClassifierId, context: &'a mut C, steps: Vec<Step>, trace: Vec<Invocation>,
    owner: u64, next_object: u32, objects: Vec<(PlannedObject, ClassifierId)>,
}
impl<C: Context> Engine<'_, C> {
    fn check(&mut self) -> R<()> { if self.context.cancelled() { Err(Problem::Cancelled) } else { Ok(()) } }
    fn emit(&mut self, source_method: &'static str, effect: Effect) -> R<()> {
        self.check()?; self.steps.push(Step { source_method, effect }); Ok(())
    }
    fn scalar(&mut self, target: &Target, feature: ScalarFeature, value: Scalar, method: &'static str) -> R<()> {
        self.emit(method, Effect::SetScalar { target: target.clone(), feature, value })
    }
    fn reference(&mut self, target: &Target, feature: ReferenceFeature, value: Option<Target>, method: &'static str) -> R<()> {
        self.emit(method, Effect::SetReference { target: target.clone(), feature, value })
    }
    fn allocate(&mut self, class: ClassifierId, method: &'static str) -> R<Target> {
        self.check()?;
        let ordinal = self.next_object;
        self.next_object = ordinal.checked_add(1).ok_or(Problem::LocalObjectSpaceExhausted)?;
        let object = PlannedObject { owner: self.owner, ordinal };
        self.emit(method, Effect::AllocateRaw { object, classifier: class, identity: IdentityRequirement::FreshSemanticUuidV4ResourceIdUnassigned })?;
        self.objects.push((object, class)); Ok(Target::Planned(object))
    }
    fn attach(&mut self, owner: &Target, feature: Containment, child: &Target, method: &'static str) -> R<()> {
        self.emit(method, Effect::Attach { owner: owner.clone(), feature, child: child.clone() })
    }
    fn class(&self, target: &Target) -> R<ClassifierId> {
        match target {
            Target::Root => Ok(self.classifier),
            Target::Planned(id) if id.owner == self.owner => self.objects.iter().find(|(key, _)| key == id).map(|(_, class)| *class).ok_or(Problem::InvalidPlannedTarget),
            _ => Err(Problem::InvalidPlannedTarget),
        }
    }
    fn unattached_raw(&self, target: &Target) -> R<bool> {
        self.class(target)?;
        if matches!(target, Target::Root) { return Ok(false); }
        if self.steps.iter().any(|step| matches!(&step.effect, Effect::Attach { child, .. } if child == target)) {
            Err(Problem::UnsupportedContext(ContextGap { contract: "planned-child owner/namespace projection", reason: "Attached planned-child derived graph queries are not implemented; do not invent a graph.".into() }))
        } else { Ok(true) }
    }
    fn count(&mut self, target: &Target, helper_call: bool) -> R<u64> {
        self.check()?;
        if helper_call { self.trace.push(Invocation { case_name: "existingElementsCount", kind: CallKind::Helper }); }
        if self.unattached_raw(target)? { return Ok(0); }
        let namespace = resolved(self.context.owning_namespace(&self.steps))?;
        match namespace {
            None => Ok(0),
            Some(namespace) => {
                self.check()?; let class = self.class(target)?; resolved(self.context.exact_class_owned_member_count(&namespace, class, &self.steps))
            }
        }
    }
    fn named_class(&mut self, target: &Target, method: &'static str) -> R<()> {
        let count = self.count(target, true)?;
        let name = format!("{}{}", self.class(target)?.metadata().name(), count);
        self.scalar(target, ScalarFeature::DeclaredName, Scalar::Text(name), method)
    }
    fn usage(&mut self, target: &Target) -> R<()> {
        let name = self.class(target)?.metadata().name();
        // All175 pinned domain class names are ASCII. No arbitrary Java string
        // case conversion or external classifier naming is claimed here.
        let mut chars = name.chars();
        let mut default_name = chars.next().ok_or(Problem::NotDomainClass)?.to_ascii_lowercase().to_string();
        default_name.extend(chars);
        if default_name.ends_with("Usage") { default_name.truncate(default_name.len() - 5); }
        let count = self.count(target, true)?;
        self.scalar(target, ScalarFeature::DeclaredName, Scalar::Text(format!("{default_name}{count}")), "caseUsage")?;
        self.scalar(target, ScalarFeature::IsComposite, Scalar::Boolean(true), "caseUsage")
    }

    fn membership(&mut self, target: &Target) -> R<Option<ExternalObject>> {
        self.check()?;
        if self.unattached_raw(target)? { Ok(None) }
        else { resolved(self.context.owning_membership(&self.steps)) }
    }
    fn owner_of(&mut self, target: &Target) -> R<Option<ExternalObject>> {
        self.check()?;
        if self.unattached_raw(target)? { Ok(None) }
        else { resolved(self.context.owner(&self.steps)) }
    }
    fn is(&mut self, object: &ExternalObject, class: ClassifierId) -> R<bool> {
        self.check()?; resolved(self.context.is_instance_of(object, class, &self.steps))
    }
    fn reference_usage(&mut self, target: &Target) -> R<()> {
        self.invoke("Usage", target, CallKind::DirectCase)?;
        // Source reads owner before testing owning membership.
        let owner = self.owner_of(target)?;
        let membership = self.membership(target)?;
        let subject = match membership {
            Some(ref member) => self.is(member, ClassifierId::SubjectMembership)?,
            None => false,
        };
        if subject {
            self.scalar(target, ScalarFeature::DeclaredName, Scalar::Text("subject".into()), "caseReferenceUsage")?;
        } else if let Some(owner) = owner {
            if self.is(&owner, ClassifierId::ActionUsage)? || self.is(&owner, ClassifierId::ActionDefinition)? {
                let count = self.count(target, true)?;
                self.scalar(target, ScalarFeature::DeclaredName, Scalar::Text(format!("parameter{count}")), "caseReferenceUsage")?;
            }
        }
        self.scalar(target, ScalarFeature::IsComposite, Scalar::Boolean(false), "caseReferenceUsage")
    }
    fn part_usage(&mut self, target: &Target) -> R<()> {
        self.invoke("Usage", target, CallKind::DirectCase)?;
        let membership = self.membership(target)?;
        if let Some(member) = membership {
            let actor = self.is(&member, ClassifierId::ActorMembership)?;
            let stakeholder = if actor { false } else { self.is(&member, ClassifierId::StakeholderMembership)? };
            if actor || stakeholder {
                // Source's lambda checks OBJECT membership, not MEMBER membership.
                // All exact-class siblings are counted under that stable snapshot;
                // it must not become a role-filtered sibling count or count+1.
                let count = self.count(target, false)?;
                let prefix = if actor { "actor" } else { "stakeholder" };
                self.scalar(target, ScalarFeature::DeclaredName, Scalar::Text(format!("{prefix}{count}")), "casePartUsage")?;
            }
        }
        Ok(())
    }
    fn requirement_usage(&mut self, target: &Target) -> R<()> {
        self.invoke("Usage", target, CallKind::DirectCase)?;
        if let Some(member) = self.membership(target)? {
            if self.is(&member, ClassifierId::ObjectiveMembership)? {
                if let Some(owner) = self.owner_of(target)? {
                    if self.is(&owner, ClassifierId::UseCaseUsage)? || self.is(&owner, ClassifierId::UseCaseDefinition)? {
                        self.check()?;
                        // Known(None) is Java null; Java string concatenation
                        // yields "null's objective", not an invented empty name.
                        let name = resolved(self.context.effective_name(&owner, &self.steps))?.unwrap_or_else(|| "null".into());
                        self.scalar(target, ScalarFeature::DeclaredName, Scalar::Text(format!("{name}'s objective")), "caseRequirementUsage")?;
                    }
                }
            }
        }
        Ok(())
    }
    fn connection_end(&mut self, target: &Target, name: &str) -> R<()> {
        self.trace.push(Invocation { case_name: "addConnectionEnd", kind: CallKind::Helper });
        let reference = self.allocate(ClassifierId::ReferenceUsage, "addConnectionEnd")?;
        self.invoke("ReferenceUsage", &reference, CallKind::DirectCase)?;
        self.scalar(&reference, ScalarFeature::DeclaredName, Scalar::Text(name.into()), "addConnectionEnd")?;
        self.scalar(&reference, ScalarFeature::IsEnd, Scalar::Boolean(true), "addConnectionEnd")?;
        let membership = self.allocate(ClassifierId::FeatureMembership, "addConnectionEnd")?;
        self.attach(&membership, Containment::OwnedRelatedElement, &reference, "addConnectionEnd")?;
        self.attach(target, Containment::OwnedRelationship, &membership, "addConnectionEnd")
    }
    fn port_end(&mut self, target: &Target, name: &str) -> R<()> {
        self.trace.push(Invocation { case_name: "addPortEnd", kind: CallKind::Helper });
        let port = self.allocate(ClassifierId::PortUsage, "addPortEnd")?;
        // DIRECT generated-null call, not full PortUsage->Usage dispatch.
        self.invoke("PortUsage", &port, CallKind::DirectCase)?;
        self.scalar(&port, ScalarFeature::DeclaredName, Scalar::Text(name.into()), "addPortEnd")?;
        self.scalar(&port, ScalarFeature::IsEnd, Scalar::Boolean(true), "addPortEnd")?;
        let membership = self.allocate(ClassifierId::FeatureMembership, "addPortEnd")?;
        self.attach(&membership, Containment::OwnedRelatedElement, &port, "addPortEnd")?;
        self.attach(target, Containment::OwnedRelationship, &membership, "addPortEnd")
    }
    fn parameter(&mut self, name: &str, direction: Direction) -> R<Target> {
        self.trace.push(Invocation { case_name: "createParameterMembershipWithReferenceUsage", kind: CallKind::Helper });
        let method = "createParameterMembershipWithReferenceUsage";
        let reference = self.allocate(ClassifierId::ReferenceUsage, method)?;
        // These source helper children stay RAW, unlike addConnectionEnd.
        self.scalar(&reference, ScalarFeature::Direction, Scalar::Direction(direction), method)?;
        self.scalar(&reference, ScalarFeature::DeclaredName, Scalar::Text(name.into()), method)?;
        let membership = self.allocate(ClassifierId::ParameterMembership, method)?;
        self.attach(&membership, Containment::OwnedRelatedElement, &reference, method)?;
        Ok(membership)
    }
    fn port_definition(&mut self, target: &Target) -> R<()> {
        let method = "casePortDefinition";
        self.named_class(target, method)?;
        let membership = self.allocate(ClassifierId::OwningMembership, method)?;
        self.attach(target, Containment::OwnedRelationship, &membership, method)?;
        let conjugated = self.allocate(ClassifierId::ConjugatedPortDefinition, method)?;
        self.attach(&membership, Containment::OwnedRelatedElement, &conjugated, method)?;
        // No recursive initializer on the conjugated child.
        let conjugation = self.allocate(ClassifierId::PortConjugation, method)?;
        self.attach(&conjugated, Containment::OwnedRelationship, &conjugation, method)?;
        self.reference(&conjugation, ReferenceFeature::ConjugatedType, Some(conjugated), method)?;
        self.reference(&conjugation, ReferenceFeature::OriginalPortDefinition, Some(target.clone()), method)
    }
    fn view_usage(&mut self, target: &Target) -> R<()> {
        self.invoke("Usage", target, CallKind::DirectCase)?;
        let method = "caseViewUsage";
        let typing = self.allocate(ClassifierId::FeatureTyping, method)?;
        self.attach(target, Containment::OwnedRelationship, &typing, method)?;
        self.check()?;
        // This contract is exactly findByNameAndType(root,
        // "StandardViewDefinitions::GeneralView", ViewDefinition).
        // Known absence assigns null; unresolved MUST NOT masquerade as absence.
        let general_view = resolved(self.context.general_view_definition(&self.steps))?;
        self.reference(&typing, ReferenceFeature::Type, general_view.map(Target::External), method)?;
        self.reference(&typing, ReferenceFeature::TypedFeature, Some(target.clone()), method)
    }

    fn invoke(&mut self, name: &'static str, target: &Target, kind: CallKind) -> R<bool> {
        self.check()?; self.trace.push(Invocation { case_name: name, kind });
        if kind == CallKind::SuperCase {
            return if GENERATED_NULL_CASES.contains(&name) { Ok(false) } else { Err(Problem::UnportedCase(name)) };
        }
        match name {
            // Non-null no-op overrides STOP inherited dispatch.
            "BindingConnectorAsUsage" | "Dependency" | "Element" | "FeatureTyping" |
            "FlowUsage" | "PerformActionUsage" | "Redefinition" | "ReferenceSubsetting" |
            "Specialization" | "Subclassification" | "Subsetting" | "SuccessionAsUsage" |
            "TransitionUsage" => Ok(true),
            "Comment" => { self.scalar(target, ScalarFeature::Body, Scalar::Text("add comment here".into()), "caseComment")?; Ok(true) }
            "Documentation" => { self.scalar(target, ScalarFeature::Body, Scalar::Text("add doc here".into()), "caseDocumentation")?; Ok(true) }
            "TextualRepresentation" => { self.scalar(target, ScalarFeature::Body, Scalar::Text("add textual representation here".into()), "caseTextualRepresentation")?; Ok(true) }
            "Definition" => { self.named_class(target, "caseDefinition")?; Ok(true) }
            "Package" => { self.named_class(target, "casePackage")?; Ok(true) }
            "EnumerationDefinition" => {
                self.scalar(target, ScalarFeature::IsVariation, Scalar::Boolean(true), "caseEnumerationDefinition")?;
                self.named_class(target, "caseEnumerationDefinition")?; Ok(true)
            }
            "Expose" => {
                self.scalar(target, ScalarFeature::Visibility, Scalar::ProtectedVisibility, "caseExpose")?;
                self.scalar(target, ScalarFeature::IsImportAll, Scalar::Boolean(true), "caseExpose")?; Ok(true)
            }
            "Usage" => { self.usage(target)?; Ok(true) }
            "AttributeUsage" => {
                self.invoke("Usage", target, CallKind::DirectCase)?;
                self.scalar(target, ScalarFeature::IsComposite, Scalar::Boolean(false), "caseAttributeUsage")?; Ok(true)
            }
            "FlowEnd" => {
                self.invoke("Feature", target, CallKind::DirectCase)?;
                self.scalar(target, ScalarFeature::IsEnd, Scalar::Boolean(true), "caseFlowEnd")?; Ok(true)
            }

            "AcceptActionUsage" => {
                self.invoke("Usage", target, CallKind::DirectCase)?;
                let payload = self.parameter("payload", Direction::InOut)?;
                self.attach(target, Containment::OwnedRelationship, &payload, "caseAcceptActionUsage")?;
                let receiver = self.parameter("receiver", Direction::In)?;
                self.attach(target, Containment::OwnedRelationship, &receiver, "caseAcceptActionUsage")?;
                Ok(true)
            }
            "ConnectionDefinition" => {
                self.scalar(target, ScalarFeature::IsSufficient, Scalar::Boolean(true), "caseConnectionDefinition")?;
                self.connection_end(target, "source")?;
                self.connection_end(target, "target")?;
                Ok(false) // Source null return continues GENERATED dispatch.
            }
            "InterfaceDefinition" => {
                self.scalar(target, ScalarFeature::IsSufficient, Scalar::Boolean(true), "caseInterfaceDefinition")?;
                self.port_end(target, "source")?;
                self.port_end(target, "target")?;
                self.invoke("InterfaceDefinition", target, CallKind::SuperCase)
            }
            "PartUsage" => { self.part_usage(target)?; Ok(true) }
            "PortDefinition" => { self.port_definition(target)?; Ok(true) }
            "ReferenceUsage" => { self.reference_usage(target)?; Ok(true) }
            "RequirementUsage" => { self.requirement_usage(target)?; Ok(true) }
            "ViewUsage" => { self.view_usage(target)?; Ok(true) }
            _ if GENERATED_NULL_CASES.contains(&name) => Ok(false),
            _ => Err(Problem::UnportedCase(name)),
        }
    }
}

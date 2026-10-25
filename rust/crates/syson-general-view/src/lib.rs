// Copyright (c) 2023, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
//! Source-point GeneralView planning over caller-supplied immutable observations.
//! No graph application, resource loading, or native visibility attestation.
#![forbid(unsafe_code)]
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};
use syson_initializer::{
    self as init, Context, ContextGap, Effect, ExternalObject, Problem, Resolution, Step, Target,
};
use syson_initializer_graph::ReferenceEncoding;
use syson_initializer_graph::native_query::{ObjectId, ResourceContext, ResourceSet};
use syson_metamodel::ClassifierId;

static NEXT_SNAPSHOT: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapshotId(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingError {
    IdentityExhausted,
    StaleSnapshot,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedTarget {
    snapshot: SnapshotId,
    object: ObjectId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Absence {
    Detached,
    MissingResourceSet,
    NoMatch,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lookup {
    NotReached,
    Absent(Absence),
    Found(ScopedTarget),
    Unresolved,
    StaleObservation,
}
/// The caller asserts that these resources represent this exact source prefix.
/// Prefix/context association is checked; native correspondence is not inferred.
pub struct PrefixSnapshot {
    id: SnapshotId,
    resources: ResourceSet,
    context: ResourceContext,
    prefix: Vec<Step>,
}
impl PrefixSnapshot {
    pub fn new(
        resources: ResourceSet,
        context: ResourceContext,
        prefix: Vec<Step>,
    ) -> Result<Self, BindingError> {
        let id = NEXT_SNAPSHOT
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| BindingError::IdentityExhausted)?;
        Ok(Self {
            id: SnapshotId(id),
            resources,
            context,
            prefix,
        })
    }
    pub fn identity(&self) -> SnapshotId {
        self.id
    }
    pub fn prefix(&self) -> &[Step] {
        &self.prefix
    }
    pub fn context(&self) -> &ResourceContext {
        &self.context
    }
    pub fn resolve<'a>(&self, target: &'a ScopedTarget) -> Result<&'a ObjectId, BindingError> {
        if target.snapshot != self.id {
            return Err(BindingError::StaleSnapshot);
        }
        Ok(&target.object)
    }
    pub fn validate(&self, plan: &Plan) -> Result<(), BindingError> {
        if plan.snapshot.as_ref().map(|s| s.id) != Some(self.id) {
            return Err(BindingError::StaleSnapshot);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoundStep {
    Local(Step),
    GeneralViewType {
        source_method: &'static str,
        typing: Target,
        definition: Option<ScopedTarget>,
    },
}
/// No legacy Plan or unqualified GeneralView ExternalObject escapes this API.
pub struct Plan {
    steps: Vec<BoundStep>,
    lookup: Lookup,
    snapshot: Option<PrefixSnapshot>,
    trace: Vec<init::Invocation>,
}
impl Plan {
    pub fn steps(&self) -> &[BoundStep] {
        &self.steps
    }
    pub fn lookup(&self) -> &Lookup {
        &self.lookup
    }
    pub fn snapshot(&self) -> Option<&PrefixSnapshot> {
        self.snapshot.as_ref()
    }
    pub fn trace(&self) -> &[init::Invocation] {
        &self.trace
    }
}
pub enum Outcome {
    Planned(Plan),
    Incomplete { prefix: Plan, problem: Problem },
}
fn gap(reason: impl Into<String>) -> ContextGap {
    ContextGap {
        contract: "resource-scoped GeneralView planning",
        reason: reason.into(),
    }
}
fn same_context(a: &ResourceContext, b: &ResourceContext) -> bool {
    match (a, b) {
        (ResourceContext::Detached, ResourceContext::Detached)
        | (ResourceContext::ResourceWithoutSet, ResourceContext::ResourceWithoutSet) => true,
        (ResourceContext::Member(a), ResourceContext::Member(b)) => a == b,
        _ => false,
    }
}
struct Binding<'a, C, F> {
    delegate: &'a mut C,
    provider: F,
    context: ResourceContext,
    encoding: ReferenceEncoding,
    snapshot: Option<PrefixSnapshot>,
    lookup: Lookup,
    token: Option<ExternalObject>,
    seen: BTreeSet<String>,
}
impl<C, F> Binding<'_, C, F> {
    fn object(
        &mut self,
        value: Resolution<Option<ExternalObject>>,
    ) -> Resolution<Option<ExternalObject>> {
        if let Resolution::Known(Some(object)) = &value {
            if self.token.as_ref() == Some(object) {
                return Resolution::Unsupported(gap(
                    "delegated handle collides with the scoped query handle",
                ));
            }
            self.seen.insert(object.0.clone());
        }
        value
    }
}
impl<C: Context, F: FnMut(&[Step]) -> Result<PrefixSnapshot, ContextGap>> Context
    for Binding<'_, C, F>
{
    fn cancelled(&mut self) -> bool {
        self.delegate.cancelled()
    }
    fn owning_namespace(&mut self, p: &[Step]) -> Resolution<Option<ExternalObject>> {
        let v = self.delegate.owning_namespace(p);
        self.object(v)
    }
    fn exact_class_owned_member_count(
        &mut self,
        n: &ExternalObject,
        c: ClassifierId,
        p: &[Step],
    ) -> Resolution<u64> {
        self.delegate.exact_class_owned_member_count(n, c, p)
    }
    fn owning_membership(&mut self, p: &[Step]) -> Resolution<Option<ExternalObject>> {
        let v = self.delegate.owning_membership(p);
        self.object(v)
    }
    fn owner(&mut self, p: &[Step]) -> Resolution<Option<ExternalObject>> {
        let v = self.delegate.owner(p);
        self.object(v)
    }
    fn is_instance_of(
        &mut self,
        o: &ExternalObject,
        c: ClassifierId,
        p: &[Step],
    ) -> Resolution<bool> {
        self.delegate.is_instance_of(o, c, p)
    }
    fn effective_name(&mut self, o: &ExternalObject, p: &[Step]) -> Resolution<Option<String>> {
        self.delegate.effective_name(o, p)
    }
    fn general_view_definition(&mut self, p: &[Step]) -> Resolution<Option<ExternalObject>> {
        let snapshot = match (self.provider)(p) {
            Ok(s) => s,
            Err(e) => {
                self.lookup = Lookup::Unresolved;
                return Resolution::Unresolved(e);
            }
        };
        if snapshot.prefix != p || !same_context(&snapshot.context, &self.context) {
            self.lookup = Lookup::StaleObservation;
            return Resolution::Unresolved(gap(
                "prefix or resource context differs from the actual query point",
            ));
        }
        let found = snapshot
            .resources
            .find_general_view(&self.context, self.encoding);
        let result = match found {
            Ok(Some(object)) => {
                let target = ScopedTarget {
                    snapshot: snapshot.id,
                    object,
                };
                self.lookup = Lookup::Found(target);
                // This symbol is temporary and never represents an element ID.
                let token = (0..=self.seen.len())
                    .map(|i| format!("\0syson.general-view:{:?}:{i}", snapshot.id))
                    .find(|s| !self.seen.contains(s))
                    .expect("N existing handles cannot exhaust N+1 symbols");
                let token = ExternalObject(token);
                self.token = Some(token.clone());
                Resolution::Known(Some(token))
            }
            Ok(None) => {
                self.lookup = Lookup::Absent(match self.context {
                    ResourceContext::Detached => Absence::Detached,
                    ResourceContext::ResourceWithoutSet => Absence::MissingResourceSet,
                    ResourceContext::Member(_) => Absence::NoMatch,
                });
                Resolution::Known(None)
            }
            Err(e) => {
                self.lookup = Lookup::Unresolved;
                Resolution::Unresolved(gap(format!("{e:?}")))
            }
        };
        self.snapshot = Some(snapshot);
        result
    }
}
fn external(target: &Target) -> bool {
    matches!(target, Target::External(_))
}
fn bind_step(
    step: Step,
    lookup: &Lookup,
    token: &Option<ExternalObject>,
) -> Result<BoundStep, ContextGap> {
    if let Effect::SetReference {
        target,
        feature: init::ReferenceFeature::Type,
        value,
    } = &step.effect
        && step.source_method == "caseViewUsage"
    {
        let definition = match (value, lookup) {
            (Some(Target::External(t)), Lookup::Found(object)) if Some(t) == token.as_ref() => {
                Some(object.clone())
            }
            (None, Lookup::Absent(_)) => None,
            _ => return Err(gap("query binding and source Type assignment disagree")),
        };
        if external(target) {
            return Err(gap("external typing owner is outside this binding"));
        }
        return Ok(BoundStep::GeneralViewType {
            source_method: step.source_method,
            typing: target.clone(),
            definition,
        });
    }
    let leaks = match &step.effect {
        Effect::AllocateRaw { .. } => false,
        Effect::Attach { owner, child, .. } => external(owner) || external(child),
        Effect::SetScalar { target, .. } => external(target),
        Effect::SetReference { target, value, .. } => {
            external(target) || value.as_ref().is_some_and(external)
        }
    };
    if leaks {
        return Err(gap("unbound external target in local planning step"));
    }
    Ok(BoundStep::Local(step))
}
/// Runs the existing ViewUsage planner; only its GeneralView query is replaced.
/// The provider is called at the actual prefix, never before naming/attachment.
pub fn plan<C: Context, F: FnMut(&[Step]) -> Result<PrefixSnapshot, ContextGap>>(
    context: ResourceContext,
    encoding: ReferenceEncoding,
    delegate: &mut C,
    provider: F,
) -> Outcome {
    let mut binding = Binding {
        delegate,
        provider,
        context,
        encoding,
        snapshot: None,
        lookup: Lookup::NotReached,
        token: None,
        seen: BTreeSet::new(),
    };
    let outcome = init::plan(ClassifierId::ViewUsage, &mut binding);
    let (steps, trace, mut problem) = match outcome {
        init::Outcome::Planned(p) => (p.steps, p.dispatch_trace, None),
        init::Outcome::Incomplete(p) => (p.prefix, p.dispatch_trace, Some(p.problem)),
    };
    let mut bound = Vec::new();
    for step in steps {
        match bind_step(step, &binding.lookup, &binding.token) {
            Ok(s) => bound.push(s),
            Err(e) => {
                problem = Some(Problem::UnsupportedContext(e));
                break;
            }
        }
    }
    let plan = Plan {
        steps: bound,
        lookup: binding.lookup,
        snapshot: binding.snapshot,
        trace,
    };
    match problem {
        Some(problem) => Outcome::Incomplete {
            prefix: plan,
            problem,
        },
        None => Outcome::Planned(plan),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use syson_initializer_graph::native_query::ResourceSnapshot;
    use syson_model::Document;
    struct Collision(ExternalObject);
    impl Context for Collision {
        fn owning_namespace(&mut self, _: &[Step]) -> Resolution<Option<ExternalObject>> {
            Resolution::Known(Some(self.0.clone()))
        }
        fn exact_class_owned_member_count(
            &mut self,
            _: &ExternalObject,
            _: ClassifierId,
            _: &[Step],
        ) -> Resolution<u64> {
            Resolution::Known(0)
        }
        fn owning_membership(&mut self, _: &[Step]) -> Resolution<Option<ExternalObject>> {
            unreachable!()
        }
        fn owner(&mut self, _: &[Step]) -> Resolution<Option<ExternalObject>> {
            unreachable!()
        }
        fn is_instance_of(
            &mut self,
            _: &ExternalObject,
            _: ClassifierId,
            _: &[Step],
        ) -> Resolution<bool> {
            unreachable!()
        }
        fn effective_name(&mut self, _: &ExternalObject, _: &[Step]) -> Resolution<Option<String>> {
            unreachable!()
        }
        fn general_view_definition(&mut self, _: &[Step]) -> Resolution<Option<ExternalObject>> {
            unreachable!()
        }
    }
    #[test]
    fn a_delegated_handle_equal_to_the_first_symbol_candidate_is_not_reused() {
        let doc = Document::from_json(&json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"id":"wire-lib","eClass":"sysml:Package","data":{"elementId":"lib","declaredName":"StandardViewDefinitions","ownedRelationship":[{"id":"wire-m","eClass":"sysml:OwningMembership","data":{"elementId":"m","ownedRelatedElement":[{"id":"wire-view","eClass":"sysml:ViewDefinition","data":{"elementId":"view","declaredName":"GeneralView"}}]}}]}}]}).to_string()).unwrap();
        let context = ResourceContext::Member(ObjectId::new("lib", "lib"));
        let snapshot = PrefixSnapshot::new(
            ResourceSet::new(vec![ResourceSnapshot::new("lib", doc)]).unwrap(),
            context.clone(),
            vec![],
        )
        .unwrap();
        let colliding = ExternalObject(format!("\0syson.general-view:{:?}:0", snapshot.id));
        let mut local = Collision(colliding.clone());
        let mut supplied = Some(snapshot);
        let mut binding = Binding {
            delegate: &mut local,
            provider: |prefix: &[Step]| {
                // Internal access keeps the exact ID for this adversarial case;
                // external callers construct an observation at this source point.
                let mut s = supplied.take().unwrap();
                s.prefix = prefix.to_vec();
                Ok(s)
            },
            context,
            encoding: ReferenceEncoding::SemanticElementIds,
            snapshot: None,
            lookup: Lookup::NotReached,
            token: None,
            seen: BTreeSet::new(),
        };
        let outcome = init::plan(ClassifierId::ViewUsage, &mut binding);
        let init::Outcome::Planned(source) = outcome else {
            panic!("binding must complete")
        };
        assert_ne!(binding.token.as_ref(), Some(&colliding));
        let bound: Vec<_> = source
            .steps
            .into_iter()
            .map(|s| bind_step(s, &binding.lookup, &binding.token).unwrap())
            .collect();
        let target = bound
            .iter()
            .find_map(|s| match s {
                BoundStep::GeneralViewType {
                    definition: Some(t),
                    ..
                } => Some(t),
                _ => None,
            })
            .unwrap();
        assert_eq!(
            binding.snapshot.as_ref().unwrap().resolve(target).unwrap(),
            &ObjectId::new("lib", "view")
        );
        assert_eq!(local.0, colliding);
    }
}

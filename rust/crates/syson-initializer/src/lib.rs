// Copyright (c) 2023, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
//! Ordered source initializer planning; no graph/Transaction application.
//! New Rust source and tests have NOT been compiled or executed.
use syson_metamodel::ClassifierId;
mod generated;
pub use generated::{DISPATCH, SOURCE_METHODS, GENERATED_NULL_CASES};
pub const BASELINE: &str = "6ba2ac64e6b16ee2b05752ad55395eebb87d4910";
#[derive(Clone, Copy, Debug)]
pub struct DispatchRow { pub classifier: ClassifierId, pub raw_constructible: bool, pub sequence: &'static [&'static str] }
#[derive(Clone, Copy, Debug)]
pub struct SourceMethod { pub name: &'static str, pub line: usize, pub body_sha256: &'static str, pub return_mode: &'static str }
pub fn dispatch_sequence(classifier: ClassifierId) -> Option<&'static [&'static str]> {
    DISPATCH.iter().find(|r| r.classifier == classifier).map(|r| r.sequence)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextGap { pub contract: &'static str, pub reason: String }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolution<T> { Known(T), Unresolved(ContextGap), Unsupported(ContextGap) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalObject(pub String);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlannedObject { owner: u64, ordinal: u32 }
impl PlannedObject { pub fn owner(self) -> u64 { self.owner } pub fn ordinal(self) -> u32 { self.ordinal } }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target { Root, Planned(PlannedObject), External(ExternalObject) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction { In, InOut }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scalar { Text(String), Boolean(bool), Direction(Direction), ProtectedVisibility }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Containment { OwnedRelationship, OwnedRelatedElement }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarFeature { DeclaredName, IsComposite, Body, IsSufficient, IsEnd, IsVariation, Visibility, IsImportAll, Direction }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceFeature { ConjugatedType, OriginalPortDefinition, Type, TypedFeature }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityRequirement { FreshSemanticUuidV4ResourceIdUnassigned }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    AllocateRaw { object: PlannedObject, classifier: ClassifierId, identity: IdentityRequirement },
    Attach { owner: Target, feature: Containment, child: Target },
    SetScalar { target: Target, feature: ScalarFeature, value: Scalar },
    SetReference { target: Target, feature: ReferenceFeature, value: Option<Target> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step { pub source_method: &'static str, pub effect: Effect }
/// Queries observe the source point after root attachment plus this ordered
/// prefix. Implementers must return unresolved instead of inventing a graph.
pub trait Context {
    fn cancelled(&mut self) -> bool { false }
    fn owning_namespace(&mut self, prefix: &[Step]) -> Resolution<Option<ExternalObject>>;
    fn exact_class_owned_member_count(&mut self, namespace: &ExternalObject, class: ClassifierId, prefix: &[Step]) -> Resolution<u64>;
    fn owning_membership(&mut self, prefix: &[Step]) -> Resolution<Option<ExternalObject>>;
    fn owner(&mut self, prefix: &[Step]) -> Resolution<Option<ExternalObject>>;
    fn is_instance_of(&mut self, object: &ExternalObject, class: ClassifierId, prefix: &[Step]) -> Resolution<bool>;
    fn effective_name(&mut self, object: &ExternalObject, prefix: &[Step]) -> Resolution<Option<String>>;
    fn general_view_definition(&mut self, prefix: &[Step]) -> Resolution<Option<ExternalObject>>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationStatus { Unported }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallKind { GeneratedDispatch, DirectCase, SuperCase, Helper, Default }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Invocation { pub case_name: &'static str, pub kind: CallKind }
#[derive(Clone, Debug)]
pub struct Plan { pub classifier: ClassifierId, pub steps: Vec<Step>, pub dispatch_trace: Vec<Invocation>, pub application: ApplicationStatus }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem { Cancelled, Context(ContextGap), UnsupportedContext(ContextGap), UnportedCase(&'static str), NotDomainClass, AbstractClass, IdentitySpaceExhausted, LocalObjectSpaceExhausted, InvalidPlannedTarget }
#[derive(Clone, Debug)]
pub struct IncompletePlan { pub classifier: ClassifierId, pub prefix: Vec<Step>, pub dispatch_trace: Vec<Invocation>, pub problem: Problem, pub application: ApplicationStatus }
#[derive(Clone, Debug)]
pub enum Outcome { Planned(Plan), Incomplete(IncompletePlan) }

mod planner;
pub use planner::plan;

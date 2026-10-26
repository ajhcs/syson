// SPDX-License-Identifier: EPL-2.0
//! Selected ViewUsage construction with materialized query-point observations.
//! No factory/cache defaults, native import evaluation or complete EMF serializer.
#![forbid(unsafe_code)]
use serde_json::{Value, json};
use std::cell::Cell;
use syson_general_view::{self as bound, BoundStep, PrefixSnapshot};
use syson_initializer::{
    Containment, Context, ContextGap, Effect, ExternalObject, PlannedObject, Problem,
    ReferenceFeature, Resolution, Scalar, ScalarFeature, Step, Target,
};
use syson_initializer_graph::{
    NativeContext, ReferenceEncoding,
    native_query::{ObjectId, ResourceContext, ResourceSet, ResourceSnapshot},
};
use syson_metamodel::ClassifierId as C;
use syson_model::{ContainmentFeature, Document, Editor, NodeIds, Transaction};
use syson_references::{FragmentIdentity, Registry, Resource};

#[derive(Debug)]
pub enum Error {
    Cancelled,
    Model(String),
    Planning(Problem),
    UnsupportedPlan,
    ResourceIdentity,
    Reference(syson_references::Error),
    TargetScope,
}
fn model(e: impl std::fmt::Debug) -> Error {
    Error::Model(format!("{e:?}"))
}
fn gap(e: impl std::fmt::Debug) -> ContextGap {
    ContextGap {
        contract: "selected ViewUsage query-point projection",
        reason: format!("{e:?}"),
    }
}
#[derive(Clone)]
pub struct Ids {
    pub membership: NodeIds,
    pub view: NodeIds,
    pub typing: NodeIds,
}
#[derive(Clone, Copy)]
pub enum Presence {
    ResourceSet,
    WithoutResourceSet,
}
struct Control<F> {
    poll: F,
    signalled: Cell<bool>,
}
impl<F: Fn() -> bool> Control<F> {
    fn cancelled(&self) -> bool {
        let value = self.signalled.get() || (self.poll)();
        self.signalled.set(value);
        value
    }
}
struct Local<'a, F> {
    context: NativeContext,
    control: &'a Control<F>,
}
impl<F: Fn() -> bool> Context for Local<'_, F> {
    fn cancelled(&mut self) -> bool {
        self.control.cancelled()
    }
    fn owning_namespace(&mut self, p: &[Step]) -> Resolution<Option<ExternalObject>> {
        self.context.owning_namespace(p)
    }
    fn exact_class_owned_member_count(
        &mut self,
        n: &ExternalObject,
        c: C,
        p: &[Step],
    ) -> Resolution<u64> {
        self.context.exact_class_owned_member_count(n, c, p)
    }
    fn owning_membership(&mut self, p: &[Step]) -> Resolution<Option<ExternalObject>> {
        self.context.owning_membership(p)
    }
    fn owner(&mut self, p: &[Step]) -> Resolution<Option<ExternalObject>> {
        self.context.owner(p)
    }
    fn is_instance_of(&mut self, o: &ExternalObject, c: C, p: &[Step]) -> Resolution<bool> {
        self.context.is_instance_of(o, c, p)
    }
    fn effective_name(&mut self, o: &ExternalObject, p: &[Step]) -> Resolution<Option<String>> {
        self.context.effective_name(o, p)
    }
    fn general_view_definition(&mut self, p: &[Step]) -> Resolution<Option<ExternalObject>> {
        self.context.general_view_definition(p)
    }
}
struct Projection {
    view: Value,
    typing: Value,
    allocated: Option<PlannedObject>,
    attached: bool,
}
fn node(ids: &NodeIds, class: &str) -> Value {
    json!({"id":ids.resource,"eClass":format!("sysml:{class}"),"data":{"elementId":ids.element}})
}
fn put(node: &mut Value, key: &str, value: Value) -> Result<(), Error> {
    if node["data"]
        .as_object_mut()
        .unwrap()
        .insert(key.into(), value)
        .is_some()
    {
        return Err(Error::UnsupportedPlan);
    }
    Ok(())
}
impl Projection {
    fn new(ids: &Ids) -> Self {
        Self {
            view: node(&ids.view, "ViewUsage"),
            typing: node(&ids.typing, "FeatureTyping"),
            allocated: None,
            attached: false,
        }
    }
    fn local(&mut self, step: &Step) -> Result<(), Error> {
        match (&step.effect, step.source_method) {
            (
                Effect::SetScalar {
                    target: Target::Root,
                    feature: ScalarFeature::DeclaredName,
                    value: Scalar::Text(name),
                },
                "caseUsage",
            ) => put(&mut self.view, "declaredName", json!(name)),
            (
                Effect::SetScalar {
                    target: Target::Root,
                    feature: ScalarFeature::IsComposite,
                    value: Scalar::Boolean(true),
                },
                "caseUsage",
            ) => put(&mut self.view, "isComposite", json!(true)),
            (
                Effect::AllocateRaw {
                    object,
                    classifier: C::FeatureTyping,
                    ..
                },
                "caseViewUsage",
            ) if self.allocated.is_none() => {
                self.allocated = Some(*object);
                Ok(())
            }
            (
                Effect::Attach {
                    owner: Target::Root,
                    feature: Containment::OwnedRelationship,
                    child: Target::Planned(object),
                },
                "caseViewUsage",
            ) if self.allocated == Some(*object) && !self.attached => {
                self.attached = true;
                Ok(())
            }
            (
                Effect::SetReference {
                    target: Target::Planned(object),
                    feature: ReferenceFeature::TypedFeature,
                    value: Some(Target::Root),
                },
                "caseViewUsage",
            ) if self.allocated == Some(*object) && self.attached => {
                let value = self.view["id"].clone();
                put(&mut self.typing, "typedFeature", value)
            }
            _ => Err(Error::UnsupportedPlan),
        }
    }
    fn selected(&self) -> Value {
        let mut view = self.view.clone();
        if self.attached {
            view["data"]["ownedRelationship"] = json!([self.typing]);
        }
        view
    }
    fn membership(&self, ids: &Ids, parent_is_type: bool) -> Value {
        let mut member = node(
            &ids.membership,
            if parent_is_type {
                "FeatureMembership"
            } else {
                "OwningMembership"
            },
        );
        member["data"]["ownedRelatedElement"] = json!([self.selected()]);
        member
    }
    fn complete(&self) -> bool {
        self.attached
            && self.view["data"]["declaredName"].is_string()
            && self.view["data"]["isComposite"] == true
            && self.typing["data"].get("type").is_some()
            && self.typing["data"]["typedFeature"] == self.view["id"]
    }
}
fn materialize(
    base: &Document,
    parent: &str,
    ids: &Ids,
    is_type: bool,
    projection: &Projection,
) -> Result<Document, Error> {
    let mut editor = Editor::new(base.clone());
    let mut tx = editor.begin();
    tx.attach_preallocated_subtree(
        parent,
        ContainmentFeature::OwnedRelationship,
        projection.membership(ids, is_type),
    )
    .map_err(model)?;
    editor.commit(tx).map_err(model)?;
    Ok(editor.document().clone())
}
fn copies(resources: &[Resource]) -> Vec<Resource> {
    resources
        .iter()
        .map(|r| Resource {
            key: r.key.clone(),
            document: r.document.clone(),
            fragments: r.fragments,
        })
        .collect()
}
fn wire(
    local: &str,
    document: &Document,
    libraries: &[Resource],
    object: &ObjectId,
) -> Result<Value, Error> {
    let value = if object.resource == local {
        let target = document
            .element(&object.element)
            .ok_or(Error::TargetScope)?;
        target["id"].clone()
    } else {
        let resource = libraries
            .iter()
            .find(|r| r.key == object.resource)
            .ok_or(Error::TargetScope)?;
        // Native force-default library references use semantic fragments. Other
        // fragment policies require another reviewed encoding, not a fallback.
        if resource.fragments != FragmentIdentity::SemanticElementId {
            return Err(Error::TargetScope);
        }
        json!(format!(
            "sysml:ViewDefinition {}#{}",
            object.resource, object.element
        ))
    };
    let mut supplied = vec![Resource {
        key: local.into(),
        document: document.clone(),
        fragments: FragmentIdentity::JsonNodeId,
    }];
    supplied.extend(copies(libraries));
    let registry = Registry::new(supplied).map_err(Error::Reference)?;
    let target = registry
        .resolve(local, Some(&value), C::ViewDefinition)
        .map_err(Error::Reference)?
        .ok_or(Error::TargetScope)?;
    if target.resource() != object.resource || target.element_id() != object.element {
        return Err(Error::TargetScope);
    }
    registry.target(&target).map_err(Error::Reference)?;
    Ok(value)
}
/// Consumes one owned transaction. No caller plan/snapshot is replayed. Each
/// query observation is materialized from its actual prefix; only one finished
/// subtree is appended to the original transaction and caller commit stays separate.
/// Libraries are owned immutable selected documents; no loading or import inference.
pub fn stage<F: Fn() -> bool>(
    mut transaction: Transaction,
    parent: &str,
    ids: Ids,
    local_resource: &str,
    libraries: Vec<Resource>,
    presence: Presence,
    cancelled: F,
) -> Result<Transaction, Error> {
    let control = Control {
        poll: cancelled,
        signalled: Cell::new(false),
    };
    if control.cancelled() {
        return Err(Error::Cancelled);
    }
    if local_resource.is_empty()
        || libraries.len() > 32
        || libraries.iter().any(|r| r.key == local_resource)
    {
        return Err(Error::ResourceIdentity);
    }
    if !transaction.document().is_instance_of(parent, C::Namespace) {
        return Err(Error::TargetScope);
    }
    let is_type = transaction.document().is_instance_of(parent, C::Type);
    let base = transaction.document().clone();
    let initial = materialize(&base, parent, &ids, is_type, &Projection::new(&ids))?;
    let mut context = Local {
        context: NativeContext::new(&initial, &ids.view.element).map_err(model)?,
        control: &control,
    };
    let query_context = match presence {
        Presence::ResourceSet => {
            ResourceContext::Member(ObjectId::new(local_resource, &ids.view.element))
        }
        Presence::WithoutResourceSet => ResourceContext::ResourceWithoutSet,
    };
    let outcome = bound::plan(
        query_context.clone(),
        ReferenceEncoding::SemanticElementIds,
        &mut context,
        |prefix| {
            if control.cancelled() {
                return Err(gap("cancelled"));
            }
            let mut projection = Projection::new(&ids);
            for step in prefix {
                projection.local(step).map_err(gap)?
            }
            let visible = materialize(&base, parent, &ids, is_type, &projection).map_err(gap)?;
            let mut resources = vec![];
            if matches!(presence, Presence::ResourceSet) {
                resources.push(ResourceSnapshot::new(local_resource, visible));
                resources.extend(
                    libraries
                        .iter()
                        .map(|r| ResourceSnapshot::new(&r.key, r.document.clone())),
                );
            }
            PrefixSnapshot::new(
                ResourceSet::new(resources).map_err(gap)?,
                query_context.clone(),
                prefix.to_vec(),
            )
            .map_err(gap)
        },
    );
    if control.signalled.get() {
        return Err(Error::Cancelled);
    }
    let plan = match outcome {
        bound::Outcome::Planned(p) => p,
        bound::Outcome::Incomplete { problem, .. } => return Err(Error::Planning(problem)),
    };
    let mut projection = Projection::new(&ids);
    for step in plan.steps() {
        match step {
            BoundStep::Local(step) => projection.local(step)?,
            BoundStep::GeneralViewType {
                source_method: "caseViewUsage",
                typing: Target::Planned(object),
                definition,
            } if projection.allocated == Some(*object) && projection.attached => {
                let value = if let Some(target) = definition {
                    let snapshot = plan.snapshot().ok_or(Error::UnsupportedPlan)?;
                    snapshot.validate(&plan).map_err(model)?;
                    let object = snapshot.resolve(target).map_err(model)?;
                    let visible = materialize(&base, parent, &ids, is_type, &projection)?;
                    wire(local_resource, &visible, &libraries, object)?
                } else {
                    Value::Null
                };
                put(&mut projection.typing, "type", value)?;
            }
            _ => return Err(Error::UnsupportedPlan),
        }
    }
    if control.cancelled() {
        return Err(Error::Cancelled);
    }
    if !projection.complete() {
        return Err(Error::UnsupportedPlan);
    }
    transaction
        .attach_preallocated_subtree(
            parent,
            ContainmentFeature::OwnedRelationship,
            projection.membership(&ids, is_type),
        )
        .map_err(model)?;
    Ok(transaction)
}

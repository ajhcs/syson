/*******************************************************************************
 * Copyright (c) 2023, 2026 Obeo.
 * This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v2.0
 * which accompanies this distribution, and is available at
 * https://www.eclipse.org/legal/epl-2.0/
 *
 * SPDX-License-Identifier: EPL-2.0
 *
 * Contributors:
 *     Obeo - initial API and implementation
 *******************************************************************************/
// SPDX-License-Identifier: EPL-2.0
// Bounded ConnectionDefinition document effects from pinned initializer source.
// Actual raw cache/flags/reference state remains reported as unrepresented.
use super::*;
use syson_model::{Transaction,Document,Editor,ContainmentFeature as F,RawAdmissionReport,RawAdmissionError};
use syson_model::factory::{RawFactory,FreshElementIdProvider,FactoryError};

#[derive(Debug)]
pub enum ConnectionHelperCause {
    Native(Error), Model(ModelError), Factory(FactoryError), Admission(RawAdmissionError),
    ResourceProvider(String), Cancelled, UnsupportedRoot(C), UnsupportedEffect(&'static str),
}
#[derive(Debug)]
pub struct ConnectionHelperReport {
    pub allocations:Vec<RawAdmissionReport>,
    pub applied_steps:Vec<Step>,
    pub declarative_plan:Option<Plan>,
}
#[derive(Debug)]
pub struct ConnectionHelperError { pub cause:ConnectionHelperCause, pub partial:ConnectionHelperReport }
struct Pending { key:PlannedObject, id:String, class:C, scratch:Option<Transaction> }
fn unsupported<T>(contract:&'static str)->Resolution<T> {
    Resolution::Unsupported(ContextGap {contract,reason:"outside the pinned ConnectionDefinition query invariant".into()})
}
/// Only root namespace and direct exact-class peer count are queried by this
/// pinned initializer. Fresh helpers attach BELOW root and cannot change root's
/// owning membership/namespace or its parent's direct ConnectionDefinition
/// member count. Other queries/effects are refused rather than guessed.
struct RootCountContext<'a,K> { native:NativeContext, cancel:&'a mut K }
fn safe_prefix(prefix:&[Step])->bool {
    prefix.iter().all(|step|match &step.effect {
        Effect::AllocateRaw {classifier,..}=>matches!(classifier,C::ReferenceUsage|C::FeatureMembership),
        Effect::SetScalar {target:Target::Root,feature,..}=>matches!(feature,ScalarFeature::IsSufficient|ScalarFeature::DeclaredName),
        Effect::SetScalar {target:Target::Planned(_),feature,..}=>matches!(feature,ScalarFeature::DeclaredName|ScalarFeature::IsComposite|ScalarFeature::IsEnd),
        Effect::Attach {owner:Target::Root,feature:Containment::OwnedRelationship,child:Target::Planned(_)}=>true,
        Effect::Attach {owner:Target::Planned(_),feature:Containment::OwnedRelatedElement,child:Target::Planned(_)}=>true,
        _=>false,
    })
}
impl<K:FnMut()->bool> Context for RootCountContext<'_,K> {
    fn cancelled(&mut self)->bool {(self.cancel)()}
    fn owning_namespace(&mut self,p:&[Step])->Resolution<Option<ExternalObject>> {
        if !safe_prefix(p) {return unsupported("connection root namespace")}
        self.native.owning_namespace(&[])
    }
    fn exact_class_owned_member_count(&mut self,n:&ExternalObject,c:C,p:&[Step])->Resolution<u64> {
        if c!=C::ConnectionDefinition || !safe_prefix(p) {return unsupported("connection exact-class peer count")}
        self.native.exact_class_owned_member_count(n,c,&[])
    }
    fn owner(&mut self,_:&[Step])->Resolution<Option<ExternalObject>> {unsupported("connection owner query")}
    fn owning_membership(&mut self,_:&[Step])->Resolution<Option<ExternalObject>> {unsupported("connection membership query")}
    fn is_instance_of(&mut self,_:&ExternalObject,_:C,_:&[Step])->Resolution<bool> {unsupported("connection subtype query")}
    fn effective_name(&mut self,_:&ExternalObject,_:&[Step])->Resolution<Option<String>> {unsupported("connection effective-name query")}
    fn general_view_definition(&mut self,_:&[Step])->Resolution<Option<ExternalObject>> {unsupported("connection library query")}
}
fn pending_index(objects:&[Pending],key:PlannedObject)->Result<usize,ConnectionHelperCause> {
    objects.iter().position(|o|o.key==key).ok_or(ConnectionHelperCause::UnsupportedEffect("unknown planned object"))
}
fn take_node(object:&mut Pending)->Result<serde_json::Value,ConnectionHelperCause> {
    let scratch=object.scratch.take().ok_or(ConnectionHelperCause::UnsupportedEffect("planned object already attached"))?;
    scratch.document().element(&object.id).cloned()
        .ok_or(ConnectionHelperCause::UnsupportedEffect("detached root absent"))
}
type CauseResult<T>=std::result::Result<T,ConnectionHelperCause>;
/// The existing root must already be staged in the OWNED transaction.
/// Fresh identities are allocated once, in source effect order; cancellation
/// drops document changes but does not refund provider/factory consumption.
/// Supported JSON scalar/containment effects share one later caller commit.
/// No reference/proxy/inverse/notification/full native-state semantics.
pub fn initialize_connection_definition<P,R,K>(
    mut transaction:Transaction,root:&str,factory:&mut RawFactory<P>,
    resources:&mut R,mut cancelled:K,
)->std::result::Result<(Transaction,ConnectionHelperReport),ConnectionHelperError>
where P:FreshElementIdProvider,R:FnMut(C,&str)->std::result::Result<String,String>,K:FnMut()->bool {
    let mut report=ConnectionHelperReport {allocations:Vec::new(),applied_steps:Vec::new(),declarative_plan:None};
    let result=(||->CauseResult<()> {
        if cancelled() {return Err(ConnectionHelperCause::Cancelled)}
        let index=Index::new(transaction.document()).map_err(ConnectionHelperCause::Native)?;
        let class=index.node(root).map_err(ConnectionHelperCause::Native)?.classifier;
        if class!=C::ConnectionDefinition {return Err(ConnectionHelperCause::UnsupportedRoot(class))}
        let native=NativeContext::new(transaction.document(),root).map_err(ConnectionHelperCause::Native)?;
        let plan={
            let mut context=RootCountContext {native,cancel:&mut cancelled};
            match syson_initializer::plan(class,&mut context) {
                Outcome::Planned(plan)=>plan,
                Outcome::Incomplete(p)=>return Err(ConnectionHelperCause::Native(Error::Planning(p.problem))),
            }
        };
        if !safe_prefix(&plan.steps) {return Err(ConnectionHelperCause::UnsupportedEffect("unproven connection effect"))}
        report.declarative_plan=Some(plan.clone());
        let mut objects:Vec<Pending>=Vec::new();let mut owner_nonce=None;
        for step in &plan.steps {
            if cancelled() {return Err(ConnectionHelperCause::Cancelled)}
            match &step.effect {
                Effect::AllocateRaw {object,classifier,..}=>{
                    if !matches!(classifier,C::ReferenceUsage|C::FeatureMembership) {return Err(ConnectionHelperCause::UnsupportedEffect("unsupported allocation classifier"))}
                    if owner_nonce.is_some_and(|n|n!=object.owner()) || objects.iter().any(|o|o.key==*object) {return Err(ConnectionHelperCause::UnsupportedEffect("foreign/duplicate planned allocation"))}
                    owner_nonce=Some(object.owner());
                    let raw=factory.allocate(*classifier).map_err(ConnectionHelperCause::Factory)?;
                    let resource=resources(*classifier,raw.semantic_id()).map_err(ConnectionHelperCause::ResourceProvider)?;
                    let (node,projection)=RawAdmissionReport::project_selected(&raw,&resource,&[]).map_err(ConnectionHelperCause::Admission)?;
                    let id=raw.semantic_id().to_owned();
                    // Private scratch transactions keep children genuinely
                    // detached while their source scalar initializer runs.
                    // They are never committed to a live Editor/history.
                    let doc=Document::from_json(&serde_json::json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[node]}).to_string()).map_err(ConnectionHelperCause::Model)?;
                    let scratch=Editor::new(doc).begin();
                    objects.push(Pending {key:*object,id,class:*classifier,scratch:Some(scratch)});
                    report.allocations.push(projection);
                }
                Effect::SetScalar {target,feature,value}=>{
                    let (name,value)=scalar(*feature,value);
                    match target {
                        Target::Root=>{
                            if *feature==ScalarFeature::IsSufficient {
                                if value!=serde_json::Value::Bool(true) {
                                    return Err(ConnectionHelperCause::UnsupportedEffect("unproven sufficient initializer value"));
                                }
                                transaction.initialize_connection_definition_sufficient(root).map_err(ConnectionHelperCause::Model)?;
                            } else {
                                transaction.set_attribute(root,name,value).map_err(ConnectionHelperCause::Model)?;
                            }
                        }
                        Target::Planned(key)=>{
                            let at=pending_index(&objects,*key)?;
                            let object=&mut objects[at];
                            if object.class!=C::ReferenceUsage {return Err(ConnectionHelperCause::UnsupportedEffect("non-reference helper scalar"))}
                            object.scratch.as_mut().ok_or(ConnectionHelperCause::UnsupportedEffect("post-attachment helper scalar"))?
                                .set_attribute(&object.id,name,value).map_err(ConnectionHelperCause::Model)?;
                        }
                        Target::External(_)=>return Err(ConnectionHelperCause::UnsupportedEffect("external scalar target")),
                    }
                }
                Effect::Attach {owner,feature,child}=>{
                    let Target::Planned(child)=child else {return Err(ConnectionHelperCause::UnsupportedEffect("non-planned child"))};
                    let child_at=pending_index(&objects,*child)?;
                    match (owner,feature) {
                        (Target::Planned(parent),Containment::OwnedRelatedElement)=>{
                            let parent_at=pending_index(&objects,*parent)?;
                            if parent_at==child_at || objects[parent_at].class!=C::FeatureMembership || objects[child_at].class!=C::ReferenceUsage {return Err(ConnectionHelperCause::UnsupportedEffect("unproven related-element attachment"))}
                            let child=take_node(&mut objects[child_at])?;
                            let parent=&mut objects[parent_at];let id=parent.id.clone();
                            parent.scratch.as_mut().ok_or(ConnectionHelperCause::UnsupportedEffect("parent already attached"))?
                                .attach_preallocated_subtree(&id,F::OwnedRelatedElement,child).map_err(ConnectionHelperCause::Model)?;
                        }
                        (Target::Root,Containment::OwnedRelationship)=>{
                            if objects[child_at].class!=C::FeatureMembership {return Err(ConnectionHelperCause::UnsupportedEffect("unproven root child"))}
                            let child=take_node(&mut objects[child_at])?;
                            transaction.attach_preallocated_subtree(root,F::OwnedRelationship,child).map_err(ConnectionHelperCause::Model)?;
                        }
                        _=>return Err(ConnectionHelperCause::UnsupportedEffect("unproven containment")),
                    }
                }
                Effect::SetReference {..}=>return Err(ConnectionHelperCause::UnsupportedEffect("reference/inverse application")),
            }
            report.applied_steps.push(step.clone());
        }
        if objects.iter().any(|o|o.scratch.is_some()) {return Err(ConnectionHelperCause::UnsupportedEffect("orphan detached helper"))}
        if cancelled() {return Err(ConnectionHelperCause::Cancelled)}
        Ok(())
    })();
    match result {
        Ok(())=>Ok((transaction,report)),
        Err(cause)=>Err(ConnectionHelperError {cause,partial:report}),
    }
}

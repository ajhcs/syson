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
// Bounded AcceptActionUsage document effects from pinned initializer source.
// Actual raw cache/flags/reference state remains reported as unrepresented.
use super::*;
use syson_model::{Transaction,Document,Editor,ContainmentFeature as F,RawAdmissionReport,RawAdmissionError};
use syson_model::factory::{RawFactory,FreshElementIdProvider,FactoryError};

#[derive(Debug)]
pub enum AcceptActionHelperCause {
    Native(Error), Model(ModelError), Factory(FactoryError), Admission(RawAdmissionError),
    ResourceProvider(String), Cancelled, UnsupportedRoot(C), UnsupportedEffect(&'static str),
}
#[derive(Debug)]
pub struct AcceptActionHelperReport {
    pub allocations:Vec<RawAdmissionReport>,
    pub applied_steps:Vec<Step>,
    pub declarative_plan:Option<Plan>,
}
#[derive(Debug)]
pub struct AcceptActionHelperError { pub cause:AcceptActionHelperCause, pub partial:AcceptActionHelperReport }
struct Pending { key:PlannedObject, id:String, class:C, scratch:Option<Transaction> }
fn unsupported<T>(contract:&'static str)->Resolution<T> {
    Resolution::Unsupported(ContextGap {contract,reason:"outside the pinned AcceptActionUsage query invariant".into()})
}
/// Both source naming queries occur before the first mutation. Later-prefix
/// queries are refused. Raw parameter children never run Usage/ReferenceUsage
/// initializers, and add no further namespace/name queries.
struct RootCountContext<'a,K> { native:NativeContext, cancel:&'a mut K }
fn safe_prefix(prefix:&[Step])->bool {
    prefix.iter().all(|step|match &step.effect {
        Effect::AllocateRaw {classifier,..}=>matches!(classifier,C::ReferenceUsage|C::ParameterMembership),
        Effect::SetScalar {target:Target::Root,feature,..}=>matches!(feature,ScalarFeature::IsComposite|ScalarFeature::DeclaredName),
        Effect::SetScalar {target:Target::Planned(_),feature,..}=>matches!(feature,ScalarFeature::DeclaredName|ScalarFeature::Direction),
        Effect::Attach {owner:Target::Root,feature:Containment::OwnedRelationship,child:Target::Planned(_)}=>true,
        Effect::Attach {owner:Target::Planned(_),feature:Containment::OwnedRelatedElement,child:Target::Planned(_)}=>true,
        _=>false,
    })
}
impl<K:FnMut()->bool> Context for RootCountContext<'_,K> {
    fn cancelled(&mut self)->bool {(self.cancel)()}
    fn owning_namespace(&mut self,p:&[Step])->Resolution<Option<ExternalObject>> {
        if !p.is_empty() {return unsupported("AcceptActionUsage query after first effect")}
        self.native.owning_namespace(&[])
    }
    fn exact_class_owned_member_count(&mut self,n:&ExternalObject,c:C,p:&[Step])->Resolution<u64> {
        if c!=C::AcceptActionUsage || !p.is_empty() {return unsupported("AcceptActionUsage exact-class count after first effect")}
        self.native.exact_class_owned_member_count(n,c,&[])
    }
    fn owner(&mut self,_:&[Step])->Resolution<Option<ExternalObject>> {unsupported("connection owner query")}
    fn owning_membership(&mut self,_:&[Step])->Resolution<Option<ExternalObject>> {unsupported("connection membership query")}
    fn is_instance_of(&mut self,_:&ExternalObject,_:C,_:&[Step])->Resolution<bool> {unsupported("connection subtype query")}
    fn effective_name(&mut self,_:&ExternalObject,_:&[Step])->Resolution<Option<String>> {unsupported("accept action effective-name query")}
    fn general_view_definition(&mut self,_:&[Step])->Resolution<Option<ExternalObject>> {unsupported("connection library query")}
}
fn pending_index(objects:&[Pending],key:PlannedObject)->Result<usize,AcceptActionHelperCause> {
    objects.iter().position(|o|o.key==key).ok_or(AcceptActionHelperCause::UnsupportedEffect("unknown planned object"))
}
fn take_node(object:&mut Pending)->Result<serde_json::Value,AcceptActionHelperCause> {
    let scratch=object.scratch.take().ok_or(AcceptActionHelperCause::UnsupportedEffect("planned object already attached"))?;
    scratch.document().element(&object.id).cloned()
        .ok_or(AcceptActionHelperCause::UnsupportedEffect("detached root absent"))
}
type CauseResult<T>=std::result::Result<T,AcceptActionHelperCause>;
/// The existing root must already be staged in the OWNED transaction.
/// Fresh identities are allocated once, in source effect order; cancellation
/// drops document changes but does not refund provider/factory consumption.
/// Supported JSON scalar/containment effects share one later caller commit.
/// No reference/proxy/inverse/notification/full native-state semantics.
pub fn initialize_accept_action<P,R,K>(
    mut transaction:Transaction,root:&str,factory:&mut RawFactory<P>,
    resources:&mut R,mut cancelled:K,
)->std::result::Result<(Transaction,AcceptActionHelperReport),AcceptActionHelperError>
where P:FreshElementIdProvider,R:FnMut(C,&str)->std::result::Result<String,String>,K:FnMut()->bool {
    let mut report=AcceptActionHelperReport {allocations:Vec::new(),applied_steps:Vec::new(),declarative_plan:None};
    let result=(||->CauseResult<()> {
        if cancelled() {return Err(AcceptActionHelperCause::Cancelled)}
        let index=Index::new(transaction.document()).map_err(AcceptActionHelperCause::Native)?;
        let class=index.node(root).map_err(AcceptActionHelperCause::Native)?.classifier;
        if class!=C::AcceptActionUsage {return Err(AcceptActionHelperCause::UnsupportedRoot(class))}
        let native=NativeContext::new(transaction.document(),root).map_err(AcceptActionHelperCause::Native)?;
        let plan={
            let mut context=RootCountContext {native,cancel:&mut cancelled};
            match syson_initializer::plan(class,&mut context) {
                Outcome::Planned(plan)=>plan,
                Outcome::Incomplete(p)=>return Err(AcceptActionHelperCause::Native(Error::Planning(p.problem))),
            }
        };
        if !safe_prefix(&plan.steps) {return Err(AcceptActionHelperCause::UnsupportedEffect("unproven accept action effect"))}
        report.declarative_plan=Some(plan.clone());
        let mut objects:Vec<Pending>=Vec::new();let mut owner_nonce=None;
        for step in &plan.steps {
            if cancelled() {return Err(AcceptActionHelperCause::Cancelled)}
            match &step.effect {
                Effect::AllocateRaw {object,classifier,..}=>{
                    if !matches!(classifier,C::ReferenceUsage|C::ParameterMembership) {return Err(AcceptActionHelperCause::UnsupportedEffect("unsupported allocation classifier"))}
                    if owner_nonce.is_some_and(|n|n!=object.owner()) || objects.iter().any(|o|o.key==*object) {return Err(AcceptActionHelperCause::UnsupportedEffect("foreign/duplicate planned allocation"))}
                    owner_nonce=Some(object.owner());
                    let raw=factory.allocate(*classifier).map_err(AcceptActionHelperCause::Factory)?;
                    let resource=resources(*classifier,raw.semantic_id()).map_err(AcceptActionHelperCause::ResourceProvider)?;
                    let (node,projection)=RawAdmissionReport::project_selected(&raw,&resource,&[]).map_err(AcceptActionHelperCause::Admission)?;
                    let id=raw.semantic_id().to_owned();
                    // Private scratch transactions keep children genuinely
                    // detached while their source scalar initializer runs.
                    // They are never committed to a live Editor/history.
                    let doc=Document::from_json(&serde_json::json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[node]}).to_string()).map_err(AcceptActionHelperCause::Model)?;
                    let scratch=Editor::new(doc).begin();
                    objects.push(Pending {key:*object,id,class:*classifier,scratch:Some(scratch)});
                    report.allocations.push(projection);
                }
                Effect::SetScalar {target,feature,value}=>{
                    let (name,value)=scalar(*feature,value);
                    match target {
                        Target::Root=>transaction.set_attribute(root,name,value).map_err(AcceptActionHelperCause::Model)?,
                        Target::Planned(key)=>{
                            let at=pending_index(&objects,*key)?;
                            let object=&mut objects[at];
                            if object.class!=C::ReferenceUsage || !matches!(feature,ScalarFeature::DeclaredName|ScalarFeature::Direction) {return Err(AcceptActionHelperCause::UnsupportedEffect("raw parameter supports only name/direction effects"))}
                            object.scratch.as_mut().ok_or(AcceptActionHelperCause::UnsupportedEffect("post-attachment helper scalar"))?
                                .set_attribute(&object.id,name,value).map_err(AcceptActionHelperCause::Model)?;
                        }
                        Target::External(_)=>return Err(AcceptActionHelperCause::UnsupportedEffect("external scalar target")),
                    }
                }
                Effect::Attach {owner,feature,child}=>{
                    let Target::Planned(child)=child else {return Err(AcceptActionHelperCause::UnsupportedEffect("non-planned child"))};
                    let child_at=pending_index(&objects,*child)?;
                    match (owner,feature) {
                        (Target::Planned(parent),Containment::OwnedRelatedElement)=>{
                            let parent_at=pending_index(&objects,*parent)?;
                            if parent_at==child_at || objects[parent_at].class!=C::ParameterMembership || objects[child_at].class!=C::ReferenceUsage {return Err(AcceptActionHelperCause::UnsupportedEffect("unproven related-element attachment"))}
                            let child=take_node(&mut objects[child_at])?;
                            let parent=&mut objects[parent_at];let id=parent.id.clone();
                            parent.scratch.as_mut().ok_or(AcceptActionHelperCause::UnsupportedEffect("parent already attached"))?
                                .attach_preallocated_subtree(&id,F::OwnedRelatedElement,child).map_err(AcceptActionHelperCause::Model)?;
                        }
                        (Target::Root,Containment::OwnedRelationship)=>{
                            if objects[child_at].class!=C::ParameterMembership {return Err(AcceptActionHelperCause::UnsupportedEffect("unproven root child"))}
                            let child=take_node(&mut objects[child_at])?;
                            transaction.attach_preallocated_subtree(root,F::OwnedRelationship,child).map_err(AcceptActionHelperCause::Model)?;
                        }
                        _=>return Err(AcceptActionHelperCause::UnsupportedEffect("unproven containment")),
                    }
                }
                Effect::SetReference {..}=>return Err(AcceptActionHelperCause::UnsupportedEffect("reference/inverse application")),
            }
            report.applied_steps.push(step.clone());
        }
        if objects.iter().any(|o|o.scratch.is_some()) {return Err(AcceptActionHelperCause::UnsupportedEffect("orphan detached helper"))}
        if cancelled() {return Err(AcceptActionHelperCause::Cancelled)}
        Ok(())
    })();
    match result {
        Ok(())=>Ok((transaction,report)),
        Err(cause)=>Err(AcceptActionHelperError {cause,partial:report}),
    }
}

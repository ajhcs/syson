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
// Bounded PortDefinition document effects from pinned initializer source.
// Actual raw cache/flags/reference state remains reported as unrepresented.
use super::*;
use syson_initializer::ReferenceFeature as RF;
use syson_model::{Transaction,ModelError,ContainmentFeature as F,RawAdmissionReport,RawAdmissionError,PortDefinitionReference as PR};
use syson_model::factory::{RawFactory,FreshElementIdProvider,FactoryError};
#[derive(Debug)]
pub enum PortDefinitionHelperCause {
    Native(Error),Model(ModelError),Factory(FactoryError),Admission(RawAdmissionError),
    ResourceProvider(String),Cancelled,UnsupportedRoot(C),UnsupportedEffect(&'static str),
}
#[derive(Debug)]
pub struct PortDefinitionHelperReport {
    pub allocations:Vec<RawAdmissionReport>,
    pub applied_steps:Vec<Step>,
    pub declarative_plan:Option<Plan>,
}
#[derive(Debug)]
pub struct PortDefinitionHelperError {pub cause:PortDefinitionHelperCause,pub partial:PortDefinitionHelperReport}
struct Pending {key:PlannedObject,id:String,class:C,node:Option<serde_json::Value>,attached:bool}
type R<T>=std::result::Result<T,PortDefinitionHelperCause>;
fn unsupported<T>(contract:&'static str)->Resolution<T> {
    Resolution::Unsupported(ContextGap {contract,reason:"outside the exact PortDefinition initializer query contract".into()})
}
struct RootContext<'a,K> {native:NativeContext,cancel:&'a mut K}
impl<K:FnMut()->bool> Context for RootContext<'_,K> {
    fn cancelled(&mut self)->bool {(self.cancel)()}
    fn owning_namespace(&mut self,p:&[Step])->Resolution<Option<ExternalObject>> {
        if !p.is_empty() {return unsupported("PortDefinition query after first effect")}
        self.native.owning_namespace(&[])
    }
    fn exact_class_owned_member_count(&mut self,n:&ExternalObject,c:C,p:&[Step])->Resolution<u64> {
        if c!=C::PortDefinition || !p.is_empty() {return unsupported("PortDefinition exact-class query invariant")}
        self.native.exact_class_owned_member_count(n,c,&[])
    }
    fn owner(&mut self,_:&[Step])->Resolution<Option<ExternalObject>> {unsupported("PortDefinition owner query")}
    fn owning_membership(&mut self,_:&[Step])->Resolution<Option<ExternalObject>> {unsupported("PortDefinition membership query")}
    fn is_instance_of(&mut self,_:&ExternalObject,_:C,_:&[Step])->Resolution<bool> {unsupported("PortDefinition subtype query")}
    fn effective_name(&mut self,_:&ExternalObject,_:&[Step])->Resolution<Option<String>> {unsupported("PortDefinition effective name")}
    fn general_view_definition(&mut self,_:&[Step])->Resolution<Option<ExternalObject>> {unsupported("PortDefinition library query")}
}
fn at(objects:&[Pending],key:PlannedObject)->R<usize> {
    objects.iter().position(|o|o.key==key).ok_or(PortDefinitionHelperCause::UnsupportedEffect("foreign/unknown planned object"))
}
/// Exact pinned PortDefinition effects over an already-staged root.
/// Raw child initializers, native aliases/caches/eIsSet/notification/proxy
/// semantics and ResourceSet lookup are not supplied by this JSON projection.
pub fn initialize_port_definition<P,Q,K>(
    mut transaction:Transaction,root:&str,factory:&mut RawFactory<P>,
    resources:&mut Q,mut cancelled:K,
)->std::result::Result<(Transaction,PortDefinitionHelperReport),PortDefinitionHelperError>
where P:FreshElementIdProvider,Q:FnMut(C,&str)->std::result::Result<String,String>,K:FnMut()->bool {
    let mut report=PortDefinitionHelperReport {allocations:Vec::new(),applied_steps:Vec::new(),declarative_plan:None};
    let result=(||->R<()> {
        if cancelled() {return Err(PortDefinitionHelperCause::Cancelled)}
        let index=Index::new(transaction.document()).map_err(PortDefinitionHelperCause::Native)?;
        let class=index.node(root).map_err(PortDefinitionHelperCause::Native)?.classifier;
        if class!=C::PortDefinition {return Err(PortDefinitionHelperCause::UnsupportedRoot(class))}
        let native=NativeContext::new(transaction.document(),root).map_err(PortDefinitionHelperCause::Native)?;
        let plan={
            let mut context=RootContext {native,cancel:&mut cancelled};
            match syson_initializer::plan(class,&mut context) {
                Outcome::Planned(plan)=>plan,
                Outcome::Incomplete(p)=>return Err(PortDefinitionHelperCause::Native(Error::Planning(p.problem))),
            }
        };
        if plan.steps.len()!=9 {return Err(PortDefinitionHelperCause::UnsupportedEffect("unexpected PortDefinition effect count"))}
        let allocation_classes=plan.steps.iter().filter_map(|s|match &s.effect {Effect::AllocateRaw {classifier,..}=>Some(*classifier),_=>None}).collect::<Vec<_>>();
        if allocation_classes!=[C::OwningMembership,C::ConjugatedPortDefinition,C::PortConjugation] {
            return Err(PortDefinitionHelperCause::UnsupportedEffect("unexpected PortDefinition allocation order"))
        }
        report.declarative_plan=Some(plan.clone());
        let mut objects:Vec<Pending>=Vec::new();let mut nonce=None;
        for step in &plan.steps {
            if cancelled() {return Err(PortDefinitionHelperCause::Cancelled)}
            match &step.effect {
                Effect::SetScalar {target:Target::Root,feature:ScalarFeature::DeclaredName,value}=>{
                    let (name,value)=scalar(ScalarFeature::DeclaredName,value);
                    transaction.set_attribute(root,name,value).map_err(PortDefinitionHelperCause::Model)?;
                }
                Effect::AllocateRaw {object,classifier,..}=>{
                    if !matches!(classifier,C::OwningMembership|C::ConjugatedPortDefinition|C::PortConjugation) {
                        return Err(PortDefinitionHelperCause::UnsupportedEffect("unexpected raw allocation class"))
                    }
                    if nonce.is_some_and(|n|n!=object.owner()) || objects.iter().any(|o|o.key==*object) {
                        return Err(PortDefinitionHelperCause::UnsupportedEffect("foreign/duplicate allocation owner"))
                    }
                    nonce=Some(object.owner());
                    let raw=factory.allocate(*classifier).map_err(PortDefinitionHelperCause::Factory)?;
                    let resource=resources(*classifier,raw.semantic_id()).map_err(PortDefinitionHelperCause::ResourceProvider)?;
                    let (node,projection)=RawAdmissionReport::project_selected(&raw,&resource,&[]).map_err(PortDefinitionHelperCause::Admission)?;
                    objects.push(Pending {key:*object,id:raw.semantic_id().into(),class:*classifier,node:Some(node),attached:false});
                    report.allocations.push(projection);
                }
                Effect::Attach {owner,feature,child:Target::Planned(child)}=>{
                    let ci=at(&objects,*child)?;
                    let (parent,typed)=match (owner,feature) {
                        (Target::Root,Containment::OwnedRelationship) if objects[ci].class==C::OwningMembership=>
                            (root.to_owned(),F::OwnedRelationship),
                        (Target::Planned(parent),Containment::OwnedRelatedElement)=>{
                            let pi=at(&objects,*parent)?;
                            if !objects[pi].attached || objects[pi].class!=C::OwningMembership || objects[ci].class!=C::ConjugatedPortDefinition {
                                return Err(PortDefinitionHelperCause::UnsupportedEffect("unproven conjugated definition ownership"))
                            }
                            (objects[pi].id.clone(),F::OwnedRelatedElement)
                        }
                        (Target::Planned(parent),Containment::OwnedRelationship)=>{
                            let pi=at(&objects,*parent)?;
                            if !objects[pi].attached || objects[pi].class!=C::ConjugatedPortDefinition || objects[ci].class!=C::PortConjugation {
                                return Err(PortDefinitionHelperCause::UnsupportedEffect("unproven conjugation ownership"))
                            }
                            (objects[pi].id.clone(),F::OwnedRelationship)
                        }
                        _=>return Err(PortDefinitionHelperCause::UnsupportedEffect("unsupported attachment target/role")),
                    };
                    let child=objects[ci].node.take().ok_or(PortDefinitionHelperCause::UnsupportedEffect("helper attached twice"))?;
                    transaction.attach_preallocated_subtree(&parent,typed,child).map_err(PortDefinitionHelperCause::Model)?;
                    objects[ci].attached=true;
                }
                Effect::SetReference {target:Target::Planned(source),feature,value}=>{
                    let si=at(&objects,*source)?;
                    if !objects[si].attached || objects[si].class!=C::PortConjugation {
                        return Err(PortDefinitionHelperCause::UnsupportedEffect("reference source is not staged PortConjugation"))
                    }
                    let (reference,target)=match (feature,value) {
                        (RF::ConjugatedType,Some(Target::Planned(target)))=>{
                            let ti=at(&objects,*target)?;
                            if !objects[ti].attached || objects[ti].class!=C::ConjugatedPortDefinition {
                                return Err(PortDefinitionHelperCause::UnsupportedEffect("conjugatedType target role"))
                            }
                            (PR::ConjugatedType,objects[ti].id.clone())
                        }
                        (RF::OriginalPortDefinition,Some(Target::Root))=>(PR::OriginalPortDefinition,root.to_owned()),
                        // None, External, foreign keys, Type/TypedFeature and all
                        // other feature/role combinations remain unsupported.
                        _=>return Err(PortDefinitionHelperCause::UnsupportedEffect("unsupported null/external/reference feature or target role")),
                    };
                    transaction.initialize_port_definition_reference(root,&objects[si].id,reference,&target).map_err(PortDefinitionHelperCause::Model)?;
                }
                _=>return Err(PortDefinitionHelperCause::UnsupportedEffect("effect outside exact PortDefinition contract")),
            }
            report.applied_steps.push(step.clone());
        }
        if objects.iter().any(|o|!o.attached || o.node.is_some()) {
            return Err(PortDefinitionHelperCause::UnsupportedEffect("unattached raw helper"))
        }
        if cancelled() {return Err(PortDefinitionHelperCause::Cancelled)}
        Ok(())
    })();
    match result {
        Ok(())=>Ok((transaction,report)),
        Err(cause)=>Err(PortDefinitionHelperError {cause,partial:report}),
    }
}

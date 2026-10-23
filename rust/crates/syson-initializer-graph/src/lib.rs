// SPDX-License-Identifier: EPL-2.0
//! Bounded local raw-containment context and allocation-free initializer staging.
//! No factory, inverse notifications, cross-resource resolver or EMF serializer.
#![forbid(unsafe_code)]
use std::collections::BTreeMap;
use serde_json::Value;
use syson_initializer::*;
use syson_metamodel::{self as mm, ClassifierId as C};
use syson_model::{Document, Editor, ModelError};
mod local_names;
pub use local_names::{ReferenceEncoding, NameKind};
mod staged;
pub use staged::{initialize_staged, apply_staged_plan};

#[derive(Debug)]
pub enum Error { Gap(ContextGap), Model(ModelError), Unresolved(ContextGap), Planning(Problem), Cancelled, ClassMismatch, StalePlan }
impl From<ModelError> for Error { fn from(e:ModelError)->Self { Self::Model(e) } }
fn gap(contract:&'static str, reason:impl Into<String>)->Error {
    Error::Gap(ContextGap { contract, reason:reason.into() })
}
#[derive(Clone, Debug)]
pub struct Node {
    pub resource_id:String,
    pub element_id:String,
    pub classifier:C,
    pub container:Option<(String, &'static str)>,
    data:Value,
}
/// One Document/resource context. ExternalObject tokens are semantic elementIds
/// in this document only; resource IDs are stored separately and never resolved
/// as elementIds. No proxy or multi-resource fallback.
pub struct Index { pub nodes:BTreeMap<String,Node> }
impl Index {
    pub fn new(document:&Document)->Result<Self,Error> {
        let mut result=Self { nodes:BTreeMap::new() };
        for node in document.value()["content"].as_array().ok_or_else(||gap("raw containment","content is not an array"))? {
            result.visit(node,None)?;
        }
        Ok(result)
    }
    fn visit(&mut self, value:&Value, container:Option<(String,&'static str)>)->Result<(),Error> {
        let id=value["data"]["elementId"].as_str().ok_or_else(||gap("identity","missing semantic elementId"))?.to_owned();
        let raw=value["eClass"].as_str().ok_or_else(||gap("classifier","missing eClass"))?;
        let classifier=raw.strip_prefix("sysml:").and_then(|n|mm::lookup(mm::Package::Domain,n))
            .filter(|c|c.metadata().kind==mm::ClassifierKind::Class)
            .ok_or_else(||gap("classifier",format!("unsupported class {raw}")))?;
        let resource_id=value["id"].as_str().ok_or_else(||gap("identity","missing resource id"))?.to_owned();
        if self.nodes.insert(id.clone(),Node { resource_id,element_id:id.clone(),classifier,container,data:value["data"].clone() }).is_some() {
            return Err(gap("identity","duplicate semantic ID"));
        }
        for feature in ["ownedRelationship","ownedRelatedElement"] {
            if let Some(children)=value["data"].get(feature) {
                for child in children.as_array().ok_or_else(||gap("raw containment","children are not an array"))? {
                    self.visit(child,Some((id.clone(),feature)))?;
                }
            }
        }
        Ok(())
    }
    fn node(&self,id:&str)->Result<&Node,Error> {
        self.nodes.get(id).ok_or_else(||gap("local object",format!("unresolved semantic elementId {id}")))
    }
    fn relationship(&self,id:&str)->Result<Option<&Node>,Error> {
        let node=self.node(id)?;
        match &node.container {
            Some((parent,"ownedRelatedElement"))=>{
                let relation=self.node(parent)?;
                if !mm::is_subtype(relation.classifier,C::Relationship) { return Err(gap("owningRelationship","raw container is not a Relationship")); }
                Ok(Some(relation))
            },
            _=>Ok(None),
        }
    }
    pub fn owner(&self,id:&str)->Result<Option<&Node>,Error> {
        let Some(relation)=self.relationship(id)? else { return Ok(None) };
        match &relation.container {
            Some((parent,"ownedRelationship"))=>Ok(Some(self.node(parent)?)),
            _=>Ok(None),
        }
    }
    pub fn owning_membership(&self,id:&str)->Result<Option<&Node>,Error> {
        Ok(self.relationship(id)?.filter(|n|mm::is_subtype(n.classifier,C::OwningMembership)))
    }
    pub fn owning_namespace(&self,id:&str)->Result<Option<&Node>,Error> {
        let Some(membership)=self.owning_membership(id)? else { return Ok(None) };
        match &membership.container {
            Some((parent,"ownedRelationship"))=>Ok(Some(self.node(parent)?).filter(|n|mm::is_subtype(n.classifier,C::Namespace))),
            _=>Ok(None),
        }
    }
    pub fn exact_class_owned_member_count(&self,namespace:&str,class:C)->Result<u64,Error> {
        let node=self.node(namespace)?;
        if !mm::is_subtype(node.classifier,C::Namespace) { return Err(gap("ownedMember","object is not a Namespace")); }
        let mut count=0;
        for relation in self.nodes.values().filter(|n|n.container.as_ref().is_some_and(|(id,f)|id==namespace && *f=="ownedRelationship") && mm::is_subtype(n.classifier,C::Membership)) {
            count+=self.nodes.values().filter(|n|n.container.as_ref().is_some_and(|(id,f)|id==&relation.element_id && *f=="ownedRelatedElement") && n.classifier==class).count() as u64;
        }
        Ok(count)
    }

}
fn scalar(feature:ScalarFeature,value:&Scalar)->(&'static str,Value) {
    let name=match feature {
        ScalarFeature::DeclaredName=>"declaredName", ScalarFeature::IsComposite=>"isComposite",
        ScalarFeature::Body=>"body", ScalarFeature::IsSufficient=>"isSufficient",
        ScalarFeature::IsEnd=>"isEnd", ScalarFeature::IsVariation=>"isVariation",
        ScalarFeature::Visibility=>"visibility", ScalarFeature::IsImportAll=>"isImportAll",
        ScalarFeature::Direction=>"direction",
    };
    let value=match value {
        Scalar::Text(s)=>Value::String(s.clone()), Scalar::Boolean(b)=>Value::Bool(*b),
        Scalar::Direction(Direction::In)=>Value::String("in".into()),
        Scalar::Direction(Direction::InOut)=>Value::String("inout".into()),
        Scalar::ProtectedVisibility=>Value::String("protected".into()),
    };
    (name,value)
}
fn apply_scalar_steps(transaction:&mut syson_model::Transaction,root:&str,steps:&[Step])->Result<(),Error> {
    for step in steps {
        match &step.effect {
            Effect::SetScalar { target,feature,value }=>{
                let id=match target {
                    Target::Root=>root,
                    Target::External(ExternalObject(id))=>id,
                    Target::Planned(_)=>return Err(gap("staging","planned target requires unaccepted factory integration")),
                };
                let (name,value)=scalar(*feature,value);
                transaction.set_attribute(id,name,value)?;
            }
            Effect::AllocateRaw { .. }=>return Err(gap("staging","raw allocation requires independently accepted factory")),
            Effect::Attach { .. }=>return Err(gap("staging","containment application unsupported")),
            Effect::SetReference { .. }=>return Err(gap("staging","reference/inverse application unsupported")),
        }
    }
    Ok(())
}
fn stage(editor:&mut Editor,root:&str,steps:&[Step])->Result<bool,Error> {
    let mut transaction=editor.begin();
    apply_scalar_steps(&mut transaction,root,steps)?;
    Ok(editor.commit(transaction)?)
}

/// Existing attached root only. Successful allocation-free plans use one native
/// Editor transaction/history entry. Errors discard that transaction. This is a
/// typed JSON scalar boundary, not Java setter/notification parity.
#[derive(Clone)]
pub struct PreparedPlan { base:Document, root:String, plan:Plan }
impl PreparedPlan {
    /// Binds an already produced declarative plan to its caller-supplied snapshot.
    /// This does not certify that the plan was generated from that snapshot.
    pub fn bind(document:&Document,root:&str,plan:Plan)->Result<Self,Error> {
        if Index::new(document)?.node(root)?.classifier!=plan.classifier { return Err(Error::ClassMismatch) }
        Ok(Self { base:document.clone(),root:root.into(),plan })
    }
}
pub fn apply(editor:&mut Editor,prepared:&PreparedPlan,cancelled:bool)->Result<bool,Error> {
    if cancelled { return Err(Error::Cancelled) }
    if editor.document()!=&prepared.base { return Err(Error::StalePlan) }
    stage(editor,&prepared.root,&prepared.plan.steps)
}
pub struct NativeContext { document:Document, root:String, pub cancelled:bool, pub reference_encoding:ReferenceEncoding }
impl NativeContext {
    pub fn new(document:&Document,root:&str)->Result<Self,Error> {
        Index::new(document)?.node(root)?;
        Ok(Self { document:document.clone(),root:root.into(),cancelled:false,reference_encoding:ReferenceEncoding::Disabled })
    }
    fn query<T>(&self,prefix:&[Step],f:impl FnOnce(&Index)->Result<T,Error>)->Resolution<T> {
        if self.cancelled { return Resolution::Unresolved(ContextGap {contract:"cancellation",reason:"cancelled".into()}) }
        let mut editor=Editor::new(self.document.clone());
        let result=stage(&mut editor,&self.root,prefix).and_then(|_|Index::new(editor.document())).and_then(|index|f(&index));
        match result {
            Ok(value)=>Resolution::Known(value),
            Err(Error::Gap(g))=>Resolution::Unsupported(g),
            Err(Error::Unresolved(g))=>Resolution::Unresolved(g),
            Err(other)=>Resolution::Unsupported(ContextGap {contract:"scalar prefix replay",reason:format!("{other:?}")}),
        }
    }
}
fn external(node:Option<&Node>)->Option<ExternalObject> { node.map(|n|ExternalObject(n.element_id.clone())) }
impl Context for NativeContext {
    fn cancelled(&mut self)->bool { self.cancelled }
    fn owner(&mut self,p:&[Step])->Resolution<Option<ExternalObject>> { self.query(p,|i|Ok(external(i.owner(&self.root)?))) }
    fn owning_membership(&mut self,p:&[Step])->Resolution<Option<ExternalObject>> { self.query(p,|i|Ok(external(i.owning_membership(&self.root)?))) }
    fn owning_namespace(&mut self,p:&[Step])->Resolution<Option<ExternalObject>> { self.query(p,|i|Ok(external(i.owning_namespace(&self.root)?))) }
    fn exact_class_owned_member_count(&mut self,n:&ExternalObject,c:C,p:&[Step])->Resolution<u64> { self.query(p,|i|i.exact_class_owned_member_count(&n.0,c)) }
    fn is_instance_of(&mut self,o:&ExternalObject,c:C,p:&[Step])->Resolution<bool> { self.query(p,|i|Ok(mm::is_subtype(i.node(&o.0)?.classifier,c))) }
    fn effective_name(&mut self,o:&ExternalObject,p:&[Step])->Resolution<Option<String>> { self.query(p,|i|i.effective_name_with_references(&o.0,NameKind::Name,self.reference_encoding)) }
    fn general_view_definition(&mut self,_:&[Step])->Resolution<Option<ExternalObject>> {
        Resolution::Unresolved(ContextGap {contract:"GeneralView lookup",reason:"no library/resource provider bound".into()})
    }
}

#[cfg(feature = "raw-factory-candidate")]
mod connection_definition;
#[cfg(feature = "raw-factory-candidate")]
pub use connection_definition::{initialize_connection_definition,ConnectionHelperReport,ConnectionHelperError,ConnectionHelperCause};

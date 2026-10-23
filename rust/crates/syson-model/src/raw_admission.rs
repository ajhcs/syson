// SPDX-License-Identifier: EPL-2.0
//! Explicit partial scalar projection; not a generic RawObject serializer.
use super::{ContainmentFeature, ModelError, NodeIds, Transaction};
use super::factory::{RawObject, ScalarValue, ScalarSnapshot, FactoryError, CachedFieldState};
use serde_json::{Value,json};
use std::collections::BTreeSet;
use syson_metamodel::ClassifierId;
#[derive(Debug)]
pub enum RawAdmissionError {
    Model(ModelError), Factory(FactoryError), ResourceAlreadyAssigned,
    IdentityAttributeSelection, DuplicateSelection(String), NonFiniteDouble(String),
}
#[derive(Debug)]
pub struct RawAdmissionReport {
    pub classifier:ClassifierId,
    pub semantic_id:String,
    pub resource_id:String,
    pub selected_scalars:Vec<(String,ScalarSnapshot)>,
    /// Native field/cache/flag/reference/list state is NOT serialized by this
    /// adapter, including slots whose getter values are explicitly selected.
    pub unrepresented_runtime_fields:Vec<CachedFieldState>,
    pub write_policy:&'static str,
}
/// Explicit partial projection for building detached initializer subtrees.
/// Returns the JSON fragment and its unrepresented native-state report together.
/// This is not a complete serializer and does not mutate the RawObject.
impl RawAdmissionReport {
pub fn project_selected(raw:&RawObject,resource_id:&str,attributes:&[&str])
 ->std::result::Result<(Value,RawAdmissionReport),RawAdmissionError> {
    if raw.resource_id().is_some() { return Err(RawAdmissionError::ResourceAlreadyAssigned) }
    NodeIds::new(resource_id,raw.semantic_id()).map_err(RawAdmissionError::Model)?;
    let mut seen=BTreeSet::new();
    let mut data=serde_json::Map::new();
    data.insert("elementId".into(),json!(raw.semantic_id()));
    let mut snapshots=Vec::new();
    for &attribute in attributes {
        if attribute=="elementId" { return Err(RawAdmissionError::IdentityAttributeSelection) }
        if !seen.insert(attribute) { return Err(RawAdmissionError::DuplicateSelection(attribute.into())) }
        let snapshot=raw.inspect_scalar(attribute).map_err(RawAdmissionError::Factory)?;
        let value=match &snapshot.value {
            ScalarValue::Null=>Value::Null,
            ScalarValue::String(v)=>Value::String(v.clone()),
            ScalarValue::Boolean(v)=>Value::Bool(*v),
            ScalarValue::Integer32(v)=>json!(*v),
            ScalarValue::Double(v)=>serde_json::Number::from_f64(*v).map(Value::Number)
                .ok_or_else(||RawAdmissionError::NonFiniteDouble(attribute.into()))?,
            ScalarValue::Enumeration {literal,..}=>Value::String(literal.clone()),
        };
        data.insert(attribute.into(),value);
        snapshots.push((attribute.into(),snapshot));
    }
    let report=RawAdmissionReport {
        classifier:raw.classifier(),semantic_id:raw.semantic_id().into(),resource_id:resource_id.into(),
        selected_scalars:snapshots,unrepresented_runtime_fields:raw.cached_fields().to_vec(),
        write_policy:"Explicit selected getter values; no source-storage inference or complete native-state projection",
    };
    let node=json!({"id":resource_id,"eClass":format!("sysml:{}",raw.classifier().metadata().name()),"data":data});
    Ok((node,report))
}
}


/// Consumes the owned transaction: any projection or attachment refusal drops
/// all pending changes. Resource identity and scalar selection remain explicit.
pub fn attach_raw_selected(
    mut transaction:Transaction,parent:&str,feature:ContainmentFeature,
    raw:&RawObject,resource_id:&str,attributes:&[&str],
)->std::result::Result<(Transaction,RawAdmissionReport),RawAdmissionError> {
    let (node,report)=RawAdmissionReport::project_selected(raw,resource_id,attributes)?;
    transaction.attach_preallocated_subtree(parent,feature,node).map_err(RawAdmissionError::Model)?;
    Ok((transaction,report))
}

/// Source-bound document effect for pinned caseConnectionDefinition only.
/// Java's actual superclass chain inherits TypeImpl.setIsSufficient unchanged.
/// This does NOT resolve EModelElement or relax generic attribute lookup.
/// Notifications, eIsSet and cached native state remain unrepresented.
impl Transaction {
pub fn initialize_connection_definition_sufficient(&mut self,element:&str)
 ->std::result::Result<(),ModelError> {
    self.apply(|doc| {
        use syson_metamodel::{self as registry,ClassifierId as C,FeatureKind,TypeReference};
        let node=doc.element(element).ok_or_else(||ModelError::MissingElement(element.into()))?;
        let class=super::scalar::node_classifier(node)?;
        if class!=C::ConnectionDefinition {
            return Err(ModelError::WrongType {element:element.into(),expected:"ConnectionDefinition"});
        }
        let found=registry::inherited_feature(class,"isSufficient");
        let feature=found.unique();
        let expected_external="platform:/plugin/org.eclipse.emf.ecore/model/Ecore.ecore#//EModelElement";
        let proven=feature.is_some_and(|f|
            f.owner==C::Type && f.name()=="isSufficient" && f.kind()==FeatureKind::Attribute
            && f.typed().lower_bound()==1 && f.typed().upper_bound()==1
            && matches!(f.typed().type_reference(),Some(TypeReference::External {
                raw:"ecore:EDataType http://www.eclipse.org/emf/2002/Ecore#//EBoolean"}))
            && f.changeable() && !f.derived() && !f.is_id()
            && !f.unsettable() && !f.volatile() && !f.transient()
            && f.explicit_default_literal()==Some("false"))
            && found.unresolved_supertypes.len()==1
            && found.unresolved_supertypes[0].raw()==expected_external;
        if !proven {
            return Err(ModelError::Attribute(super::AttributeError::UnresolvedAttributeSemantics {
                feature:"isSufficient".into(),
                external_supertypes:found.unresolved_supertypes.iter().map(|r|r.raw().into()).collect(),
            }));
        }
        let node=super::find_node_mut(doc.value["content"].as_array_mut().unwrap(),element).unwrap();
        node["data"]["isSufficient"]=Value::Bool(true);
        doc.validate()
    })
}
}

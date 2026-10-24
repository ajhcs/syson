// SPDX-License-Identifier: EPL-2.0
// Bounded reference effects derived from pinned Eclipse SysON Java/Ecore.
// Native source notices: Copyright (c) 2023, 2025 Obeo.
// Native caches, aliases, eIsSet, proxies and notifications are not JSON state.
use super::{Document, ModelError, Transaction};
use serde_json::Value;
use syson_metamodel::{self as registry, ClassifierId as C, FeatureKind, TypeReference};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDefinitionReference { ConjugatedType, OriginalPortDefinition }

fn wrong(element:&str,expected:&'static str)->ModelError {
    ModelError::WrongType {element:element.into(),expected}
}
fn node_class(node:&Value)->std::result::Result<C,ModelError> {
    super::scalar::node_classifier(node)
}
fn children<'a>(node:&'a Value,feature:&str)->impl Iterator<Item=&'a Value> {
    node["data"].get(feature).and_then(Value::as_array).into_iter().flatten()
}
fn semantic_id(node:&Value)->std::result::Result<&str,ModelError> {
    node["data"]["elementId"].as_str().ok_or_else(||ModelError::Shape("missing semantic identity in port topology".into()))
}
fn conjugated_owner(doc:&Document,root:&str,conjugation:&str)->std::result::Result<String,ModelError> {
    let root_node=doc.element(root).ok_or_else(||ModelError::MissingElement(root.into()))?;
    if node_class(root_node)?!=C::PortDefinition {return Err(wrong(root,"PortDefinition"))}
    for member in children(root_node,"ownedRelationship") {
        if node_class(member)?!=C::OwningMembership {continue}
        for port in children(member,"ownedRelatedElement") {
            if node_class(port)?!=C::ConjugatedPortDefinition {continue}
            for relation in children(port,"ownedRelationship") {
                if semantic_id(relation)?==conjugation {
                    if node_class(relation)?!=C::PortConjugation {return Err(wrong(conjugation,"PortConjugation"))}
                    return Ok(semantic_id(port)?.into())
                }
            }
        }
    }
    Err(ModelError::Shape("PortConjugation is outside the supplied PortDefinition initializer topology".into()))
}
impl Transaction {
    /// Two source-fixed LOCAL initializer bindings, not a generic reference API.
    /// Targets use semantic identifiers; resource identifiers remain separate.
    /// Any refusal poisons the owned transaction through the existing apply path.
    pub fn initialize_port_definition_reference(
        &mut self,root:&str,conjugation:&str,reference:PortDefinitionReference,target:&str,
    )->std::result::Result<(),ModelError> {
        self.apply(|doc| {
            let source=doc.element(conjugation).ok_or_else(||ModelError::MissingElement(conjugation.into()))?;
            if node_class(source)?!=C::PortConjugation {return Err(wrong(conjugation,"PortConjugation"))}
            let (name,owner,declared_type)=match reference {
                PortDefinitionReference::ConjugatedType=>("conjugatedType",C::Conjugation,C::Type),
                PortDefinitionReference::OriginalPortDefinition=>("originalPortDefinition",C::PortConjugation,C::PortDefinition),
            };
            let found=registry::inherited_feature(C::PortConjugation,name);
            // These lookups STOP at local declarations. The unresolved GLOBAL
            // EModelElement ancestor must not be inserted into either lookup.
            let proven=found.unresolved_supertypes.is_empty() && found.unique().is_some_and(|f|
                f.owner==owner && f.name()==name && f.kind()==FeatureKind::Reference
                && matches!(f.typed().type_reference(),Some(TypeReference::Local {id,..}) if id==declared_type)
                && f.typed().lower_bound()==1 && f.typed().upper_bound()==1
                && !f.typed().ordered() && f.typed().unique()
                && f.changeable() && !f.containment() && !f.derived()
                && !f.volatile() && !f.transient() && !f.unsettable()
                && f.explicit_default_literal().is_none()
                && f.node.attribute("eOpposite").is_none());
            if !proven {
                return Err(ModelError::Attribute(super::AttributeError::UnresolvedAttributeSemantics {
                    feature:name.into(),external_supertypes:found.unresolved_supertypes.iter().map(|x|x.raw().into()).collect(),
                }));
            }
            let port=conjugated_owner(doc,root,conjugation)?;
            let expected=match reference {PortDefinitionReference::ConjugatedType=>port.as_str(),PortDefinitionReference::OriginalPortDefinition=>root};
            let target_node=doc.element(target).ok_or_else(||ModelError::MissingElement(target.into()))?;
            let expected_class=match reference {PortDefinitionReference::ConjugatedType=>C::ConjugatedPortDefinition,PortDefinitionReference::OriginalPortDefinition=>C::PortDefinition};
            if node_class(target_node)?!=expected_class {return Err(wrong(target,match reference {
                PortDefinitionReference::ConjugatedType=>"ConjugatedPortDefinition",
                PortDefinitionReference::OriginalPortDefinition=>"PortDefinition",
            }))}
            if target!=expected {return Err(ModelError::Shape("reference target does not match the verified PortDefinition role".into()))}
            let source=super::find_node_mut(doc.value["content"].as_array_mut().unwrap(),conjugation).unwrap();
            source["data"][name]=Value::String(target.into());
            // Alias keys/derived getters/native caches/eIsSet are not synthesized.
            doc.validate()
        })
    }
}

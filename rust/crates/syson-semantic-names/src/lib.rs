// SPDX-License-Identifier: EPL-2.0
//! Selected immutable Element operations over explicit JSON resources.
//! No proxy loading, graph mutation, native caches or import evaluation.
#![forbid(unsafe_code)]
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use syson_domain::names::printable_name;
use syson_initializer_graph::native_query::{ObjectId, ResourceSet, ResourceSnapshot};
use syson_metamodel::{self as mm, ClassifierId as C};
use syson_model::Document;
use syson_references::{Registry, Resource};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Admission(String),
    MissingObject(ObjectId),
    StaleSnapshot,
    InvalidName,
    Reference(syson_references::Error),
    Cycle,
    Depth,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameKind {
    Name,
    ShortName,
}
/// Only its owning Snapshot can mint or consume a handle.
#[derive(Clone, Debug)]
pub struct Object {
    snapshot: Arc<()>,
    id: ObjectId,
}
impl Object {
    pub fn resource(&self) -> &str {
        &self.id.resource
    }
    pub fn element_id(&self) -> &str {
        &self.id.element
    }
}
pub struct Snapshot {
    token: Arc<()>,
    documents: BTreeMap<String, Document>,
    owners: ResourceSet,
    references: Registry,
}
impl Snapshot {
    /// Registry fragment policy is explicit; no semantic-ID fallback is added.
    pub fn new(resources: Vec<Resource>) -> Result<Self, Error> {
        let owners = ResourceSet::new(
            resources
                .iter()
                .map(|r| ResourceSnapshot::new(r.key.clone(), r.document.clone()))
                .collect(),
        )
        .map_err(|e| Error::Admission(format!("{e:?}")))?;
        let documents = resources
            .iter()
            .map(|r| (r.key.clone(), r.document.clone()))
            .collect();
        let references = Registry::new(resources).map_err(Error::Reference)?;
        Ok(Self {
            token: Arc::new(()),
            documents,
            owners,
            references,
        })
    }
    fn make(&self, id: ObjectId) -> Object {
        Object {
            snapshot: self.token.clone(),
            id,
        }
    }
    pub fn object(&self, resource: &str, element: &str) -> Result<Object, Error> {
        let object = self.make(ObjectId::new(resource, element));
        self.node(&object)?;
        Ok(object)
    }
    fn node(&self, object: &Object) -> Result<&Value, Error> {
        if !Arc::ptr_eq(&self.token, &object.snapshot) {
            return Err(Error::StaleSnapshot);
        }
        self.documents
            .get(&object.id.resource)
            .and_then(|d| d.element(&object.id.element))
            .ok_or_else(|| Error::MissingObject(object.id.clone()))
    }
    fn class(&self, object: &Object) -> Result<C, Error> {
        let node = self.node(object)?;
        node["eClass"]
            .as_str()
            .and_then(|s| s.strip_prefix("sysml:"))
            .and_then(|s| mm::lookup(mm::Package::Domain, s))
            .ok_or_else(|| Error::Admission("unknown class".into()))
    }
    /// Element.basicGetOwner, using the existing public raw-containment adapter.
    /// A relationship's eContainer is not silently substituted for getOwner.
    pub fn owner(&self, object: &Object) -> Result<Option<Object>, Error> {
        self.node(object)?;
        self.owners
            .semantic_owner(&object.id)
            .map(|id| id.map(|id| self.make(id)))
            .map_err(|e| Error::Admission(format!("{e:?}")))
    }
    /// Flatten each ownedRelationship's ownedRelatedElement in source order.
    pub fn owned_elements(&self, object: &Object) -> Result<Vec<Object>, Error> {
        let mut result = Vec::new();
        for relation in self.children(object, "ownedRelationship")? {
            result.extend(self.children(&relation, "ownedRelatedElement")?);
        }
        Ok(result)
    }
    fn children(&self, object: &Object, feature: &str) -> Result<Vec<Object>, Error> {
        let node = self.node(object)?;
        Ok(node["data"][feature]
            .as_array()
            .into_iter()
            .flatten()
            .map(|child| {
                self.make(ObjectId::new(
                    &object.id.resource,
                    child["data"]["elementId"]
                        .as_str()
                        .expect("admitted containment"),
                ))
            })
            .collect())
    }
    fn reference(
        &self,
        object: &Object,
        feature: &str,
        expected: C,
    ) -> Result<Option<Object>, Error> {
        let value = self.node(object)?["data"].get(feature);
        self.references
            .resolve(&object.id.resource, value, expected)
            .map(|target| target.map(|t| self.make(ObjectId::new(t.resource(), t.element_id()))))
            .map_err(Error::Reference)
    }
    fn declared(&self, object: &Object, field: &str) -> Result<Option<String>, Error> {
        match self.node(object)?["data"].get(field) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            _ => Err(Error::InvalidName),
        }
    }
    /// Feature inheritance follows only the first Redefinition and only when
    /// both declared names are absent. Conjugation affects the primary name.
    pub fn name(&self, object: &Object, kind: NameKind) -> Result<Option<String>, Error> {
        let mut current = object.clone();
        let mut seen = BTreeSet::new();
        let mut prefixes = String::new();
        loop {
            self.node(&current)?;
            if !seen.insert(current.id.clone()) {
                return Err(Error::Cycle);
            }
            if seen.len() > 128 {
                return Err(Error::Depth);
            }
            let class = self.class(&current)?;
            let children = self.children(&current, "ownedRelationship")?;
            let (relation_class, feature, expected) =
                if kind == NameKind::Name && mm::is_subtype(class, C::ConjugatedPortDefinition) {
                    prefixes.push('~');
                    (
                        C::PortConjugation,
                        "originalPortDefinition",
                        C::PortDefinition,
                    )
                } else {
                    let name = self.declared(&current, "declaredName")?;
                    let short = self.declared(&current, "declaredShortName")?;
                    if !mm::is_subtype(class, C::Feature) || name.is_some() || short.is_some() {
                        let result = if kind == NameKind::Name { name } else { short };
                        return Ok(result.map(|s| prefixes + &s));
                    }
                    (C::Redefinition, "redefinedFeature", C::Feature)
                };
            let mut relation = None;
            for child in children {
                if mm::is_subtype(self.class(&child)?, relation_class) {
                    relation = Some(child);
                    break;
                }
            }
            let Some(relation) = relation else {
                return Ok(None);
            };
            let Some(next) = self.reference(&relation, feature, expected)? else {
                return Ok(None);
            };
            if relation_class == C::Redefinition && next.id == current.id {
                return Ok(None);
            }
            current = next;
        }
    }
    /// Actual current Element.getQualifiedName policy, including printable
    /// names and the membership-owner skip. It does not consult imports.
    pub fn qualified_name(&self, object: &Object) -> Result<Option<String>, Error> {
        let mut current = object.clone();
        let mut pieces = Vec::new();
        let mut seen = BTreeSet::new();
        loop {
            self.node(&current)?;
            if !seen.insert(current.id.clone()) {
                return Err(Error::Cycle);
            }
            if seen.len() > 128 {
                return Err(Error::Depth);
            }
            let name = printable_name(self.name(&current, NameKind::Name)?.as_deref(), true);
            if name.is_empty() {
                break;
            }
            pieces.push(name);
            let Some(mut owner) = self.owner(&current)? else {
                break;
            };
            if mm::is_subtype(self.class(&owner)?, C::Membership) {
                let Some(parent) = self.owner(&owner)? else {
                    break;
                };
                owner = parent;
            }
            current = owner;
        }
        pieces.reverse();
        Ok((!pieces.is_empty()).then(|| pieces.join("::")))
    }
}

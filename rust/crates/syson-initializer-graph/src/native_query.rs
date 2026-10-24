// Copyright (c) 2024, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Source: pinned ElementUtil, ElementImpl, FeatureImpl, NamespaceImpl and
// ConjugatedPortDefinitionImpl. Rust snapshot/resource adapter: port contributors.
//! Ordered, immutable selected-JSON resources. No URI loading, proxies, EMF
//! caches/notifications, or automatic import evaluation. Results retain resource
//! identity; process-local query handles and resource-node IDs are never exposed
//! as semantic identities. See porting/domain/native-query/README.md.
use super::{Error, Index, NameKind, Node, ReferenceEncoding, gap};
use std::collections::{BTreeMap, BTreeSet};
use syson_domain::{
    model::{ElementId, Visibility},
    names::name_matches,
    query::{self, QueryGraph},
};
use syson_metamodel::{self as mm, ClassifierId as C};
use syson_model::Document;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId {
    pub resource: String,
    pub element: String,
}
impl ObjectId {
    pub fn new(resource: impl Into<String>, element: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            element: element.into(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct ResourceSnapshot {
    pub key: String,
    pub document: Document,
}
impl ResourceSnapshot {
    /// A caller-owned key, not a URI to fetch or an EMF resource UUID.
    pub fn new(key: impl Into<String>, document: Document) -> Self {
        Self {
            key: key.into(),
            document,
        }
    }
}
#[derive(Clone, Debug)]
pub enum ResourceContext {
    Detached,
    ResourceWithoutSet,
    Member(ObjectId),
}
/// An explicit observation of the derived getImportedMembership/getRelatedElement
/// results, in source order. This is not inferred from raw JSON or recomputed
/// import semantics, and does not establish native cache/reference parity.
#[derive(Clone, Debug)]
pub struct ImportedMembershipObservation {
    pub visibility: Visibility,
    pub related: Vec<ObjectId>,
}
pub struct ResourceSet {
    resources: Vec<ResourceSnapshot>,
    indexes: Vec<Index>,
    imported: BTreeMap<ObjectId, Vec<ImportedMembershipObservation>>,
}
fn unresolved(contract: &'static str, reason: impl Into<String>) -> Error {
    Error::Unresolved(syson_initializer::ContextGap {
        contract,
        reason: reason.into(),
    })
}
impl ResourceSet {
    /// Admission is limited to known SysML classes and the two selected native
    /// containment features. Unknown classes and malformed containment refuse
    /// the snapshot instead of silently disappearing from a search.
    pub fn new(resources: Vec<ResourceSnapshot>) -> Result<Self, Error> {
        let mut keys = BTreeSet::new();
        let mut indexes = Vec::new();
        for resource in &resources {
            if resource.key.is_empty() || !keys.insert(resource.key.clone()) {
                return Err(gap(
                    "ResourceSet identity",
                    "empty or duplicate resource key",
                ));
            }
            let index = Index::new(&resource.document)?;
            for node in index.nodes.values() {
                if !mm::is_subtype(node.classifier, C::Element) {
                    return Err(gap(
                        "ResourceSet admission",
                        "non-Element objects are not represented",
                    ));
                }
                for (feature, required) in [
                    ("ownedRelationship", C::Relationship),
                    ("ownedRelatedElement", C::Element),
                ] {
                    if feature == "ownedRelatedElement"
                        && node.data.get(feature).is_some()
                        && !mm::is_subtype(node.classifier, C::Relationship)
                    {
                        return Err(gap(
                            "raw containment",
                            "ownedRelatedElement owner is not a Relationship",
                        ));
                    }
                    for id in child_ids(node, feature) {
                        if !mm::is_subtype(index.node(id)?.classifier, required) {
                            return Err(gap(
                                "raw containment",
                                "child class does not match its containment feature",
                            ));
                        }
                    }
                }
            }
            indexes.push(index);
        }
        Ok(Self {
            resources,
            indexes,
            imported: BTreeMap::new(),
        })
    }
    pub fn with_imported_memberships(
        mut self,
        namespace: ObjectId,
        memberships: Vec<ImportedMembershipObservation>,
    ) -> Result<Self, Error> {
        if !mm::is_subtype(self.node(&namespace)?.classifier, C::Namespace) {
            return Err(gap("import observation", "object is not a Namespace"));
        }
        self.imported.insert(namespace, memberships);
        Ok(self)
    }
    fn resource_at(&self, key: &str) -> Result<usize, Error> {
        self.resources
            .iter()
            .position(|r| r.key == key)
            .ok_or_else(|| {
                unresolved(
                    "ResourceSet context",
                    format!("resource not in snapshot: {key}"),
                )
            })
    }
    fn node(&self, object: &ObjectId) -> Result<&Node, Error> {
        let at = self.resource_at(&object.resource)?;
        self.indexes[at].nodes.get(&object.element).ok_or_else(|| {
            unresolved(
                "resource-scoped object",
                format!(
                    "element not in resource {}: {}",
                    object.resource, object.element
                ),
            )
        })
    }
    pub fn semantic_owner(&self, object: &ObjectId) -> Result<Option<ObjectId>, Error> {
        let at = self.resource_at(&object.resource)?;
        self.node(object)?;
        Ok(self.indexes[at]
            .owner(&object.element)?
            .map(|node| ObjectId::new(&object.resource, &node.element_id)))
    }
    pub fn effective_name(
        &self,
        object: &ObjectId,
        kind: NameKind,
        encoding: ReferenceEncoding,
    ) -> Result<Option<String>, Error> {
        self.name_inner(object, kind, encoding, &mut BTreeSet::new())
    }
    fn name_inner(
        &self,
        object: &ObjectId,
        kind: NameKind,
        encoding: ReferenceEncoding,
        seen: &mut BTreeSet<ObjectId>,
    ) -> Result<Option<String>, Error> {
        if !seen.insert(object.clone()) {
            return Err(gap(
                "effective name",
                "local name cycle; native recursive getter not executed",
            ));
        }
        let node = self.node(object)?;
        if kind == NameKind::Name && mm::is_subtype(node.classifier, C::ConjugatedPortDefinition) {
            let at = self.resource_at(&object.resource)?;
            for id in child_ids(node, "ownedRelationship") {
                let relation = self.indexes[at].node(id)?;
                if mm::is_subtype(relation.classifier, C::PortConjugation) {
                    let Some(target) = self.local_reference(
                        object,
                        relation,
                        "originalPortDefinition",
                        C::PortDefinition,
                        encoding,
                    )?
                    else {
                        return Ok(None);
                    };
                    return self
                        .name_inner(&target, NameKind::Name, encoding, seen)
                        .map(|name| name.map(|name| format!("~{name}")));
                }
            }
            return Ok(None);
        }
        if mm::is_subtype(node.classifier, C::Feature) {
            return self.indexes[self.resource_at(&object.resource)?]
                .effective_name_with_references(&object.element, kind, encoding);
        }
        let feature = if kind == NameKind::Name {
            "declaredName"
        } else {
            "declaredShortName"
        };
        match node.data.get(feature) {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(serde_json::Value::String(name)) => Ok(Some(name.clone())),
            _ => Err(gap("effective name", "declared name is not a string")),
        }
    }
    fn local_reference(
        &self,
        owner: &ObjectId,
        relation: &Node,
        feature: &'static str,
        expected: C,
        encoding: ReferenceEncoding,
    ) -> Result<Option<ObjectId>, Error> {
        let value = relation.data.get(feature);
        if matches!(value, None | Some(serde_json::Value::Null)) {
            return Ok(None);
        }
        if encoding != ReferenceEncoding::SemanticElementIds {
            return Err(gap(
                feature,
                "local semantic-ID reference encoding was not selected",
            ));
        }
        let Some(serde_json::Value::String(element)) = value else {
            return Err(gap(
                feature,
                "proxy/object reference wire shape is not admitted",
            ));
        };
        let target = ObjectId::new(&owner.resource, element);
        if !mm::is_subtype(self.node(&target)?.classifier, expected) {
            return Err(gap(feature, "reference target has the wrong class"));
        }
        Ok(Some(target))
    }
    /// Mirrors getAllRootsInResourceSet: an explicit detached/no-set context
    /// yields no roots. A missing claimed member is an unresolved adapter input.
    pub fn find_from(
        &self,
        context: &ResourceContext,
        name: &str,
        expected: C,
        encoding: ReferenceEncoding,
    ) -> Result<Option<ObjectId>, Error> {
        match context {
            ResourceContext::Detached | ResourceContext::ResourceWithoutSet => Ok(None),
            ResourceContext::Member(object) => {
                self.node(object)?;
                let graph = NativeGraph::new(self, encoding)?;
                graph.find(&graph.roots, name, expected)
            }
        }
    }
    /// Explicit root collection overload; input order is preserved verbatim.
    pub fn find_in_roots(
        &self,
        roots: &[ObjectId],
        name: &str,
        expected: C,
        encoding: ReferenceEncoding,
    ) -> Result<Option<ObjectId>, Error> {
        let graph = NativeGraph::new(self, encoding)?;
        let roots = roots
            .iter()
            .map(|id| graph.handle(id))
            .collect::<Result<Vec<_>, _>>()?;
        graph.find(&roots, name, expected)
    }
    pub fn find_general_view(
        &self,
        context: &ResourceContext,
        encoding: ReferenceEncoding,
    ) -> Result<Option<ObjectId>, Error> {
        self.find_from(
            context,
            "StandardViewDefinitions::GeneralView",
            C::ViewDefinition,
            encoding,
        )
    }
}
fn child_ids<'a>(node: &'a Node, feature: &str) -> impl Iterator<Item = &'a str> {
    node.data
        .get(feature)
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|child| child["data"]["elementId"].as_str())
}
struct NativeGraph<'a> {
    set: &'a ResourceSet,
    objects: Vec<ObjectId>,
    handles: BTreeMap<ObjectId, ElementId>,
    roots: Vec<ElementId>,
    encoding: ReferenceEncoding,
}
impl<'a> NativeGraph<'a> {
    fn new(set: &'a ResourceSet, encoding: ReferenceEncoding) -> Result<Self, Error> {
        let mut graph = Self {
            set,
            objects: Vec::new(),
            handles: BTreeMap::new(),
            roots: Vec::new(),
            encoding,
        };
        for resource in &set.resources {
            for root in resource.document.value()["content"].as_array().unwrap() {
                let root_id = graph.visit(&resource.key, root)?;
                graph.roots.push(root_id);
            }
        }
        Ok(graph)
    }
    fn visit(&mut self, resource: &str, value: &serde_json::Value) -> Result<ElementId, Error> {
        let object = ObjectId::new(resource, value["data"]["elementId"].as_str().unwrap());
        let id = ElementId(self.objects.len());
        if self.handles.insert(object.clone(), id).is_some() {
            return Err(gap("query graph", "duplicate resource-scoped identity"));
        }
        self.objects.push(object);
        for feature in ["ownedRelationship", "ownedRelatedElement"] {
            if let Some(children) = value["data"][feature].as_array() {
                for child in children {
                    self.visit(resource, child)?;
                }
            }
        }
        Ok(id)
    }
    fn object(&self, id: ElementId) -> Result<&ObjectId, Error> {
        self.objects
            .get(id.0)
            .ok_or_else(|| gap("query handle", "handle outside this immutable snapshot"))
    }
    fn handle(&self, object: &ObjectId) -> Result<ElementId, Error> {
        self.handles.get(object).copied().ok_or_else(|| {
            unresolved(
                "resource-scoped object",
                format!("object not in query graph: {object:?}"),
            )
        })
    }
    fn node(&self, id: ElementId) -> Result<&Node, Error> {
        self.set.node(self.object(id)?)
    }
    fn children(&self, id: ElementId, feature: &str) -> Result<Vec<ElementId>, Error> {
        let object = self.object(id)?;
        child_ids(self.node(id)?, feature)
            .map(|child| self.handle(&ObjectId::new(&object.resource, child)))
            .collect()
    }
    fn find(
        &self,
        roots: &[ElementId],
        name: &str,
        expected: C,
    ) -> Result<Option<ObjectId>, Error> {
        if expected.metadata().kind != mm::ClassifierKind::Class
            || !mm::is_subtype(expected, C::Element)
        {
            return Err(gap(
                "query type",
                "expected type is not a SysML Element class",
            ));
        }
        query::find_by_name_and_type(self, roots, name, |id| {
            Ok(mm::is_subtype(self.node(id)?.classifier, expected))
        })?
        .map(|id| self.object(id).cloned())
        .transpose()
    }
    fn import_observations(
        &self,
        id: ElementId,
    ) -> Result<&[ImportedMembershipObservation], Error> {
        let object = self.object(id)?;
        if let Some(observations) = self.set.imported.get(object) {
            return Ok(observations);
        }
        for relation in self.children(id, "ownedRelationship")? {
            if mm::is_subtype(self.node(relation)?.classifier, C::Import) {
                return Err(unresolved(
                    "importedMembership",
                    "native import evaluation is not bound; supply explicit ordered derived observations",
                ));
            }
        }
        Ok(&[])
    }
}
impl QueryGraph for NativeGraph<'_> {
    type Error = Error;
    fn raw_children(&self, id: ElementId) -> Result<Vec<ElementId>, Error> {
        let node = self.node(id)?;
        if node
            .data
            .get("eAnnotations")
            .is_some_and(|value| !value.is_null() && value.as_array().is_none_or(|a| !a.is_empty()))
        {
            return Err(gap(
                "raw containment",
                "external EAnnotation containment is not represented",
            ));
        }
        let mut children = self.children(id, "ownedRelationship")?;
        children.extend(self.children(id, "ownedRelatedElement")?);
        Ok(children)
    }
    fn owned_elements(&self, id: ElementId) -> Result<Vec<ElementId>, Error> {
        let mut elements = Vec::new();
        for relation in self.children(id, "ownedRelationship")? {
            elements.extend(self.children(relation, "ownedRelatedElement")?);
        }
        Ok(elements)
    }
    fn is_namespace(&self, id: ElementId) -> Result<bool, Error> {
        Ok(mm::is_subtype(self.node(id)?.classifier, C::Namespace))
    }
    fn has_raw_container(&self, id: ElementId) -> Result<bool, Error> {
        Ok(self.node(id)?.container.is_some())
    }
    fn is_unnamed(&self, id: ElementId) -> Result<bool, Error> {
        Ok(self
            .set
            .effective_name(self.object(id)?, NameKind::Name, self.encoding)?
            .is_none())
    }
    fn name_matches(&self, id: ElementId, name: &str) -> Result<bool, Error> {
        let object = self.object(id)?;
        if name_matches(
            self.set
                .effective_name(object, NameKind::Name, self.encoding)?
                .as_deref(),
            Some(name),
        ) {
            return Ok(true);
        }
        Ok(name_matches(
            self.set
                .effective_name(object, NameKind::ShortName, self.encoding)?
                .as_deref(),
            Some(name),
        ))
    }
    fn public_imported_elements(&self, id: ElementId) -> Result<Vec<ElementId>, Error> {
        self.import_observations(id)?
            .iter()
            .filter(|m| m.visibility == Visibility::Public)
            .flat_map(|m| m.related.iter())
            .map(|object| self.handle(object))
            .collect()
    }
    fn find_public_imported_match(
        &self,
        id: ElementId,
        name: &str,
    ) -> Result<Option<ElementId>, Error> {
        for membership in self.import_observations(id)? {
            if membership.visibility != Visibility::Public {
                continue;
            }
            for object in &membership.related {
                let child = self.handle(object)?;
                if self.name_matches(child, name)? {
                    return Ok(Some(child));
                }
            }
        }
        Ok(None)
    }
}

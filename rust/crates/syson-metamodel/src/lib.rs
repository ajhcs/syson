// Copyright (c) 2024, 2026 Obeo.
// This program and accompanying materials are made available under the terms
// of the Eclipse Public License v2.0, https://www.eclipse.org/legal/epl-2.0/.
// SPDX-License-Identifier: EPL-2.0
// Contributors: Obeo - initial metamodel
// Rust schema metadata adapter: SysON Rust port contributors.

//! Complete pinned Ecore metadata registry, not generated model behavior.
//! External supertypes and data types retain full identities and stay unresolved.
//! Derived values, annotations and operations are metadata only, not execution.
#![forbid(unsafe_code)]
use std::collections::BTreeSet;
mod generated;
pub use generated::{CUSTOM_NODE_CLASSIFIERS, ClassifierId, DOMAIN_CLASSIFIERS, SOURCES};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Package {
    Domain,
    CustomNodes,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassifierKind {
    Class,
    Enum,
}
#[derive(Debug)]
pub struct Source {
    pub package: Package,
    pub path: &'static str,
    pub revision: &'static str,
    pub sha256: &'static str,
    pub attributes: &'static [(&'static str, &'static str)],
}
/// Every attribute and child is preserved, including namespaced XML attributes.
#[derive(Debug, PartialEq, Eq)]
pub struct MetadataNode {
    pub tag: &'static str,
    pub attributes: &'static [(&'static str, &'static str)],
    pub children: &'static [MetadataNode],
}
impl MetadataNode {
    pub fn attribute(&self, name: &str) -> Option<&'static str> {
        self.attributes
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| *value)
    }
    pub fn name(&self) -> Option<&'static str> {
        self.attribute("name")
    }
    fn flag(&self, name: &str, default: bool) -> bool {
        self.attribute(name).map_or(default, |s| s == "true")
    }
    fn number(&self, name: &str, default: i32) -> i32 {
        self.attribute(name)
            .map_or(default, |s| s.parse().expect("pinned Ecore integer"))
    }
    pub fn annotations(&self) -> impl Iterator<Item = &MetadataNode> {
        self.children.iter().filter(|n| n.tag == "eAnnotations")
    }
}
#[derive(Debug)]
pub struct Classifier {
    pub id: ClassifierId,
    pub package: Package,
    pub kind: ClassifierKind,
    pub node: MetadataNode,
}
impl Classifier {
    pub fn name(&self) -> &'static str {
        self.node.name().unwrap()
    }
    pub fn is_abstract(&self) -> bool {
        self.node.flag("abstract", false)
    }
    pub fn is_interface(&self) -> bool {
        self.node.flag("interface", false)
    }
    pub fn supertypes(&self) -> impl Iterator<Item = TypeReference> {
        self.node
            .attribute("eSuperTypes")
            .unwrap_or("")
            .split_whitespace()
            .map(|raw| resolve_type(self.package, raw))
    }
    pub fn features(&'static self) -> impl Iterator<Item = Feature> {
        self.node
            .children
            .iter()
            .filter(|n| n.tag == "eStructuralFeatures")
            .map(|node| Feature {
                owner: self.id,
                node,
            })
    }
    pub fn operations(&'static self) -> impl Iterator<Item = Operation> {
        self.node
            .children
            .iter()
            .filter(|n| n.tag == "eOperations")
            .map(|node| Operation {
                owner: self.id,
                node,
            })
    }
    pub fn enum_literals(&self) -> impl Iterator<Item = EnumLiteral> {
        self.node
            .children
            .iter()
            .filter(|n| n.tag == "eLiterals")
            .map(|node| EnumLiteral {
                name: node.name().unwrap(),
                value: node.number("value", 0),
                literal: node.attribute("literal").unwrap_or(node.name().unwrap()),
            })
    }
}
impl ClassifierId {
    pub fn metadata(self) -> &'static Classifier {
        all()
            .find(|c| c.id == self)
            .expect("generated ClassifierId")
    }
}
pub fn all() -> impl Iterator<Item = &'static Classifier> {
    DOMAIN_CLASSIFIERS.iter().chain(CUSTOM_NODE_CLASSIFIERS)
}
pub fn lookup(package: Package, name: &str) -> Option<ClassifierId> {
    all()
        .find(|c| c.package == package && c.name() == name)
        .map(|c| c.id)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeReference {
    Local { id: ClassifierId, raw: &'static str },
    External { raw: &'static str },
    UnresolvedLocal { raw: &'static str },
}
impl TypeReference {
    pub fn raw(self) -> &'static str {
        match self {
            Self::Local { raw, .. } | Self::External { raw } | Self::UnresolvedLocal { raw } => raw,
        }
    }
    pub fn local(self) -> Option<ClassifierId> {
        if let Self::Local { id, .. } = self {
            Some(id)
        } else {
            None
        }
    }
}
fn resolve_type(package: Package, raw: &'static str) -> TypeReference {
    if let Some(name) = raw.strip_prefix("#//") {
        lookup(package, name).map_or(TypeReference::UnresolvedLocal { raw }, |id| {
            TypeReference::Local { id, raw }
        })
    } else {
        TypeReference::External { raw }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureKind {
    Attribute,
    Reference,
}
/// ETypedElement metadata. Omitted XML flags retain Ecore schema defaults.
#[derive(Debug, Clone, Copy)]
pub struct TypedElement {
    pub package: Package,
    pub node: &'static MetadataNode,
}
impl TypedElement {
    pub fn name(self) -> &'static str {
        self.node.name().unwrap()
    }
    pub fn type_reference(self) -> Option<TypeReference> {
        self.node
            .attribute("eType")
            .map(|raw| resolve_type(self.package, raw))
    }
    pub fn lower_bound(self) -> i32 {
        self.node.number("lowerBound", 0)
    }
    /// -1 is unbounded; no conversion to an arbitrary finite limit.
    pub fn upper_bound(self) -> i32 {
        self.node.number("upperBound", 1)
    }
    pub fn ordered(self) -> bool {
        self.node.flag("ordered", true)
    }
    pub fn unique(self) -> bool {
        self.node.flag("unique", true)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Feature {
    pub owner: ClassifierId,
    pub node: &'static MetadataNode,
}
impl Feature {
    pub fn typed(self) -> TypedElement {
        TypedElement {
            package: self.owner.metadata().package,
            node: self.node,
        }
    }
    pub fn name(self) -> &'static str {
        self.typed().name()
    }
    pub fn kind(self) -> FeatureKind {
        if self
            .node
            .attribute("{http://www.w3.org/2001/XMLSchema-instance}type")
            == Some("ecore:EReference")
        {
            FeatureKind::Reference
        } else {
            FeatureKind::Attribute
        }
    }
    pub fn containment(self) -> bool {
        self.node.flag("containment", false)
    }
    pub fn resolve_proxies(self) -> bool {
        self.node.flag("resolveProxies", true)
    }
    pub fn derived(self) -> bool {
        self.node.flag("derived", false)
    }
    pub fn volatile(self) -> bool {
        self.node.flag("volatile", false)
    }
    pub fn transient(self) -> bool {
        self.node.flag("transient", false)
    }
    pub fn changeable(self) -> bool {
        self.node.flag("changeable", true)
    }
    pub fn unsettable(self) -> bool {
        self.node.flag("unsettable", false)
    }
    pub fn is_id(self) -> bool {
        self.node.flag("iD", false)
    }
    pub fn opposite(self) -> Option<&'static str> {
        self.node.attribute("eOpposite")
    }
    /// Literal from Ecore only. This does not execute Java initializers, derive
    /// values, choose enum defaults or synthesize external data-type defaults.
    pub fn explicit_default_literal(self) -> Option<&'static str> {
        self.node.attribute("defaultValueLiteral")
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Operation {
    pub owner: ClassifierId,
    pub node: &'static MetadataNode,
}
impl Operation {
    pub fn typed(self) -> TypedElement {
        TypedElement {
            package: self.owner.metadata().package,
            node: self.node,
        }
    }
    pub fn name(self) -> &'static str {
        self.typed().name()
    }
    pub fn parameters(self) -> impl Iterator<Item = TypedElement> {
        self.node
            .children
            .iter()
            .filter(|n| n.tag == "eParameters")
            .map(move |node| TypedElement {
                package: self.owner.metadata().package,
                node,
            })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumLiteral {
    pub name: &'static str,
    pub value: i32,
    pub literal: &'static str,
}

/// True only for a reflexive/local transitive EClass proof. False does not
/// resolve external ancestors: inspect unresolved_supertypes for that boundary.
pub fn is_subtype(child: ClassifierId, parent: ClassifierId) -> bool {
    if child.metadata().kind != ClassifierKind::Class
        || parent.metadata().kind != ClassifierKind::Class
    {
        return false;
    }
    let mut stack = vec![child];
    let mut visited = BTreeSet::new();
    while let Some(id) = stack.pop() {
        if id == parent {
            return true;
        }
        if visited.insert(id) {
            stack.extend(id.metadata().supertypes().filter_map(TypeReference::local));
        }
    }
    false
}
pub fn unresolved_supertypes(class: ClassifierId) -> Vec<TypeReference> {
    let mut stack = vec![class];
    let mut visited = BTreeSet::new();
    let mut unresolved = Vec::new();
    while let Some(id) = stack.pop() {
        if !visited.insert(id) {
            continue;
        }
        for supertype in id.metadata().supertypes() {
            if let Some(local) = supertype.local() {
                stack.push(local)
            } else if !unresolved.contains(&supertype) {
                unresolved.push(supertype)
            }
        }
    }
    unresolved
}
/// Structural feature-name lookup, not evaluation of redefines/subsets.
/// Keeps independent inherited candidates and unresolved external ancestors.
#[derive(Debug)]
pub struct FeatureLookup {
    pub candidates: Vec<Feature>,
    pub unresolved_supertypes: Vec<TypeReference>,
}
impl FeatureLookup {
    /// Returns one local candidate, not proof of uniqueness across unresolved
    /// external ancestor branches. Inspect unresolved_supertypes separately.
    pub fn unique(&self) -> Option<Feature> {
        if self.candidates.len() == 1 {
            Some(self.candidates[0])
        } else {
            None
        }
    }
}
pub fn inherited_feature(class: ClassifierId, name: &str) -> FeatureLookup {
    let mut result = FeatureLookup {
        candidates: Vec::new(),
        unresolved_supertypes: Vec::new(),
    };
    fn visit(
        id: ClassifierId,
        name: &str,
        seen: &mut BTreeSet<ClassifierId>,
        result: &mut FeatureLookup,
    ) {
        if !seen.insert(id) {
            return;
        }
        let own: Vec<_> = id
            .metadata()
            .features()
            .filter(|f| f.name() == name)
            .collect();
        if !own.is_empty() {
            result.candidates.extend(own);
            return;
        }
        for supertype in id.metadata().supertypes() {
            if let Some(local) = supertype.local() {
                visit(local, name, seen, result)
            } else if !result.unresolved_supertypes.contains(&supertype) {
                result.unresolved_supertypes.push(supertype)
            }
        }
    }
    visit(class, name, &mut BTreeSet::new(), &mut result);
    result
}

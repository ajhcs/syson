// SPDX-License-Identifier: EPL-2.0
//! Explicit bridge from the selected model's local semantic-ID typing to native
//! node-UUID wires. This is not encoding detection or a full EMF serializer.
use std::collections::BTreeMap;

use serde_json::Value;
use syson_model::{Document, NodeIds};
use syson_references::FragmentIdentity;

use crate::{Bundle, Input, MAX_NODES, MAX_RESOURCE_BYTES};

/// The caller asserts the source representation. Native-wire documents continue
/// to use Bundle::load; references are never tried in a second identity domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceEncoding {
    LocalSemanticElementIds,
}
#[derive(Debug)]
pub enum Error {
    Limit,
    Model(String),
    UnsupportedClass(String),
    Shape {
        element: String,
        problem: &'static str,
    },
    Reference {
        element: String,
        feature: &'static str,
        problem: &'static str,
    },
    Native(crate::Error),
}
struct Target {
    node: String,
    class: String,
}
fn id(node: &Value) -> &str {
    node["data"]["elementId"].as_str().unwrap()
}
fn shape(node: &Value, problem: &'static str) -> Error {
    Error::Shape {
        element: id(node).into(),
        problem,
    }
}
fn reference(node: &Value, feature: &'static str, problem: &'static str) -> Error {
    Error::Reference {
        element: id(node).into(),
        feature,
        problem,
    }
}
fn class(node: &Value) -> Result<&str, Error> {
    let class = node["eClass"].as_str().unwrap();
    match class {
        "sysml:Package"
        | "sysml:PartDefinition"
        | "sysml:PartUsage"
        | "sysml:OwningMembership"
        | "sysml:FeatureMembership"
        | "sysml:FeatureTyping" => Ok(class),
        _ => Err(Error::UnsupportedClass(class.into())),
    }
}
fn children<'a>(node: &'a Value, feature: &str) -> Result<&'a [Value], Error> {
    match node["data"].get(feature) {
        None => Ok(&[]),
        Some(Value::Array(rows)) => Ok(rows),
        _ => Err(shape(node, "containment must be an array")),
    }
}
fn index(node: &Value, depth: usize, targets: &mut BTreeMap<String, Target>) -> Result<(), Error> {
    if depth > 64 || targets.len() >= MAX_NODES {
        return Err(Error::Limit);
    }
    let class = class(node)?;
    let node_id = node["id"].as_str().unwrap();
    NodeIds::new(node_id, id(node)).map_err(|e| Error::Model(e.to_string()))?;
    targets.insert(
        id(node).into(),
        Target {
            node: node_id.into(),
            class: class.into(),
        },
    );
    for feature in ["ownedRelationship", "ownedRelatedElement"] {
        for child in children(node, feature)? {
            index(child, depth + 1, targets)?;
        }
    }
    Ok(())
}
fn semantic_id<'a>(node: &'a Value, feature: &'static str) -> Result<&'a str, Error> {
    node["data"][feature]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| reference(node, feature, "a local semantic-ID string is required"))
}
fn leaf(node: &Value) -> Result<(), Error> {
    if !children(node, "ownedRelationship")?.is_empty()
        || !children(node, "ownedRelatedElement")?.is_empty()
    {
        return Err(shape(node, "FeatureTyping must be a leaf"));
    }
    Ok(())
}
fn convert(node: &mut Value, targets: &BTreeMap<String, Target>) -> Result<(), Error> {
    let kind = class(node)?.to_owned();
    if !matches!(
        kind.as_str(),
        "sysml:Package" | "sysml:PartDefinition" | "sysml:PartUsage"
    ) || !children(node, "ownedRelatedElement")?.is_empty()
    {
        return Err(shape(
            node,
            "declarations must own membership relationships",
        ));
    }
    let owner = id(node).to_owned();
    let mut typings = 0;
    if let Some(rows) = node["data"].get_mut("ownedRelationship") {
        // The first pass validated every known containment array and node ID.
        for relation in rows.as_array_mut().unwrap() {
            match class(relation)? {
                "sysml:FeatureTyping" if kind == "sysml:PartUsage" => {
                    typings += 1;
                    leaf(relation)?;
                    if semantic_id(relation, "typedFeature")? != owner {
                        return Err(reference(
                            relation,
                            "typedFeature",
                            "the semantic ID must identify the containing PartUsage",
                        ));
                    }
                    let semantic = semantic_id(relation, "type")?;
                    let target = targets.get(semantic).ok_or_else(|| {
                        reference(relation, "type", "the local semantic ID is unresolved")
                    })?;
                    if target.class != "sysml:PartDefinition" {
                        return Err(reference(
                            relation,
                            "type",
                            "the semantic target must be a PartDefinition",
                        ));
                    }
                    relation["data"]["type"] = Value::String(target.node.clone());
                    relation["data"]["typedFeature"] = Value::String(targets[&owner].node.clone());
                }
                "sysml:OwningMembership" | "sysml:FeatureMembership" => {
                    if !children(relation, "ownedRelationship")?.is_empty()
                        || children(relation, "ownedRelatedElement")?.len() != 1
                    {
                        return Err(shape(relation, "one declaration per owning membership"));
                    }
                    let child = &relation["data"]["ownedRelatedElement"][0];
                    let expected =
                        if kind != "sysml:Package" && child["eClass"] == "sysml:PartUsage" {
                            "sysml:FeatureMembership"
                        } else {
                            "sysml:OwningMembership"
                        };
                    if relation["eClass"] != expected {
                        return Err(shape(
                            relation,
                            "membership kind does not match containment",
                        ));
                    }
                    convert(&mut relation["data"]["ownedRelatedElement"][0], targets)?;
                }
                _ => {
                    return Err(shape(
                        relation,
                        "only owning memberships or direct PartUsage typing are supported",
                    ));
                }
            }
        }
    }
    if kind == "sysml:PartUsage" && typings != 1 {
        return Err(shape(
            node,
            "one FeatureTyping per selected PartUsage is required",
        ));
    }
    Ok(())
}
/// Serialized boundaries use Document::from_json through the literal-safe model
/// authority; no generic Value pre-parse or identity-domain inference is used.
/// Convert one caller-selected resource atomically, then run normal Bundle::load.
/// Only FeatureTyping.type/typedFeature change. All node/semantic IDs, envelope
/// fields and opaque unknown values remain intact. Input is immutable on failure
/// or success. Only exact local semantic-ID lookup is performed; no library,
/// URI loading, reference fallback or native encoding inference is available.
///
/// Admits the six F14 package/part classes and their selected owning shapes;
/// this is a reference-representation bridge, not a SysML validity verdict.
pub fn load(key: &str, document: &Document, encoding: SourceEncoding) -> Result<Bundle, Error> {
    match encoding {
        SourceEncoding::LocalSemanticElementIds => (),
    }
    if document
        .to_json()
        .map_err(|e| Error::Model(e.to_string()))?
        .len()
        > MAX_RESOURCE_BYTES
    {
        return Err(Error::Limit);
    }
    let mut targets = BTreeMap::new();
    for node in document.value()["content"].as_array().unwrap() {
        if node["eClass"] != "sysml:Package" {
            return Err(shape(node, "root packages are required"));
        }
        index(node, 0, &mut targets)?;
    }
    let mut converted = document.value().clone();
    for node in converted["content"].as_array_mut().unwrap() {
        convert(node, &targets)?;
    }
    Bundle::load(vec![Input {
        key: key.into(),
        json: serde_json::to_string(&converted).map_err(|e| Error::Model(e.to_string()))?,
        fragments: FragmentIdentity::JsonNodeId,
    }])
    .map_err(Error::Native)
}

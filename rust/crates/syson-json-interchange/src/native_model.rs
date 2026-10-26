// SPDX-License-Identifier: EPL-2.0
//! Explicit inverse of the finite selected-model bridge; no encoding inference.
use std::collections::BTreeMap;

use serde_json::Value;
use syson_model::Document;
use syson_references::{Decoded, FragmentIdentity, decode};

use crate::{Bundle, Input, MAX_NODES, MAX_RESOURCE_BYTES, selected_model};

/// The caller asserts native local JSON node-ID wires. Semantic-ID wires are
/// never tried as a fallback, even when they happen to identify another node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceEncoding {
    LocalJsonNodeIds,
}
#[derive(Debug)]
pub enum Error {
    Limit,
    Model(String),
    UnsupportedClass(String),
    Reference {
        element: String,
        feature: &'static str,
        problem: &'static str,
    },
    Native(Box<crate::Error>),
    Selected(Box<selected_model::Error>),
}
struct Target {
    semantic: String,
    class: String,
}
fn reference(node: &Value, feature: &'static str, problem: &'static str) -> Error {
    Error::Reference {
        element: node["data"]["elementId"].as_str().unwrap().into(),
        feature,
        problem,
    }
}
fn index(node: &Value, depth: usize, targets: &mut BTreeMap<String, Target>) -> Result<(), Error> {
    if depth > 64 || targets.len() >= MAX_NODES {
        return Err(Error::Limit);
    }
    let class = node["eClass"].as_str().unwrap();
    if !matches!(
        class,
        "sysml:Package"
            | "sysml:PartDefinition"
            | "sysml:PartUsage"
            | "sysml:OwningMembership"
            | "sysml:FeatureMembership"
            | "sysml:FeatureTyping"
    ) {
        return Err(Error::UnsupportedClass(class.into()));
    }
    targets.insert(
        node["id"].as_str().unwrap().into(),
        Target {
            semantic: node["data"]["elementId"].as_str().unwrap().into(),
            class: class.into(),
        },
    );
    for feature in ["ownedRelationship", "ownedRelatedElement"] {
        if let Some(rows) = node["data"][feature].as_array() {
            for child in rows {
                index(child, depth + 1, targets)?;
            }
        }
    }
    Ok(())
}
fn convert(node: &mut Value, targets: &BTreeMap<String, Target>) -> Result<(), Error> {
    if node["eClass"] == "sysml:FeatureTyping" {
        for (feature, expected) in [
            ("type", "sysml:PartDefinition"),
            ("typedFeature", "sysml:PartUsage"),
        ] {
            let Ok(Decoded::LocalNode(fragment)) = decode(node["data"].get(feature)) else {
                return Err(reference(
                    node,
                    feature,
                    "an untyped local node-ID wire is required",
                ));
            };
            let target = targets
                .get(&fragment)
                .ok_or_else(|| reference(node, feature, "the local node ID is unresolved"))?;
            if target.class != expected {
                return Err(reference(
                    node,
                    feature,
                    "the local node target has the wrong class",
                ));
            }
            node["data"][feature] = Value::String(target.semantic.clone());
        }
    }
    // Only known containment is visited; arbitrary vendor JSON is opaque.
    for feature in ["ownedRelationship", "ownedRelatedElement"] {
        if let Some(rows) = node["data"].get_mut(feature).and_then(Value::as_array_mut) {
            for child in rows {
                convert(child, targets)?;
            }
        }
    }
    Ok(())
}
/// Atomically convert one native local-node-ID resource into the six-class F14
/// selected representation. Only FeatureTyping.type/typedFeature are rewritten;
/// both UUID domains and unknown JSON values are preserved. Input is immutable.
///
/// Native reference validation runs first; the existing selected bridge then
/// validates package roots, memberships, exactly one typing per usage and its
/// association with that containing usage. No URI loading, cross-resource wires,
/// class expansion, encoding detection or complete SysML validity is provided.
/// Callers explicitly publish the returned document into an Editor or importer.
pub fn load(key: &str, document: &Document, encoding: SourceEncoding) -> Result<Document, Error> {
    match encoding {
        SourceEncoding::LocalJsonNodeIds => (),
    }
    let json = document
        .to_json()
        .map_err(|e| Error::Model(e.to_string()))?;
    if json.len() > MAX_RESOURCE_BYTES {
        return Err(Error::Limit);
    }
    let mut targets = BTreeMap::new();
    for node in document.value()["content"].as_array().unwrap() {
        index(node, 0, &mut targets)?;
    }
    Bundle::load(vec![Input {
        key: key.into(),
        json,
        fragments: FragmentIdentity::JsonNodeId,
    }])
    .map_err(|e| Error::Native(Box::new(e)))?;
    let mut converted = document.value().clone();
    for node in converted["content"].as_array_mut().unwrap() {
        convert(node, &targets)?;
    }
    let result =
        Document::from_json(&converted.to_string()).map_err(|e| Error::Model(e.to_string()))?;
    selected_model::load(
        key,
        &result,
        selected_model::SourceEncoding::LocalSemanticElementIds,
    )
    .map_err(|e| Error::Selected(Box::new(e)))?;
    Ok(result)
}

// SPDX-License-Identifier: EPL-2.0
//! Value-preserving selected EMF JSON bundles; no loader or complete serializer.
#![forbid(unsafe_code)]
pub mod native_model;
pub mod selected_model;

use serde_json::Value;
use syson_metamodel::{self as mm, ClassifierId as C, FeatureKind};
use syson_model::{Document, NodeIds};
use syson_references::{FragmentIdentity, Registry, Resource, Target};

pub const MAX_RESOURCES: usize = 32;
pub const MAX_RESOURCE_BYTES: usize = 1_048_576;
pub const MAX_NODES: usize = 4096;

/// Keys are caller-supplied identities, never paths to discover or load.
#[derive(Clone, Debug)]
pub struct Input {
    pub key: String,
    pub json: String,
    pub fragments: FragmentIdentity,
}
#[derive(Debug)]
pub enum Error {
    Limit,
    Identity,
    Model(String),
    UnsupportedClass(String),
    UnsupportedReference {
        class: String,
        feature: String,
    },
    Reference {
        resource: String,
        element: String,
        feature: String,
        cause: syson_references::Error,
    },
}
/// Constructed only after all documents and admitted references pass. An error
/// drops all private staging; publishing/replacing a caller's bundle is explicit.
/// Unknown fields survive as JSON values, without semantic interpretation.
pub struct Bundle {
    resources: Vec<(String, FragmentIdentity, Document)>,
    registry: Registry,
}
fn admitted(class: C) -> bool {
    matches!(
        class,
        C::Package
            | C::PartDefinition
            | C::PartUsage
            | C::PortDefinition
            | C::PortUsage
            | C::ViewDefinition
            | C::ViewUsage
            | C::OwningMembership
            | C::FeatureMembership
            | C::FeatureTyping
    )
}
fn class(node: &Value) -> Result<C, Error> {
    let name = node["eClass"].as_str().unwrap();
    name.strip_prefix("sysml:")
        .and_then(|s| mm::lookup(mm::Package::Domain, s))
        .filter(|c| admitted(*c))
        .ok_or_else(|| Error::UnsupportedClass(name.into()))
}
impl Bundle {
    pub fn load(inputs: Vec<Input>) -> Result<Self, Error> {
        if inputs.is_empty() || inputs.len() > MAX_RESOURCES {
            return Err(Error::Limit);
        }
        let mut resources = Vec::new();
        let mut count = 0;
        for input in inputs {
            if input.json.len() > MAX_RESOURCE_BYTES {
                return Err(Error::Limit);
            }
            if input.key.is_empty()
                || input.key.len() > 4096
                || input.key.chars().any(char::is_control)
                || resources.iter().any(|(key, _, _)| key == &input.key)
            {
                return Err(Error::Identity);
            }
            let document = Document::from_json(&input.json).map_err(|error| match error {
                syson_model::ModelError::JsonAdmission(syson_json_value::Error::Limit(_)) => {
                    Error::Limit
                }
                other => Error::Model(other.to_string()),
            })?;
            if document
                .to_json()
                .map_err(|e| Error::Model(e.to_string()))?
                .len()
                > MAX_RESOURCE_BYTES
            {
                return Err(Error::Limit);
            }
            let nodes = document.elements();
            count += nodes.len();
            if count > MAX_NODES {
                return Err(Error::Limit);
            }
            for summary in nodes {
                NodeIds::new(&summary.resource_id, &summary.element_id)
                    .map_err(|e| Error::Model(e.to_string()))?;
                class(document.element(&summary.element_id).unwrap())?;
            }
            resources.push((input.key, input.fragments, document));
        }
        let registry = Registry::new(
            resources
                .iter()
                .map(|(key, fragments, document)| Resource {
                    key: key.clone(),
                    fragments: *fragments,
                    document: document.clone(),
                })
                .collect(),
        )
        .map_err(|e| Error::Model(format!("{e:?}")))?;
        let bundle = Self {
            resources,
            registry,
        };
        for (key, _, document) in &bundle.resources {
            for summary in document.elements() {
                let node = document.element(&summary.element_id).unwrap();
                let classifier = class(node)?;
                for (feature, value) in node["data"].as_object().unwrap() {
                    let expected = match (classifier, feature.as_str()) {
                        (C::FeatureTyping, "type") => Some(C::Type),
                        (C::FeatureTyping, "typedFeature") => Some(C::Feature),
                        (_, "ownedRelationship" | "ownedRelatedElement") => None,
                        _ => {
                            if mm::inherited_feature(classifier, feature)
                                .candidates
                                .iter()
                                .any(|f| f.kind() == FeatureKind::Reference)
                            {
                                return Err(Error::UnsupportedReference {
                                    class: summary.class.clone(),
                                    feature: feature.clone(),
                                });
                            }
                            None
                        }
                    };
                    if let Some(expected) = expected {
                        bundle
                            .registry
                            .resolve(key, Some(value), expected)
                            .map_err(|cause| Error::Reference {
                                resource: key.clone(),
                                element: summary.element_id.clone(),
                                feature: feature.clone(),
                                cause,
                            })?;
                    }
                }
            }
        }
        Ok(bundle)
    }
    pub fn document(&self, key: &str) -> Option<&Document> {
        self.resources
            .iter()
            .find(|(k, _, _)| k == key)
            .map(|(_, _, d)| d)
    }
    /// Value-preserving canonical pretty JSON; whitespace and object key order
    /// from upload bytes are not retained. No reference rewriting or URI loading.
    pub fn save(&self) -> Result<Vec<Input>, Error> {
        self.resources
            .iter()
            .map(|(key, fragments, document)| {
                Ok(Input {
                    key: key.clone(),
                    fragments: *fragments,
                    json: document
                        .to_json()
                        .map_err(|e| Error::Model(e.to_string()))?,
                })
            })
            .collect()
    }
    pub fn resolve(
        &self,
        key: &str,
        wire: &Value,
        expected: C,
    ) -> Result<Option<Target>, syson_references::Error> {
        self.registry.resolve(key, Some(wire), expected)
    }
    pub fn target(&self, target: &Target) -> Result<&Value, syson_references::Error> {
        self.registry.target(target)
    }
}

/**
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
 */
/**
 * Copyright (c) 2023, 2025 Obeo.
 * This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v2.0
 * which accompanies this distribution, and is available at
 * https://www.eclipse.org/legal/epl-2.0/
 *
 * SPDX-License-Identifier: EPL-2.0
 *
 * Contributors:
 *     Obeo - initial API and implementation
 */
// SPDX-License-Identifier: EPL-2.0
// Typed raw JSON containment for preallocated detached subtrees.
// No identity allocation, EObject notifications or generic EMF serialization.
use super::*;
use syson_metamodel::{self as registry, ClassifierId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainmentFeature {
    OwnedRelationship,
    OwnedRelatedElement,
}
impl ContainmentFeature {
    fn name(self) -> &'static str {
        match self {
            Self::OwnedRelationship => "ownedRelationship",
            Self::OwnedRelatedElement => "ownedRelatedElement",
        }
    }
    fn parent_type(self) -> ClassifierId {
        match self {
            Self::OwnedRelationship => ClassifierId::Element,
            Self::OwnedRelatedElement => ClassifierId::Relationship,
        }
    }
    fn child_type(self) -> ClassifierId {
        match self {
            Self::OwnedRelationship => ClassifierId::Relationship,
            Self::OwnedRelatedElement => ClassifierId::Element,
        }
    }
}
fn validate_preallocated(node: &Value, depth: usize) -> Result<ClassifierId> {
    if depth > 128 {
        return Err(ModelError::Shape(
            "preallocated containment exceeds bounded depth".into(),
        ));
    }
    let data = node
        .get("data")
        .and_then(Value::as_object)
        .ok_or_else(|| ModelError::Shape("preallocated data must be object".into()))?;
    let resource = node
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| ModelError::Shape("preallocated id must be string".into()))?;
    let element = data
        .get("elementId")
        .and_then(Value::as_str)
        .ok_or_else(|| ModelError::Shape("preallocated elementId must be string".into()))?;
    NodeIds::new(resource, element)?;
    if node.get("eClass").and_then(Value::as_str).is_none() {
        return Err(ModelError::Shape(
            "preallocated eClass must be string".into(),
        ));
    }
    let classifier = scalar::node_classifier(node)?;
    if classifier.metadata().is_abstract() || classifier.metadata().is_interface() {
        return Err(ModelError::UnsupportedClass(
            classifier.metadata().name().into(),
        ));
    }
    for (name, parent_type, child_type) in [
        (
            "ownedRelationship",
            ClassifierId::Element,
            ClassifierId::Relationship,
        ),
        (
            "ownedRelatedElement",
            ClassifierId::Relationship,
            ClassifierId::Element,
        ),
    ] {
        if let Some(children) = data.get(name) {
            if !registry::is_subtype(classifier, parent_type) {
                return Err(ModelError::WrongType {
                    element: element.into(),
                    expected: parent_type.metadata().name(),
                });
            }
            for child in children
                .as_array()
                .ok_or_else(|| ModelError::Shape(format!("preallocated {name} must be array")))?
            {
                let actual = validate_preallocated(child, depth + 1)?;
                if !registry::is_subtype(actual, child_type) {
                    return Err(ModelError::WrongType {
                        element: child["data"]["elementId"].as_str().unwrap().into(),
                        expected: child_type.metadata().name(),
                    });
                }
            }
        }
    }
    Ok(classifier)
}
impl Transaction {
    /// Appends a detached subtree without allocating or rewriting identities.
    /// All fragment IDs must pass the existing NodeIds admission rule; that rule
    /// checks UUID spelling, not freshness, v4 provenance or entropy.
    /// Errors poison this transaction, preserving Editor commit refusal. Caller
    /// cancellation consumes the transaction through the existing cancel API.
    pub fn attach_preallocated_subtree(
        &mut self,
        parent: &str,
        feature: ContainmentFeature,
        subtree: Value,
    ) -> Result<()> {
        self.apply(move |doc| {
            let child_class = validate_preallocated(&subtree, 0)?;
            if !registry::is_subtype(child_class, feature.child_type()) {
                return Err(ModelError::WrongType {
                    element: subtree["data"]["elementId"].as_str().unwrap().into(),
                    expected: feature.child_type().metadata().name(),
                });
            }
            let content = doc.value["content"].as_array_mut().unwrap();
            let parent = find_node_mut(content, parent)
                .ok_or_else(|| ModelError::MissingElement(parent.into()))?;
            let parent_class = scalar::node_classifier(parent)?;
            if !registry::is_subtype(parent_class, feature.parent_type()) {
                return Err(ModelError::WrongType {
                    element: parent["data"]["elementId"].as_str().unwrap().into(),
                    expected: feature.parent_type().metadata().name(),
                });
            }
            let children = parent["data"]
                .as_object_mut()
                .unwrap()
                .entry(feature.name())
                .or_insert_with(|| json!([]));
            children
                .as_array_mut()
                .ok_or_else(|| ModelError::Shape("parent containment is not an array".into()))?
                .push(subtree);
            // Detect both semantic/resource collisions against the complete
            // staged document, including case-only duplicates and nested nodes.
            doc.validate()
        })
    }
}

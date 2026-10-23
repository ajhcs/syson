// Copyright (c) 2024, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Source metadata and Java setters: pinned SysON 6ba2ac64.
// Rust typed JSON edit adapter; semantic operations and notifications unported.
use super::*;
use syson_metamodel::{
    self as registry, ClassifierId, ClassifierKind, Feature, FeatureKind, TypeReference,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalarType {
    String,
    Boolean,
    Integer32,
    Double,
    Enumeration {
        classifier: ClassifierId,
        literals: Vec<&'static str>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeBlock {
    Derived,
    NotChangeable,
    ManyValued,
    IdentityManaged,
    UnsupportedType(String),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeError {
    UnknownAttribute(String),
    AmbiguousAttribute(String),
    UnresolvedAttributeSemantics {
        feature: String,
        external_supertypes: Vec<String>,
    },
    ReferenceEditUnsupported {
        feature: String,
        containment: bool,
        type_identity: Option<String>,
    },
    Blocked {
        feature: String,
        reason: AttributeBlock,
    },
    WrongValueType {
        feature: String,
        expected: &'static str,
    },
    InvalidEnumLiteral {
        feature: String,
    },
    UnsupportedNullReset {
        feature: String,
    },
    RequiredValue {
        feature: String,
    },
}
#[derive(Debug, Clone)]
pub struct AttributeDescriptor {
    pub name: &'static str,
    pub declared_by: ClassifierId,
    pub lower_bound: i32,
    pub upper_bound: i32,
    pub unsettable: bool,
    pub explicit_default_literal: Option<&'static str>,
    /// None means absent in stored JSON. No schema defaults are materialized.
    pub value: Option<Value>,
    pub value_type: Option<ScalarType>,
    pub blocked: Option<AttributeBlock>,
}
impl AttributeDescriptor {
    pub fn writable(&self) -> bool {
        self.blocked.is_none()
    }
    pub fn accepts_null(&self) -> bool {
        self.lower_bound == 0 && self.value_type == Some(ScalarType::String)
    }
}
#[derive(Debug, Clone)]
pub struct FeatureDiagnostic {
    pub name: String,
    pub issue: AttributeError,
}
#[derive(Debug, Clone)]
pub struct NodeInspection {
    pub classifier: ClassifierId,
    pub attributes: Vec<AttributeDescriptor>,
    pub diagnostics: Vec<FeatureDiagnostic>,
    pub unresolved_supertypes: Vec<String>,
}
impl NodeInspection {
    pub fn writable_attributes(&self) -> impl Iterator<Item = &AttributeDescriptor> {
        self.attributes.iter().filter(|a| a.writable())
    }
}
impl Document {
    /// Inspects known domain EClasses without mutating imported/unknown fields.
    pub fn inspect_node(&self, element: &str) -> Result<NodeInspection> {
        let node = self
            .element(element)
            .ok_or_else(|| ModelError::MissingElement(element.into()))?;
        let classifier = node_classifier(node)?;
        let mut names = BTreeSet::new();
        let mut visited = BTreeSet::new();
        let mut stack = vec![classifier];
        while let Some(id) = stack.pop() {
            if !visited.insert(id) {
                continue;
            }
            names.extend(id.metadata().features().map(|f| f.name()));
            stack.extend(id.metadata().supertypes().filter_map(TypeReference::local));
        }
        let mut attributes = Vec::new();
        let mut diagnostics = Vec::new();
        for name in names {
            match scalar_feature(classifier, name) {
                Ok(feature) => {
                    attributes.push(descriptor(feature, node["data"].get(name).cloned()))
                }
                Err(issue) => diagnostics.push(FeatureDiagnostic {
                    name: name.into(),
                    issue,
                }),
            }
        }
        Ok(NodeInspection {
            classifier,
            attributes,
            diagnostics,
            unresolved_supertypes: registry::unresolved_supertypes(classifier)
                .into_iter()
                .map(|r| r.raw().to_owned())
                .collect(),
        })
    }
}
impl Projection {
    /// Exact known-local subtype predicate from the pinned registry. Unknown
    /// imported classes never become positive matches through name guessing.
    pub fn find_by_type(&self, name: &str, classifier: ClassifierId) -> Result<Option<&str>> {
        let result = self
            .model
            .find_by_name_and_type(name, |e| {
                registry::lookup(registry::Package::Domain, &e.class)
                    .is_some_and(|actual| registry::is_subtype(actual, classifier))
            })
            .map_err(|e| ModelError::Shape(format!("projection lookup: {e:?}")))?;
        Ok(result.and_then(|id| self.stable_id(id)))
    }
}
impl Transaction {
    /// Typed scalar edit, not Jackson input coercion or arbitrary Java eSet.
    /// Reference/containment and identity changes are deliberately rejected.
    pub fn set_attribute(&mut self, element: &str, name: &str, value: Value) -> Result<()> {
        self.apply(|doc| {
            let node = doc
                .element(element)
                .ok_or_else(|| ModelError::MissingElement(element.into()))?;
            let classifier = node_classifier(node)?;
            let feature = scalar_feature(classifier, name).map_err(ModelError::Attribute)?;
            let info = descriptor(feature, None);
            if let Some(reason) = info.blocked {
                return Err(ModelError::Attribute(AttributeError::Blocked {
                    feature: name.into(),
                    reason,
                }));
            }
            let normalized = normalize_value(&info, value).map_err(ModelError::Attribute)?;
            let node =
                find_node_mut(doc.value["content"].as_array_mut().unwrap(), element).unwrap();
            node["data"][name] = normalized;
            doc.validate()
        })
    }
}
pub(super) fn node_classifier(node: &Value) -> Result<ClassifierId> {
    let raw = node["eClass"].as_str().unwrap();
    raw.strip_prefix("sysml:")
        .and_then(|name| registry::lookup(registry::Package::Domain, name))
        .filter(|id| id.metadata().kind == ClassifierKind::Class)
        .ok_or_else(|| ModelError::UnsupportedClass(raw.into()))
}
fn scalar_feature(
    classifier: ClassifierId,
    name: &str,
) -> std::result::Result<Feature, AttributeError> {
    let found = registry::inherited_feature(classifier, name);
    if found.candidates.is_empty() {
        return Err(AttributeError::UnknownAttribute(name.into()));
    }
    let feature = found
        .unique()
        .ok_or_else(|| AttributeError::AmbiguousAttribute(name.into()))?;
    if !found.unresolved_supertypes.is_empty() {
        return Err(AttributeError::UnresolvedAttributeSemantics {
            feature: name.into(),
            external_supertypes: found
                .unresolved_supertypes
                .iter()
                .map(|r| r.raw().to_owned())
                .collect(),
        });
    }
    if feature.kind() != FeatureKind::Attribute {
        return Err(AttributeError::ReferenceEditUnsupported {
            feature: name.into(),
            containment: feature.containment(),
            type_identity: feature.typed().type_reference().map(|r| r.raw().to_owned()),
        });
    }
    Ok(feature)
}
fn descriptor(feature: Feature, value: Option<Value>) -> AttributeDescriptor {
    let resolved = scalar_type(feature.typed().type_reference());
    let blocked = if feature.derived() {
        Some(AttributeBlock::Derived)
    } else if !feature.changeable() {
        Some(AttributeBlock::NotChangeable)
    } else if feature.typed().upper_bound() != 1 {
        Some(AttributeBlock::ManyValued)
    } else if feature.is_id() {
        Some(AttributeBlock::IdentityManaged)
    } else {
        resolved.as_ref().err().cloned()
    };
    AttributeDescriptor {
        name: feature.name(),
        declared_by: feature.owner,
        lower_bound: feature.typed().lower_bound(),
        upper_bound: feature.typed().upper_bound(),
        unsettable: feature.unsettable(),
        explicit_default_literal: feature.explicit_default_literal(),
        value,
        value_type: resolved.ok(),
        blocked,
    }
}
fn scalar_type(
    reference: Option<TypeReference>,
) -> std::result::Result<ScalarType, AttributeBlock> {
    match reference {
        Some(TypeReference::External {
            raw: "ecore:EDataType http://www.eclipse.org/emf/2002/Ecore#//EString",
        }) => Ok(ScalarType::String),
        Some(TypeReference::External {
            raw: "ecore:EDataType http://www.eclipse.org/emf/2002/Ecore#//EBoolean",
        }) => Ok(ScalarType::Boolean),
        Some(TypeReference::External {
            raw: "ecore:EDataType http://www.eclipse.org/emf/2002/Ecore#//EInt",
        }) => Ok(ScalarType::Integer32),
        Some(TypeReference::External {
            raw: "ecore:EDataType http://www.eclipse.org/emf/2002/Ecore#//EDouble",
        }) => Ok(ScalarType::Double),
        Some(TypeReference::Local { id, .. }) if id.metadata().kind == ClassifierKind::Enum => {
            Ok(ScalarType::Enumeration {
                classifier: id,
                literals: id.metadata().enum_literals().map(|l| l.literal).collect(),
            })
        }
        other => Err(AttributeBlock::UnsupportedType(
            other.map_or("<no eType>".into(), |r| r.raw().into()),
        )),
    }
}
fn normalize_value(
    info: &AttributeDescriptor,
    value: Value,
) -> std::result::Result<Value, AttributeError> {
    let feature = info.name.to_owned();
    if value.is_null() {
        return if info.accepts_null() {
            Ok(Value::Null)
        } else if matches!(info.value_type, Some(ScalarType::Enumeration { .. })) {
            Err(AttributeError::UnsupportedNullReset { feature })
        } else {
            Err(AttributeError::RequiredValue { feature })
        };
    }
    let wrong = |expected| AttributeError::WrongValueType {
        feature: feature.clone(),
        expected,
    };
    match info.value_type.as_ref().unwrap() {
        ScalarType::String => {
            if value.is_string() {
                Ok(value)
            } else {
                Err(wrong("string"))
            }
        }
        ScalarType::Boolean => {
            if value.is_boolean() {
                Ok(value)
            } else {
                Err(wrong("boolean"))
            }
        }
        ScalarType::Integer32 => {
            if value.as_i64().is_some_and(|n| i32::try_from(n).is_ok()) {
                Ok(value)
            } else {
                Err(wrong("signed 32-bit integer"))
            }
        }
        ScalarType::Double => {
            let n = value
                .as_f64()
                .filter(|n| n.is_finite())
                .ok_or_else(|| wrong("finite IEEE754 double"))?;
            Ok(Value::Number(serde_json::Number::from_f64(n).unwrap()))
        }
        ScalarType::Enumeration { literals, .. } => {
            let literal = value.as_str().ok_or_else(|| wrong("enum literal string"))?;
            if literals.contains(&literal) {
                Ok(value)
            } else {
                Err(AttributeError::InvalidEnumLiteral { feature })
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unrecognized_external_datatype_has_explicit_unsupported_state() {
        let raw = "ecore:EDataType http://example.invalid/Unimplemented#//Scalar";
        assert_eq!(
            scalar_type(Some(TypeReference::External { raw })),
            Err(AttributeBlock::UnsupportedType(raw.into()))
        );
    }
}

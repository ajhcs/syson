// Copyright (c) 2023, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Contributors: Obeo - source metamodel/factory; Rust port contributors - native adapter.
//! Raw, unattached domain allocation and source-proven local cached state.
//! No Document mutation, context initializer, route, serializer or EObject engine.
//! Fresh semantic IDs come from an explicit provider; resource IDs stay unassigned.
use std::collections::BTreeSet;
use syson_metamodel::{
    self as registry, ClassifierId, ClassifierKind, Feature, FeatureKind, Package,
};

pub const BASELINE: &str = "6ba2ac64e6b16ee2b05752ad55395eebb87d4910";
pub const DOMAIN_NAMESPACE: &str = "http://www.eclipse.org/syson/sysml";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceRef {
    pub path: &'static str,
    pub line: usize,
    pub sha256: &'static str,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldId(pub usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarKind {
    String,
    Boolean,
    Integer32,
    Double,
    Enumeration(ClassifierId),
}
#[derive(Clone, Debug, PartialEq)]
pub enum ScalarValue {
    Null,
    String(String),
    Boolean(bool),
    Integer32(i32),
    Double(f64),
    Enumeration {
        classifier: ClassifierId,
        literal: String,
    },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DefaultValue {
    Null,
    Boolean(bool),
    Integer32(i32),
    Double(f64),
    Enumeration {
        classifier: ClassifierId,
        literal: &'static str,
    },
}
impl DefaultValue {
    pub fn value(self) -> ScalarValue {
        match self {
            Self::Null => ScalarValue::Null,
            Self::Boolean(v) => ScalarValue::Boolean(v),
            Self::Integer32(v) => ScalarValue::Integer32(v),
            Self::Double(v) => ScalarValue::Double(v),
            Self::Enumeration {
                classifier,
                literal,
            } => ScalarValue::Enumeration {
                classifier,
                literal: literal.into(),
            },
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldKind {
    Scalar(ScalarKind),
    SetFlag,
    Reference,
    LazyCollection,
}
#[derive(Clone, Copy, Debug)]
pub struct DefaultConstant {
    pub owner: ClassifierId,
    pub name: &'static str,
    pub java_type: &'static str,
    pub value: DefaultValue,
    pub source: SourceRef,
}
#[derive(Clone, Copy, Debug)]
pub struct CachedFieldSpec {
    pub id: FieldId,
    pub owner: ClassifierId,
    pub name: &'static str,
    pub java_type: &'static str,
    pub kind: FieldKind,
    pub initial: DefaultValue,
    pub default_constant: Option<usize>,
    pub source: SourceRef,
}
#[derive(Clone, Copy, Debug)]
pub struct RawClassSpec {
    pub classifier: ClassifierId,
    pub java_parent: Option<ClassifierId>,
    pub external_parent: Option<&'static str>,
    pub raw_constructible: bool,
    pub factory_line: Option<usize>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GetterBehavior {
    Cached(FieldId),
    Custom,
}
#[derive(Clone, Copy, Debug)]
pub struct GetterSpec {
    pub owner: ClassifierId,
    pub attribute: &'static str,
    pub method: &'static str,
    pub behavior: GetterBehavior,
    pub source: SourceRef,
}
#[derive(Clone, Copy, Debug)]
pub struct SetterSpec {
    pub owner: ClassifierId,
    pub attribute: &'static str,
    pub field: Option<FieldId>,
    pub null_reset_default: Option<usize>,
    pub set_flag: Option<FieldId>,
    pub source: SourceRef,
}
#[derive(Clone, Copy, Debug)]
pub enum IsSetBehavior {
    Difference { field: FieldId, default: usize },
    Flag(FieldId),
    Unsupported,
}
#[derive(Clone, Copy, Debug)]
pub struct IsSetSpec {
    pub owner: ClassifierId,
    pub attribute: &'static str,
    pub behavior: IsSetBehavior,
    pub source: SourceRef,
}
#[derive(Clone, Copy, Debug)]
pub struct UnsetterSpec {
    pub owner: ClassifierId,
    pub attribute: &'static str,
    pub field: FieldId,
    pub default: usize,
    pub set_flag: FieldId,
    pub source: SourceRef,
}
#[derive(Clone, Copy, Debug)]
pub enum ResetBehavior {
    SetterDefault(usize),
    UnsetMethod,
}
#[derive(Clone, Copy, Debug)]
pub struct ResetSpec {
    pub owner: ClassifierId,
    pub attribute: &'static str,
    pub behavior: ResetBehavior,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnsupportedReason {
    Derived,
    Collection,
    Reference,
    Ambiguous,
    CustomGetter {
        method: &'static str,
        source: SourceRef,
    },
    UnknownOrExternal {
        external_supertypes: Vec<String>,
    },
    MissingGetter,
    UnsafeSetter,
    UnsafeUnset,
    IdentityManaged,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FactoryError {
    WrongNamespace(String),
    UnknownClassifier(String),
    WrongPackage(ClassifierId),
    NotClass(ClassifierId),
    AbstractClass(ClassifierId),
    MissingEvidence(String),
    IdentityProvider(String),
    InvalidFreshV4,
    ReusedSemanticId,
    TypeMismatch(ScalarKind),
    InvalidEnumLiteral,
    Unsupported {
        attribute: String,
        reason: UnsupportedReason,
    },
}
impl std::fmt::Display for FactoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for FactoryError {}
type Result<T> = std::result::Result<T, FactoryError>;
/// Provider owns its entropy source. A valid UUID spelling alone does not prove
/// cryptographic randomness; callers must supply a genuine fresh-v4 provider.
pub trait FreshElementIdProvider {
    fn fresh_v4(&mut self) -> std::result::Result<String, String>;
}
impl<F: FnMut() -> std::result::Result<String, String>> FreshElementIdProvider for F {
    fn fresh_v4(&mut self) -> std::result::Result<String, String> {
        self()
    }
}
pub struct RawFactory<P> {
    provider: P,
    allocated: BTreeSet<String>,
}
impl<P: FreshElementIdProvider> RawFactory<P> {
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            allocated: BTreeSet::new(),
        }
    }
    pub fn allocate_named(&mut self, namespace: &str, name: &str) -> Result<RawObject> {
        if namespace != DOMAIN_NAMESPACE {
            return Err(FactoryError::WrongNamespace(namespace.into()));
        }
        let classifier = registry::lookup(Package::Domain, name)
            .ok_or_else(|| FactoryError::UnknownClassifier(name.into()))?;
        self.allocate(classifier)
    }
    pub fn allocate(&mut self, classifier: ClassifierId) -> Result<RawObject> {
        let metadata = classifier.metadata();
        if metadata.package != Package::Domain {
            return Err(FactoryError::WrongPackage(classifier));
        }
        if metadata.kind != ClassifierKind::Class {
            return Err(FactoryError::NotClass(classifier));
        }
        if metadata.is_abstract() || metadata.is_interface() {
            return Err(FactoryError::AbstractClass(classifier));
        }
        let spec = class_spec(classifier)
            .ok_or_else(|| FactoryError::MissingEvidence(metadata.name().into()))?;
        if !spec.raw_constructible {
            return Err(FactoryError::MissingEvidence(metadata.name().into()));
        }
        let semantic_id = self
            .provider
            .fresh_v4()
            .map_err(FactoryError::IdentityProvider)?;
        if !valid_fresh_v4(&semantic_id) {
            return Err(FactoryError::InvalidFreshV4);
        }
        if self.allocated.contains(&semantic_id) {
            return Err(FactoryError::ReusedSemanticId);
        }
        let chain = java_chain(classifier);
        let mut fields = Vec::new();
        // Java initializes superclass fields before subclass fields; keep both
        // copies when an implementation shadows an inherited field name.
        for owner in chain.iter().rev() {
            fields.extend(
                CACHED_FIELDS
                    .iter()
                    .filter(|f| f.owner == *owner)
                    .map(|spec| CachedFieldState {
                        spec,
                        value: spec.initial.value(),
                        origin: AssignmentOrigin::ConstructorDefault,
                    }),
            );
        }
        let identity = fields
            .iter_mut()
            .find(|f| f.spec.owner == ClassifierId::Element && f.spec.name == "elementId")
            .ok_or_else(|| FactoryError::MissingEvidence("Element.elementId cache".into()))?;
        identity.value = ScalarValue::String(semantic_id.clone());
        identity.origin = AssignmentOrigin::FactoryElementId;
        self.allocated.insert(semantic_id.clone());
        Ok(RawObject {
            classifier,
            semantic_id,
            fields,
        })
    }
}
fn valid_fresh_v4(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
            }
        })
        && id.as_bytes()[14] == b'4'
        && b"89ab".contains(&id.as_bytes()[19])
}
pub fn class_spec(classifier: ClassifierId) -> Option<&'static RawClassSpec> {
    CLASS_SPECS.iter().find(|s| s.classifier == classifier)
}
fn java_chain(classifier: ClassifierId) -> Vec<ClassifierId> {
    let mut chain = Vec::new();
    let mut current = Some(classifier);
    while let Some(class) = current {
        chain.push(class);
        current = class_spec(class).and_then(|s| s.java_parent);
    }
    chain
}
#[derive(Clone, Debug, PartialEq)]
pub enum AssignmentOrigin {
    ConstructorDefault,
    FactoryElementId,
    ExplicitInput(ScalarValue),
    Reset,
}
#[derive(Clone, Debug)]
pub struct CachedFieldState {
    pub spec: &'static CachedFieldSpec,
    pub value: ScalarValue,
    pub origin: AssignmentOrigin,
}
/// No storage observation is inferred from native defaults or native assignments.
/// Absent and Present(Null) remain distinct when a caller supplies an observation.
#[derive(Clone, Debug, PartialEq)]
pub enum StorageObservation {
    Unobserved,
    Absent,
    Present(ScalarValue),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetState {
    Known(bool),
    Unsupported { source: Option<SourceRef> },
}
#[derive(Clone, Debug, PartialEq)]
pub struct ScalarSnapshot {
    pub value: ScalarValue,
    pub set_state: SetState,
    pub origin: AssignmentOrigin,
    pub storage_observation: StorageObservation,
    pub getter_source: SourceRef,
}
#[derive(Clone, Debug)]
pub struct RawObject {
    classifier: ClassifierId,
    semantic_id: String,
    fields: Vec<CachedFieldState>,
}
impl RawObject {
    pub fn classifier(&self) -> ClassifierId {
        self.classifier
    }
    pub fn semantic_id(&self) -> &str {
        &self.semantic_id
    }
    /// Resource identity is deliberately not allocated by this raw factory.
    pub fn resource_id(&self) -> Option<&str> {
        None
    }
    pub fn cached_fields(&self) -> &[CachedFieldState] {
        &self.fields
    }
    pub fn cached_field(&self, owner: ClassifierId, name: &str) -> Option<&CachedFieldState> {
        self.fields
            .iter()
            .find(|f| f.spec.owner == owner && f.spec.name == name)
    }
    pub fn external_state_gaps(&self) -> Vec<String> {
        registry::unresolved_supertypes(self.classifier)
            .into_iter()
            .map(|r| r.raw().into())
            .collect()
    }
    fn slot(&self, id: FieldId) -> Result<&CachedFieldState> {
        self.fields
            .iter()
            .find(|f| f.spec.id == id)
            .ok_or_else(|| FactoryError::MissingEvidence(format!("cache field {}", id.0)))
    }
    fn write_slot(&mut self, id: FieldId, value: ScalarValue, origin: AssignmentOrigin) {
        // Every mutation checks all addressed slots before reaching this helper.
        if let Some(slot) = self.fields.iter_mut().find(|f| f.spec.id == id) {
            slot.value = value;
            slot.origin = origin;
        }
    }
    fn unsupported(&self, name: &str, reason: UnsupportedReason) -> FactoryError {
        FactoryError::Unsupported {
            attribute: name.into(),
            reason,
        }
    }
    fn binding(&self, name: &str) -> Result<(Feature, &'static GetterSpec, FieldId)> {
        let found = registry::inherited_feature(self.classifier, name);
        if found.candidates.is_empty() {
            return Err(self.unsupported(
                name,
                UnsupportedReason::UnknownOrExternal {
                    external_supertypes: found
                        .unresolved_supertypes
                        .iter()
                        .map(|r| r.raw().into())
                        .collect(),
                },
            ));
        }
        let feature = found
            .unique()
            .ok_or_else(|| self.unsupported(name, UnsupportedReason::Ambiguous))?;
        if feature.kind() != FeatureKind::Attribute {
            return Err(self.unsupported(name, UnsupportedReason::Reference));
        }
        if feature.derived() {
            return Err(self.unsupported(name, UnsupportedReason::Derived));
        }
        if feature.typed().upper_bound() != 1 {
            return Err(self.unsupported(name, UnsupportedReason::Collection));
        }
        let getter = java_chain(self.classifier)
            .into_iter()
            .find_map(|class| {
                GETTERS
                    .iter()
                    .find(|s| s.owner == class && s.attribute == name)
            })
            .ok_or_else(|| self.unsupported(name, UnsupportedReason::MissingGetter))?;
        match getter.behavior {
            GetterBehavior::Cached(field) => {
                self.slot(field)?;
                Ok((feature, getter, field))
            }
            GetterBehavior::Custom => Err(self.unsupported(
                name,
                UnsupportedReason::CustomGetter {
                    method: getter.method,
                    source: getter.source,
                },
            )),
        }
    }
    fn source_is_set(&self, name: &str) -> SetState {
        let spec = java_chain(self.classifier).into_iter().find_map(|class| {
            IS_SET_SPECS
                .iter()
                .find(|s| s.owner == class && s.attribute == name)
        });
        match spec {
            Some(IsSetSpec {
                behavior: IsSetBehavior::Difference { field, default },
                ..
            }) => match self.slot(*field) {
                Ok(slot) => {
                    SetState::Known(slot.value != DEFAULT_CONSTANTS[*default].value.value())
                }
                Err(_) => SetState::Unsupported {
                    source: spec.map(|s| s.source),
                },
            },
            Some(IsSetSpec {
                behavior: IsSetBehavior::Flag(field),
                ..
            }) => match self.slot(*field) {
                Ok(CachedFieldState {
                    value: ScalarValue::Boolean(flag),
                    ..
                }) => SetState::Known(*flag),
                _ => SetState::Unsupported {
                    source: spec.map(|s| s.source),
                },
            },
            _ => SetState::Unsupported {
                source: spec.map(|s| s.source),
            },
        }
    }
    pub fn inspect_scalar(&self, name: &str) -> Result<ScalarSnapshot> {
        let (_, getter, field) = self.binding(name)?;
        let slot = self.slot(field)?;
        if !matches!(slot.spec.kind, FieldKind::Scalar(_)) {
            return Err(self.unsupported(
                name,
                UnsupportedReason::CustomGetter {
                    method: getter.method,
                    source: getter.source,
                },
            ));
        }
        Ok(ScalarSnapshot {
            value: slot.value.clone(),
            set_state: self.source_is_set(name),
            origin: slot.origin.clone(),
            storage_observation: StorageObservation::Unobserved,
            getter_source: getter.source,
        })
    }
    /// Attaches an explicitly caller-supplied observation to the returned value
    /// only. It neither imports JSON nor applies that observation to this object.
    pub fn inspect_scalar_with_storage_observation(
        &self,
        name: &str,
        observation: StorageObservation,
    ) -> Result<ScalarSnapshot> {
        let mut snapshot = self.inspect_scalar(name)?;
        snapshot.storage_observation = observation;
        Ok(snapshot)
    }
    fn writable_binding(&self, name: &str) -> Result<(FieldId, &'static SetterSpec)> {
        let (feature, _, field) = self.binding(name)?;
        if feature.is_id() {
            return Err(self.unsupported(name, UnsupportedReason::IdentityManaged));
        }
        if !feature.changeable() {
            return Err(self.unsupported(name, UnsupportedReason::UnsafeSetter));
        }
        let setter = java_chain(self.classifier)
            .into_iter()
            .find_map(|class| {
                SETTERS
                    .iter()
                    .find(|s| s.owner == class && s.attribute == name)
            })
            .ok_or_else(|| self.unsupported(name, UnsupportedReason::UnsafeSetter))?;
        if setter.field != Some(field) {
            return Err(self.unsupported(name, UnsupportedReason::UnsafeSetter));
        }
        if let Some(flag) = setter.set_flag {
            self.slot(flag)?;
        }
        Ok((field, setter))
    }
    /// Applies only source-proven local cached setter semantics. No EObject
    /// notifications, inverse updates, Document edits or serialized writes occur.
    pub fn assign_cached_scalar(
        &mut self,
        name: &str,
        input: ScalarValue,
    ) -> Result<ScalarSnapshot> {
        let (field, setter) = self.writable_binding(name)?;
        let FieldKind::Scalar(kind) = self.slot(field)?.spec.kind else {
            return Err(self.unsupported(name, UnsupportedReason::UnsafeSetter));
        };
        let value = if input == ScalarValue::Null {
            setter
                .null_reset_default
                .map(|i| DEFAULT_CONSTANTS[i].value.value())
                .unwrap_or_else(|| input.clone())
        } else {
            input.clone()
        };
        validate_scalar(kind, &value, setter.null_reset_default.is_some())?;
        self.write_slot(field, value, AssignmentOrigin::ExplicitInput(input));
        if let Some(flag) = setter.set_flag {
            self.write_slot(
                flag,
                ScalarValue::Boolean(true),
                AssignmentOrigin::ExplicitInput(ScalarValue::Boolean(true)),
            );
        }
        self.inspect_scalar(name)
    }
    /// Replays only proven eUnset scalar dispatch, including enum set flags.
    /// Identity and custom/reference/list mutations remain explicitly unsupported.
    pub fn unset_cached_scalar(&mut self, name: &str) -> Result<ScalarSnapshot> {
        let (field, _) = self.writable_binding(name)?;
        let reset = java_chain(self.classifier)
            .into_iter()
            .find_map(|class| {
                RESETS
                    .iter()
                    .find(|s| s.owner == class && s.attribute == name)
            })
            .ok_or_else(|| self.unsupported(name, UnsupportedReason::UnsafeUnset))?;
        match reset.behavior {
            ResetBehavior::SetterDefault(default) => {
                self.assign_cached_scalar(name, DEFAULT_CONSTANTS[default].value.value())?;
                self.write_slot(
                    field,
                    self.slot(field)?.value.clone(),
                    AssignmentOrigin::Reset,
                );
            }
            ResetBehavior::UnsetMethod => {
                let unsetter = java_chain(self.classifier)
                    .into_iter()
                    .find_map(|class| {
                        UNSETTERS
                            .iter()
                            .find(|s| s.owner == class && s.attribute == name)
                    })
                    .ok_or_else(|| self.unsupported(name, UnsupportedReason::UnsafeUnset))?;
                if unsetter.field != field {
                    return Err(self.unsupported(name, UnsupportedReason::UnsafeUnset));
                }
                self.slot(unsetter.set_flag)?;
                self.write_slot(
                    field,
                    DEFAULT_CONSTANTS[unsetter.default].value.value(),
                    AssignmentOrigin::Reset,
                );
                self.write_slot(
                    unsetter.set_flag,
                    ScalarValue::Boolean(false),
                    AssignmentOrigin::Reset,
                );
            }
        }
        self.inspect_scalar(name)
    }
}
fn validate_scalar(kind: ScalarKind, value: &ScalarValue, enum_null: bool) -> Result<()> {
    let valid = match (kind, value) {
        (ScalarKind::String, ScalarValue::String(_) | ScalarValue::Null) => true,
        (ScalarKind::Boolean, ScalarValue::Boolean(_))
        | (ScalarKind::Integer32, ScalarValue::Integer32(_))
        | (ScalarKind::Double, ScalarValue::Double(_)) => true,
        (
            ScalarKind::Enumeration(expected),
            ScalarValue::Enumeration {
                classifier,
                literal,
            },
        ) => {
            if *classifier != expected {
                return Err(FactoryError::TypeMismatch(kind));
            }
            if !classifier
                .metadata()
                .enum_literals()
                .any(|v| v.literal == literal)
            {
                return Err(FactoryError::InvalidEnumLiteral);
            }
            true
        }
        (ScalarKind::Enumeration(_), ScalarValue::Null) => enum_null,
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(FactoryError::TypeMismatch(kind))
    }
}

// BEGIN PINNED SOURCE TABLES - generated by porting/domain/factory-implementation/generate-source.py
pub static DEFAULT_CONSTANTS: &[DefaultConstant] = &[
    DefaultConstant {
        owner: ClassifierId::AssertConstraintUsage,
        name: "IS_NEGATED_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssertConstraintUsageImpl.java",
            line: 47,
            sha256: "26a68637a08cdce51f972c837fd8cb151cfe303be9a90691a96d06676cda3454",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Association,
        name: "IS_IMPLIED_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssociationImpl.java",
            line: 67,
            sha256: "aaaf65656b531086ac47abfe06dc0f6715c3a34cb459b4f57f171b8d6089c8d7",
        },
    },
    DefaultConstant {
        owner: ClassifierId::CalculationDefinition,
        name: "IS_MODEL_LEVEL_EVALUABLE_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CalculationDefinitionImpl.java",
            line: 55,
            sha256: "1a14004d79980eeb6ecb677674ef9e64cfbdf807e674cc90daa468b2dc4f5822",
        },
    },
    DefaultConstant {
        owner: ClassifierId::CalculationUsage,
        name: "IS_MODEL_LEVEL_EVALUABLE_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CalculationUsageImpl.java",
            line: 57,
            sha256: "0357c1e27ba49ef525da2dd45530ad1841b0cbccdd28143f72513d791c10f0c6",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Comment,
        name: "BODY_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CommentImpl.java",
            line: 42,
            sha256: "4f2913fe8128a200498793edfd90c594f19169854dada56ce0b5ff36842f8bf8",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Comment,
        name: "LOCALE_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CommentImpl.java",
            line: 62,
            sha256: "4f2913fe8128a200498793edfd90c594f19169854dada56ce0b5ff36842f8bf8",
        },
    },
    DefaultConstant {
        owner: ClassifierId::ConnectionDefinition,
        name: "IS_IMPLIED_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionDefinitionImpl.java",
            line: 75,
            sha256: "99aac0b55d655179e84f0dd8419111616a91bf2eb0b7ab8ab880acf063813352",
        },
    },
    DefaultConstant {
        owner: ClassifierId::ConnectionUsage,
        name: "IS_INDIVIDUAL_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 68,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    DefaultConstant {
        owner: ClassifierId::ConnectionUsage,
        name: "PORTION_KIND_EDEFAULT",
        java_type: "PortionKind",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 88,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Connector,
        name: "IS_IMPLIED_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorImpl.java",
            line: 72,
            sha256: "1f49aa6b7e11ffec691b129403fadbc7dc1e179855b7ae9f7303dc84d11d8528",
        },
    },
    DefaultConstant {
        owner: ClassifierId::ConnectorAsUsage,
        name: "IS_IMPLIED_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorAsUsageImpl.java",
            line: 76,
            sha256: "9a34426f1fe469817000d26eb0399ee29991838e1bb45501f8477fe4dc084857",
        },
    },
    DefaultConstant {
        owner: ClassifierId::ConstraintDefinition,
        name: "IS_MODEL_LEVEL_EVALUABLE_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConstraintDefinitionImpl.java",
            line: 59,
            sha256: "67edcc045820f764f1c47e318cbf0ecac46f3416f7291aa7b2fef3e89b51373c",
        },
    },
    DefaultConstant {
        owner: ClassifierId::ConstraintUsage,
        name: "IS_MODEL_LEVEL_EVALUABLE_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConstraintUsageImpl.java",
            line: 75,
            sha256: "9a782d4050e58d8bdcf061bfea5fca441039b9b94a5469922e29b45bc82d5a6d",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Definition,
        name: "IS_VARIATION_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/DefinitionImpl.java",
            line: 109,
            sha256: "b8c1f6391eb0fd2b022ece2e8ea239413a2b6b562ee7209f80fc1a35fa9145ec",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Element,
        name: "DECLARED_NAME_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 90,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Element,
        name: "DECLARED_SHORT_NAME_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 110,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Element,
        name: "ELEMENT_ID_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 130,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Element,
        name: "IS_IMPLIED_INCLUDED_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 150,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Element,
        name: "IS_LIBRARY_ELEMENT_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 170,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Element,
        name: "NAME_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 180,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Element,
        name: "QUALIFIED_NAME_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 190,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Element,
        name: "SHORT_NAME_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 200,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Expression,
        name: "IS_MODEL_LEVEL_EVALUABLE_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ExpressionImpl.java",
            line: 54,
            sha256: "bbbaec6f943ed89c79ce7a58c30ad5d895bd3e7c356e1b04a584639292dbf15f",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Feature,
        name: "DIRECTION_EDEFAULT",
        java_type: "FeatureDirectionKind",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 104,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Feature,
        name: "IS_COMPOSITE_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 132,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Feature,
        name: "IS_CONSTANT_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 152,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Feature,
        name: "IS_DERIVED_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 172,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Feature,
        name: "IS_END_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 192,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Feature,
        name: "IS_NONUNIQUE_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 212,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Feature,
        name: "IS_ORDERED_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 222,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Feature,
        name: "IS_PORTION_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 242,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Feature,
        name: "IS_UNIQUE_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(true),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 262,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Feature,
        name: "IS_VARIABLE_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 282,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    DefaultConstant {
        owner: ClassifierId::FeatureValue,
        name: "IS_DEFAULT_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureValueImpl.java",
            line: 49,
            sha256: "06d158252791fccd475b35f55547369dd63bd92d6b27a2649e0d736553a66cc7",
        },
    },
    DefaultConstant {
        owner: ClassifierId::FeatureValue,
        name: "IS_INITIAL_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureValueImpl.java",
            line: 69,
            sha256: "06d158252791fccd475b35f55547369dd63bd92d6b27a2649e0d736553a66cc7",
        },
    },
    DefaultConstant {
        owner: ClassifierId::FlowDefinition,
        name: "IS_IMPLIED_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowDefinitionImpl.java",
            line: 75,
            sha256: "244d80d8b39947bd40618e40af85847d44035c8220b238658960e4e1082b01ed",
        },
    },
    DefaultConstant {
        owner: ClassifierId::FlowUsage,
        name: "IS_INDIVIDUAL_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 82,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    DefaultConstant {
        owner: ClassifierId::FlowUsage,
        name: "PORTION_KIND_EDEFAULT",
        java_type: "PortionKind",
        value: DefaultValue::Enumeration {
            classifier: ClassifierId::PortionKind,
            literal: "snapshot",
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 102,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Function,
        name: "IS_MODEL_LEVEL_EVALUABLE_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FunctionImpl.java",
            line: 52,
            sha256: "8a5dbc66196a21a04c260a3bda8a363aebff5bdff9d3dcce91fff73f41024434",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Import,
        name: "IS_IMPORT_ALL_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 55,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Import,
        name: "IS_RECURSIVE_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 75,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Import,
        name: "VISIBILITY_EDEFAULT",
        java_type: "VisibilityKind",
        value: DefaultValue::Enumeration {
            classifier: ClassifierId::VisibilityKind,
            literal: "private",
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 95,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Invariant,
        name: "IS_NEGATED_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/InvariantImpl.java",
            line: 41,
            sha256: "1e5440a1324cb70f52d4a2bb313ac4ade34e1ac0cd608c5e720e3d424a09bbdf",
        },
    },
    DefaultConstant {
        owner: ClassifierId::LibraryPackage,
        name: "IS_STANDARD_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LibraryPackageImpl.java",
            line: 43,
            sha256: "9103340eb814396cb02705d14b9062fa660b0577879c31538eae5534a552e99f",
        },
    },
    DefaultConstant {
        owner: ClassifierId::LiteralBoolean,
        name: "VALUE_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralBooleanImpl.java",
            line: 42,
            sha256: "1cdf2c81a78ab9a5ae2437a4cfe959aa31ee9f37107949638abb3d1fb70d52d2",
        },
    },
    DefaultConstant {
        owner: ClassifierId::LiteralInteger,
        name: "VALUE_EDEFAULT",
        java_type: "int",
        value: DefaultValue::Integer32(0),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralIntegerImpl.java",
            line: 42,
            sha256: "ea23684de620c444cd6cd43c5562e6c4f1a180d9f8cfb02ed58284134d06f96c",
        },
    },
    DefaultConstant {
        owner: ClassifierId::LiteralRational,
        name: "VALUE_EDEFAULT",
        java_type: "double",
        value: DefaultValue::Double(0.0),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralRationalImpl.java",
            line: 42,
            sha256: "0e99ac7f50b7d612c5f1450d76195d8b5f4c8f0b3ef7db87ab94b82145f8d632",
        },
    },
    DefaultConstant {
        owner: ClassifierId::LiteralString,
        name: "VALUE_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralStringImpl.java",
            line: 41,
            sha256: "0f51f25d8b70036c25befc883f7dc4e88e3b63b60555d3b8ecd633a3b4f1441f",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Membership,
        name: "MEMBER_ELEMENT_ID_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 60,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Membership,
        name: "MEMBER_NAME_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 70,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Membership,
        name: "MEMBER_SHORT_NAME_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 90,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Membership,
        name: "VISIBILITY_EDEFAULT",
        java_type: "VisibilityKind",
        value: DefaultValue::Enumeration {
            classifier: ClassifierId::VisibilityKind,
            literal: "public",
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 110,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    DefaultConstant {
        owner: ClassifierId::OccurrenceDefinition,
        name: "IS_INDIVIDUAL_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceDefinitionImpl.java",
            line: 42,
            sha256: "da09e6d912522b3ce8080ed9d8598abee5652d238df2ea6e0b5f44994871ef75",
        },
    },
    DefaultConstant {
        owner: ClassifierId::OccurrenceUsage,
        name: "IS_INDIVIDUAL_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 57,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
    DefaultConstant {
        owner: ClassifierId::OccurrenceUsage,
        name: "PORTION_KIND_EDEFAULT",
        java_type: "PortionKind",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 77,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
    DefaultConstant {
        owner: ClassifierId::OperatorExpression,
        name: "OPERATOR_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OperatorExpressionImpl.java",
            line: 42,
            sha256: "af2646607ccbe2e363494a3b54c2004ab7777766fc93e08590d591a16b8d5b68",
        },
    },
    DefaultConstant {
        owner: ClassifierId::OwningMembership,
        name: "OWNED_MEMBER_ELEMENT_ID_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OwningMembershipImpl.java",
            line: 49,
            sha256: "c9b001a480fc6ee3ca6005160bdcd2edefdbf195477f7f126eac9026c32923d5",
        },
    },
    DefaultConstant {
        owner: ClassifierId::OwningMembership,
        name: "OWNED_MEMBER_NAME_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OwningMembershipImpl.java",
            line: 59,
            sha256: "c9b001a480fc6ee3ca6005160bdcd2edefdbf195477f7f126eac9026c32923d5",
        },
    },
    DefaultConstant {
        owner: ClassifierId::OwningMembership,
        name: "OWNED_MEMBER_SHORT_NAME_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OwningMembershipImpl.java",
            line: 69,
            sha256: "c9b001a480fc6ee3ca6005160bdcd2edefdbf195477f7f126eac9026c32923d5",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Relationship,
        name: "IS_IMPLIED_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RelationshipImpl.java",
            line: 61,
            sha256: "fbd56ac8a6e5792869844e2a91c8b55fa7aecd6cc2e04b3e5315382a457bbe3d",
        },
    },
    DefaultConstant {
        owner: ClassifierId::RequirementConstraintMembership,
        name: "KIND_EDEFAULT",
        java_type: "RequirementConstraintKind",
        value: DefaultValue::Enumeration {
            classifier: ClassifierId::RequirementConstraintKind,
            literal: "assumption",
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementConstraintMembershipImpl.java",
            line: 52,
            sha256: "96bd03d8e2490604aa1b474eb8a69427ac45c35b96af4157337f4f78221f3e45",
        },
    },
    DefaultConstant {
        owner: ClassifierId::RequirementDefinition,
        name: "REQ_ID_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementDefinitionImpl.java",
            line: 69,
            sha256: "877901d3a8da850c15105ce9b633e45d825f52cdbeef41bf8b1db3ff4cacd917",
        },
    },
    DefaultConstant {
        owner: ClassifierId::RequirementUsage,
        name: "REQ_ID_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementUsageImpl.java",
            line: 72,
            sha256: "e6e28cd8c181652e257b31369cf8e2697a628a1a2b2a8d73bdf70f36530accf2",
        },
    },
    DefaultConstant {
        owner: ClassifierId::SatisfyRequirementUsage,
        name: "IS_NEGATED_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/SatisfyRequirementUsageImpl.java",
            line: 58,
            sha256: "f55cf0eddbbb8d5af72f925674eec7290bf065e0d728a6e96649cce0cb429de3",
        },
    },
    DefaultConstant {
        owner: ClassifierId::StateDefinition,
        name: "IS_PARALLEL_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateDefinitionImpl.java",
            line: 57,
            sha256: "192b52b80e56b4816bf6b8bdf12478acb0f65223b897422a3014a95ae19bebf0",
        },
    },
    DefaultConstant {
        owner: ClassifierId::StateSubactionMembership,
        name: "KIND_EDEFAULT",
        java_type: "StateSubactionKind",
        value: DefaultValue::Enumeration {
            classifier: ClassifierId::StateSubactionKind,
            literal: "do",
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateSubactionMembershipImpl.java",
            line: 47,
            sha256: "f362f4fdc765663f74f3a3ea0aa2645779d528e07cf8a3511520c4d4b4990644",
        },
    },
    DefaultConstant {
        owner: ClassifierId::StateUsage,
        name: "IS_PARALLEL_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateUsageImpl.java",
            line: 60,
            sha256: "235b477331ac3e3311e37b7a3ec1bebd2ace7bdc56ef0a30781f4213ff224865",
        },
    },
    DefaultConstant {
        owner: ClassifierId::TextualRepresentation,
        name: "BODY_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TextualRepresentationImpl.java",
            line: 50,
            sha256: "31d09a2b2639048566c9f56f04f7c12d53d107c5420effb2e4ff7d8ba5c52b3e",
        },
    },
    DefaultConstant {
        owner: ClassifierId::TextualRepresentation,
        name: "LANGUAGE_EDEFAULT",
        java_type: "String",
        value: DefaultValue::Null,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TextualRepresentationImpl.java",
            line: 70,
            sha256: "31d09a2b2639048566c9f56f04f7c12d53d107c5420effb2e4ff7d8ba5c52b3e",
        },
    },
    DefaultConstant {
        owner: ClassifierId::TransitionFeatureMembership,
        name: "KIND_EDEFAULT",
        java_type: "TransitionFeatureKind",
        value: DefaultValue::Enumeration {
            classifier: ClassifierId::TransitionFeatureKind,
            literal: "effect",
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TransitionFeatureMembershipImpl.java",
            line: 49,
            sha256: "a8cd905da455a039bfe81b0ead676c0167fc53ff78064de4872b8c1913d4115f",
        },
    },
    DefaultConstant {
        owner: ClassifierId::TriggerInvocationExpression,
        name: "KIND_EDEFAULT",
        java_type: "TriggerKind",
        value: DefaultValue::Enumeration {
            classifier: ClassifierId::TriggerKind,
            literal: "after",
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TriggerInvocationExpressionImpl.java",
            line: 43,
            sha256: "35f26a38a2c17afcac3ed32d568179dff68854722f9270a3be6a165a0e7a102f",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Type,
        name: "IS_ABSTRACT_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 99,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Type,
        name: "IS_CONJUGATED_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 119,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Type,
        name: "IS_SUFFICIENT_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 129,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Usage,
        name: "IS_REFERENCE_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UsageImpl.java",
            line: 121,
            sha256: "88ff141769debfe9a8c9cf575b0ec05a289f267479af5800d6526fc470cabe26",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Usage,
        name: "IS_VARIATION_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UsageImpl.java",
            line: 131,
            sha256: "88ff141769debfe9a8c9cf575b0ec05a289f267479af5800d6526fc470cabe26",
        },
    },
    DefaultConstant {
        owner: ClassifierId::Usage,
        name: "MAY_TIME_VARY_EDEFAULT",
        java_type: "boolean",
        value: DefaultValue::Boolean(false),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UsageImpl.java",
            line: 151,
            sha256: "88ff141769debfe9a8c9cf575b0ec05a289f267479af5800d6526fc470cabe26",
        },
    },
];
pub static CACHED_FIELDS: &[CachedFieldSpec] = &[
    CachedFieldSpec {
        id: FieldId(0),
        owner: ClassifierId::Annotation,
        name: "annotatedElement",
        java_type: "Element",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AnnotationImpl.java",
            line: 61,
            sha256: "8415e6d021cb956701c50f813c6d3c025ad9f4d6bacc3ccd90eb15496d05b18b",
        },
    },
    CachedFieldSpec {
        id: FieldId(1),
        owner: ClassifierId::AssertConstraintUsage,
        name: "isNegated",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(0),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssertConstraintUsageImpl.java",
            line: 57,
            sha256: "26a68637a08cdce51f972c837fd8cb151cfe303be9a90691a96d06676cda3454",
        },
    },
    CachedFieldSpec {
        id: FieldId(2),
        owner: ClassifierId::Association,
        name: "isImplied",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(1),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssociationImpl.java",
            line: 77,
            sha256: "aaaf65656b531086ac47abfe06dc0f6715c3a34cb459b4f57f171b8d6089c8d7",
        },
    },
    CachedFieldSpec {
        id: FieldId(3),
        owner: ClassifierId::Association,
        name: "ownedRelatedElement",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssociationImpl.java",
            line: 87,
            sha256: "aaaf65656b531086ac47abfe06dc0f6715c3a34cb459b4f57f171b8d6089c8d7",
        },
    },
    CachedFieldSpec {
        id: FieldId(4),
        owner: ClassifierId::Association,
        name: "source",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssociationImpl.java",
            line: 97,
            sha256: "aaaf65656b531086ac47abfe06dc0f6715c3a34cb459b4f57f171b8d6089c8d7",
        },
    },
    CachedFieldSpec {
        id: FieldId(5),
        owner: ClassifierId::Association,
        name: "target",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssociationImpl.java",
            line: 107,
            sha256: "aaaf65656b531086ac47abfe06dc0f6715c3a34cb459b4f57f171b8d6089c8d7",
        },
    },
    CachedFieldSpec {
        id: FieldId(6),
        owner: ClassifierId::Comment,
        name: "body",
        java_type: "String",
        kind: FieldKind::Scalar(ScalarKind::String),
        initial: DefaultValue::Null,
        default_constant: Some(4),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CommentImpl.java",
            line: 52,
            sha256: "4f2913fe8128a200498793edfd90c594f19169854dada56ce0b5ff36842f8bf8",
        },
    },
    CachedFieldSpec {
        id: FieldId(7),
        owner: ClassifierId::Comment,
        name: "locale",
        java_type: "String",
        kind: FieldKind::Scalar(ScalarKind::String),
        initial: DefaultValue::Null,
        default_constant: Some(5),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CommentImpl.java",
            line: 72,
            sha256: "4f2913fe8128a200498793edfd90c594f19169854dada56ce0b5ff36842f8bf8",
        },
    },
    CachedFieldSpec {
        id: FieldId(8),
        owner: ClassifierId::ConjugatedPortTyping,
        name: "conjugatedPortDefinition",
        java_type: "ConjugatedPortDefinition",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConjugatedPortTypingImpl.java",
            line: 48,
            sha256: "7adf3faa4057653986cd9965bcd55f96ae686b633c4a19880ca242c5c003cd4f",
        },
    },
    CachedFieldSpec {
        id: FieldId(9),
        owner: ClassifierId::Conjugation,
        name: "conjugatedType",
        java_type: "Type",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConjugationImpl.java",
            line: 51,
            sha256: "4bb692fbd63a6522e7e0d1614e747fe9139cdd6a00309eb5030981b114c79952",
        },
    },
    CachedFieldSpec {
        id: FieldId(10),
        owner: ClassifierId::Conjugation,
        name: "originalType",
        java_type: "Type",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConjugationImpl.java",
            line: 61,
            sha256: "4bb692fbd63a6522e7e0d1614e747fe9139cdd6a00309eb5030981b114c79952",
        },
    },
    CachedFieldSpec {
        id: FieldId(11),
        owner: ClassifierId::ConnectionDefinition,
        name: "isImplied",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(6),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionDefinitionImpl.java",
            line: 85,
            sha256: "99aac0b55d655179e84f0dd8419111616a91bf2eb0b7ab8ab880acf063813352",
        },
    },
    CachedFieldSpec {
        id: FieldId(12),
        owner: ClassifierId::ConnectionDefinition,
        name: "ownedRelatedElement",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionDefinitionImpl.java",
            line: 95,
            sha256: "99aac0b55d655179e84f0dd8419111616a91bf2eb0b7ab8ab880acf063813352",
        },
    },
    CachedFieldSpec {
        id: FieldId(13),
        owner: ClassifierId::ConnectionDefinition,
        name: "source",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionDefinitionImpl.java",
            line: 105,
            sha256: "99aac0b55d655179e84f0dd8419111616a91bf2eb0b7ab8ab880acf063813352",
        },
    },
    CachedFieldSpec {
        id: FieldId(14),
        owner: ClassifierId::ConnectionDefinition,
        name: "target",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionDefinitionImpl.java",
            line: 115,
            sha256: "99aac0b55d655179e84f0dd8419111616a91bf2eb0b7ab8ab880acf063813352",
        },
    },
    CachedFieldSpec {
        id: FieldId(15),
        owner: ClassifierId::ConnectionUsage,
        name: "isIndividual",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(7),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 78,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    CachedFieldSpec {
        id: FieldId(16),
        owner: ClassifierId::ConnectionUsage,
        name: "portionKind",
        java_type: "PortionKind",
        kind: FieldKind::Scalar(ScalarKind::Enumeration(ClassifierId::PortionKind)),
        initial: DefaultValue::Null,
        default_constant: Some(8),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 98,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    CachedFieldSpec {
        id: FieldId(17),
        owner: ClassifierId::ConnectionUsage,
        name: "portionKindESet",
        java_type: "boolean",
        kind: FieldKind::SetFlag,
        initial: DefaultValue::Boolean(false),
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 106,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    CachedFieldSpec {
        id: FieldId(18),
        owner: ClassifierId::Connector,
        name: "isImplied",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(9),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorImpl.java",
            line: 82,
            sha256: "1f49aa6b7e11ffec691b129403fadbc7dc1e179855b7ae9f7303dc84d11d8528",
        },
    },
    CachedFieldSpec {
        id: FieldId(19),
        owner: ClassifierId::Connector,
        name: "ownedRelatedElement",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorImpl.java",
            line: 92,
            sha256: "1f49aa6b7e11ffec691b129403fadbc7dc1e179855b7ae9f7303dc84d11d8528",
        },
    },
    CachedFieldSpec {
        id: FieldId(20),
        owner: ClassifierId::Connector,
        name: "source",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorImpl.java",
            line: 102,
            sha256: "1f49aa6b7e11ffec691b129403fadbc7dc1e179855b7ae9f7303dc84d11d8528",
        },
    },
    CachedFieldSpec {
        id: FieldId(21),
        owner: ClassifierId::Connector,
        name: "target",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorImpl.java",
            line: 112,
            sha256: "1f49aa6b7e11ffec691b129403fadbc7dc1e179855b7ae9f7303dc84d11d8528",
        },
    },
    CachedFieldSpec {
        id: FieldId(22),
        owner: ClassifierId::ConnectorAsUsage,
        name: "isImplied",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(10),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorAsUsageImpl.java",
            line: 86,
            sha256: "9a34426f1fe469817000d26eb0399ee29991838e1bb45501f8477fe4dc084857",
        },
    },
    CachedFieldSpec {
        id: FieldId(23),
        owner: ClassifierId::ConnectorAsUsage,
        name: "ownedRelatedElement",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorAsUsageImpl.java",
            line: 96,
            sha256: "9a34426f1fe469817000d26eb0399ee29991838e1bb45501f8477fe4dc084857",
        },
    },
    CachedFieldSpec {
        id: FieldId(24),
        owner: ClassifierId::ConnectorAsUsage,
        name: "source",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorAsUsageImpl.java",
            line: 106,
            sha256: "9a34426f1fe469817000d26eb0399ee29991838e1bb45501f8477fe4dc084857",
        },
    },
    CachedFieldSpec {
        id: FieldId(25),
        owner: ClassifierId::ConnectorAsUsage,
        name: "target",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorAsUsageImpl.java",
            line: 116,
            sha256: "9a34426f1fe469817000d26eb0399ee29991838e1bb45501f8477fe4dc084857",
        },
    },
    CachedFieldSpec {
        id: FieldId(26),
        owner: ClassifierId::CrossSubsetting,
        name: "crossedFeature",
        java_type: "Feature",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CrossSubsettingImpl.java",
            line: 46,
            sha256: "f3e1c73df149e78f552ccd434618232addbd91be65d461f636a96e526d11adc4",
        },
    },
    CachedFieldSpec {
        id: FieldId(27),
        owner: ClassifierId::Definition,
        name: "isVariation",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(13),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/DefinitionImpl.java",
            line: 119,
            sha256: "b8c1f6391eb0fd2b022ece2e8ea239413a2b6b562ee7209f80fc1a35fa9145ec",
        },
    },
    CachedFieldSpec {
        id: FieldId(28),
        owner: ClassifierId::Dependency,
        name: "client",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/DependencyImpl.java",
            line: 45,
            sha256: "3c402989f5657227e3a375b31283d211c9f3f4134ca29573ce3bab1b12585e55",
        },
    },
    CachedFieldSpec {
        id: FieldId(29),
        owner: ClassifierId::Dependency,
        name: "supplier",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/DependencyImpl.java",
            line: 55,
            sha256: "3c402989f5657227e3a375b31283d211c9f3f4134ca29573ce3bab1b12585e55",
        },
    },
    CachedFieldSpec {
        id: FieldId(30),
        owner: ClassifierId::Differencing,
        name: "differencingType",
        java_type: "Type",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/DifferencingImpl.java",
            line: 50,
            sha256: "161e52e27eb8eee697ff5cf0ad5b3e4efbf82136eda4d8fe8b50232097dff38f",
        },
    },
    CachedFieldSpec {
        id: FieldId(31),
        owner: ClassifierId::Disjoining,
        name: "disjoiningType",
        java_type: "Type",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/DisjoiningImpl.java",
            line: 51,
            sha256: "7403e2d15948965fbd93969e411eb7ed09f94b276d00b990f0bf017a826d813c",
        },
    },
    CachedFieldSpec {
        id: FieldId(32),
        owner: ClassifierId::Disjoining,
        name: "typeDisjoined",
        java_type: "Type",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/DisjoiningImpl.java",
            line: 61,
            sha256: "7403e2d15948965fbd93969e411eb7ed09f94b276d00b990f0bf017a826d813c",
        },
    },
    CachedFieldSpec {
        id: FieldId(33),
        owner: ClassifierId::Element,
        name: "aliasIds",
        java_type: "EList<String>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 80,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    CachedFieldSpec {
        id: FieldId(34),
        owner: ClassifierId::Element,
        name: "declaredName",
        java_type: "String",
        kind: FieldKind::Scalar(ScalarKind::String),
        initial: DefaultValue::Null,
        default_constant: Some(14),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 100,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    CachedFieldSpec {
        id: FieldId(35),
        owner: ClassifierId::Element,
        name: "declaredShortName",
        java_type: "String",
        kind: FieldKind::Scalar(ScalarKind::String),
        initial: DefaultValue::Null,
        default_constant: Some(15),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 120,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    CachedFieldSpec {
        id: FieldId(36),
        owner: ClassifierId::Element,
        name: "elementId",
        java_type: "String",
        kind: FieldKind::Scalar(ScalarKind::String),
        initial: DefaultValue::Null,
        default_constant: Some(16),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 140,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    CachedFieldSpec {
        id: FieldId(37),
        owner: ClassifierId::Element,
        name: "isImpliedIncluded",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(17),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 160,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    CachedFieldSpec {
        id: FieldId(38),
        owner: ClassifierId::Element,
        name: "ownedRelationship",
        java_type: "EList<Relationship>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 210,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    CachedFieldSpec {
        id: FieldId(39),
        owner: ClassifierId::Feature,
        name: "direction",
        java_type: "FeatureDirectionKind",
        kind: FieldKind::Scalar(ScalarKind::Enumeration(ClassifierId::FeatureDirectionKind)),
        initial: DefaultValue::Null,
        default_constant: Some(23),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 114,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    CachedFieldSpec {
        id: FieldId(40),
        owner: ClassifierId::Feature,
        name: "directionESet",
        java_type: "boolean",
        kind: FieldKind::SetFlag,
        initial: DefaultValue::Boolean(false),
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 122,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    CachedFieldSpec {
        id: FieldId(41),
        owner: ClassifierId::Feature,
        name: "isComposite",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(24),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 142,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    CachedFieldSpec {
        id: FieldId(42),
        owner: ClassifierId::Feature,
        name: "isConstant",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(25),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 162,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    CachedFieldSpec {
        id: FieldId(43),
        owner: ClassifierId::Feature,
        name: "isDerived",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(26),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 182,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    CachedFieldSpec {
        id: FieldId(44),
        owner: ClassifierId::Feature,
        name: "isEnd",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(27),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 202,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    CachedFieldSpec {
        id: FieldId(45),
        owner: ClassifierId::Feature,
        name: "isOrdered",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(29),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 232,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    CachedFieldSpec {
        id: FieldId(46),
        owner: ClassifierId::Feature,
        name: "isPortion",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(30),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 252,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    CachedFieldSpec {
        id: FieldId(47),
        owner: ClassifierId::Feature,
        name: "isUnique",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(true),
        default_constant: Some(31),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 272,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    CachedFieldSpec {
        id: FieldId(48),
        owner: ClassifierId::Feature,
        name: "isVariable",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(32),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 292,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    CachedFieldSpec {
        id: FieldId(49),
        owner: ClassifierId::FeatureChaining,
        name: "chainingFeature",
        java_type: "Feature",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureChainingImpl.java",
            line: 51,
            sha256: "70de16073dfc7a7513290c34d51bf79d306462e96e19fbd779a8ac3c8843806a",
        },
    },
    CachedFieldSpec {
        id: FieldId(50),
        owner: ClassifierId::FeatureInverting,
        name: "featureInverted",
        java_type: "Feature",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureInvertingImpl.java",
            line: 52,
            sha256: "a47f66905d0a0499943fe6759698a2eca6c10367b7d1c76f662fbc44708a8530",
        },
    },
    CachedFieldSpec {
        id: FieldId(51),
        owner: ClassifierId::FeatureInverting,
        name: "invertingFeature",
        java_type: "Feature",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureInvertingImpl.java",
            line: 62,
            sha256: "a47f66905d0a0499943fe6759698a2eca6c10367b7d1c76f662fbc44708a8530",
        },
    },
    CachedFieldSpec {
        id: FieldId(52),
        owner: ClassifierId::FeatureTyping,
        name: "type",
        java_type: "Type",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureTypingImpl.java",
            line: 47,
            sha256: "68dcee25dd646be7b2240906e42209b5ccc03ffe68cf931c4f1d49a23d6ca097",
        },
    },
    CachedFieldSpec {
        id: FieldId(53),
        owner: ClassifierId::FeatureTyping,
        name: "typedFeature",
        java_type: "Feature",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureTypingImpl.java",
            line: 57,
            sha256: "68dcee25dd646be7b2240906e42209b5ccc03ffe68cf931c4f1d49a23d6ca097",
        },
    },
    CachedFieldSpec {
        id: FieldId(54),
        owner: ClassifierId::FeatureValue,
        name: "isDefault",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(33),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureValueImpl.java",
            line: 59,
            sha256: "06d158252791fccd475b35f55547369dd63bd92d6b27a2649e0d736553a66cc7",
        },
    },
    CachedFieldSpec {
        id: FieldId(55),
        owner: ClassifierId::FeatureValue,
        name: "isInitial",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(34),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureValueImpl.java",
            line: 79,
            sha256: "06d158252791fccd475b35f55547369dd63bd92d6b27a2649e0d736553a66cc7",
        },
    },
    CachedFieldSpec {
        id: FieldId(56),
        owner: ClassifierId::FlowDefinition,
        name: "isImplied",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(35),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowDefinitionImpl.java",
            line: 85,
            sha256: "244d80d8b39947bd40618e40af85847d44035c8220b238658960e4e1082b01ed",
        },
    },
    CachedFieldSpec {
        id: FieldId(57),
        owner: ClassifierId::FlowDefinition,
        name: "ownedRelatedElement",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowDefinitionImpl.java",
            line: 95,
            sha256: "244d80d8b39947bd40618e40af85847d44035c8220b238658960e4e1082b01ed",
        },
    },
    CachedFieldSpec {
        id: FieldId(58),
        owner: ClassifierId::FlowDefinition,
        name: "source",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowDefinitionImpl.java",
            line: 105,
            sha256: "244d80d8b39947bd40618e40af85847d44035c8220b238658960e4e1082b01ed",
        },
    },
    CachedFieldSpec {
        id: FieldId(59),
        owner: ClassifierId::FlowDefinition,
        name: "target",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowDefinitionImpl.java",
            line: 115,
            sha256: "244d80d8b39947bd40618e40af85847d44035c8220b238658960e4e1082b01ed",
        },
    },
    CachedFieldSpec {
        id: FieldId(60),
        owner: ClassifierId::FlowUsage,
        name: "isIndividual",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(36),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 92,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    CachedFieldSpec {
        id: FieldId(61),
        owner: ClassifierId::FlowUsage,
        name: "portionKind",
        java_type: "PortionKind",
        kind: FieldKind::Scalar(ScalarKind::Enumeration(ClassifierId::PortionKind)),
        initial: DefaultValue::Enumeration {
            classifier: ClassifierId::PortionKind,
            literal: "snapshot",
        },
        default_constant: Some(37),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 112,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    CachedFieldSpec {
        id: FieldId(62),
        owner: ClassifierId::FlowUsage,
        name: "portionKindESet",
        java_type: "boolean",
        kind: FieldKind::SetFlag,
        initial: DefaultValue::Boolean(false),
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 120,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    CachedFieldSpec {
        id: FieldId(63),
        owner: ClassifierId::Import,
        name: "isImportAll",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(39),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 65,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    CachedFieldSpec {
        id: FieldId(64),
        owner: ClassifierId::Import,
        name: "isRecursive",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(40),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 85,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    CachedFieldSpec {
        id: FieldId(65),
        owner: ClassifierId::Import,
        name: "visibility",
        java_type: "VisibilityKind",
        kind: FieldKind::Scalar(ScalarKind::Enumeration(ClassifierId::VisibilityKind)),
        initial: DefaultValue::Enumeration {
            classifier: ClassifierId::VisibilityKind,
            literal: "private",
        },
        default_constant: Some(41),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 105,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    CachedFieldSpec {
        id: FieldId(66),
        owner: ClassifierId::Intersecting,
        name: "intersectingType",
        java_type: "Type",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/IntersectingImpl.java",
            line: 50,
            sha256: "c63eec9cca22961dca97a7f1c77c89e1033bd0a907f45d1d1f21e126e15169a8",
        },
    },
    CachedFieldSpec {
        id: FieldId(67),
        owner: ClassifierId::Invariant,
        name: "isNegated",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(42),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/InvariantImpl.java",
            line: 51,
            sha256: "1e5440a1324cb70f52d4a2bb313ac4ade34e1ac0cd608c5e720e3d424a09bbdf",
        },
    },
    CachedFieldSpec {
        id: FieldId(68),
        owner: ClassifierId::LibraryPackage,
        name: "isStandard",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(43),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LibraryPackageImpl.java",
            line: 53,
            sha256: "9103340eb814396cb02705d14b9062fa660b0577879c31538eae5534a552e99f",
        },
    },
    CachedFieldSpec {
        id: FieldId(69),
        owner: ClassifierId::LiteralBoolean,
        name: "value",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(44),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralBooleanImpl.java",
            line: 52,
            sha256: "1cdf2c81a78ab9a5ae2437a4cfe959aa31ee9f37107949638abb3d1fb70d52d2",
        },
    },
    CachedFieldSpec {
        id: FieldId(70),
        owner: ClassifierId::LiteralInteger,
        name: "value",
        java_type: "int",
        kind: FieldKind::Scalar(ScalarKind::Integer32),
        initial: DefaultValue::Integer32(0),
        default_constant: Some(45),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralIntegerImpl.java",
            line: 52,
            sha256: "ea23684de620c444cd6cd43c5562e6c4f1a180d9f8cfb02ed58284134d06f96c",
        },
    },
    CachedFieldSpec {
        id: FieldId(71),
        owner: ClassifierId::LiteralRational,
        name: "value",
        java_type: "double",
        kind: FieldKind::Scalar(ScalarKind::Double),
        initial: DefaultValue::Double(0.0),
        default_constant: Some(46),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralRationalImpl.java",
            line: 52,
            sha256: "0e99ac7f50b7d612c5f1450d76195d8b5f4c8f0b3ef7db87ab94b82145f8d632",
        },
    },
    CachedFieldSpec {
        id: FieldId(72),
        owner: ClassifierId::LiteralString,
        name: "value",
        java_type: "String",
        kind: FieldKind::Scalar(ScalarKind::String),
        initial: DefaultValue::Null,
        default_constant: Some(47),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralStringImpl.java",
            line: 51,
            sha256: "0f51f25d8b70036c25befc883f7dc4e88e3b63b60555d3b8ecd633a3b4f1441f",
        },
    },
    CachedFieldSpec {
        id: FieldId(73),
        owner: ClassifierId::Membership,
        name: "memberName",
        java_type: "String",
        kind: FieldKind::Scalar(ScalarKind::String),
        initial: DefaultValue::Null,
        default_constant: Some(49),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 80,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    CachedFieldSpec {
        id: FieldId(74),
        owner: ClassifierId::Membership,
        name: "memberShortName",
        java_type: "String",
        kind: FieldKind::Scalar(ScalarKind::String),
        initial: DefaultValue::Null,
        default_constant: Some(50),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 100,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    CachedFieldSpec {
        id: FieldId(75),
        owner: ClassifierId::Membership,
        name: "visibility",
        java_type: "VisibilityKind",
        kind: FieldKind::Scalar(ScalarKind::Enumeration(ClassifierId::VisibilityKind)),
        initial: DefaultValue::Enumeration {
            classifier: ClassifierId::VisibilityKind,
            literal: "public",
        },
        default_constant: Some(51),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 120,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    CachedFieldSpec {
        id: FieldId(76),
        owner: ClassifierId::Membership,
        name: "memberElement",
        java_type: "Element",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 130,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    CachedFieldSpec {
        id: FieldId(77),
        owner: ClassifierId::MembershipImport,
        name: "importedMembership",
        java_type: "Membership",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImportImpl.java",
            line: 52,
            sha256: "6247435c125bfe21beddc99aa04fe34001d40ae337f9c652740eec1baf1bf780",
        },
    },
    CachedFieldSpec {
        id: FieldId(78),
        owner: ClassifierId::NamespaceImport,
        name: "importedNamespace",
        java_type: "Namespace",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/NamespaceImportImpl.java",
            line: 52,
            sha256: "93ff9f7a44d6c737b2543320e94da73d4a6dfb76aea503f641b4513ffda6d74c",
        },
    },
    CachedFieldSpec {
        id: FieldId(79),
        owner: ClassifierId::OccurrenceDefinition,
        name: "isIndividual",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(52),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceDefinitionImpl.java",
            line: 52,
            sha256: "da09e6d912522b3ce8080ed9d8598abee5652d238df2ea6e0b5f44994871ef75",
        },
    },
    CachedFieldSpec {
        id: FieldId(80),
        owner: ClassifierId::OccurrenceUsage,
        name: "isIndividual",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(53),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 67,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
    CachedFieldSpec {
        id: FieldId(81),
        owner: ClassifierId::OccurrenceUsage,
        name: "portionKind",
        java_type: "PortionKind",
        kind: FieldKind::Scalar(ScalarKind::Enumeration(ClassifierId::PortionKind)),
        initial: DefaultValue::Null,
        default_constant: Some(54),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 87,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
    CachedFieldSpec {
        id: FieldId(82),
        owner: ClassifierId::OccurrenceUsage,
        name: "portionKindESet",
        java_type: "boolean",
        kind: FieldKind::SetFlag,
        initial: DefaultValue::Boolean(false),
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 95,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
    CachedFieldSpec {
        id: FieldId(83),
        owner: ClassifierId::OperatorExpression,
        name: "operator",
        java_type: "String",
        kind: FieldKind::Scalar(ScalarKind::String),
        initial: DefaultValue::Null,
        default_constant: Some(55),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OperatorExpressionImpl.java",
            line: 52,
            sha256: "af2646607ccbe2e363494a3b54c2004ab7777766fc93e08590d591a16b8d5b68",
        },
    },
    CachedFieldSpec {
        id: FieldId(84),
        owner: ClassifierId::PortConjugation,
        name: "originalPortDefinition",
        java_type: "PortDefinition",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/PortConjugationImpl.java",
            line: 49,
            sha256: "a30ccdde5bdc1a5d540b68960fef3e6aba076d8ecdc695f5231c7df1c587f926",
        },
    },
    CachedFieldSpec {
        id: FieldId(85),
        owner: ClassifierId::Redefinition,
        name: "redefinedFeature",
        java_type: "Feature",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RedefinitionImpl.java",
            line: 44,
            sha256: "9f2664cdcd1d727e1d9432ead6104a4e26e2318a8831f1714d15264fb3254738",
        },
    },
    CachedFieldSpec {
        id: FieldId(86),
        owner: ClassifierId::Redefinition,
        name: "redefiningFeature",
        java_type: "Feature",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RedefinitionImpl.java",
            line: 54,
            sha256: "9f2664cdcd1d727e1d9432ead6104a4e26e2318a8831f1714d15264fb3254738",
        },
    },
    CachedFieldSpec {
        id: FieldId(87),
        owner: ClassifierId::ReferenceSubsetting,
        name: "referencedFeature",
        java_type: "Feature",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ReferenceSubsettingImpl.java",
            line: 48,
            sha256: "ce72d83a5434cb5a5436575b526c1496fb01a32f771e7f6258604a6182a24684",
        },
    },
    CachedFieldSpec {
        id: FieldId(88),
        owner: ClassifierId::Relationship,
        name: "isImplied",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(59),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RelationshipImpl.java",
            line: 71,
            sha256: "fbd56ac8a6e5792869844e2a91c8b55fa7aecd6cc2e04b3e5315382a457bbe3d",
        },
    },
    CachedFieldSpec {
        id: FieldId(89),
        owner: ClassifierId::Relationship,
        name: "ownedRelatedElement",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RelationshipImpl.java",
            line: 81,
            sha256: "fbd56ac8a6e5792869844e2a91c8b55fa7aecd6cc2e04b3e5315382a457bbe3d",
        },
    },
    CachedFieldSpec {
        id: FieldId(90),
        owner: ClassifierId::Relationship,
        name: "source",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RelationshipImpl.java",
            line: 91,
            sha256: "fbd56ac8a6e5792869844e2a91c8b55fa7aecd6cc2e04b3e5315382a457bbe3d",
        },
    },
    CachedFieldSpec {
        id: FieldId(91),
        owner: ClassifierId::Relationship,
        name: "target",
        java_type: "EList<Element>",
        kind: FieldKind::LazyCollection,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RelationshipImpl.java",
            line: 101,
            sha256: "fbd56ac8a6e5792869844e2a91c8b55fa7aecd6cc2e04b3e5315382a457bbe3d",
        },
    },
    CachedFieldSpec {
        id: FieldId(92),
        owner: ClassifierId::RequirementConstraintMembership,
        name: "kind",
        java_type: "RequirementConstraintKind",
        kind: FieldKind::Scalar(ScalarKind::Enumeration(
            ClassifierId::RequirementConstraintKind,
        )),
        initial: DefaultValue::Enumeration {
            classifier: ClassifierId::RequirementConstraintKind,
            literal: "assumption",
        },
        default_constant: Some(60),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementConstraintMembershipImpl.java",
            line: 62,
            sha256: "96bd03d8e2490604aa1b474eb8a69427ac45c35b96af4157337f4f78221f3e45",
        },
    },
    CachedFieldSpec {
        id: FieldId(93),
        owner: ClassifierId::RequirementDefinition,
        name: "reqId",
        java_type: "String",
        kind: FieldKind::Scalar(ScalarKind::String),
        initial: DefaultValue::Null,
        default_constant: Some(61),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementDefinitionImpl.java",
            line: 79,
            sha256: "877901d3a8da850c15105ce9b633e45d825f52cdbeef41bf8b1db3ff4cacd917",
        },
    },
    CachedFieldSpec {
        id: FieldId(94),
        owner: ClassifierId::RequirementUsage,
        name: "reqId",
        java_type: "String",
        kind: FieldKind::Scalar(ScalarKind::String),
        initial: DefaultValue::Null,
        default_constant: Some(62),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementUsageImpl.java",
            line: 82,
            sha256: "e6e28cd8c181652e257b31369cf8e2697a628a1a2b2a8d73bdf70f36530accf2",
        },
    },
    CachedFieldSpec {
        id: FieldId(95),
        owner: ClassifierId::SatisfyRequirementUsage,
        name: "isNegated",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(63),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/SatisfyRequirementUsageImpl.java",
            line: 68,
            sha256: "f55cf0eddbbb8d5af72f925674eec7290bf065e0d728a6e96649cce0cb429de3",
        },
    },
    CachedFieldSpec {
        id: FieldId(96),
        owner: ClassifierId::Specialization,
        name: "general",
        java_type: "Type",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/SpecializationImpl.java",
            line: 52,
            sha256: "a86d15b8f7dfee3e9abe7d2c6b823f77f370aff018d7a49376bd17895d92fae1",
        },
    },
    CachedFieldSpec {
        id: FieldId(97),
        owner: ClassifierId::Specialization,
        name: "specific",
        java_type: "Type",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/SpecializationImpl.java",
            line: 62,
            sha256: "a86d15b8f7dfee3e9abe7d2c6b823f77f370aff018d7a49376bd17895d92fae1",
        },
    },
    CachedFieldSpec {
        id: FieldId(98),
        owner: ClassifierId::StateDefinition,
        name: "isParallel",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(64),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateDefinitionImpl.java",
            line: 67,
            sha256: "192b52b80e56b4816bf6b8bdf12478acb0f65223b897422a3014a95ae19bebf0",
        },
    },
    CachedFieldSpec {
        id: FieldId(99),
        owner: ClassifierId::StateSubactionMembership,
        name: "kind",
        java_type: "StateSubactionKind",
        kind: FieldKind::Scalar(ScalarKind::Enumeration(ClassifierId::StateSubactionKind)),
        initial: DefaultValue::Enumeration {
            classifier: ClassifierId::StateSubactionKind,
            literal: "do",
        },
        default_constant: Some(65),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateSubactionMembershipImpl.java",
            line: 57,
            sha256: "f362f4fdc765663f74f3a3ea0aa2645779d528e07cf8a3511520c4d4b4990644",
        },
    },
    CachedFieldSpec {
        id: FieldId(100),
        owner: ClassifierId::StateUsage,
        name: "isParallel",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(66),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateUsageImpl.java",
            line: 70,
            sha256: "235b477331ac3e3311e37b7a3ec1bebd2ace7bdc56ef0a30781f4213ff224865",
        },
    },
    CachedFieldSpec {
        id: FieldId(101),
        owner: ClassifierId::Subclassification,
        name: "subclassifier",
        java_type: "Classifier",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/SubclassificationImpl.java",
            line: 47,
            sha256: "12a4e1e07c8b3578fbc378ee46acb1ed6d3127408dad5af78a70c6dea6fffe5b",
        },
    },
    CachedFieldSpec {
        id: FieldId(102),
        owner: ClassifierId::Subclassification,
        name: "superclassifier",
        java_type: "Classifier",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/SubclassificationImpl.java",
            line: 57,
            sha256: "12a4e1e07c8b3578fbc378ee46acb1ed6d3127408dad5af78a70c6dea6fffe5b",
        },
    },
    CachedFieldSpec {
        id: FieldId(103),
        owner: ClassifierId::Subsetting,
        name: "subsettedFeature",
        java_type: "Feature",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/SubsettingImpl.java",
            line: 47,
            sha256: "d6a461caa225b6572394e43c6ece3e61ff034c6b3ef2a116287d08e31eba98d2",
        },
    },
    CachedFieldSpec {
        id: FieldId(104),
        owner: ClassifierId::Subsetting,
        name: "subsettingFeature",
        java_type: "Feature",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/SubsettingImpl.java",
            line: 57,
            sha256: "d6a461caa225b6572394e43c6ece3e61ff034c6b3ef2a116287d08e31eba98d2",
        },
    },
    CachedFieldSpec {
        id: FieldId(105),
        owner: ClassifierId::TextualRepresentation,
        name: "body",
        java_type: "String",
        kind: FieldKind::Scalar(ScalarKind::String),
        initial: DefaultValue::Null,
        default_constant: Some(67),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TextualRepresentationImpl.java",
            line: 60,
            sha256: "31d09a2b2639048566c9f56f04f7c12d53d107c5420effb2e4ff7d8ba5c52b3e",
        },
    },
    CachedFieldSpec {
        id: FieldId(106),
        owner: ClassifierId::TextualRepresentation,
        name: "language",
        java_type: "String",
        kind: FieldKind::Scalar(ScalarKind::String),
        initial: DefaultValue::Null,
        default_constant: Some(68),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TextualRepresentationImpl.java",
            line: 80,
            sha256: "31d09a2b2639048566c9f56f04f7c12d53d107c5420effb2e4ff7d8ba5c52b3e",
        },
    },
    CachedFieldSpec {
        id: FieldId(107),
        owner: ClassifierId::TransitionFeatureMembership,
        name: "kind",
        java_type: "TransitionFeatureKind",
        kind: FieldKind::Scalar(ScalarKind::Enumeration(ClassifierId::TransitionFeatureKind)),
        initial: DefaultValue::Enumeration {
            classifier: ClassifierId::TransitionFeatureKind,
            literal: "effect",
        },
        default_constant: Some(69),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TransitionFeatureMembershipImpl.java",
            line: 59,
            sha256: "a8cd905da455a039bfe81b0ead676c0167fc53ff78064de4872b8c1913d4115f",
        },
    },
    CachedFieldSpec {
        id: FieldId(108),
        owner: ClassifierId::TriggerInvocationExpression,
        name: "kind",
        java_type: "TriggerKind",
        kind: FieldKind::Scalar(ScalarKind::Enumeration(ClassifierId::TriggerKind)),
        initial: DefaultValue::Enumeration {
            classifier: ClassifierId::TriggerKind,
            literal: "after",
        },
        default_constant: Some(70),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TriggerInvocationExpressionImpl.java",
            line: 53,
            sha256: "35f26a38a2c17afcac3ed32d568179dff68854722f9270a3be6a165a0e7a102f",
        },
    },
    CachedFieldSpec {
        id: FieldId(109),
        owner: ClassifierId::Type,
        name: "isAbstract",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(71),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 109,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    CachedFieldSpec {
        id: FieldId(110),
        owner: ClassifierId::Type,
        name: "isSufficient",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(73),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 139,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    CachedFieldSpec {
        id: FieldId(111),
        owner: ClassifierId::TypeFeaturing,
        name: "featureOfType",
        java_type: "Feature",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeFeaturingImpl.java",
            line: 53,
            sha256: "9b69dc00c229566f7393b44be2ec3a2fcf3a66566d133d6248019116f7563d4c",
        },
    },
    CachedFieldSpec {
        id: FieldId(112),
        owner: ClassifierId::TypeFeaturing,
        name: "featuringType",
        java_type: "Type",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeFeaturingImpl.java",
            line: 63,
            sha256: "9b69dc00c229566f7393b44be2ec3a2fcf3a66566d133d6248019116f7563d4c",
        },
    },
    CachedFieldSpec {
        id: FieldId(113),
        owner: ClassifierId::Unioning,
        name: "unioningType",
        java_type: "Type",
        kind: FieldKind::Reference,
        initial: DefaultValue::Null,
        default_constant: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UnioningImpl.java",
            line: 50,
            sha256: "ce89aab6a74f140305f4d3f8caf3dd1e28e2abb65664572f9e351daa5d84c6b8",
        },
    },
    CachedFieldSpec {
        id: FieldId(114),
        owner: ClassifierId::Usage,
        name: "isVariation",
        java_type: "boolean",
        kind: FieldKind::Scalar(ScalarKind::Boolean),
        initial: DefaultValue::Boolean(false),
        default_constant: Some(75),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UsageImpl.java",
            line: 141,
            sha256: "88ff141769debfe9a8c9cf575b0ec05a289f267479af5800d6526fc470cabe26",
        },
    },
];
pub static CLASS_SPECS: &[RawClassSpec] = &[
    RawClassSpec {
        classifier: ClassifierId::AcceptActionUsage,
        java_parent: Some(ClassifierId::ActionUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(463),
    },
    RawClassSpec {
        classifier: ClassifierId::ActionDefinition,
        java_parent: Some(ClassifierId::OccurrenceDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(475),
    },
    RawClassSpec {
        classifier: ClassifierId::ActionUsage,
        java_parent: Some(ClassifierId::OccurrenceUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(487),
    },
    RawClassSpec {
        classifier: ClassifierId::ActorMembership,
        java_parent: Some(ClassifierId::ParameterMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(499),
    },
    RawClassSpec {
        classifier: ClassifierId::AllocationDefinition,
        java_parent: Some(ClassifierId::ConnectionDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(511),
    },
    RawClassSpec {
        classifier: ClassifierId::AllocationUsage,
        java_parent: Some(ClassifierId::ConnectionUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(523),
    },
    RawClassSpec {
        classifier: ClassifierId::AnalysisCaseDefinition,
        java_parent: Some(ClassifierId::CaseDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(535),
    },
    RawClassSpec {
        classifier: ClassifierId::AnalysisCaseUsage,
        java_parent: Some(ClassifierId::CaseUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(547),
    },
    RawClassSpec {
        classifier: ClassifierId::AnnotatingElement,
        java_parent: Some(ClassifierId::Element),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(559),
    },
    RawClassSpec {
        classifier: ClassifierId::Annotation,
        java_parent: Some(ClassifierId::Relationship),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(571),
    },
    RawClassSpec {
        classifier: ClassifierId::AssertConstraintUsage,
        java_parent: Some(ClassifierId::ConstraintUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(583),
    },
    RawClassSpec {
        classifier: ClassifierId::AssignmentActionUsage,
        java_parent: Some(ClassifierId::ActionUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(595),
    },
    RawClassSpec {
        classifier: ClassifierId::Association,
        java_parent: Some(ClassifierId::Classifier),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(607),
    },
    RawClassSpec {
        classifier: ClassifierId::AssociationStructure,
        java_parent: Some(ClassifierId::Association),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(619),
    },
    RawClassSpec {
        classifier: ClassifierId::AttributeDefinition,
        java_parent: Some(ClassifierId::Definition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(631),
    },
    RawClassSpec {
        classifier: ClassifierId::AttributeUsage,
        java_parent: Some(ClassifierId::Usage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(643),
    },
    RawClassSpec {
        classifier: ClassifierId::Behavior,
        java_parent: Some(ClassifierId::Class),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(655),
    },
    RawClassSpec {
        classifier: ClassifierId::BindingConnector,
        java_parent: Some(ClassifierId::Connector),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(667),
    },
    RawClassSpec {
        classifier: ClassifierId::BindingConnectorAsUsage,
        java_parent: Some(ClassifierId::ConnectorAsUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(679),
    },
    RawClassSpec {
        classifier: ClassifierId::BooleanExpression,
        java_parent: Some(ClassifierId::Expression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(691),
    },
    RawClassSpec {
        classifier: ClassifierId::CalculationDefinition,
        java_parent: Some(ClassifierId::ActionDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(703),
    },
    RawClassSpec {
        classifier: ClassifierId::CalculationUsage,
        java_parent: Some(ClassifierId::ActionUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(715),
    },
    RawClassSpec {
        classifier: ClassifierId::CaseDefinition,
        java_parent: Some(ClassifierId::CalculationDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(727),
    },
    RawClassSpec {
        classifier: ClassifierId::CaseUsage,
        java_parent: Some(ClassifierId::CalculationUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(739),
    },
    RawClassSpec {
        classifier: ClassifierId::Class,
        java_parent: Some(ClassifierId::Classifier),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(751),
    },
    RawClassSpec {
        classifier: ClassifierId::Classifier,
        java_parent: Some(ClassifierId::Type),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(763),
    },
    RawClassSpec {
        classifier: ClassifierId::CollectExpression,
        java_parent: Some(ClassifierId::OperatorExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(775),
    },
    RawClassSpec {
        classifier: ClassifierId::Comment,
        java_parent: Some(ClassifierId::AnnotatingElement),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(787),
    },
    RawClassSpec {
        classifier: ClassifierId::ConcernDefinition,
        java_parent: Some(ClassifierId::RequirementDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(799),
    },
    RawClassSpec {
        classifier: ClassifierId::ConcernUsage,
        java_parent: Some(ClassifierId::RequirementUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(811),
    },
    RawClassSpec {
        classifier: ClassifierId::ConjugatedPortDefinition,
        java_parent: Some(ClassifierId::PortDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(823),
    },
    RawClassSpec {
        classifier: ClassifierId::ConjugatedPortTyping,
        java_parent: Some(ClassifierId::FeatureTyping),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(835),
    },
    RawClassSpec {
        classifier: ClassifierId::Conjugation,
        java_parent: Some(ClassifierId::Relationship),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(847),
    },
    RawClassSpec {
        classifier: ClassifierId::ConnectionDefinition,
        java_parent: Some(ClassifierId::PartDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(859),
    },
    RawClassSpec {
        classifier: ClassifierId::ConnectionUsage,
        java_parent: Some(ClassifierId::ConnectorAsUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(871),
    },
    RawClassSpec {
        classifier: ClassifierId::Connector,
        java_parent: Some(ClassifierId::Feature),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(883),
    },
    RawClassSpec {
        classifier: ClassifierId::ConnectorAsUsage,
        java_parent: Some(ClassifierId::Usage),
        external_parent: None,
        raw_constructible: false,
        factory_line: None,
    },
    RawClassSpec {
        classifier: ClassifierId::ConstraintDefinition,
        java_parent: Some(ClassifierId::OccurrenceDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(895),
    },
    RawClassSpec {
        classifier: ClassifierId::ConstraintUsage,
        java_parent: Some(ClassifierId::OccurrenceUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(907),
    },
    RawClassSpec {
        classifier: ClassifierId::ConstructorExpression,
        java_parent: Some(ClassifierId::InstantiationExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(919),
    },
    RawClassSpec {
        classifier: ClassifierId::ControlNode,
        java_parent: Some(ClassifierId::ActionUsage),
        external_parent: None,
        raw_constructible: false,
        factory_line: None,
    },
    RawClassSpec {
        classifier: ClassifierId::CrossSubsetting,
        java_parent: Some(ClassifierId::Subsetting),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(931),
    },
    RawClassSpec {
        classifier: ClassifierId::DataType,
        java_parent: Some(ClassifierId::Classifier),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(943),
    },
    RawClassSpec {
        classifier: ClassifierId::DecisionNode,
        java_parent: Some(ClassifierId::ControlNode),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(955),
    },
    RawClassSpec {
        classifier: ClassifierId::Definition,
        java_parent: Some(ClassifierId::Classifier),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(967),
    },
    RawClassSpec {
        classifier: ClassifierId::Dependency,
        java_parent: Some(ClassifierId::Relationship),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(979),
    },
    RawClassSpec {
        classifier: ClassifierId::Differencing,
        java_parent: Some(ClassifierId::Relationship),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(991),
    },
    RawClassSpec {
        classifier: ClassifierId::Disjoining,
        java_parent: Some(ClassifierId::Relationship),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1003),
    },
    RawClassSpec {
        classifier: ClassifierId::Documentation,
        java_parent: Some(ClassifierId::Comment),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1015),
    },
    RawClassSpec {
        classifier: ClassifierId::Element,
        java_parent: None,
        external_parent: Some("EModelElementImpl"),
        raw_constructible: false,
        factory_line: None,
    },
    RawClassSpec {
        classifier: ClassifierId::ElementFilterMembership,
        java_parent: Some(ClassifierId::OwningMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1027),
    },
    RawClassSpec {
        classifier: ClassifierId::EndFeatureMembership,
        java_parent: Some(ClassifierId::FeatureMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1039),
    },
    RawClassSpec {
        classifier: ClassifierId::EnumerationDefinition,
        java_parent: Some(ClassifierId::AttributeDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1051),
    },
    RawClassSpec {
        classifier: ClassifierId::EnumerationUsage,
        java_parent: Some(ClassifierId::AttributeUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1063),
    },
    RawClassSpec {
        classifier: ClassifierId::EventOccurrenceUsage,
        java_parent: Some(ClassifierId::OccurrenceUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1075),
    },
    RawClassSpec {
        classifier: ClassifierId::ExhibitStateUsage,
        java_parent: Some(ClassifierId::StateUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1087),
    },
    RawClassSpec {
        classifier: ClassifierId::Expose,
        java_parent: Some(ClassifierId::Import),
        external_parent: None,
        raw_constructible: false,
        factory_line: None,
    },
    RawClassSpec {
        classifier: ClassifierId::Expression,
        java_parent: Some(ClassifierId::Step),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1099),
    },
    RawClassSpec {
        classifier: ClassifierId::Feature,
        java_parent: Some(ClassifierId::Type),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1111),
    },
    RawClassSpec {
        classifier: ClassifierId::FeatureChainExpression,
        java_parent: Some(ClassifierId::OperatorExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1123),
    },
    RawClassSpec {
        classifier: ClassifierId::FeatureChaining,
        java_parent: Some(ClassifierId::Relationship),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1135),
    },
    RawClassSpec {
        classifier: ClassifierId::FeatureInverting,
        java_parent: Some(ClassifierId::Relationship),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1147),
    },
    RawClassSpec {
        classifier: ClassifierId::FeatureMembership,
        java_parent: Some(ClassifierId::OwningMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1159),
    },
    RawClassSpec {
        classifier: ClassifierId::FeatureReferenceExpression,
        java_parent: Some(ClassifierId::Expression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1171),
    },
    RawClassSpec {
        classifier: ClassifierId::FeatureTyping,
        java_parent: Some(ClassifierId::Specialization),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1183),
    },
    RawClassSpec {
        classifier: ClassifierId::FeatureValue,
        java_parent: Some(ClassifierId::OwningMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1195),
    },
    RawClassSpec {
        classifier: ClassifierId::Flow,
        java_parent: Some(ClassifierId::Connector),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1207),
    },
    RawClassSpec {
        classifier: ClassifierId::FlowDefinition,
        java_parent: Some(ClassifierId::ActionDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1219),
    },
    RawClassSpec {
        classifier: ClassifierId::FlowEnd,
        java_parent: Some(ClassifierId::Feature),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1231),
    },
    RawClassSpec {
        classifier: ClassifierId::FlowUsage,
        java_parent: Some(ClassifierId::ConnectorAsUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1243),
    },
    RawClassSpec {
        classifier: ClassifierId::ForkNode,
        java_parent: Some(ClassifierId::ControlNode),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1255),
    },
    RawClassSpec {
        classifier: ClassifierId::ForLoopActionUsage,
        java_parent: Some(ClassifierId::LoopActionUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1267),
    },
    RawClassSpec {
        classifier: ClassifierId::FramedConcernMembership,
        java_parent: Some(ClassifierId::RequirementConstraintMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1279),
    },
    RawClassSpec {
        classifier: ClassifierId::Function,
        java_parent: Some(ClassifierId::Behavior),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1291),
    },
    RawClassSpec {
        classifier: ClassifierId::IfActionUsage,
        java_parent: Some(ClassifierId::ActionUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1303),
    },
    RawClassSpec {
        classifier: ClassifierId::Import,
        java_parent: Some(ClassifierId::Relationship),
        external_parent: None,
        raw_constructible: false,
        factory_line: None,
    },
    RawClassSpec {
        classifier: ClassifierId::IncludeUseCaseUsage,
        java_parent: Some(ClassifierId::UseCaseUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1315),
    },
    RawClassSpec {
        classifier: ClassifierId::IndexExpression,
        java_parent: Some(ClassifierId::OperatorExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1327),
    },
    RawClassSpec {
        classifier: ClassifierId::InstantiationExpression,
        java_parent: Some(ClassifierId::Expression),
        external_parent: None,
        raw_constructible: false,
        factory_line: None,
    },
    RawClassSpec {
        classifier: ClassifierId::Interaction,
        java_parent: Some(ClassifierId::Association),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1339),
    },
    RawClassSpec {
        classifier: ClassifierId::InterfaceDefinition,
        java_parent: Some(ClassifierId::ConnectionDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1351),
    },
    RawClassSpec {
        classifier: ClassifierId::InterfaceUsage,
        java_parent: Some(ClassifierId::ConnectionUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1363),
    },
    RawClassSpec {
        classifier: ClassifierId::Intersecting,
        java_parent: Some(ClassifierId::Relationship),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1375),
    },
    RawClassSpec {
        classifier: ClassifierId::Invariant,
        java_parent: Some(ClassifierId::BooleanExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1387),
    },
    RawClassSpec {
        classifier: ClassifierId::InvocationExpression,
        java_parent: Some(ClassifierId::InstantiationExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1399),
    },
    RawClassSpec {
        classifier: ClassifierId::ItemDefinition,
        java_parent: Some(ClassifierId::OccurrenceDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1411),
    },
    RawClassSpec {
        classifier: ClassifierId::ItemUsage,
        java_parent: Some(ClassifierId::OccurrenceUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1423),
    },
    RawClassSpec {
        classifier: ClassifierId::JoinNode,
        java_parent: Some(ClassifierId::ControlNode),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1435),
    },
    RawClassSpec {
        classifier: ClassifierId::LibraryPackage,
        java_parent: Some(ClassifierId::Package),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1447),
    },
    RawClassSpec {
        classifier: ClassifierId::LiteralBoolean,
        java_parent: Some(ClassifierId::LiteralExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1459),
    },
    RawClassSpec {
        classifier: ClassifierId::LiteralExpression,
        java_parent: Some(ClassifierId::Expression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1471),
    },
    RawClassSpec {
        classifier: ClassifierId::LiteralInfinity,
        java_parent: Some(ClassifierId::LiteralExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1483),
    },
    RawClassSpec {
        classifier: ClassifierId::LiteralInteger,
        java_parent: Some(ClassifierId::LiteralExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1495),
    },
    RawClassSpec {
        classifier: ClassifierId::LiteralRational,
        java_parent: Some(ClassifierId::LiteralExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1507),
    },
    RawClassSpec {
        classifier: ClassifierId::LiteralString,
        java_parent: Some(ClassifierId::LiteralExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1519),
    },
    RawClassSpec {
        classifier: ClassifierId::LoopActionUsage,
        java_parent: Some(ClassifierId::ActionUsage),
        external_parent: None,
        raw_constructible: false,
        factory_line: None,
    },
    RawClassSpec {
        classifier: ClassifierId::Membership,
        java_parent: Some(ClassifierId::Relationship),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1531),
    },
    RawClassSpec {
        classifier: ClassifierId::MembershipExpose,
        java_parent: Some(ClassifierId::MembershipImport),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1543),
    },
    RawClassSpec {
        classifier: ClassifierId::MembershipImport,
        java_parent: Some(ClassifierId::Import),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1555),
    },
    RawClassSpec {
        classifier: ClassifierId::MergeNode,
        java_parent: Some(ClassifierId::ControlNode),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1567),
    },
    RawClassSpec {
        classifier: ClassifierId::Metaclass,
        java_parent: Some(ClassifierId::Structure),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1579),
    },
    RawClassSpec {
        classifier: ClassifierId::MetadataAccessExpression,
        java_parent: Some(ClassifierId::Expression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1591),
    },
    RawClassSpec {
        classifier: ClassifierId::MetadataDefinition,
        java_parent: Some(ClassifierId::ItemDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1603),
    },
    RawClassSpec {
        classifier: ClassifierId::MetadataFeature,
        java_parent: Some(ClassifierId::Feature),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1615),
    },
    RawClassSpec {
        classifier: ClassifierId::MetadataUsage,
        java_parent: Some(ClassifierId::ItemUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1627),
    },
    RawClassSpec {
        classifier: ClassifierId::Multiplicity,
        java_parent: Some(ClassifierId::Feature),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1639),
    },
    RawClassSpec {
        classifier: ClassifierId::MultiplicityRange,
        java_parent: Some(ClassifierId::Multiplicity),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1651),
    },
    RawClassSpec {
        classifier: ClassifierId::Namespace,
        java_parent: Some(ClassifierId::Element),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1663),
    },
    RawClassSpec {
        classifier: ClassifierId::NamespaceExpose,
        java_parent: Some(ClassifierId::NamespaceImport),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1675),
    },
    RawClassSpec {
        classifier: ClassifierId::NamespaceImport,
        java_parent: Some(ClassifierId::Import),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1687),
    },
    RawClassSpec {
        classifier: ClassifierId::NullExpression,
        java_parent: Some(ClassifierId::Expression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1699),
    },
    RawClassSpec {
        classifier: ClassifierId::ObjectiveMembership,
        java_parent: Some(ClassifierId::FeatureMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1711),
    },
    RawClassSpec {
        classifier: ClassifierId::OccurrenceDefinition,
        java_parent: Some(ClassifierId::Definition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1723),
    },
    RawClassSpec {
        classifier: ClassifierId::OccurrenceUsage,
        java_parent: Some(ClassifierId::Usage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1735),
    },
    RawClassSpec {
        classifier: ClassifierId::OperatorExpression,
        java_parent: Some(ClassifierId::InvocationExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1747),
    },
    RawClassSpec {
        classifier: ClassifierId::OwningMembership,
        java_parent: Some(ClassifierId::Membership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1759),
    },
    RawClassSpec {
        classifier: ClassifierId::Package,
        java_parent: Some(ClassifierId::Namespace),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1771),
    },
    RawClassSpec {
        classifier: ClassifierId::ParameterMembership,
        java_parent: Some(ClassifierId::FeatureMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1783),
    },
    RawClassSpec {
        classifier: ClassifierId::PartDefinition,
        java_parent: Some(ClassifierId::ItemDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1795),
    },
    RawClassSpec {
        classifier: ClassifierId::PartUsage,
        java_parent: Some(ClassifierId::ItemUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1807),
    },
    RawClassSpec {
        classifier: ClassifierId::PayloadFeature,
        java_parent: Some(ClassifierId::Feature),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1819),
    },
    RawClassSpec {
        classifier: ClassifierId::PerformActionUsage,
        java_parent: Some(ClassifierId::ActionUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1831),
    },
    RawClassSpec {
        classifier: ClassifierId::PortConjugation,
        java_parent: Some(ClassifierId::Conjugation),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1843),
    },
    RawClassSpec {
        classifier: ClassifierId::PortDefinition,
        java_parent: Some(ClassifierId::OccurrenceDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1855),
    },
    RawClassSpec {
        classifier: ClassifierId::PortUsage,
        java_parent: Some(ClassifierId::OccurrenceUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1867),
    },
    RawClassSpec {
        classifier: ClassifierId::Predicate,
        java_parent: Some(ClassifierId::Function),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1879),
    },
    RawClassSpec {
        classifier: ClassifierId::Redefinition,
        java_parent: Some(ClassifierId::Subsetting),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1891),
    },
    RawClassSpec {
        classifier: ClassifierId::ReferenceSubsetting,
        java_parent: Some(ClassifierId::Subsetting),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1903),
    },
    RawClassSpec {
        classifier: ClassifierId::ReferenceUsage,
        java_parent: Some(ClassifierId::Usage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1915),
    },
    RawClassSpec {
        classifier: ClassifierId::Relationship,
        java_parent: Some(ClassifierId::Element),
        external_parent: None,
        raw_constructible: false,
        factory_line: None,
    },
    RawClassSpec {
        classifier: ClassifierId::RenderingDefinition,
        java_parent: Some(ClassifierId::PartDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1927),
    },
    RawClassSpec {
        classifier: ClassifierId::RenderingUsage,
        java_parent: Some(ClassifierId::PartUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1939),
    },
    RawClassSpec {
        classifier: ClassifierId::RequirementConstraintMembership,
        java_parent: Some(ClassifierId::FeatureMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1951),
    },
    RawClassSpec {
        classifier: ClassifierId::RequirementDefinition,
        java_parent: Some(ClassifierId::ConstraintDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1963),
    },
    RawClassSpec {
        classifier: ClassifierId::RequirementUsage,
        java_parent: Some(ClassifierId::ConstraintUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1975),
    },
    RawClassSpec {
        classifier: ClassifierId::RequirementVerificationMembership,
        java_parent: Some(ClassifierId::RequirementConstraintMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1987),
    },
    RawClassSpec {
        classifier: ClassifierId::ResultExpressionMembership,
        java_parent: Some(ClassifierId::FeatureMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(1999),
    },
    RawClassSpec {
        classifier: ClassifierId::ReturnParameterMembership,
        java_parent: Some(ClassifierId::ParameterMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2011),
    },
    RawClassSpec {
        classifier: ClassifierId::SatisfyRequirementUsage,
        java_parent: Some(ClassifierId::RequirementUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2023),
    },
    RawClassSpec {
        classifier: ClassifierId::SelectExpression,
        java_parent: Some(ClassifierId::OperatorExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2035),
    },
    RawClassSpec {
        classifier: ClassifierId::SendActionUsage,
        java_parent: Some(ClassifierId::ActionUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2047),
    },
    RawClassSpec {
        classifier: ClassifierId::Specialization,
        java_parent: Some(ClassifierId::Relationship),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2059),
    },
    RawClassSpec {
        classifier: ClassifierId::StakeholderMembership,
        java_parent: Some(ClassifierId::ParameterMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2071),
    },
    RawClassSpec {
        classifier: ClassifierId::StateDefinition,
        java_parent: Some(ClassifierId::ActionDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2083),
    },
    RawClassSpec {
        classifier: ClassifierId::StateSubactionMembership,
        java_parent: Some(ClassifierId::FeatureMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2095),
    },
    RawClassSpec {
        classifier: ClassifierId::StateUsage,
        java_parent: Some(ClassifierId::ActionUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2107),
    },
    RawClassSpec {
        classifier: ClassifierId::Step,
        java_parent: Some(ClassifierId::Feature),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2119),
    },
    RawClassSpec {
        classifier: ClassifierId::Structure,
        java_parent: Some(ClassifierId::Class),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2131),
    },
    RawClassSpec {
        classifier: ClassifierId::Subclassification,
        java_parent: Some(ClassifierId::Specialization),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2143),
    },
    RawClassSpec {
        classifier: ClassifierId::SubjectMembership,
        java_parent: Some(ClassifierId::ParameterMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2155),
    },
    RawClassSpec {
        classifier: ClassifierId::Subsetting,
        java_parent: Some(ClassifierId::Specialization),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2167),
    },
    RawClassSpec {
        classifier: ClassifierId::Succession,
        java_parent: Some(ClassifierId::Connector),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2179),
    },
    RawClassSpec {
        classifier: ClassifierId::SuccessionAsUsage,
        java_parent: Some(ClassifierId::ConnectorAsUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2191),
    },
    RawClassSpec {
        classifier: ClassifierId::SuccessionFlow,
        java_parent: Some(ClassifierId::Flow),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2203),
    },
    RawClassSpec {
        classifier: ClassifierId::SuccessionFlowUsage,
        java_parent: Some(ClassifierId::FlowUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2215),
    },
    RawClassSpec {
        classifier: ClassifierId::TerminateActionUsage,
        java_parent: Some(ClassifierId::ActionUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2227),
    },
    RawClassSpec {
        classifier: ClassifierId::TextualRepresentation,
        java_parent: Some(ClassifierId::AnnotatingElement),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2239),
    },
    RawClassSpec {
        classifier: ClassifierId::TransitionFeatureMembership,
        java_parent: Some(ClassifierId::FeatureMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2251),
    },
    RawClassSpec {
        classifier: ClassifierId::TransitionUsage,
        java_parent: Some(ClassifierId::ActionUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2263),
    },
    RawClassSpec {
        classifier: ClassifierId::TriggerInvocationExpression,
        java_parent: Some(ClassifierId::InvocationExpression),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2275),
    },
    RawClassSpec {
        classifier: ClassifierId::Type,
        java_parent: Some(ClassifierId::Namespace),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2287),
    },
    RawClassSpec {
        classifier: ClassifierId::TypeFeaturing,
        java_parent: Some(ClassifierId::Relationship),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2299),
    },
    RawClassSpec {
        classifier: ClassifierId::Unioning,
        java_parent: Some(ClassifierId::Relationship),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2311),
    },
    RawClassSpec {
        classifier: ClassifierId::Usage,
        java_parent: Some(ClassifierId::Feature),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2323),
    },
    RawClassSpec {
        classifier: ClassifierId::UseCaseDefinition,
        java_parent: Some(ClassifierId::CaseDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2335),
    },
    RawClassSpec {
        classifier: ClassifierId::UseCaseUsage,
        java_parent: Some(ClassifierId::CaseUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2347),
    },
    RawClassSpec {
        classifier: ClassifierId::VariantMembership,
        java_parent: Some(ClassifierId::OwningMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2359),
    },
    RawClassSpec {
        classifier: ClassifierId::VerificationCaseDefinition,
        java_parent: Some(ClassifierId::CaseDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2371),
    },
    RawClassSpec {
        classifier: ClassifierId::VerificationCaseUsage,
        java_parent: Some(ClassifierId::CaseUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2383),
    },
    RawClassSpec {
        classifier: ClassifierId::ViewDefinition,
        java_parent: Some(ClassifierId::PartDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2395),
    },
    RawClassSpec {
        classifier: ClassifierId::ViewpointDefinition,
        java_parent: Some(ClassifierId::RequirementDefinition),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2407),
    },
    RawClassSpec {
        classifier: ClassifierId::ViewpointUsage,
        java_parent: Some(ClassifierId::RequirementUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2419),
    },
    RawClassSpec {
        classifier: ClassifierId::ViewRenderingMembership,
        java_parent: Some(ClassifierId::FeatureMembership),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2431),
    },
    RawClassSpec {
        classifier: ClassifierId::ViewUsage,
        java_parent: Some(ClassifierId::PartUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2443),
    },
    RawClassSpec {
        classifier: ClassifierId::WhileLoopActionUsage,
        java_parent: Some(ClassifierId::LoopActionUsage),
        external_parent: None,
        raw_constructible: true,
        factory_line: Some(2455),
    },
];
pub static GETTERS: &[GetterSpec] = &[
    GetterSpec {
        owner: ClassifierId::AssertConstraintUsage,
        attribute: "isNegated",
        method: "isIsNegated",
        behavior: GetterBehavior::Cached(FieldId(1)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssertConstraintUsageImpl.java",
            line: 84,
            sha256: "26a68637a08cdce51f972c837fd8cb151cfe303be9a90691a96d06676cda3454",
        },
    },
    GetterSpec {
        owner: ClassifierId::Association,
        attribute: "isImplied",
        method: "isIsImplied",
        behavior: GetterBehavior::Cached(FieldId(2)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssociationImpl.java",
            line: 134,
            sha256: "aaaf65656b531086ac47abfe06dc0f6715c3a34cb459b4f57f171b8d6089c8d7",
        },
    },
    GetterSpec {
        owner: ClassifierId::AttributeUsage,
        attribute: "isReference",
        method: "isIsReference",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AttributeUsageImpl.java",
            line: 78,
            sha256: "8c7ea5714920d5614e5c2973b750ebe1fea6770c7a8b731b052b0aa88b48e0b2",
        },
    },
    GetterSpec {
        owner: ClassifierId::CalculationDefinition,
        attribute: "isModelLevelEvaluable",
        method: "isIsModelLevelEvaluable",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CalculationDefinitionImpl.java",
            line: 93,
            sha256: "1a14004d79980eeb6ecb677674ef9e64cfbdf807e674cc90daa468b2dc4f5822",
        },
    },
    GetterSpec {
        owner: ClassifierId::CalculationUsage,
        attribute: "isModelLevelEvaluable",
        method: "isIsModelLevelEvaluable",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CalculationUsageImpl.java",
            line: 104,
            sha256: "0357c1e27ba49ef525da2dd45530ad1841b0cbccdd28143f72513d791c10f0c6",
        },
    },
    GetterSpec {
        owner: ClassifierId::Comment,
        attribute: "body",
        method: "getBody",
        behavior: GetterBehavior::Cached(FieldId(6)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CommentImpl.java",
            line: 99,
            sha256: "4f2913fe8128a200498793edfd90c594f19169854dada56ce0b5ff36842f8bf8",
        },
    },
    GetterSpec {
        owner: ClassifierId::Comment,
        attribute: "locale",
        method: "getLocale",
        behavior: GetterBehavior::Cached(FieldId(7)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CommentImpl.java",
            line: 123,
            sha256: "4f2913fe8128a200498793edfd90c594f19169854dada56ce0b5ff36842f8bf8",
        },
    },
    GetterSpec {
        owner: ClassifierId::ConjugatedPortDefinition,
        attribute: "isConjugated",
        method: "isIsConjugated",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConjugatedPortDefinitionImpl.java",
            line: 160,
            sha256: "5899f7a6a681b11a80beeef46c1b24d0f84afd5e7a1c039af3b4fcff9e88535f",
        },
    },
    GetterSpec {
        owner: ClassifierId::ConnectionDefinition,
        attribute: "isImplied",
        method: "isIsImplied",
        behavior: GetterBehavior::Cached(FieldId(11)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionDefinitionImpl.java",
            line: 142,
            sha256: "99aac0b55d655179e84f0dd8419111616a91bf2eb0b7ab8ab880acf063813352",
        },
    },
    GetterSpec {
        owner: ClassifierId::ConnectionUsage,
        attribute: "isIndividual",
        method: "isIsIndividual",
        behavior: GetterBehavior::Cached(FieldId(15)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 158,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    GetterSpec {
        owner: ClassifierId::ConnectionUsage,
        attribute: "portionKind",
        method: "getPortionKind",
        behavior: GetterBehavior::Cached(FieldId(16)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 197,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    GetterSpec {
        owner: ClassifierId::Connector,
        attribute: "isImplied",
        method: "isIsImplied",
        behavior: GetterBehavior::Cached(FieldId(18)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorImpl.java",
            line: 139,
            sha256: "1f49aa6b7e11ffec691b129403fadbc7dc1e179855b7ae9f7303dc84d11d8528",
        },
    },
    GetterSpec {
        owner: ClassifierId::ConnectorAsUsage,
        attribute: "isImplied",
        method: "isIsImplied",
        behavior: GetterBehavior::Cached(FieldId(22)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorAsUsageImpl.java",
            line: 143,
            sha256: "9a34426f1fe469817000d26eb0399ee29991838e1bb45501f8477fe4dc084857",
        },
    },
    GetterSpec {
        owner: ClassifierId::ConstraintDefinition,
        attribute: "isModelLevelEvaluable",
        method: "isIsModelLevelEvaluable",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConstraintDefinitionImpl.java",
            line: 125,
            sha256: "67edcc045820f764f1c47e318cbf0ecac46f3416f7291aa7b2fef3e89b51373c",
        },
    },
    GetterSpec {
        owner: ClassifierId::ConstraintUsage,
        attribute: "isModelLevelEvaluable",
        method: "isIsModelLevelEvaluable",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConstraintUsageImpl.java",
            line: 153,
            sha256: "9a782d4050e58d8bdcf061bfea5fca441039b9b94a5469922e29b45bc82d5a6d",
        },
    },
    GetterSpec {
        owner: ClassifierId::Definition,
        attribute: "isVariation",
        method: "isIsVariation",
        behavior: GetterBehavior::Cached(FieldId(27)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/DefinitionImpl.java",
            line: 157,
            sha256: "b8c1f6391eb0fd2b022ece2e8ea239413a2b6b562ee7209f80fc1a35fa9145ec",
        },
    },
    GetterSpec {
        owner: ClassifierId::Element,
        attribute: "aliasIds",
        method: "getAliasIds",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 237,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    GetterSpec {
        owner: ClassifierId::Element,
        attribute: "declaredName",
        method: "getDeclaredName",
        behavior: GetterBehavior::Cached(FieldId(34)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 250,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    GetterSpec {
        owner: ClassifierId::Element,
        attribute: "declaredShortName",
        method: "getDeclaredShortName",
        behavior: GetterBehavior::Cached(FieldId(35)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 274,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    GetterSpec {
        owner: ClassifierId::Element,
        attribute: "elementId",
        method: "getElementId",
        behavior: GetterBehavior::Cached(FieldId(36)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 313,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    GetterSpec {
        owner: ClassifierId::Element,
        attribute: "isImpliedIncluded",
        method: "isIsImpliedIncluded",
        behavior: GetterBehavior::Cached(FieldId(37)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 337,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    GetterSpec {
        owner: ClassifierId::Element,
        attribute: "isLibraryElement",
        method: "isIsLibraryElement",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 361,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    GetterSpec {
        owner: ClassifierId::Element,
        attribute: "name",
        method: "getName",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 371,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    GetterSpec {
        owner: ClassifierId::Element,
        attribute: "qualifiedName",
        method: "getQualifiedName",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 568,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    GetterSpec {
        owner: ClassifierId::Element,
        attribute: "shortName",
        method: "getShortName",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 618,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    GetterSpec {
        owner: ClassifierId::EventOccurrenceUsage,
        attribute: "isReference",
        method: "isIsReference",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/EventOccurrenceUsageImpl.java",
            line: 84,
            sha256: "5d258e8f52a3da5e8628c3e1db55ff095e489bde840ed91c01b19634f97ae8eb",
        },
    },
    GetterSpec {
        owner: ClassifierId::Expression,
        attribute: "isModelLevelEvaluable",
        method: "isIsModelLevelEvaluable",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ExpressionImpl.java",
            line: 105,
            sha256: "bbbaec6f943ed89c79ce7a58c30ad5d895bd3e7c356e1b04a584639292dbf15f",
        },
    },
    GetterSpec {
        owner: ClassifierId::Feature,
        attribute: "direction",
        method: "getDirection",
        behavior: GetterBehavior::Cached(FieldId(39)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 361,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    GetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isComposite",
        method: "isIsComposite",
        behavior: GetterBehavior::Cached(FieldId(41)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 509,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    GetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isConstant",
        method: "isIsConstant",
        behavior: GetterBehavior::Cached(FieldId(42)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 533,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    GetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isDerived",
        method: "isIsDerived",
        behavior: GetterBehavior::Cached(FieldId(43)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 557,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    GetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isEnd",
        method: "isIsEnd",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 581,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    GetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isNonunique",
        method: "isIsNonunique",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 609,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    GetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isOrdered",
        method: "isIsOrdered",
        behavior: GetterBehavior::Cached(FieldId(45)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 619,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    GetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isPortion",
        method: "isIsPortion",
        behavior: GetterBehavior::Cached(FieldId(46)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 643,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    GetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isUnique",
        method: "isIsUnique",
        behavior: GetterBehavior::Cached(FieldId(47)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 667,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    GetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isVariable",
        method: "isIsVariable",
        behavior: GetterBehavior::Cached(FieldId(48)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 691,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    GetterSpec {
        owner: ClassifierId::FeatureMembership,
        attribute: "memberName",
        method: "getMemberName",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureMembershipImpl.java",
            line: 116,
            sha256: "f69d795fe211675cddb22ceacc399d5dc4b6b9c8c3c0e2f9a3fe9b0de96c7dfc",
        },
    },
    GetterSpec {
        owner: ClassifierId::FeatureMembership,
        attribute: "memberShortName",
        method: "getMemberShortName",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureMembershipImpl.java",
            line: 126,
            sha256: "f69d795fe211675cddb22ceacc399d5dc4b6b9c8c3c0e2f9a3fe9b0de96c7dfc",
        },
    },
    GetterSpec {
        owner: ClassifierId::FeatureValue,
        attribute: "isDefault",
        method: "isIsDefault",
        behavior: GetterBehavior::Cached(FieldId(54)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureValueImpl.java",
            line: 131,
            sha256: "06d158252791fccd475b35f55547369dd63bd92d6b27a2649e0d736553a66cc7",
        },
    },
    GetterSpec {
        owner: ClassifierId::FeatureValue,
        attribute: "isInitial",
        method: "isIsInitial",
        behavior: GetterBehavior::Cached(FieldId(55)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureValueImpl.java",
            line: 155,
            sha256: "06d158252791fccd475b35f55547369dd63bd92d6b27a2649e0d736553a66cc7",
        },
    },
    GetterSpec {
        owner: ClassifierId::FlowDefinition,
        attribute: "isImplied",
        method: "isIsImplied",
        behavior: GetterBehavior::Cached(FieldId(56)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowDefinitionImpl.java",
            line: 142,
            sha256: "244d80d8b39947bd40618e40af85847d44035c8220b238658960e4e1082b01ed",
        },
    },
    GetterSpec {
        owner: ClassifierId::FlowUsage,
        attribute: "isIndividual",
        method: "isIsIndividual",
        behavior: GetterBehavior::Cached(FieldId(60)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 147,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    GetterSpec {
        owner: ClassifierId::FlowUsage,
        attribute: "portionKind",
        method: "getPortionKind",
        behavior: GetterBehavior::Cached(FieldId(61)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 171,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    GetterSpec {
        owner: ClassifierId::Function,
        attribute: "isModelLevelEvaluable",
        method: "isIsModelLevelEvaluable",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FunctionImpl.java",
            line: 90,
            sha256: "8a5dbc66196a21a04c260a3bda8a363aebff5bdff9d3dcce91fff73f41024434",
        },
    },
    GetterSpec {
        owner: ClassifierId::Import,
        attribute: "isImportAll",
        method: "isIsImportAll",
        behavior: GetterBehavior::Cached(FieldId(63)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 177,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    GetterSpec {
        owner: ClassifierId::Import,
        attribute: "isRecursive",
        method: "isIsRecursive",
        behavior: GetterBehavior::Cached(FieldId(64)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 201,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    GetterSpec {
        owner: ClassifierId::Import,
        attribute: "visibility",
        method: "getVisibility",
        behavior: GetterBehavior::Cached(FieldId(65)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 225,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    GetterSpec {
        owner: ClassifierId::Invariant,
        attribute: "isNegated",
        method: "isIsNegated",
        behavior: GetterBehavior::Cached(FieldId(67)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/InvariantImpl.java",
            line: 78,
            sha256: "1e5440a1324cb70f52d4a2bb313ac4ade34e1ac0cd608c5e720e3d424a09bbdf",
        },
    },
    GetterSpec {
        owner: ClassifierId::LibraryPackage,
        attribute: "isStandard",
        method: "isIsStandard",
        behavior: GetterBehavior::Cached(FieldId(68)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LibraryPackageImpl.java",
            line: 80,
            sha256: "9103340eb814396cb02705d14b9062fa660b0577879c31538eae5534a552e99f",
        },
    },
    GetterSpec {
        owner: ClassifierId::LiteralBoolean,
        attribute: "value",
        method: "isValue",
        behavior: GetterBehavior::Cached(FieldId(69)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralBooleanImpl.java",
            line: 79,
            sha256: "1cdf2c81a78ab9a5ae2437a4cfe959aa31ee9f37107949638abb3d1fb70d52d2",
        },
    },
    GetterSpec {
        owner: ClassifierId::LiteralInteger,
        attribute: "value",
        method: "getValue",
        behavior: GetterBehavior::Cached(FieldId(70)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralIntegerImpl.java",
            line: 79,
            sha256: "ea23684de620c444cd6cd43c5562e6c4f1a180d9f8cfb02ed58284134d06f96c",
        },
    },
    GetterSpec {
        owner: ClassifierId::LiteralRational,
        attribute: "value",
        method: "getValue",
        behavior: GetterBehavior::Cached(FieldId(71)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralRationalImpl.java",
            line: 79,
            sha256: "0e99ac7f50b7d612c5f1450d76195d8b5f4c8f0b3ef7db87ab94b82145f8d632",
        },
    },
    GetterSpec {
        owner: ClassifierId::LiteralString,
        attribute: "value",
        method: "getValue",
        behavior: GetterBehavior::Cached(FieldId(72)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralStringImpl.java",
            line: 78,
            sha256: "0f51f25d8b70036c25befc883f7dc4e88e3b63b60555d3b8ecd633a3b4f1441f",
        },
    },
    GetterSpec {
        owner: ClassifierId::Membership,
        attribute: "memberElementId",
        method: "getMemberElementId",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 199,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    GetterSpec {
        owner: ClassifierId::Membership,
        attribute: "memberName",
        method: "getMemberName",
        behavior: GetterBehavior::Cached(FieldId(73)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 213,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    GetterSpec {
        owner: ClassifierId::Membership,
        attribute: "memberShortName",
        method: "getMemberShortName",
        behavior: GetterBehavior::Cached(FieldId(74)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 261,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    GetterSpec {
        owner: ClassifierId::Membership,
        attribute: "visibility",
        method: "getVisibility",
        behavior: GetterBehavior::Cached(FieldId(75)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 285,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    GetterSpec {
        owner: ClassifierId::OccurrenceDefinition,
        attribute: "isIndividual",
        method: "isIsIndividual",
        behavior: GetterBehavior::Cached(FieldId(79)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceDefinitionImpl.java",
            line: 79,
            sha256: "da09e6d912522b3ce8080ed9d8598abee5652d238df2ea6e0b5f44994871ef75",
        },
    },
    GetterSpec {
        owner: ClassifierId::OccurrenceUsage,
        attribute: "isIndividual",
        method: "isIsIndividual",
        behavior: GetterBehavior::Cached(FieldId(80)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 147,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
    GetterSpec {
        owner: ClassifierId::OccurrenceUsage,
        attribute: "portionKind",
        method: "getPortionKind",
        behavior: GetterBehavior::Cached(FieldId(81)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 186,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
    GetterSpec {
        owner: ClassifierId::OperatorExpression,
        attribute: "operator",
        method: "getOperator",
        behavior: GetterBehavior::Cached(FieldId(83)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OperatorExpressionImpl.java",
            line: 79,
            sha256: "af2646607ccbe2e363494a3b54c2004ab7777766fc93e08590d591a16b8d5b68",
        },
    },
    GetterSpec {
        owner: ClassifierId::OwningMembership,
        attribute: "memberName",
        method: "getMemberName",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OwningMembershipImpl.java",
            line: 162,
            sha256: "c9b001a480fc6ee3ca6005160bdcd2edefdbf195477f7f126eac9026c32923d5",
        },
    },
    GetterSpec {
        owner: ClassifierId::OwningMembership,
        attribute: "memberShortName",
        method: "getMemberShortName",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OwningMembershipImpl.java",
            line: 172,
            sha256: "c9b001a480fc6ee3ca6005160bdcd2edefdbf195477f7f126eac9026c32923d5",
        },
    },
    GetterSpec {
        owner: ClassifierId::OwningMembership,
        attribute: "ownedMemberElementId",
        method: "getOwnedMemberElementId",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OwningMembershipImpl.java",
            line: 120,
            sha256: "c9b001a480fc6ee3ca6005160bdcd2edefdbf195477f7f126eac9026c32923d5",
        },
    },
    GetterSpec {
        owner: ClassifierId::OwningMembership,
        attribute: "ownedMemberName",
        method: "getOwnedMemberName",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OwningMembershipImpl.java",
            line: 134,
            sha256: "c9b001a480fc6ee3ca6005160bdcd2edefdbf195477f7f126eac9026c32923d5",
        },
    },
    GetterSpec {
        owner: ClassifierId::OwningMembership,
        attribute: "ownedMemberShortName",
        method: "getOwnedMemberShortName",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OwningMembershipImpl.java",
            line: 148,
            sha256: "c9b001a480fc6ee3ca6005160bdcd2edefdbf195477f7f126eac9026c32923d5",
        },
    },
    GetterSpec {
        owner: ClassifierId::ReferenceUsage,
        attribute: "isReference",
        method: "isIsReference",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ReferenceUsageImpl.java",
            line: 60,
            sha256: "5104dd7999afabab2f8760c41d56b04866227387b4f1aaa555e1c7a290db3e6e",
        },
    },
    GetterSpec {
        owner: ClassifierId::Relationship,
        attribute: "isImplied",
        method: "isIsImplied",
        behavior: GetterBehavior::Cached(FieldId(88)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RelationshipImpl.java",
            line: 128,
            sha256: "fbd56ac8a6e5792869844e2a91c8b55fa7aecd6cc2e04b3e5315382a457bbe3d",
        },
    },
    GetterSpec {
        owner: ClassifierId::RequirementConstraintMembership,
        attribute: "kind",
        method: "getKind",
        behavior: GetterBehavior::Cached(FieldId(92)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementConstraintMembershipImpl.java",
            line: 89,
            sha256: "96bd03d8e2490604aa1b474eb8a69427ac45c35b96af4157337f4f78221f3e45",
        },
    },
    GetterSpec {
        owner: ClassifierId::RequirementDefinition,
        attribute: "declaredShortName",
        method: "getDeclaredShortName",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementDefinitionImpl.java",
            line: 358,
            sha256: "877901d3a8da850c15105ce9b633e45d825f52cdbeef41bf8b1db3ff4cacd917",
        },
    },
    GetterSpec {
        owner: ClassifierId::RequirementDefinition,
        attribute: "reqId",
        method: "getReqId",
        behavior: GetterBehavior::Cached(FieldId(93)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementDefinitionImpl.java",
            line: 156,
            sha256: "877901d3a8da850c15105ce9b633e45d825f52cdbeef41bf8b1db3ff4cacd917",
        },
    },
    GetterSpec {
        owner: ClassifierId::RequirementDefinition,
        attribute: "text",
        method: "getText",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementDefinitionImpl.java",
            line: 239,
            sha256: "877901d3a8da850c15105ce9b633e45d825f52cdbeef41bf8b1db3ff4cacd917",
        },
    },
    GetterSpec {
        owner: ClassifierId::RequirementUsage,
        attribute: "declaredShortName",
        method: "getDeclaredShortName",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementUsageImpl.java",
            line: 410,
            sha256: "e6e28cd8c181652e257b31369cf8e2697a628a1a2b2a8d73bdf70f36530accf2",
        },
    },
    GetterSpec {
        owner: ClassifierId::RequirementUsage,
        attribute: "reqId",
        method: "getReqId",
        behavior: GetterBehavior::Cached(FieldId(94)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementUsageImpl.java",
            line: 159,
            sha256: "e6e28cd8c181652e257b31369cf8e2697a628a1a2b2a8d73bdf70f36530accf2",
        },
    },
    GetterSpec {
        owner: ClassifierId::RequirementUsage,
        attribute: "text",
        method: "getText",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementUsageImpl.java",
            line: 266,
            sha256: "e6e28cd8c181652e257b31369cf8e2697a628a1a2b2a8d73bdf70f36530accf2",
        },
    },
    GetterSpec {
        owner: ClassifierId::SatisfyRequirementUsage,
        attribute: "isNegated",
        method: "isIsNegated",
        behavior: GetterBehavior::Cached(FieldId(95)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/SatisfyRequirementUsageImpl.java",
            line: 95,
            sha256: "f55cf0eddbbb8d5af72f925674eec7290bf065e0d728a6e96649cce0cb429de3",
        },
    },
    GetterSpec {
        owner: ClassifierId::StateDefinition,
        attribute: "isParallel",
        method: "isIsParallel",
        behavior: GetterBehavior::Cached(FieldId(98)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateDefinitionImpl.java",
            line: 178,
            sha256: "192b52b80e56b4816bf6b8bdf12478acb0f65223b897422a3014a95ae19bebf0",
        },
    },
    GetterSpec {
        owner: ClassifierId::StateSubactionMembership,
        attribute: "kind",
        method: "getKind",
        behavior: GetterBehavior::Cached(FieldId(99)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateSubactionMembershipImpl.java",
            line: 116,
            sha256: "f362f4fdc765663f74f3a3ea0aa2645779d528e07cf8a3511520c4d4b4990644",
        },
    },
    GetterSpec {
        owner: ClassifierId::StateUsage,
        attribute: "isParallel",
        method: "isIsParallel",
        behavior: GetterBehavior::Cached(FieldId(100)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateUsageImpl.java",
            line: 181,
            sha256: "235b477331ac3e3311e37b7a3ec1bebd2ace7bdc56ef0a30781f4213ff224865",
        },
    },
    GetterSpec {
        owner: ClassifierId::TextualRepresentation,
        attribute: "body",
        method: "getBody",
        behavior: GetterBehavior::Cached(FieldId(105)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TextualRepresentationImpl.java",
            line: 107,
            sha256: "31d09a2b2639048566c9f56f04f7c12d53d107c5420effb2e4ff7d8ba5c52b3e",
        },
    },
    GetterSpec {
        owner: ClassifierId::TextualRepresentation,
        attribute: "language",
        method: "getLanguage",
        behavior: GetterBehavior::Cached(FieldId(106)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TextualRepresentationImpl.java",
            line: 131,
            sha256: "31d09a2b2639048566c9f56f04f7c12d53d107c5420effb2e4ff7d8ba5c52b3e",
        },
    },
    GetterSpec {
        owner: ClassifierId::TransitionFeatureMembership,
        attribute: "kind",
        method: "getKind",
        behavior: GetterBehavior::Cached(FieldId(107)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TransitionFeatureMembershipImpl.java",
            line: 86,
            sha256: "a8cd905da455a039bfe81b0ead676c0167fc53ff78064de4872b8c1913d4115f",
        },
    },
    GetterSpec {
        owner: ClassifierId::TriggerInvocationExpression,
        attribute: "kind",
        method: "getKind",
        behavior: GetterBehavior::Cached(FieldId(108)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TriggerInvocationExpressionImpl.java",
            line: 80,
            sha256: "35f26a38a2c17afcac3ed32d568179dff68854722f9270a3be6a165a0e7a102f",
        },
    },
    GetterSpec {
        owner: ClassifierId::Type,
        attribute: "isAbstract",
        method: "isIsAbstract",
        behavior: GetterBehavior::Cached(FieldId(109)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 299,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    GetterSpec {
        owner: ClassifierId::Type,
        attribute: "isConjugated",
        method: "isIsConjugated",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 323,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    GetterSpec {
        owner: ClassifierId::Type,
        attribute: "isSufficient",
        method: "isIsSufficient",
        behavior: GetterBehavior::Cached(FieldId(110)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 333,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    GetterSpec {
        owner: ClassifierId::Usage,
        attribute: "isConstant",
        method: "isIsConstant",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UsageImpl.java",
            line: 283,
            sha256: "88ff141769debfe9a8c9cf575b0ec05a289f267479af5800d6526fc470cabe26",
        },
    },
    GetterSpec {
        owner: ClassifierId::Usage,
        attribute: "isReference",
        method: "isIsReference",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UsageImpl.java",
            line: 228,
            sha256: "88ff141769debfe9a8c9cf575b0ec05a289f267479af5800d6526fc470cabe26",
        },
    },
    GetterSpec {
        owner: ClassifierId::Usage,
        attribute: "isVariation",
        method: "isIsVariation",
        behavior: GetterBehavior::Cached(FieldId(114)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UsageImpl.java",
            line: 238,
            sha256: "88ff141769debfe9a8c9cf575b0ec05a289f267479af5800d6526fc470cabe26",
        },
    },
    GetterSpec {
        owner: ClassifierId::Usage,
        attribute: "mayTimeVary",
        method: "isMayTimeVary",
        behavior: GetterBehavior::Custom,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UsageImpl.java",
            line: 268,
            sha256: "88ff141769debfe9a8c9cf575b0ec05a289f267479af5800d6526fc470cabe26",
        },
    },
];
pub static SETTERS: &[SetterSpec] = &[
    SetterSpec {
        owner: ClassifierId::AssertConstraintUsage,
        attribute: "isNegated",
        field: Some(FieldId(1)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssertConstraintUsageImpl.java",
            line: 94,
            sha256: "26a68637a08cdce51f972c837fd8cb151cfe303be9a90691a96d06676cda3454",
        },
    },
    SetterSpec {
        owner: ClassifierId::Association,
        attribute: "isImplied",
        field: Some(FieldId(2)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssociationImpl.java",
            line: 144,
            sha256: "aaaf65656b531086ac47abfe06dc0f6715c3a34cb459b4f57f171b8d6089c8d7",
        },
    },
    SetterSpec {
        owner: ClassifierId::Comment,
        attribute: "body",
        field: Some(FieldId(6)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CommentImpl.java",
            line: 109,
            sha256: "4f2913fe8128a200498793edfd90c594f19169854dada56ce0b5ff36842f8bf8",
        },
    },
    SetterSpec {
        owner: ClassifierId::Comment,
        attribute: "locale",
        field: Some(FieldId(7)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CommentImpl.java",
            line: 133,
            sha256: "4f2913fe8128a200498793edfd90c594f19169854dada56ce0b5ff36842f8bf8",
        },
    },
    SetterSpec {
        owner: ClassifierId::ConnectionDefinition,
        attribute: "isImplied",
        field: Some(FieldId(11)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionDefinitionImpl.java",
            line: 152,
            sha256: "99aac0b55d655179e84f0dd8419111616a91bf2eb0b7ab8ab880acf063813352",
        },
    },
    SetterSpec {
        owner: ClassifierId::ConnectionUsage,
        attribute: "isIndividual",
        field: Some(FieldId(15)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 168,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    SetterSpec {
        owner: ClassifierId::ConnectionUsage,
        attribute: "portionKind",
        field: Some(FieldId(16)),
        null_reset_default: Some(8),
        set_flag: Some(FieldId(17)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 207,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    SetterSpec {
        owner: ClassifierId::Connector,
        attribute: "isImplied",
        field: Some(FieldId(18)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorImpl.java",
            line: 149,
            sha256: "1f49aa6b7e11ffec691b129403fadbc7dc1e179855b7ae9f7303dc84d11d8528",
        },
    },
    SetterSpec {
        owner: ClassifierId::ConnectorAsUsage,
        attribute: "isImplied",
        field: Some(FieldId(22)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorAsUsageImpl.java",
            line: 153,
            sha256: "9a34426f1fe469817000d26eb0399ee29991838e1bb45501f8477fe4dc084857",
        },
    },
    SetterSpec {
        owner: ClassifierId::Definition,
        attribute: "isVariation",
        field: Some(FieldId(27)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/DefinitionImpl.java",
            line: 167,
            sha256: "b8c1f6391eb0fd2b022ece2e8ea239413a2b6b562ee7209f80fc1a35fa9145ec",
        },
    },
    SetterSpec {
        owner: ClassifierId::Element,
        attribute: "declaredName",
        field: Some(FieldId(34)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 260,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    SetterSpec {
        owner: ClassifierId::Element,
        attribute: "declaredShortName",
        field: Some(FieldId(35)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 284,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    SetterSpec {
        owner: ClassifierId::Element,
        attribute: "elementId",
        field: Some(FieldId(36)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 323,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    SetterSpec {
        owner: ClassifierId::Element,
        attribute: "isImpliedIncluded",
        field: Some(FieldId(37)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 347,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    SetterSpec {
        owner: ClassifierId::Feature,
        attribute: "direction",
        field: Some(FieldId(39)),
        null_reset_default: Some(23),
        set_flag: Some(FieldId(40)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 371,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    SetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isComposite",
        field: Some(FieldId(41)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 519,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    SetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isConstant",
        field: Some(FieldId(42)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 543,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    SetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isDerived",
        field: Some(FieldId(43)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 567,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    SetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isEnd",
        field: Some(FieldId(44)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 595,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    SetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isOrdered",
        field: Some(FieldId(45)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 629,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    SetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isPortion",
        field: Some(FieldId(46)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 653,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    SetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isUnique",
        field: Some(FieldId(47)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 677,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    SetterSpec {
        owner: ClassifierId::Feature,
        attribute: "isVariable",
        field: Some(FieldId(48)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 701,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    SetterSpec {
        owner: ClassifierId::FeatureValue,
        attribute: "isDefault",
        field: Some(FieldId(54)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureValueImpl.java",
            line: 141,
            sha256: "06d158252791fccd475b35f55547369dd63bd92d6b27a2649e0d736553a66cc7",
        },
    },
    SetterSpec {
        owner: ClassifierId::FeatureValue,
        attribute: "isInitial",
        field: Some(FieldId(55)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureValueImpl.java",
            line: 165,
            sha256: "06d158252791fccd475b35f55547369dd63bd92d6b27a2649e0d736553a66cc7",
        },
    },
    SetterSpec {
        owner: ClassifierId::FlowDefinition,
        attribute: "isImplied",
        field: Some(FieldId(56)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowDefinitionImpl.java",
            line: 152,
            sha256: "244d80d8b39947bd40618e40af85847d44035c8220b238658960e4e1082b01ed",
        },
    },
    SetterSpec {
        owner: ClassifierId::FlowUsage,
        attribute: "isIndividual",
        field: Some(FieldId(60)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 157,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    SetterSpec {
        owner: ClassifierId::FlowUsage,
        attribute: "portionKind",
        field: Some(FieldId(61)),
        null_reset_default: Some(37),
        set_flag: Some(FieldId(62)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 181,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    SetterSpec {
        owner: ClassifierId::Import,
        attribute: "isImportAll",
        field: Some(FieldId(63)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 187,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    SetterSpec {
        owner: ClassifierId::Import,
        attribute: "isRecursive",
        field: Some(FieldId(64)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 211,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    SetterSpec {
        owner: ClassifierId::Import,
        attribute: "visibility",
        field: Some(FieldId(65)),
        null_reset_default: Some(41),
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 235,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    SetterSpec {
        owner: ClassifierId::Invariant,
        attribute: "isNegated",
        field: Some(FieldId(67)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/InvariantImpl.java",
            line: 88,
            sha256: "1e5440a1324cb70f52d4a2bb313ac4ade34e1ac0cd608c5e720e3d424a09bbdf",
        },
    },
    SetterSpec {
        owner: ClassifierId::LibraryPackage,
        attribute: "isStandard",
        field: Some(FieldId(68)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LibraryPackageImpl.java",
            line: 90,
            sha256: "9103340eb814396cb02705d14b9062fa660b0577879c31538eae5534a552e99f",
        },
    },
    SetterSpec {
        owner: ClassifierId::LiteralBoolean,
        attribute: "value",
        field: Some(FieldId(69)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralBooleanImpl.java",
            line: 89,
            sha256: "1cdf2c81a78ab9a5ae2437a4cfe959aa31ee9f37107949638abb3d1fb70d52d2",
        },
    },
    SetterSpec {
        owner: ClassifierId::LiteralInteger,
        attribute: "value",
        field: Some(FieldId(70)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralIntegerImpl.java",
            line: 89,
            sha256: "ea23684de620c444cd6cd43c5562e6c4f1a180d9f8cfb02ed58284134d06f96c",
        },
    },
    SetterSpec {
        owner: ClassifierId::LiteralRational,
        attribute: "value",
        field: Some(FieldId(71)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralRationalImpl.java",
            line: 89,
            sha256: "0e99ac7f50b7d612c5f1450d76195d8b5f4c8f0b3ef7db87ab94b82145f8d632",
        },
    },
    SetterSpec {
        owner: ClassifierId::LiteralString,
        attribute: "value",
        field: Some(FieldId(72)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralStringImpl.java",
            line: 88,
            sha256: "0f51f25d8b70036c25befc883f7dc4e88e3b63b60555d3b8ecd633a3b4f1441f",
        },
    },
    SetterSpec {
        owner: ClassifierId::Membership,
        attribute: "memberName",
        field: Some(FieldId(73)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 223,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    SetterSpec {
        owner: ClassifierId::Membership,
        attribute: "memberShortName",
        field: Some(FieldId(74)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 271,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    SetterSpec {
        owner: ClassifierId::Membership,
        attribute: "visibility",
        field: Some(FieldId(75)),
        null_reset_default: Some(51),
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 295,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    SetterSpec {
        owner: ClassifierId::OccurrenceDefinition,
        attribute: "isIndividual",
        field: Some(FieldId(79)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceDefinitionImpl.java",
            line: 89,
            sha256: "da09e6d912522b3ce8080ed9d8598abee5652d238df2ea6e0b5f44994871ef75",
        },
    },
    SetterSpec {
        owner: ClassifierId::OccurrenceUsage,
        attribute: "isIndividual",
        field: Some(FieldId(80)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 157,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
    SetterSpec {
        owner: ClassifierId::OccurrenceUsage,
        attribute: "portionKind",
        field: Some(FieldId(81)),
        null_reset_default: Some(54),
        set_flag: Some(FieldId(82)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 196,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
    SetterSpec {
        owner: ClassifierId::OperatorExpression,
        attribute: "operator",
        field: Some(FieldId(83)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OperatorExpressionImpl.java",
            line: 89,
            sha256: "af2646607ccbe2e363494a3b54c2004ab7777766fc93e08590d591a16b8d5b68",
        },
    },
    SetterSpec {
        owner: ClassifierId::Relationship,
        attribute: "isImplied",
        field: Some(FieldId(88)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RelationshipImpl.java",
            line: 138,
            sha256: "fbd56ac8a6e5792869844e2a91c8b55fa7aecd6cc2e04b3e5315382a457bbe3d",
        },
    },
    SetterSpec {
        owner: ClassifierId::RequirementConstraintMembership,
        attribute: "kind",
        field: Some(FieldId(92)),
        null_reset_default: Some(60),
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementConstraintMembershipImpl.java",
            line: 99,
            sha256: "96bd03d8e2490604aa1b474eb8a69427ac45c35b96af4157337f4f78221f3e45",
        },
    },
    SetterSpec {
        owner: ClassifierId::RequirementDefinition,
        attribute: "declaredShortName",
        field: None,
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementDefinitionImpl.java",
            line: 368,
            sha256: "877901d3a8da850c15105ce9b633e45d825f52cdbeef41bf8b1db3ff4cacd917",
        },
    },
    SetterSpec {
        owner: ClassifierId::RequirementDefinition,
        attribute: "reqId",
        field: Some(FieldId(93)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementDefinitionImpl.java",
            line: 166,
            sha256: "877901d3a8da850c15105ce9b633e45d825f52cdbeef41bf8b1db3ff4cacd917",
        },
    },
    SetterSpec {
        owner: ClassifierId::RequirementUsage,
        attribute: "declaredShortName",
        field: None,
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementUsageImpl.java",
            line: 420,
            sha256: "e6e28cd8c181652e257b31369cf8e2697a628a1a2b2a8d73bdf70f36530accf2",
        },
    },
    SetterSpec {
        owner: ClassifierId::RequirementUsage,
        attribute: "reqId",
        field: Some(FieldId(94)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementUsageImpl.java",
            line: 169,
            sha256: "e6e28cd8c181652e257b31369cf8e2697a628a1a2b2a8d73bdf70f36530accf2",
        },
    },
    SetterSpec {
        owner: ClassifierId::SatisfyRequirementUsage,
        attribute: "isNegated",
        field: Some(FieldId(95)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/SatisfyRequirementUsageImpl.java",
            line: 105,
            sha256: "f55cf0eddbbb8d5af72f925674eec7290bf065e0d728a6e96649cce0cb429de3",
        },
    },
    SetterSpec {
        owner: ClassifierId::StateDefinition,
        attribute: "isParallel",
        field: Some(FieldId(98)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateDefinitionImpl.java",
            line: 188,
            sha256: "192b52b80e56b4816bf6b8bdf12478acb0f65223b897422a3014a95ae19bebf0",
        },
    },
    SetterSpec {
        owner: ClassifierId::StateSubactionMembership,
        attribute: "kind",
        field: Some(FieldId(99)),
        null_reset_default: Some(65),
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateSubactionMembershipImpl.java",
            line: 126,
            sha256: "f362f4fdc765663f74f3a3ea0aa2645779d528e07cf8a3511520c4d4b4990644",
        },
    },
    SetterSpec {
        owner: ClassifierId::StateUsage,
        attribute: "isParallel",
        field: Some(FieldId(100)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateUsageImpl.java",
            line: 191,
            sha256: "235b477331ac3e3311e37b7a3ec1bebd2ace7bdc56ef0a30781f4213ff224865",
        },
    },
    SetterSpec {
        owner: ClassifierId::TextualRepresentation,
        attribute: "body",
        field: Some(FieldId(105)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TextualRepresentationImpl.java",
            line: 117,
            sha256: "31d09a2b2639048566c9f56f04f7c12d53d107c5420effb2e4ff7d8ba5c52b3e",
        },
    },
    SetterSpec {
        owner: ClassifierId::TextualRepresentation,
        attribute: "language",
        field: Some(FieldId(106)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TextualRepresentationImpl.java",
            line: 141,
            sha256: "31d09a2b2639048566c9f56f04f7c12d53d107c5420effb2e4ff7d8ba5c52b3e",
        },
    },
    SetterSpec {
        owner: ClassifierId::TransitionFeatureMembership,
        attribute: "kind",
        field: Some(FieldId(107)),
        null_reset_default: Some(69),
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TransitionFeatureMembershipImpl.java",
            line: 96,
            sha256: "a8cd905da455a039bfe81b0ead676c0167fc53ff78064de4872b8c1913d4115f",
        },
    },
    SetterSpec {
        owner: ClassifierId::TriggerInvocationExpression,
        attribute: "kind",
        field: Some(FieldId(108)),
        null_reset_default: Some(70),
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TriggerInvocationExpressionImpl.java",
            line: 90,
            sha256: "35f26a38a2c17afcac3ed32d568179dff68854722f9270a3be6a165a0e7a102f",
        },
    },
    SetterSpec {
        owner: ClassifierId::Type,
        attribute: "isAbstract",
        field: Some(FieldId(109)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 309,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    SetterSpec {
        owner: ClassifierId::Type,
        attribute: "isSufficient",
        field: Some(FieldId(110)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 343,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    SetterSpec {
        owner: ClassifierId::Usage,
        attribute: "isVariation",
        field: Some(FieldId(114)),
        null_reset_default: None,
        set_flag: None,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UsageImpl.java",
            line: 248,
            sha256: "88ff141769debfe9a8c9cf575b0ec05a289f267479af5800d6526fc470cabe26",
        },
    },
];
pub static IS_SET_SPECS: &[IsSetSpec] = &[
    IsSetSpec {
        owner: ClassifierId::AssertConstraintUsage,
        attribute: "isNegated",
        behavior: IsSetBehavior::Difference {
            field: FieldId(1),
            default: 0,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssertConstraintUsageImpl.java",
            line: 184,
            sha256: "26a68637a08cdce51f972c837fd8cb151cfe303be9a90691a96d06676cda3454",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Association,
        attribute: "isImplied",
        behavior: IsSetBehavior::Difference {
            field: FieldId(2),
            default: 1,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssociationImpl.java",
            line: 433,
            sha256: "aaaf65656b531086ac47abfe06dc0f6715c3a34cb459b4f57f171b8d6089c8d7",
        },
    },
    IsSetSpec {
        owner: ClassifierId::CalculationDefinition,
        attribute: "isModelLevelEvaluable",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CalculationDefinitionImpl.java",
            line: 160,
            sha256: "1a14004d79980eeb6ecb677674ef9e64cfbdf807e674cc90daa468b2dc4f5822",
        },
    },
    IsSetSpec {
        owner: ClassifierId::CalculationUsage,
        attribute: "isModelLevelEvaluable",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CalculationUsageImpl.java",
            line: 223,
            sha256: "0357c1e27ba49ef525da2dd45530ad1841b0cbccdd28143f72513d791c10f0c6",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Comment,
        attribute: "body",
        behavior: IsSetBehavior::Difference {
            field: FieldId(6),
            default: 4,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CommentImpl.java",
            line: 199,
            sha256: "4f2913fe8128a200498793edfd90c594f19169854dada56ce0b5ff36842f8bf8",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Comment,
        attribute: "locale",
        behavior: IsSetBehavior::Difference {
            field: FieldId(7),
            default: 5,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CommentImpl.java",
            line: 199,
            sha256: "4f2913fe8128a200498793edfd90c594f19169854dada56ce0b5ff36842f8bf8",
        },
    },
    IsSetSpec {
        owner: ClassifierId::ConnectionDefinition,
        attribute: "isImplied",
        behavior: IsSetBehavior::Difference {
            field: FieldId(11),
            default: 6,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionDefinitionImpl.java",
            line: 486,
            sha256: "99aac0b55d655179e84f0dd8419111616a91bf2eb0b7ab8ab880acf063813352",
        },
    },
    IsSetSpec {
        owner: ClassifierId::ConnectionUsage,
        attribute: "isIndividual",
        behavior: IsSetBehavior::Difference {
            field: FieldId(15),
            default: 7,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 363,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    IsSetSpec {
        owner: ClassifierId::ConnectionUsage,
        attribute: "portionKind",
        behavior: IsSetBehavior::Flag(FieldId(17)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 363,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Connector,
        attribute: "isImplied",
        behavior: IsSetBehavior::Difference {
            field: FieldId(18),
            default: 9,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorImpl.java",
            line: 506,
            sha256: "1f49aa6b7e11ffec691b129403fadbc7dc1e179855b7ae9f7303dc84d11d8528",
        },
    },
    IsSetSpec {
        owner: ClassifierId::ConnectorAsUsage,
        attribute: "isImplied",
        behavior: IsSetBehavior::Difference {
            field: FieldId(22),
            default: 10,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorAsUsageImpl.java",
            line: 539,
            sha256: "9a34426f1fe469817000d26eb0399ee29991838e1bb45501f8477fe4dc084857",
        },
    },
    IsSetSpec {
        owner: ClassifierId::ConstraintDefinition,
        attribute: "isModelLevelEvaluable",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConstraintDefinitionImpl.java",
            line: 183,
            sha256: "67edcc045820f764f1c47e318cbf0ecac46f3416f7291aa7b2fef3e89b51373c",
        },
    },
    IsSetSpec {
        owner: ClassifierId::ConstraintUsage,
        attribute: "isModelLevelEvaluable",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConstraintUsageImpl.java",
            line: 343,
            sha256: "9a782d4050e58d8bdcf061bfea5fca441039b9b94a5469922e29b45bc82d5a6d",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Definition,
        attribute: "isVariation",
        behavior: IsSetBehavior::Difference {
            field: FieldId(27),
            default: 13,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/DefinitionImpl.java",
            line: 829,
            sha256: "b8c1f6391eb0fd2b022ece2e8ea239413a2b6b562ee7209f80fc1a35fa9145ec",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Element,
        attribute: "aliasIds",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 876,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Element,
        attribute: "declaredName",
        behavior: IsSetBehavior::Difference {
            field: FieldId(34),
            default: 14,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 876,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Element,
        attribute: "declaredShortName",
        behavior: IsSetBehavior::Difference {
            field: FieldId(35),
            default: 15,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 876,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Element,
        attribute: "elementId",
        behavior: IsSetBehavior::Difference {
            field: FieldId(36),
            default: 16,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 876,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Element,
        attribute: "isImpliedIncluded",
        behavior: IsSetBehavior::Difference {
            field: FieldId(37),
            default: 17,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 876,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Element,
        attribute: "isLibraryElement",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 876,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Element,
        attribute: "name",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 876,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Element,
        attribute: "qualifiedName",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 876,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Element,
        attribute: "shortName",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 876,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Expression,
        attribute: "isModelLevelEvaluable",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ExpressionImpl.java",
            line: 197,
            sha256: "bbbaec6f943ed89c79ce7a58c30ad5d895bd3e7c356e1b04a584639292dbf15f",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Feature,
        attribute: "direction",
        behavior: IsSetBehavior::Flag(FieldId(40)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1375,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Feature,
        attribute: "isComposite",
        behavior: IsSetBehavior::Difference {
            field: FieldId(41),
            default: 24,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1375,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Feature,
        attribute: "isConstant",
        behavior: IsSetBehavior::Difference {
            field: FieldId(42),
            default: 25,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1375,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Feature,
        attribute: "isDerived",
        behavior: IsSetBehavior::Difference {
            field: FieldId(43),
            default: 26,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1375,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Feature,
        attribute: "isEnd",
        behavior: IsSetBehavior::Difference {
            field: FieldId(44),
            default: 27,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1375,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Feature,
        attribute: "isNonunique",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1375,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Feature,
        attribute: "isOrdered",
        behavior: IsSetBehavior::Difference {
            field: FieldId(45),
            default: 29,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1375,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Feature,
        attribute: "isPortion",
        behavior: IsSetBehavior::Difference {
            field: FieldId(46),
            default: 30,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1375,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Feature,
        attribute: "isUnique",
        behavior: IsSetBehavior::Difference {
            field: FieldId(47),
            default: 31,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1375,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Feature,
        attribute: "isVariable",
        behavior: IsSetBehavior::Difference {
            field: FieldId(48),
            default: 32,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1375,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    IsSetSpec {
        owner: ClassifierId::FeatureValue,
        attribute: "isDefault",
        behavior: IsSetBehavior::Difference {
            field: FieldId(54),
            default: 33,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureValueImpl.java",
            line: 265,
            sha256: "06d158252791fccd475b35f55547369dd63bd92d6b27a2649e0d736553a66cc7",
        },
    },
    IsSetSpec {
        owner: ClassifierId::FeatureValue,
        attribute: "isInitial",
        behavior: IsSetBehavior::Difference {
            field: FieldId(55),
            default: 34,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureValueImpl.java",
            line: 265,
            sha256: "06d158252791fccd475b35f55547369dd63bd92d6b27a2649e0d736553a66cc7",
        },
    },
    IsSetSpec {
        owner: ClassifierId::FlowDefinition,
        attribute: "isImplied",
        behavior: IsSetBehavior::Difference {
            field: FieldId(56),
            default: 35,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowDefinitionImpl.java",
            line: 501,
            sha256: "244d80d8b39947bd40618e40af85847d44035c8220b238658960e4e1082b01ed",
        },
    },
    IsSetSpec {
        owner: ClassifierId::FlowUsage,
        attribute: "isIndividual",
        behavior: IsSetBehavior::Difference {
            field: FieldId(60),
            default: 36,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 596,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    IsSetSpec {
        owner: ClassifierId::FlowUsage,
        attribute: "portionKind",
        behavior: IsSetBehavior::Flag(FieldId(62)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 596,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Function,
        attribute: "isModelLevelEvaluable",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FunctionImpl.java",
            line: 147,
            sha256: "8a5dbc66196a21a04c260a3bda8a363aebff5bdff9d3dcce91fff73f41024434",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Import,
        attribute: "isImportAll",
        behavior: IsSetBehavior::Difference {
            field: FieldId(63),
            default: 39,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 331,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Import,
        attribute: "isRecursive",
        behavior: IsSetBehavior::Difference {
            field: FieldId(64),
            default: 40,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 331,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Import,
        attribute: "visibility",
        behavior: IsSetBehavior::Difference {
            field: FieldId(65),
            default: 41,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 331,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Invariant,
        attribute: "isNegated",
        behavior: IsSetBehavior::Difference {
            field: FieldId(67),
            default: 42,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/InvariantImpl.java",
            line: 146,
            sha256: "1e5440a1324cb70f52d4a2bb313ac4ade34e1ac0cd608c5e720e3d424a09bbdf",
        },
    },
    IsSetSpec {
        owner: ClassifierId::LibraryPackage,
        attribute: "isStandard",
        behavior: IsSetBehavior::Difference {
            field: FieldId(68),
            default: 43,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LibraryPackageImpl.java",
            line: 158,
            sha256: "9103340eb814396cb02705d14b9062fa660b0577879c31538eae5534a552e99f",
        },
    },
    IsSetSpec {
        owner: ClassifierId::LiteralBoolean,
        attribute: "value",
        behavior: IsSetBehavior::Difference {
            field: FieldId(69),
            default: 44,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralBooleanImpl.java",
            line: 147,
            sha256: "1cdf2c81a78ab9a5ae2437a4cfe959aa31ee9f37107949638abb3d1fb70d52d2",
        },
    },
    IsSetSpec {
        owner: ClassifierId::LiteralInteger,
        attribute: "value",
        behavior: IsSetBehavior::Difference {
            field: FieldId(70),
            default: 45,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralIntegerImpl.java",
            line: 147,
            sha256: "ea23684de620c444cd6cd43c5562e6c4f1a180d9f8cfb02ed58284134d06f96c",
        },
    },
    IsSetSpec {
        owner: ClassifierId::LiteralRational,
        attribute: "value",
        behavior: IsSetBehavior::Difference {
            field: FieldId(71),
            default: 46,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralRationalImpl.java",
            line: 147,
            sha256: "0e99ac7f50b7d612c5f1450d76195d8b5f4c8f0b3ef7db87ab94b82145f8d632",
        },
    },
    IsSetSpec {
        owner: ClassifierId::LiteralString,
        attribute: "value",
        behavior: IsSetBehavior::Difference {
            field: FieldId(72),
            default: 47,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralStringImpl.java",
            line: 146,
            sha256: "0f51f25d8b70036c25befc883f7dc4e88e3b63b60555d3b8ecd633a3b4f1441f",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Membership,
        attribute: "memberElementId",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 415,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Membership,
        attribute: "memberName",
        behavior: IsSetBehavior::Difference {
            field: FieldId(73),
            default: 49,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 415,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Membership,
        attribute: "memberShortName",
        behavior: IsSetBehavior::Difference {
            field: FieldId(74),
            default: 50,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 415,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Membership,
        attribute: "visibility",
        behavior: IsSetBehavior::Difference {
            field: FieldId(75),
            default: 51,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 415,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    IsSetSpec {
        owner: ClassifierId::OccurrenceDefinition,
        attribute: "isIndividual",
        behavior: IsSetBehavior::Difference {
            field: FieldId(79),
            default: 52,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceDefinitionImpl.java",
            line: 147,
            sha256: "da09e6d912522b3ce8080ed9d8598abee5652d238df2ea6e0b5f44994871ef75",
        },
    },
    IsSetSpec {
        owner: ClassifierId::OccurrenceUsage,
        attribute: "isIndividual",
        behavior: IsSetBehavior::Difference {
            field: FieldId(80),
            default: 53,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 298,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
    IsSetSpec {
        owner: ClassifierId::OccurrenceUsage,
        attribute: "portionKind",
        behavior: IsSetBehavior::Flag(FieldId(82)),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 298,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
    IsSetSpec {
        owner: ClassifierId::OperatorExpression,
        attribute: "operator",
        behavior: IsSetBehavior::Difference {
            field: FieldId(83),
            default: 55,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OperatorExpressionImpl.java",
            line: 148,
            sha256: "af2646607ccbe2e363494a3b54c2004ab7777766fc93e08590d591a16b8d5b68",
        },
    },
    IsSetSpec {
        owner: ClassifierId::OwningMembership,
        attribute: "ownedMemberElementId",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OwningMembershipImpl.java",
            line: 205,
            sha256: "c9b001a480fc6ee3ca6005160bdcd2edefdbf195477f7f126eac9026c32923d5",
        },
    },
    IsSetSpec {
        owner: ClassifierId::OwningMembership,
        attribute: "ownedMemberName",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OwningMembershipImpl.java",
            line: 205,
            sha256: "c9b001a480fc6ee3ca6005160bdcd2edefdbf195477f7f126eac9026c32923d5",
        },
    },
    IsSetSpec {
        owner: ClassifierId::OwningMembership,
        attribute: "ownedMemberShortName",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OwningMembershipImpl.java",
            line: 205,
            sha256: "c9b001a480fc6ee3ca6005160bdcd2edefdbf195477f7f126eac9026c32923d5",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Relationship,
        attribute: "isImplied",
        behavior: IsSetBehavior::Difference {
            field: FieldId(88),
            default: 59,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RelationshipImpl.java",
            line: 403,
            sha256: "fbd56ac8a6e5792869844e2a91c8b55fa7aecd6cc2e04b3e5315382a457bbe3d",
        },
    },
    IsSetSpec {
        owner: ClassifierId::RequirementConstraintMembership,
        attribute: "kind",
        behavior: IsSetBehavior::Difference {
            field: FieldId(92),
            default: 60,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementConstraintMembershipImpl.java",
            line: 217,
            sha256: "96bd03d8e2490604aa1b474eb8a69427ac45c35b96af4157337f4f78221f3e45",
        },
    },
    IsSetSpec {
        owner: ClassifierId::RequirementDefinition,
        attribute: "reqId",
        behavior: IsSetBehavior::Difference {
            field: FieldId(93),
            default: 61,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementDefinitionImpl.java",
            line: 312,
            sha256: "877901d3a8da850c15105ce9b633e45d825f52cdbeef41bf8b1db3ff4cacd917",
        },
    },
    IsSetSpec {
        owner: ClassifierId::RequirementDefinition,
        attribute: "text",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementDefinitionImpl.java",
            line: 312,
            sha256: "877901d3a8da850c15105ce9b633e45d825f52cdbeef41bf8b1db3ff4cacd917",
        },
    },
    IsSetSpec {
        owner: ClassifierId::RequirementUsage,
        attribute: "reqId",
        behavior: IsSetBehavior::Difference {
            field: FieldId(94),
            default: 62,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementUsageImpl.java",
            line: 352,
            sha256: "e6e28cd8c181652e257b31369cf8e2697a628a1a2b2a8d73bdf70f36530accf2",
        },
    },
    IsSetSpec {
        owner: ClassifierId::RequirementUsage,
        attribute: "text",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementUsageImpl.java",
            line: 352,
            sha256: "e6e28cd8c181652e257b31369cf8e2697a628a1a2b2a8d73bdf70f36530accf2",
        },
    },
    IsSetSpec {
        owner: ClassifierId::SatisfyRequirementUsage,
        attribute: "isNegated",
        behavior: IsSetBehavior::Difference {
            field: FieldId(95),
            default: 63,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/SatisfyRequirementUsageImpl.java",
            line: 277,
            sha256: "f55cf0eddbbb8d5af72f925674eec7290bf065e0d728a6e96649cce0cb429de3",
        },
    },
    IsSetSpec {
        owner: ClassifierId::StateDefinition,
        attribute: "isParallel",
        behavior: IsSetBehavior::Difference {
            field: FieldId(98),
            default: 64,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateDefinitionImpl.java",
            line: 275,
            sha256: "192b52b80e56b4816bf6b8bdf12478acb0f65223b897422a3014a95ae19bebf0",
        },
    },
    IsSetSpec {
        owner: ClassifierId::StateSubactionMembership,
        attribute: "kind",
        behavior: IsSetBehavior::Difference {
            field: FieldId(99),
            default: 65,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateSubactionMembershipImpl.java",
            line: 189,
            sha256: "f362f4fdc765663f74f3a3ea0aa2645779d528e07cf8a3511520c4d4b4990644",
        },
    },
    IsSetSpec {
        owner: ClassifierId::StateUsage,
        attribute: "isParallel",
        behavior: IsSetBehavior::Difference {
            field: FieldId(100),
            default: 66,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateUsageImpl.java",
            line: 318,
            sha256: "235b477331ac3e3311e37b7a3ec1bebd2ace7bdc56ef0a30781f4213ff224865",
        },
    },
    IsSetSpec {
        owner: ClassifierId::TextualRepresentation,
        attribute: "body",
        behavior: IsSetBehavior::Difference {
            field: FieldId(105),
            default: 67,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TextualRepresentationImpl.java",
            line: 239,
            sha256: "31d09a2b2639048566c9f56f04f7c12d53d107c5420effb2e4ff7d8ba5c52b3e",
        },
    },
    IsSetSpec {
        owner: ClassifierId::TextualRepresentation,
        attribute: "language",
        behavior: IsSetBehavior::Difference {
            field: FieldId(106),
            default: 68,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TextualRepresentationImpl.java",
            line: 239,
            sha256: "31d09a2b2639048566c9f56f04f7c12d53d107c5420effb2e4ff7d8ba5c52b3e",
        },
    },
    IsSetSpec {
        owner: ClassifierId::TransitionFeatureMembership,
        attribute: "kind",
        behavior: IsSetBehavior::Difference {
            field: FieldId(107),
            default: 69,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TransitionFeatureMembershipImpl.java",
            line: 184,
            sha256: "a8cd905da455a039bfe81b0ead676c0167fc53ff78064de4872b8c1913d4115f",
        },
    },
    IsSetSpec {
        owner: ClassifierId::TriggerInvocationExpression,
        attribute: "kind",
        behavior: IsSetBehavior::Difference {
            field: FieldId(108),
            default: 70,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TriggerInvocationExpressionImpl.java",
            line: 148,
            sha256: "35f26a38a2c17afcac3ed32d568179dff68854722f9270a3be6a165a0e7a102f",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Type,
        attribute: "isAbstract",
        behavior: IsSetBehavior::Difference {
            field: FieldId(109),
            default: 71,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 988,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Type,
        attribute: "isConjugated",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 988,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Type,
        attribute: "isSufficient",
        behavior: IsSetBehavior::Difference {
            field: FieldId(110),
            default: 73,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 988,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Usage,
        attribute: "isReference",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UsageImpl.java",
            line: 953,
            sha256: "88ff141769debfe9a8c9cf575b0ec05a289f267479af5800d6526fc470cabe26",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Usage,
        attribute: "isVariation",
        behavior: IsSetBehavior::Difference {
            field: FieldId(114),
            default: 75,
        },
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UsageImpl.java",
            line: 953,
            sha256: "88ff141769debfe9a8c9cf575b0ec05a289f267479af5800d6526fc470cabe26",
        },
    },
    IsSetSpec {
        owner: ClassifierId::Usage,
        attribute: "mayTimeVary",
        behavior: IsSetBehavior::Unsupported,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UsageImpl.java",
            line: 953,
            sha256: "88ff141769debfe9a8c9cf575b0ec05a289f267479af5800d6526fc470cabe26",
        },
    },
];
pub static UNSETTERS: &[UnsetterSpec] = &[
    UnsetterSpec {
        owner: ClassifierId::ConnectionUsage,
        attribute: "portionKind",
        field: FieldId(16),
        default: 8,
        set_flag: FieldId(17),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 223,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    UnsetterSpec {
        owner: ClassifierId::Feature,
        attribute: "direction",
        field: FieldId(39),
        default: 23,
        set_flag: FieldId(40),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 387,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    UnsetterSpec {
        owner: ClassifierId::FlowUsage,
        attribute: "portionKind",
        field: FieldId(61),
        default: 37,
        set_flag: FieldId(62),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 197,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    UnsetterSpec {
        owner: ClassifierId::OccurrenceUsage,
        attribute: "portionKind",
        field: FieldId(81),
        default: 54,
        set_flag: FieldId(82),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 212,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
];
pub static RESETS: &[ResetSpec] = &[
    ResetSpec {
        owner: ClassifierId::AssertConstraintUsage,
        attribute: "isNegated",
        behavior: ResetBehavior::SetterDefault(0),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssertConstraintUsageImpl.java",
            line: 169,
            sha256: "26a68637a08cdce51f972c837fd8cb151cfe303be9a90691a96d06676cda3454",
        },
    },
    ResetSpec {
        owner: ClassifierId::Association,
        attribute: "isImplied",
        behavior: ResetBehavior::SetterDefault(1),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/AssociationImpl.java",
            line: 406,
            sha256: "aaaf65656b531086ac47abfe06dc0f6715c3a34cb459b4f57f171b8d6089c8d7",
        },
    },
    ResetSpec {
        owner: ClassifierId::Comment,
        attribute: "body",
        behavior: ResetBehavior::SetterDefault(4),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CommentImpl.java",
            line: 181,
            sha256: "4f2913fe8128a200498793edfd90c594f19169854dada56ce0b5ff36842f8bf8",
        },
    },
    ResetSpec {
        owner: ClassifierId::Comment,
        attribute: "locale",
        behavior: ResetBehavior::SetterDefault(5),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/CommentImpl.java",
            line: 181,
            sha256: "4f2913fe8128a200498793edfd90c594f19169854dada56ce0b5ff36842f8bf8",
        },
    },
    ResetSpec {
        owner: ClassifierId::ConnectionDefinition,
        attribute: "isImplied",
        behavior: ResetBehavior::SetterDefault(6),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionDefinitionImpl.java",
            line: 459,
            sha256: "99aac0b55d655179e84f0dd8419111616a91bf2eb0b7ab8ab880acf063813352",
        },
    },
    ResetSpec {
        owner: ClassifierId::ConnectionUsage,
        attribute: "isIndividual",
        behavior: ResetBehavior::SetterDefault(7),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 345,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    ResetSpec {
        owner: ClassifierId::ConnectionUsage,
        attribute: "portionKind",
        behavior: ResetBehavior::UnsetMethod,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectionUsageImpl.java",
            line: 345,
            sha256: "ac0a9090a9605fc9718d7e84ef289079e58899ac95a2fa4c6116d77f0f69f07e",
        },
    },
    ResetSpec {
        owner: ClassifierId::Connector,
        attribute: "isImplied",
        behavior: ResetBehavior::SetterDefault(9),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorImpl.java",
            line: 479,
            sha256: "1f49aa6b7e11ffec691b129403fadbc7dc1e179855b7ae9f7303dc84d11d8528",
        },
    },
    ResetSpec {
        owner: ClassifierId::ConnectorAsUsage,
        attribute: "isImplied",
        behavior: ResetBehavior::SetterDefault(10),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ConnectorAsUsageImpl.java",
            line: 512,
            sha256: "9a34426f1fe469817000d26eb0399ee29991838e1bb45501f8477fe4dc084857",
        },
    },
    ResetSpec {
        owner: ClassifierId::Definition,
        attribute: "isVariation",
        behavior: ResetBehavior::SetterDefault(13),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/DefinitionImpl.java",
            line: 814,
            sha256: "b8c1f6391eb0fd2b022ece2e8ea239413a2b6b562ee7209f80fc1a35fa9145ec",
        },
    },
    ResetSpec {
        owner: ClassifierId::Element,
        attribute: "declaredName",
        behavior: ResetBehavior::SetterDefault(14),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 843,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    ResetSpec {
        owner: ClassifierId::Element,
        attribute: "declaredShortName",
        behavior: ResetBehavior::SetterDefault(15),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 843,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    ResetSpec {
        owner: ClassifierId::Element,
        attribute: "elementId",
        behavior: ResetBehavior::SetterDefault(16),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 843,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    ResetSpec {
        owner: ClassifierId::Element,
        attribute: "isImpliedIncluded",
        behavior: ResetBehavior::SetterDefault(17),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ElementImpl.java",
            line: 843,
            sha256: "094e31ec652f2df9eb7c424721a2f83501dfd0a771eeebfdfeab18fe8bceef30",
        },
    },
    ResetSpec {
        owner: ClassifierId::Feature,
        attribute: "direction",
        behavior: ResetBehavior::UnsetMethod,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1294,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    ResetSpec {
        owner: ClassifierId::Feature,
        attribute: "isComposite",
        behavior: ResetBehavior::SetterDefault(24),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1294,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    ResetSpec {
        owner: ClassifierId::Feature,
        attribute: "isConstant",
        behavior: ResetBehavior::SetterDefault(25),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1294,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    ResetSpec {
        owner: ClassifierId::Feature,
        attribute: "isDerived",
        behavior: ResetBehavior::SetterDefault(26),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1294,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    ResetSpec {
        owner: ClassifierId::Feature,
        attribute: "isEnd",
        behavior: ResetBehavior::SetterDefault(27),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1294,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    ResetSpec {
        owner: ClassifierId::Feature,
        attribute: "isOrdered",
        behavior: ResetBehavior::SetterDefault(29),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1294,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    ResetSpec {
        owner: ClassifierId::Feature,
        attribute: "isPortion",
        behavior: ResetBehavior::SetterDefault(30),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1294,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    ResetSpec {
        owner: ClassifierId::Feature,
        attribute: "isUnique",
        behavior: ResetBehavior::SetterDefault(31),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1294,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    ResetSpec {
        owner: ClassifierId::Feature,
        attribute: "isVariable",
        behavior: ResetBehavior::SetterDefault(32),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureImpl.java",
            line: 1294,
            sha256: "063f7d9d2d9f560ffaa279d73ec34cf8d97b27f9e050f54020826ada0427db69",
        },
    },
    ResetSpec {
        owner: ClassifierId::FeatureValue,
        attribute: "isDefault",
        behavior: ResetBehavior::SetterDefault(33),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureValueImpl.java",
            line: 247,
            sha256: "06d158252791fccd475b35f55547369dd63bd92d6b27a2649e0d736553a66cc7",
        },
    },
    ResetSpec {
        owner: ClassifierId::FeatureValue,
        attribute: "isInitial",
        behavior: ResetBehavior::SetterDefault(34),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FeatureValueImpl.java",
            line: 247,
            sha256: "06d158252791fccd475b35f55547369dd63bd92d6b27a2649e0d736553a66cc7",
        },
    },
    ResetSpec {
        owner: ClassifierId::FlowDefinition,
        attribute: "isImplied",
        behavior: ResetBehavior::SetterDefault(35),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowDefinitionImpl.java",
            line: 474,
            sha256: "244d80d8b39947bd40618e40af85847d44035c8220b238658960e4e1082b01ed",
        },
    },
    ResetSpec {
        owner: ClassifierId::FlowUsage,
        attribute: "isIndividual",
        behavior: ResetBehavior::SetterDefault(36),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 578,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    ResetSpec {
        owner: ClassifierId::FlowUsage,
        attribute: "portionKind",
        behavior: ResetBehavior::UnsetMethod,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/FlowUsageImpl.java",
            line: 578,
            sha256: "145466ffac630c716684d9e45cd8d84923191a117ad7eec2b82d183c69cb24a1",
        },
    },
    ResetSpec {
        owner: ClassifierId::Import,
        attribute: "isImportAll",
        behavior: ResetBehavior::SetterDefault(39),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 310,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    ResetSpec {
        owner: ClassifierId::Import,
        attribute: "isRecursive",
        behavior: ResetBehavior::SetterDefault(40),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 310,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    ResetSpec {
        owner: ClassifierId::Import,
        attribute: "visibility",
        behavior: ResetBehavior::SetterDefault(41),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/ImportImpl.java",
            line: 310,
            sha256: "2582b6d61ae20df3ef64bb1d55c09018e3c000bdd0deeeceb109d9868b7e2158",
        },
    },
    ResetSpec {
        owner: ClassifierId::Invariant,
        attribute: "isNegated",
        behavior: ResetBehavior::SetterDefault(42),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/InvariantImpl.java",
            line: 131,
            sha256: "1e5440a1324cb70f52d4a2bb313ac4ade34e1ac0cd608c5e720e3d424a09bbdf",
        },
    },
    ResetSpec {
        owner: ClassifierId::LibraryPackage,
        attribute: "isStandard",
        behavior: ResetBehavior::SetterDefault(43),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LibraryPackageImpl.java",
            line: 143,
            sha256: "9103340eb814396cb02705d14b9062fa660b0577879c31538eae5534a552e99f",
        },
    },
    ResetSpec {
        owner: ClassifierId::LiteralBoolean,
        attribute: "value",
        behavior: ResetBehavior::SetterDefault(44),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralBooleanImpl.java",
            line: 132,
            sha256: "1cdf2c81a78ab9a5ae2437a4cfe959aa31ee9f37107949638abb3d1fb70d52d2",
        },
    },
    ResetSpec {
        owner: ClassifierId::LiteralInteger,
        attribute: "value",
        behavior: ResetBehavior::SetterDefault(45),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralIntegerImpl.java",
            line: 132,
            sha256: "ea23684de620c444cd6cd43c5562e6c4f1a180d9f8cfb02ed58284134d06f96c",
        },
    },
    ResetSpec {
        owner: ClassifierId::LiteralRational,
        attribute: "value",
        behavior: ResetBehavior::SetterDefault(46),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralRationalImpl.java",
            line: 132,
            sha256: "0e99ac7f50b7d612c5f1450d76195d8b5f4c8f0b3ef7db87ab94b82145f8d632",
        },
    },
    ResetSpec {
        owner: ClassifierId::LiteralString,
        attribute: "value",
        behavior: ResetBehavior::SetterDefault(47),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/LiteralStringImpl.java",
            line: 131,
            sha256: "0f51f25d8b70036c25befc883f7dc4e88e3b63b60555d3b8ecd633a3b4f1441f",
        },
    },
    ResetSpec {
        owner: ClassifierId::Membership,
        attribute: "memberName",
        behavior: ResetBehavior::SetterDefault(49),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 391,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    ResetSpec {
        owner: ClassifierId::Membership,
        attribute: "memberShortName",
        behavior: ResetBehavior::SetterDefault(50),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 391,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    ResetSpec {
        owner: ClassifierId::Membership,
        attribute: "visibility",
        behavior: ResetBehavior::SetterDefault(51),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/MembershipImpl.java",
            line: 391,
            sha256: "4458b25fa330b95a673013337b46b69627064c5b4c716663b62bd71bcdf4bf8b",
        },
    },
    ResetSpec {
        owner: ClassifierId::OccurrenceDefinition,
        attribute: "isIndividual",
        behavior: ResetBehavior::SetterDefault(52),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceDefinitionImpl.java",
            line: 132,
            sha256: "da09e6d912522b3ce8080ed9d8598abee5652d238df2ea6e0b5f44994871ef75",
        },
    },
    ResetSpec {
        owner: ClassifierId::OccurrenceUsage,
        attribute: "isIndividual",
        behavior: ResetBehavior::SetterDefault(53),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 280,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
    ResetSpec {
        owner: ClassifierId::OccurrenceUsage,
        attribute: "portionKind",
        behavior: ResetBehavior::UnsetMethod,
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OccurrenceUsageImpl.java",
            line: 280,
            sha256: "b1c2101c39b1906a600de57f0f9976a115a36019a54995caf42753e8ee66df75",
        },
    },
    ResetSpec {
        owner: ClassifierId::OperatorExpression,
        attribute: "operator",
        behavior: ResetBehavior::SetterDefault(55),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/OperatorExpressionImpl.java",
            line: 133,
            sha256: "af2646607ccbe2e363494a3b54c2004ab7777766fc93e08590d591a16b8d5b68",
        },
    },
    ResetSpec {
        owner: ClassifierId::Relationship,
        attribute: "isImplied",
        behavior: ResetBehavior::SetterDefault(59),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RelationshipImpl.java",
            line: 376,
            sha256: "fbd56ac8a6e5792869844e2a91c8b55fa7aecd6cc2e04b3e5315382a457bbe3d",
        },
    },
    ResetSpec {
        owner: ClassifierId::RequirementConstraintMembership,
        attribute: "kind",
        behavior: ResetBehavior::SetterDefault(60),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementConstraintMembershipImpl.java",
            line: 202,
            sha256: "96bd03d8e2490604aa1b474eb8a69427ac45c35b96af4157337f4f78221f3e45",
        },
    },
    ResetSpec {
        owner: ClassifierId::RequirementDefinition,
        attribute: "reqId",
        behavior: ResetBehavior::SetterDefault(61),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementDefinitionImpl.java",
            line: 297,
            sha256: "877901d3a8da850c15105ce9b633e45d825f52cdbeef41bf8b1db3ff4cacd917",
        },
    },
    ResetSpec {
        owner: ClassifierId::RequirementUsage,
        attribute: "reqId",
        behavior: ResetBehavior::SetterDefault(62),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/RequirementUsageImpl.java",
            line: 337,
            sha256: "e6e28cd8c181652e257b31369cf8e2697a628a1a2b2a8d73bdf70f36530accf2",
        },
    },
    ResetSpec {
        owner: ClassifierId::SatisfyRequirementUsage,
        attribute: "isNegated",
        behavior: ResetBehavior::SetterDefault(63),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/SatisfyRequirementUsageImpl.java",
            line: 262,
            sha256: "f55cf0eddbbb8d5af72f925674eec7290bf065e0d728a6e96649cce0cb429de3",
        },
    },
    ResetSpec {
        owner: ClassifierId::StateDefinition,
        attribute: "isParallel",
        behavior: ResetBehavior::SetterDefault(64),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateDefinitionImpl.java",
            line: 260,
            sha256: "192b52b80e56b4816bf6b8bdf12478acb0f65223b897422a3014a95ae19bebf0",
        },
    },
    ResetSpec {
        owner: ClassifierId::StateSubactionMembership,
        attribute: "kind",
        behavior: ResetBehavior::SetterDefault(65),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateSubactionMembershipImpl.java",
            line: 174,
            sha256: "f362f4fdc765663f74f3a3ea0aa2645779d528e07cf8a3511520c4d4b4990644",
        },
    },
    ResetSpec {
        owner: ClassifierId::StateUsage,
        attribute: "isParallel",
        behavior: ResetBehavior::SetterDefault(66),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/StateUsageImpl.java",
            line: 303,
            sha256: "235b477331ac3e3311e37b7a3ec1bebd2ace7bdc56ef0a30781f4213ff224865",
        },
    },
    ResetSpec {
        owner: ClassifierId::TextualRepresentation,
        attribute: "body",
        behavior: ResetBehavior::SetterDefault(67),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TextualRepresentationImpl.java",
            line: 221,
            sha256: "31d09a2b2639048566c9f56f04f7c12d53d107c5420effb2e4ff7d8ba5c52b3e",
        },
    },
    ResetSpec {
        owner: ClassifierId::TextualRepresentation,
        attribute: "language",
        behavior: ResetBehavior::SetterDefault(68),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TextualRepresentationImpl.java",
            line: 221,
            sha256: "31d09a2b2639048566c9f56f04f7c12d53d107c5420effb2e4ff7d8ba5c52b3e",
        },
    },
    ResetSpec {
        owner: ClassifierId::TransitionFeatureMembership,
        attribute: "kind",
        behavior: ResetBehavior::SetterDefault(69),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TransitionFeatureMembershipImpl.java",
            line: 169,
            sha256: "a8cd905da455a039bfe81b0ead676c0167fc53ff78064de4872b8c1913d4115f",
        },
    },
    ResetSpec {
        owner: ClassifierId::TriggerInvocationExpression,
        attribute: "kind",
        behavior: ResetBehavior::SetterDefault(70),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TriggerInvocationExpressionImpl.java",
            line: 133,
            sha256: "35f26a38a2c17afcac3ed32d568179dff68854722f9270a3be6a165a0e7a102f",
        },
    },
    ResetSpec {
        owner: ClassifierId::Type,
        attribute: "isAbstract",
        behavior: ResetBehavior::SetterDefault(71),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 970,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    ResetSpec {
        owner: ClassifierId::Type,
        attribute: "isSufficient",
        behavior: ResetBehavior::SetterDefault(73),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/TypeImpl.java",
            line: 970,
            sha256: "3b084a9bd887cc70131c630997830f6e61875890c6919c9137a62a365d91617d",
        },
    },
    ResetSpec {
        owner: ClassifierId::Usage,
        attribute: "isVariation",
        behavior: ResetBehavior::SetterDefault(75),
        source: SourceRef {
            path: "backend/metamodel/syson-sysml-metamodel/src/main/java/org/eclipse/syson/sysml/impl/UsageImpl.java",
            line: 938,
            sha256: "88ff141769debfe9a8c9cf575b0ec05a289f267479af5800d6526fc470cabe26",
        },
    },
];
// END PINNED SOURCE TABLES

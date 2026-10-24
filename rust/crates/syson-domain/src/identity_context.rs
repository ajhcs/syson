// Copyright (c) 2024, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Rust source mapping: porting/domain/identity-context/SOURCE-CONTRACT.json.
//! UUID decisions over explicit immutable source observations, not native EMF adapters.
use crate::uuid::{Uuid, standard_library_uuid, uuid_v5};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observed<T> {
    Known(T),
    Unresolved,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementClass {
    LibraryPackage { standard: bool },
    Import,
    Membership,
    Specialization,
    TypeFeaturing,
    Other,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceObservation<'a> {
    Absent,
    Present { uri: Observed<Option<&'a str>> },
    Unresolved,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryNamespaceObservation<'a> {
    Absent,
    OtherNamespace,
    LibraryPackage {
        standard: bool,
        element_id: Observed<Option<&'a str>>,
    },
    Unresolved,
}
/// Caller-supplied source getter facts; no native values or class predicates are inferred.
/// One immutable observation cannot reproduce changing/side-effectful getter calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElementContext<'a> {
    pub class: ElementClass,
    pub resource: ResourceObservation<'a>,
    pub library_namespace: LibraryNamespaceObservation<'a>,
    pub declared_name: Observed<Option<&'a str>>,
    pub qualified_name: Observed<Option<&'a str>>,
}
pub trait FreshIdProvider {
    type Error;
    fn next_v4(&mut self) -> Result<Uuid, Self::Error>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SemanticId(pub Uuid);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResourceId(pub Uuid);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityOrigin {
    FreshV4,
    StandardLibraryRootV5,
    StandardLibraryMemberV5,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeneratedSemanticId {
    pub id: SemanticId,
    pub origin: IdentityOrigin,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UuidTextError {
    NonCanonicalSpelling,
    InvalidHex,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextField {
    Resource,
    ResourceUri,
    LibraryNamespace,
    DeclaredName,
    QualifiedName,
    NamespaceElementId,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityError<E> {
    Fresh(E),
    ProviderNotV4,
    NullElement,
    NullResourceUri,
    NullNamespaceElementId,
    Unresolved(ContextField),
    NamespaceUuid(UuidTextError),
}
fn resolve<T, E>(value: Observed<T>, field: ContextField) -> Result<T, IdentityError<E>> {
    match value {
        Observed::Known(value) => Ok(value),
        Observed::Unresolved => Err(IdentityError::Unresolved(field)),
    }
}

/// Preserve eager UUID.randomUUID consumption even when a deterministic branch wins.
/// Failed/exhausted providers are never retried, replaced or silently bypassed.
pub fn generate_semantic_id<P: FreshIdProvider>(
    context: Option<ElementContext<'_>>,
    provider: &mut P,
) -> Result<GeneratedSemanticId, IdentityError<P::Error>> {
    let fresh = provider.next_v4().map_err(IdentityError::Fresh)?;
    if fresh.0[6] >> 4 != 4 || fresh.0[8] & 0xc0 != 0x80 {
        return Err(IdentityError::ProviderNotV4);
    }
    let context = context.ok_or(IdentityError::NullElement)?;
    let mut result = GeneratedSemanticId {
        id: SemanticId(fresh),
        origin: IdentityOrigin::FreshV4,
    };
    if matches!(
        context.class,
        ElementClass::LibraryPackage { standard: true }
    ) {
        match context.resource {
            ResourceObservation::Absent => {}
            ResourceObservation::Unresolved => {
                return Err(IdentityError::Unresolved(ContextField::Resource));
            }
            ResourceObservation::Present { uri } => {
                let uri = resolve(uri, ContextField::ResourceUri)?
                    .ok_or(IdentityError::NullResourceUri)?;
                if let Some(name) = resolve(context.qualified_name, ContextField::QualifiedName)? {
                    result = GeneratedSemanticId {
                        id: SemanticId(standard_library_uuid(uri, name)),
                        origin: IdentityOrigin::StandardLibraryRootV5,
                    };
                }
            }
        }
    } else {
        match context.library_namespace {
            LibraryNamespaceObservation::Unresolved => {
                return Err(IdentityError::Unresolved(ContextField::LibraryNamespace));
            }
            LibraryNamespaceObservation::LibraryPackage {
                standard: true,
                element_id,
            } => {
                let declared = resolve(context.declared_name, ContextField::DeclaredName)?;
                let excluded = matches!(
                    context.class,
                    ElementClass::Import
                        | ElementClass::Membership
                        | ElementClass::Specialization
                        | ElementClass::TypeFeaturing
                );
                if declared.is_some()
                    && !excluded
                    && let Some(name) =
                        resolve(context.qualified_name, ContextField::QualifiedName)?
                {
                    let namespace = resolve(element_id, ContextField::NamespaceElementId)?
                        .ok_or(IdentityError::NullNamespaceElementId)?;
                    let namespace = parse_uuid(namespace).map_err(IdentityError::NamespaceUuid)?;
                    result = GeneratedSemanticId {
                        id: SemanticId(uuid_v5(namespace, name)),
                        origin: IdentityOrigin::StandardLibraryMemberV5,
                    };
                }
            }
            _ => {}
        }
    }
    Ok(result)
}

/// Selected domain: 36 ASCII UUID characters, accepting either hex case.
/// Additional Java UUID.fromString spellings remain explicitly unsupported.
fn parse_uuid(text: &str) -> Result<Uuid, UuidTextError> {
    let bytes = text.as_bytes();
    if bytes.len() != 36 || [8, 13, 18, 23].iter().any(|i| bytes[*i] != b'-') {
        return Err(UuidTextError::NonCanonicalSpelling);
    }
    let mut output = [0u8; 16];
    let mut nibble = 0;
    for (i, byte) in bytes.iter().enumerate() {
        if [8, 13, 18, 23].contains(&i) {
            continue;
        }
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => return Err(UuidTextError::InvalidHex),
        };
        output[nibble / 2] = (output[nibble / 2] << 4) | digit;
        nibble += 1;
    }
    Ok(Uuid(output))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectedIdentityError {
    MissingResource,
    MissingSemantic,
    ResourceText(UuidTextError),
    SemanticText(UuidTextError),
}
/// Distinct identity roles with retained original spelling; equal values are allowed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectedIdentity<'a> {
    resource: ResourceId,
    semantic: SemanticId,
    resource_text: &'a str,
    semantic_text: &'a str,
}
impl<'a> SelectedIdentity<'a> {
    pub fn from_fields(
        resource: Option<&'a str>,
        semantic: Option<&'a str>,
    ) -> Result<Self, SelectedIdentityError> {
        let resource_text = resource.ok_or(SelectedIdentityError::MissingResource)?;
        let semantic_text = semantic.ok_or(SelectedIdentityError::MissingSemantic)?;
        Ok(Self {
            resource: ResourceId(
                parse_uuid(resource_text).map_err(SelectedIdentityError::ResourceText)?,
            ),
            semantic: SemanticId(
                parse_uuid(semantic_text).map_err(SelectedIdentityError::SemanticText)?,
            ),
            resource_text,
            semantic_text,
        })
    }
    pub fn resource(&self) -> ResourceId {
        self.resource
    }
    pub fn semantic(&self) -> SemanticId {
        self.semantic
    }
    /// Selected EMF JSON id and data.elementId fields unchanged; not a serializer.
    pub fn fields(&self) -> (&'a str, &'a str) {
        (self.resource_text, self.semantic_text)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collision {
    Resource(ResourceId),
    Semantic(SemanticId),
}
/// Proposed selected-document reservation facility, separate from upstream generation.
/// Separate instances define separate scopes; no global/native resource registry is implied.
#[derive(Debug, Default)]
pub struct SelectedIdentityRegistry {
    resources: HashSet<ResourceId>,
    semantics: HashSet<SemanticId>,
}
impl SelectedIdentityRegistry {
    /// Check both roles before mutation; a late semantic collision reserves neither field.
    pub fn admit(&mut self, identity: SelectedIdentity<'_>) -> Result<(), Collision> {
        if self.resources.contains(&identity.resource) {
            return Err(Collision::Resource(identity.resource));
        }
        if self.semantics.contains(&identity.semantic) {
            return Err(Collision::Semantic(identity.semantic));
        }
        self.resources.insert(identity.resource);
        self.semantics.insert(identity.semantic);
        Ok(())
    }
    pub fn restore<'a>(
        identities: impl IntoIterator<Item = SelectedIdentity<'a>>,
    ) -> Result<Self, Collision> {
        let mut registry = Self::default();
        for identity in identities {
            registry.admit(identity)?;
        }
        Ok(registry)
    }
    pub fn contains_resource(&self, id: ResourceId) -> bool {
        self.resources.contains(&id)
    }
    pub fn contains_semantic(&self, id: SemanticId) -> bool {
        self.semantics.contains(&id)
    }
    pub fn len(&self) -> usize {
        self.resources.len()
    }
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}

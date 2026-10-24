// Copyright (c) 2024, 2025, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Contributors: Obeo - Java read/serializer contracts; SysON Rust port contributors - typed adapters.
//! Two source-frozen REST read contracts, without transport, native getters or a server.
#![forbid(unsafe_code)]
use serde_json::Value;
use std::collections::BTreeSet;

pub const ELEMENT_PATH: &str =
    "/api/rest/projects/{projectId}/commits/{commitId}/elements/{elementId}";
pub const RELATIONSHIPS_PATH: &str =
    "/api/rest/projects/{projectId}/commits/{commitId}/elements/{relatedElementId}/relationships";
pub const QUERY_TIMEOUT_SECONDS: u64 = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gap {
    InvalidJson(String),
    ExpectedObject,
    ExpectedArray,
    MissingType,
    InvalidId,
    NullElementId,
    NullSuccessElement,
    DuplicateOrReservedField(String),
    UnresolvedFeature(String),
    ContextUnresolved,
    UnmappedDispatchFailure,
    InverseReferencesUnresolved,
    ConflictingRelationshipObservation,
}

/// Validation uses serde_json; wire emission retains original text and number
/// spellings. Parsed structural views must not be substituted for original wire bytes.
#[derive(Debug, Clone)]
pub struct Json {
    raw: String,
    value: Value,
}
impl Json {
    pub fn parse(text: &str) -> Result<Self, Gap> {
        let value = serde_json::from_str(text)
            .map_err(|e: serde_json::Error| Gap::InvalidJson(e.to_string()))?;
        Ok(Self {
            raw: text.into(),
            value,
        })
    }
    pub fn wire(&self) -> &str {
        &self.raw
    }
    pub fn value(&self) -> &Value {
        &self.value
    }
}

/// Unknown properties are opaque and retained. No native model class is inferred.
#[derive(Debug, Clone)]
pub struct ElementDto(Json);
impl ElementDto {
    pub fn parse(text: &str) -> Result<Self, Gap> {
        let json = Json::parse(text)?;
        let object = json.value().as_object().ok_or(Gap::ExpectedObject)?;
        if !object.get("@type").is_some_and(Value::is_string) {
            return Err(Gap::MissingType);
        }
        if object.get("@id").is_some_and(|id| !id.is_string()) {
            return Err(Gap::InvalidId);
        }
        Ok(Self(json))
    }
    pub fn json(&self) -> &Json {
        &self.0
    }
    pub fn into_json(self) -> Json {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct RelationshipsDto(Json);
impl RelationshipsDto {
    pub fn parse(text: &str) -> Result<Self, Gap> {
        let json = Json::parse(text)?;
        if !json.value().is_array() {
            return Err(Gap::ExpectedArray);
        }
        Ok(Self(json))
    }
    pub fn from_elements(elements: &[ElementDto]) -> Result<Self, Gap> {
        Self::parse(&format!(
            "[{}]",
            elements
                .iter()
                .map(|e| e.json().wire())
                .collect::<Vec<_>>()
                .join(",")
        ))
    }
    pub fn json(&self) -> &Json {
        &self.0
    }
    pub fn into_json(self) -> Json {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    In,
    Out,
    Both,
}
impl Direction {
    pub fn observed_query(value: Option<Self>) -> Self {
        value.unwrap_or(Self::Both)
    }
    pub fn source_name(self) -> &'static str {
        match self {
            Self::In => "IN",
            Self::Out => "OUT",
            Self::Both => "BOTH",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    ElementById,
    Relationships,
}
#[derive(Debug, Clone, Copy)]
pub enum Context {
    Absent,
    Present,
    Unresolved,
}
/// These are observations of internal dispatcher outcomes, not HTTP envelopes.
#[derive(Debug, Clone)]
pub enum Dispatch {
    ElementSuccess(Json),
    RelationshipsSuccess(RelationshipsDto),
    NonSuccess,
    CompletedWithoutPayload,
    UnqualifiedFailure,
}
#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub body: Option<Json>,
}

/// Commit IDs are not checked by these controller bodies. UUID/query binding,
/// authentication and exceptions outside this observed branch are not simulated.
pub fn controller_response(
    operation: Operation,
    context: Context,
    dispatch: Dispatch,
) -> Result<Response, Gap> {
    let not_found = || Response {
        status: 404,
        body: None,
    };
    match context {
        Context::Absent => return Ok(not_found()),
        Context::Unresolved => return Err(Gap::ContextUnresolved),
        Context::Present => {}
    }
    let body = match (operation, dispatch) {
        (Operation::ElementById, Dispatch::ElementSuccess(json)) => {
            if json.value().is_null() {
                return Err(Gap::NullSuccessElement);
            }
            json
        }
        (Operation::Relationships, Dispatch::RelationshipsSuccess(json)) => json.into_json(),
        (_, Dispatch::UnqualifiedFailure) => return Err(Gap::UnmappedDispatchFailure),
        _ => return Ok(not_found()),
    };
    Ok(Response {
        status: 200,
        body: Some(body),
    })
}

#[derive(Debug, Clone)]
pub struct Target<'a> {
    pub is_element: bool,
    pub element_id: Option<&'a str>,
    pub virtual_link: bool,
    pub standard_library: bool,
}
#[derive(Debug, Clone)]
pub enum Reference<'a> {
    Null,
    Target(Target<'a>),
    List(Vec<Target<'a>>),
    OtherNonNull,
}
#[derive(Debug, Clone)]
pub enum FeatureValue<'a> {
    Attribute(Json),
    EnumeratorLiteral(Option<&'a str>),
    Reference {
        many: bool,
        eannotations_identity: bool,
        value: Reference<'a>,
    },
    Unresolved,
}
#[derive(Debug, Clone)]
pub struct Feature<'a> {
    pub name: &'a str,
    pub value: FeatureValue<'a>,
}

fn quote(value: &str) -> String {
    Value::String(value.into()).to_string()
}
fn target_json(target: &Target<'_>, many: bool) -> Option<String> {
    if target.is_element && !target.virtual_link && (!many || !target.standard_library) {
        target
            .element_id
            .map(|id| format!("{{\"@id\":{}}}", quote(id)))
    } else {
        None
    }
}

/// Serialize caller-observed native values. Getter computation, proxy behavior,
/// Java malformed UTF16 and Jackson numeric/object conversion remain unqualified.
/// Raw JSON attributes are validated inputs; no float/string coercion is applied.
pub fn serialize_element(
    element_id: Option<&str>,
    type_name: &str,
    features: &[Feature<'_>],
) -> Result<ElementDto, Gap> {
    let id = element_id.ok_or(Gap::NullElementId)?;
    let mut fields = vec![
        format!("\"@id\":{}", quote(id)),
        format!("\"@type\":{}", quote(type_name)),
    ];
    let mut ordered = features.iter().collect::<Vec<_>>();
    ordered.sort_by(|a, b| a.name.encode_utf16().cmp(b.name.encode_utf16()));
    let mut names = BTreeSet::from(["@id", "@type"]);
    for feature in ordered {
        if !names.insert(feature.name) {
            return Err(Gap::DuplicateOrReservedField(feature.name.into()));
        }
        let raw = match &feature.value {
            FeatureValue::Attribute(value) => Some(value.wire().to_owned()),
            FeatureValue::EnumeratorLiteral(value) => Some(value.map_or("null".into(), quote)),
            FeatureValue::Unresolved => return Err(Gap::UnresolvedFeature(feature.name.into())),
            FeatureValue::Reference {
                many,
                eannotations_identity,
                value,
            } => {
                if *eannotations_identity {
                    None
                } else {
                    match value {
                        Reference::Null => Some("null".into()),
                        Reference::List(items) if *many => Some(format!(
                            "[{}]",
                            items
                                .iter()
                                .filter_map(|t| target_json(t, true))
                                .collect::<Vec<_>>()
                                .join(",")
                        )),
                        Reference::Target(target) if !many => target_json(target, false),
                        _ if *many => Some("[]".into()),
                        _ => None,
                    }
                }
            }
        };
        if let Some(raw) = raw {
            fields.push(format!("{}:{}", quote(feature.name), raw));
        }
    }
    ElementDto::parse(&format!("{{{}}}", fields.join(",")))
}

/// Already-admitted root snapshots. This is not ResourceSet loading or namespace unwrapping.
#[derive(Debug)]
pub struct TraversalNode {
    pub token: usize,
    pub is_element: bool,
    pub implied_relationship: bool,
    pub children: Vec<Self>,
}
pub fn visible_root_contents(roots: &[TraversalNode]) -> Vec<usize> {
    let mut out = Vec::new();
    for root in roots.iter().filter(|r| r.is_element) {
        out.push(root.token); // The source does not apply the descendant filter to roots.
        let mut pending = root.children.iter().rev().collect::<Vec<_>>();
        while let Some(node) = pending.pop() {
            if !node.implied_relationship {
                out.push(node.token);
            }
            pending.extend(node.children.iter().rev()); // Omitting a node does not prune descendants.
        }
    }
    out
}

#[derive(Debug)]
pub struct RelationshipFact {
    pub object_identity: usize,
    pub implied: bool,
}
/// Caller identity is scoped to one native observation. Iteration order is only
/// a Rust presentation order, never asserted equal to Java HashSet wire order.
#[derive(Debug)]
pub struct UnorderedRelationships<'a>(pub Vec<&'a RelationshipFact>);
#[derive(Debug, Clone, Copy)]
pub enum InverseRelationships<'a> {
    KnownAdapterAbsent,
    Observed(&'a [&'a RelationshipFact]),
    Unresolved,
}
pub fn relationship_members<'a>(
    target_is_element: bool,
    direction: Direction,
    owned: &[&'a RelationshipFact],
    inverse_adapter: InverseRelationships<'a>,
) -> Result<UnorderedRelationships<'a>, Gap> {
    if !target_is_element {
        return Ok(UnorderedRelationships(Vec::new()));
    }
    let mut candidates = Vec::new();
    if matches!(direction, Direction::Out | Direction::Both) {
        candidates.extend_from_slice(owned);
    }
    if matches!(direction, Direction::In | Direction::Both) {
        match inverse_adapter {
            InverseRelationships::KnownAdapterAbsent => {}
            InverseRelationships::Observed(items) => candidates.extend_from_slice(items),
            InverseRelationships::Unresolved => return Err(Gap::InverseReferencesUnresolved),
        }
    }
    let mut seen = std::collections::BTreeMap::new();
    let mut members = Vec::new();
    for candidate in candidates {
        match seen.insert(candidate.object_identity, candidate.implied) {
            Some(prior) if prior != candidate.implied => {
                return Err(Gap::ConflictingRelationshipObservation);
            }
            Some(_) => {}
            None if !candidate.implied => members.push(candidate),
            None => {}
        }
    }
    Ok(UnorderedRelationships(members))
}

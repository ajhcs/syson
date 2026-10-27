// SPDX-License-Identifier: EPL-2.0
//! Two source-derived rules over finite selected package/part documents.
#![forbid(unsafe_code)]
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use syson_ast::{Arena, NodeId, Slot, Span, Value};
use syson_ast_adapter::{Adapter, Limits};
use syson_metamodel::{self as mm, ClassifierId as C};
use syson_model::{Document, ModelError};
use syson_references::{FragmentIdentity, Resource};
use syson_semantic_names::{NameKind, Snapshot};

/// Selected validation admission, not native SysON project capacity. The bound
/// applies before summaries, parser construction or the names snapshot clone.
pub const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;
pub const NAMESPACE: &str = "validateNamespaceDistinguishibility";
pub const PART: &str = "validatePartUsagePartDefinition";
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UnsupportedRule(String),
    UnsupportedClassifier(String),
    UnsupportedShape(&'static str),
    SourceAssociation,
    Parse(syson_ast_adapter::Error),
    Names(syson_semantic_names::Error),
    UnresolvedTyping(String),
    Cancelled,
}
impl From<syson_semantic_names::Error> for Error {
    fn from(e: syson_semantic_names::Error) -> Self {
        Self::Names(e)
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub rule: &'static str,
    pub element: String,
    pub qualified_name: Option<String>,
    /// Current upstream SysML rules are warnings, even when a check fails.
    pub severity: &'static str,
    pub span: Span,
    /// One-based source line and UTF-16 column, derived from the same F11 input.
    pub line: usize,
    pub column_utf16: usize,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Check {
    pub rule: &'static str,
    pub element: String,
    pub passed: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Report {
    /// Only requested, applicable checks; this is never an all-rules verdict.
    pub checks: Vec<Check>,
    pub diagnostics: Vec<Diagnostic>,
}
fn check(cancel: &AtomicBool) -> Result<(), Error> {
    if cancel.load(Ordering::Relaxed) {
        Err(Error::Cancelled)
    } else {
        Ok(())
    }
}
fn class(value: &serde_json::Value) -> Result<C, Error> {
    value["eClass"]
        .as_str()
        .and_then(|s| s.strip_prefix("sysml:"))
        .and_then(|s| mm::lookup(mm::Package::Domain, s))
        .ok_or_else(|| Error::UnsupportedClassifier(value["eClass"].to_string()))
}
fn declaration(c: C) -> bool {
    matches!(c, C::Package | C::PartDefinition | C::PartUsage)
}
fn ast_parent(arena: &Arena, id: NodeId) -> Option<NodeId> {
    let mut current = id;
    while let Some(parent) = arena.node(current)?.container() {
        current = parent.node;
        if matches!(
            arena.node(current)?.kind(),
            Some("Package" | "PartDefinition" | "PartUsage")
        ) {
            return Some(current);
        }
    }
    None
}
/// Source association is deliberately finite: declarations must occur in the
/// same containment order, class, cleaned declared-name and semantic owner.
/// Other scalar/typing data may differ, allowing diagnostics after model edits.
/// The source text is a caller observation, not a native parser/source-map claim.
/// Local FeatureTyping.type uses F14's explicit semantic-ID representation only.
/// No native node-ID fallback, resource loading, inherited type or full AQL runs.
pub fn validate(
    source: &str,
    document: &Document,
    rules: &[&str],
    cancel: Arc<AtomicBool>,
) -> Result<Report, Error> {
    check(&cancel)?;
    if rules.is_empty() {
        return Err(Error::UnsupportedShape(
            "explicit nonempty rule selection required",
        ));
    }
    let mut selected = Vec::new();
    for rule in rules {
        let rule = match *rule {
            NAMESPACE => NAMESPACE,
            PART => PART,
            other => return Err(Error::UnsupportedRule(other.into())),
        };
        if !selected.contains(&rule) {
            selected.push(rule);
        }
    }
    // Use the existing model serializer authority for admission, without owning
    // output. Even otherwise ignored envelope data must fit before a full names
    // snapshot is cloned. This is an explicit selected-validator budget.
    document
        .json_size(MAX_DOCUMENT_BYTES, || cancel.load(Ordering::Relaxed))
        .map_err(|error| match error {
            ModelError::JsonAdmission(syson_json_value::Error::Cancelled) => Error::Cancelled,
            _ => Error::UnsupportedShape("selected document byte/depth budget"),
        })?;
    let arena = Adapter::pinned()
        .map_err(Error::Parse)?
        .construct(source, Limits::default(), cancel.clone())
        .map_err(Error::Parse)?;
    let mut ast: Vec<_> = arena
        .nodes()
        .iter()
        .filter(|(_, n)| matches!(n.kind(), Some("Package" | "PartDefinition" | "PartUsage")))
        .collect();
    ast.sort_by_key(|(_, n)| n.origin().map(|o| o.span.bytes.start));
    let rows = document.elements();
    if rows.len() > 2048 {
        return Err(Error::UnsupportedShape("selected document budget"));
    }
    // Document already owns the validated envelope. Borrow it instead of a
    // full canonical String plus generic Value reparse of opaque JSON.
    if document.value()["content"]
        .as_array()
        .is_some_and(|roots| roots.iter().any(|root| root["eClass"] != "sysml:Package"))
    {
        return Err(Error::UnsupportedShape("root packages only"));
    }
    let mut declarations = Vec::new();
    for row in &rows {
        check(&cancel)?;
        let node = document
            .element(&row.element_id)
            .ok_or(Error::SourceAssociation)?;
        let c = class(node)?;
        if declaration(c) {
            declarations.push((row, c));
        } else if !matches!(
            c,
            C::OwningMembership | C::FeatureMembership | C::FeatureTyping
        ) {
            return Err(Error::UnsupportedClassifier(row.class.clone()));
        }
        let allowed: &[&str] = match c {
            C::OwningMembership | C::FeatureMembership => {
                &["elementId", "ownedRelatedElement", "ownedRelationship"]
            }
            C::FeatureTyping => &[
                "elementId",
                "type",
                "typedFeature",
                "ownedRelatedElement",
                "ownedRelationship",
            ],
            C::PartUsage => &[
                "elementId",
                "declaredName",
                "declaredShortName",
                "isComposite",
                "ownedRelationship",
                "ownedRelatedElement",
            ],
            _ => &[
                "elementId",
                "declaredName",
                "declaredShortName",
                "ownedRelationship",
                "ownedRelatedElement",
            ],
        };
        if node.as_object().is_none_or(|o| {
            o.keys()
                .any(|k| !["id", "eClass", "data"].contains(&k.as_str()))
        }) || node["data"]
            .as_object()
            .is_none_or(|o| o.keys().any(|k| !allowed.contains(&k.as_str())))
            || node["data"]
                .get("isComposite")
                .is_some_and(|v| !v.is_boolean())
        {
            return Err(Error::UnsupportedShape("fields outside finite projection"));
        }
        // Extra containment channels make correspondence and ownership ambiguous.
        for field in ["ownedRelationship", "ownedRelatedElement"] {
            if node["data"].get(field).is_some_and(|v| !v.is_array()) {
                return Err(Error::UnsupportedShape("containment must be arrays"));
            }
        }
        match c {
            C::OwningMembership | C::FeatureMembership => {
                if node["data"]["ownedRelatedElement"].as_array().map(Vec::len) != Some(1)
                    || node["data"]["ownedRelationship"]
                        .as_array()
                        .is_some_and(|a| !a.is_empty())
                {
                    return Err(Error::UnsupportedShape("one owned member per membership"));
                }
            }
            C::FeatureTyping => {
                if node["data"]["ownedRelatedElement"]
                    .as_array()
                    .is_some_and(|a| !a.is_empty())
                    || node["data"]["ownedRelationship"]
                        .as_array()
                        .is_some_and(|a| !a.is_empty())
                {
                    return Err(Error::UnsupportedShape("leaf typing"));
                }
            }
            _ if node["data"]["ownedRelatedElement"]
                .as_array()
                .is_some_and(|a| !a.is_empty()) =>
            {
                return Err(Error::UnsupportedShape("declarations own memberships"));
            }
            _ => (),
        }
    }
    if ast.len() != declarations.len() {
        return Err(Error::SourceAssociation);
    }
    let names = Snapshot::new(vec![Resource {
        key: "selected".into(),
        document: document.clone(),
        fragments: FragmentIdentity::SemanticElementId,
    }])?;
    let mut mapping = BTreeMap::new();
    let mut spans = BTreeMap::new();
    for ((id, node), (row, c)) in ast.iter().zip(&declarations) {
        check(&cancel)?;
        let raw = match node.slot("declaredName") {
            Slot::Present(d) => match &d.value {
                Value::String(s) => s,
                _ => return Err(Error::SourceAssociation),
            },
            _ => return Err(Error::SourceAssociation),
        };
        if node.kind() != Some(c.metadata().name())
            || row.name.as_deref() != Some(raw.replace('\'', "").as_str())
        {
            return Err(Error::SourceAssociation);
        }
        let expected = ast_parent(&arena, **id)
            .and_then(|p| mapping.get(&p))
            .cloned();
        let object = names.object("selected", &row.element_id)?;
        let actual = names.owner(&object)?.map(|o| o.element_id().to_owned());
        if actual != expected {
            return Err(Error::SourceAssociation);
        }
        mapping.insert(**id, row.element_id.clone());
        spans.insert(
            row.element_id.clone(),
            node.origin().ok_or(Error::SourceAssociation)?.span.clone(),
        );
    }
    let mut report = Report {
        checks: vec![],
        diagnostics: vec![],
    };
    for (row, c) in declarations {
        let object = names.object("selected", &row.element_id)?;
        let node = document
            .element(&row.element_id)
            .ok_or(Error::SourceAssociation)?;
        let relations = node["data"]["ownedRelationship"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        for relation in relations {
            let rc = class(relation)?;
            if matches!(rc, C::OwningMembership | C::FeatureMembership) {
                let child = &relation["data"]["ownedRelatedElement"][0];
                if !declaration(class(child)?) || (rc == C::FeatureMembership && c == C::Package) {
                    return Err(Error::UnsupportedShape("finite membership containment"));
                }
            } else if rc == C::FeatureTyping && c == C::PartUsage {
                if relation["data"]["typedFeature"].as_str() != Some(row.element_id.as_str()) {
                    return Err(Error::UnsupportedShape(
                        "typing must explicitly identify its semantic owner",
                    ));
                }
            } else {
                return Err(Error::UnsupportedShape("typing only on PartUsage"));
            }
        }
        for rule in &selected {
            check(&cancel)?;
            let passed = if *rule == NAMESPACE {
                let children = names.owned_elements(&object)?;
                let mut members = Vec::new();
                for child in &children {
                    let value = document
                        .element(child.element_id())
                        .ok_or(Error::SourceAssociation)?;
                    members.push((
                        class(value)?,
                        names.name(child, NameKind::Name)?,
                        names.name(child, NameKind::ShortName)?,
                    ));
                }
                let mut distinguishable = true;
                for (i, (a, an, asn)) in members.iter().enumerate() {
                    for (j, (b, bn, bsn)) in members.iter().enumerate() {
                        check(&cancel)?;
                        if i != j
                            && (mm::is_subtype(*a, *b) || mm::is_subtype(*b, *a))
                            && [an, asn].iter().any(|name| {
                                name.as_ref().is_some_and(|s| {
                                    bn.as_ref() == Some(s) || bsn.as_ref() == Some(s)
                                })
                            })
                        {
                            distinguishable = false;
                        }
                    }
                }
                distinguishable
            } else if c == C::PartUsage {
                let mut found = false;
                for relation in relations {
                    if class(relation)? == C::FeatureTyping {
                        match relation["data"].get("type") {
                            None | Some(serde_json::Value::Null) => (),
                            Some(serde_json::Value::String(id)) => {
                                let target = document
                                    .element(id)
                                    .ok_or_else(|| Error::UnresolvedTyping(id.clone()))?;
                                found |= class(target)? == C::PartDefinition;
                            }
                            _ => {
                                return Err(Error::UnsupportedShape(
                                    "local semantic typing string",
                                ));
                            }
                        }
                    }
                }
                found
            } else {
                continue;
            };
            report.checks.push(Check {
                rule,
                element: row.element_id.clone(),
                passed,
            });
            if !passed {
                let span = spans[&row.element_id].clone();
                let prefix = &source[..span.bytes.start];
                let (mut line, mut column_utf16) = (1, 1);
                let mut chars = prefix.chars().peekable();
                while let Some(c) = chars.next() {
                    match c {
                        '\r' => {
                            if chars.peek() == Some(&'\n') {
                                chars.next();
                            }
                            line += 1;
                            column_utf16 = 1;
                        }
                        '\n' => {
                            line += 1;
                            column_utf16 = 1;
                        }
                        _ => column_utf16 += c.len_utf16(),
                    }
                }
                report.diagnostics.push(Diagnostic {
                    rule,
                    element: row.element_id.clone(),
                    qualified_name: names.qualified_name(&object)?,
                    severity: "WARNING",
                    span,
                    line,
                    column_utf16,
                });
            }
        }
    }
    check(&cancel)?;
    Ok(report)
}

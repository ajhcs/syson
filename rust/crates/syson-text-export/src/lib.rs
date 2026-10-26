// SPDX-License-Identifier: EPL-2.0
//! Finite selected-model package/part export with an actual F14 import gate.
#![forbid(unsafe_code)]
use serde_json::Value;
use std::sync::{Arc, atomic::AtomicBool};
use syson_domain::names::printable_name;
use syson_model::{Document, Editor, NodeIds};
use syson_text_import::Importer;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Unsupported(&'static str),
    NonRoundTripName,
    DuplicateName,
    InvalidTyping,
    UnresolvableType,
    Limit,
    NotImportable(syson_text_import::Error),
    RoundTripMismatch,
}
/// Explicit bytes independent of the platform; no trailing newline or BOM.
#[derive(Clone, Copy, Debug, Default)]
pub enum Newline {
    #[default]
    Lf,
    CrLf,
}
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub newline: Newline,
    pub indentation: &'static str,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            newline: Newline::Lf,
            indentation: "    ",
        }
    }
}
struct Declaration<'a> {
    value: &'a Value,
    name: &'a str,
    parent: Option<usize>,
    children: Vec<usize>,
    typing: Option<&'a str>,
}
fn fields(value: &Value, allowed: &[&str]) -> Result<(), Error> {
    if !value
        .as_object()
        .is_some_and(|o| o.keys().all(|k| allowed.contains(&k.as_str())))
    {
        return Err(Error::Unsupported("field outside finite projection"));
    }
    Ok(())
}
fn array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value], Error> {
    match value.get(key) {
        None => Ok(&[]),
        Some(Value::Array(a)) => Ok(a),
        _ => Err(Error::Unsupported("containment array")),
    }
}
fn node(value: &Value, allowed: &[&str]) -> Result<(), Error> {
    fields(value, &["id", "eClass", "data"])?;
    fields(&value["data"], allowed)
}
fn collect<'a>(
    value: &'a Value,
    parent: Option<usize>,
    depth: usize,
    rows: &mut Vec<Declaration<'a>>,
) -> Result<usize, Error> {
    if depth >= 64 || rows.len() >= 256 {
        return Err(Error::Limit);
    }
    let class = value["eClass"].as_str().unwrap_or("");
    if !matches!(
        class,
        "sysml:Package" | "sysml:PartDefinition" | "sysml:PartUsage"
    ) {
        return Err(Error::Unsupported("declaration classifier"));
    }
    if parent.is_none() && class != "sysml:Package" {
        return Err(Error::Unsupported("root package required"));
    }
    node(
        value,
        &[
            "elementId",
            "declaredName",
            "ownedRelationship",
            "isComposite",
        ],
    )?;
    let data = &value["data"];
    if (class == "sysml:PartUsage" && data["isComposite"] != true)
        || (class != "sysml:PartUsage" && data.get("isComposite").is_some())
    {
        return Err(Error::Unsupported("composite/default modifiers"));
    }
    let name = data["declaredName"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or(Error::NonRoundTripName)?;
    if name
        .chars()
        .any(|c| c == '\'' || c == '\\' || c.is_control())
    {
        return Err(Error::NonRoundTripName);
    }
    if rows.iter().any(|r| r.parent == parent && r.name == name) {
        return Err(Error::DuplicateName);
    }
    let index = rows.len();
    rows.push(Declaration {
        value,
        name,
        parent,
        children: vec![],
        typing: None,
    });
    for relation in array(data, "ownedRelationship")? {
        match relation["eClass"].as_str() {
            Some("sysml:FeatureTyping") if class == "sysml:PartUsage" => {
                node(relation, &["elementId", "type", "typedFeature"])?;
                let typing = &relation["data"];
                if rows[index].typing.is_some() || typing["typedFeature"] != data["elementId"] {
                    return Err(Error::InvalidTyping);
                }
                rows[index].typing = Some(typing["type"].as_str().ok_or(Error::InvalidTyping)?);
            }
            Some("sysml:OwningMembership" | "sysml:FeatureMembership") => {
                node(relation, &["elementId", "ownedRelatedElement"])?;
                let children = array(&relation["data"], "ownedRelatedElement")?;
                if children.len() != 1 {
                    return Err(Error::Unsupported("one membership target required"));
                }
                let child = &children[0];
                let expected = if class != "sysml:Package" && child["eClass"] == "sysml:PartUsage" {
                    "sysml:FeatureMembership"
                } else {
                    "sysml:OwningMembership"
                };
                if relation["eClass"] != expected {
                    return Err(Error::Unsupported("membership kind"));
                }
                let child = collect(child, Some(index), depth + 1, rows)?;
                rows[index].children.push(child);
            }
            _ => return Err(Error::Unsupported("relationship classifier")),
        }
    }
    if class == "sysml:PartUsage" && rows[index].typing.is_none() {
        return Err(Error::Unsupported("untyped part outside F14 family"));
    }
    Ok(index)
}
fn resolve(rows: &[Declaration<'_>], mut scope: Option<usize>, parts: &[&str]) -> Option<usize> {
    let mut found = loop {
        if let Some(index) = rows
            .iter()
            .position(|r| r.parent == scope && r.name == parts[0])
        {
            break index;
        }
        scope = rows.get(scope?)?.parent;
    };
    for part in &parts[1..] {
        found = rows[found]
            .children
            .iter()
            .copied()
            .find(|i| rows[*i].name == *part)?;
    }
    Some(found)
}
fn type_name(rows: &[Declaration<'_>], usage: usize, target: &str) -> Result<String, Error> {
    // The selected-model API uses semantic elementIds, without node-ID fallback.
    let target = rows
        .iter()
        .position(|r| r.value["data"]["elementId"] == target)
        .ok_or(Error::InvalidTyping)?;
    if rows[target].value["eClass"] != "sysml:PartDefinition" {
        return Err(Error::InvalidTyping);
    }
    let mut path = vec![];
    let mut next = Some(target);
    while let Some(i) = next {
        path.push(rows[i].name);
        next = rows[i].parent;
    }
    path.reverse();
    let mut candidates = vec![];
    for start in 0..path.len() {
        let suffix = &path[start..];
        if resolve(rows, rows[usage].parent, suffix) == Some(target) {
            candidates.push(
                suffix
                    .iter()
                    .map(|s| printable_name(Some(s), true))
                    .collect::<Vec<_>>()
                    .join("::"),
            );
        }
    }
    // Native resolver sorts by Java String.length; preserve first candidate on ties.
    candidates
        .into_iter()
        .min_by_key(|s| s.encode_utf16().count())
        .ok_or(Error::UnresolvableType)
}
fn render(
    rows: &[Declaration<'_>],
    index: usize,
    depth: usize,
    options: Options,
    out: &mut String,
) -> Result<(), Error> {
    let nl = match options.newline {
        Newline::Lf => "\n",
        Newline::CrLf => "\r\n",
    };
    let row = &rows[index];
    out.push_str(&options.indentation.repeat(depth));
    out.push_str(match row.value["eClass"].as_str() {
        Some("sysml:Package") => "package ",
        Some("sysml:PartDefinition") => "part def ",
        _ => "part ",
    });
    out.push_str(&printable_name(Some(row.name), true));
    if let Some(target) = row.typing {
        out.push_str(" : ");
        out.push_str(&type_name(rows, index, target)?);
    }
    if row.children.is_empty() {
        out.push(';');
    } else {
        out.push_str(" {");
        for child in &row.children {
            out.push_str(nl);
            render(rows, *child, depth + 1, options, out)?;
        }
        out.push_str(nl);
        out.push_str(&options.indentation.repeat(depth));
        out.push('}');
    }
    if out.len() > 65_536 {
        return Err(Error::Limit);
    }
    Ok(())
}
/// Returns a complete owned UTF-8 string; on failure no partial output escapes.
/// Uses only the existing F14 selected semantic-ID projection, never native wires.
/// Resource/node/semantic identities are intentionally absent from text.
pub fn export(document: &Document, options: Options) -> Result<String, Error> {
    if options.indentation.len() > 8
        || !options.indentation.bytes().all(|b| b == b' ' || b == b'\t')
    {
        return Err(Error::Unsupported("indentation"));
    }
    fields(document.value(), &["json", "ns", "content"])?;
    fields(&document.value()["json"], &["version", "encoding"])?;
    fields(&document.value()["ns"], &["sysml"])?;
    if document.value()["json"]["encoding"] != "utf-8" {
        return Err(Error::Unsupported("encoding"));
    }
    let mut rows = vec![];
    let mut roots = vec![];
    for root in array(document.value(), "content")? {
        roots.push(collect(root, None, 0, &mut rows)?);
    }
    let mut out = String::new();
    let nl = match options.newline {
        Newline::Lf => "\n",
        Newline::CrLf => "\r\n",
    };
    for root in roots {
        if !out.is_empty() {
            out.push_str(nl);
        }
        render(&rows, root, 0, options, &mut out)?;
    }
    let importer = Importer::pinned().map_err(Error::NotImportable)?;
    let editor = Editor::new(Document::empty());
    let ids = (1..=1024).map(|n| {
        NodeIds::new(
            &format!("10000000-0000-4000-8000-{n:012x}"),
            &format!("20000000-0000-4000-8000-{n:012x}"),
        )
        .unwrap()
    });
    let staged = importer
        .stage(
            editor.begin(),
            &out,
            None,
            ids,
            Arc::new(AtomicBool::new(false)),
        )
        .map_err(Error::NotImportable)?;
    let mut reloaded = vec![];
    for root in array(staged.document().value(), "content")? {
        collect(root, None, 0, &mut reloaded)?;
    }
    if rows.len() != reloaded.len() {
        return Err(Error::RoundTripMismatch);
    }
    let target = |family: &[Declaration<'_>], row: &Declaration<'_>| {
        row.typing.and_then(|id| {
            family
                .iter()
                .position(|r| r.value["data"]["elementId"] == id)
        })
    };
    for (left, right) in rows.iter().zip(&reloaded) {
        if left.name != right.name
            || left.parent != right.parent
            || left.value["eClass"] != right.value["eClass"]
            || target(&rows, left) != target(&reloaded, right)
        {
            return Err(Error::RoundTripMismatch);
        }
    }
    staged.cancel();
    Ok(out)
}

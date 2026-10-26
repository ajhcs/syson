// SPDX-License-Identifier: EPL-2.0
#![forbid(unsafe_code)]
//! Ordered JSON observations only: no normalization, attestation or parity claim.
use serde::Deserializer;
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde_json::{Value, json, value::RawValue};
use std::{collections::BTreeSet, fmt};

pub const MAX_INPUT: usize = 8 * 1024 * 1024;
pub const MAX_VALUES: usize = 100_000;
pub const MAX_DEPTH: usize = 64;
pub const MAX_CHILDREN: usize = 20_000;
pub const MAX_POINTER: usize = 4096;
pub const MAX_DIFFS: usize = 64;
pub const MAX_PREVIEW: usize = 160;
type Result<T> = std::result::Result<T, String>;
#[derive(Debug)]
enum Node<'a> {
    Null,
    Boolean(bool),
    Number(&'a str),
    Text(String),
    Array(Vec<Node<'a>>),
    Object(Vec<(String, Node<'a>)>),
}
enum Parts<'a> {
    Array(Vec<&'a RawValue>),
    Object(Vec<(String, &'a RawValue)>),
}
struct Children<'a>(&'a mut usize);
impl Children<'_> {
    fn reserve<E: serde::de::Error>(&mut self, length: usize) -> std::result::Result<(), E> {
        if length == MAX_CHILDREN || *self.0 == MAX_VALUES {
            return Err(E::custom("container/value limit"));
        }
        *self.0 += 1;
        Ok(())
    }
}
impl<'de> Visitor<'de> for Children<'_> {
    type Value = Parts<'de>;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("JSON container")
    }
    fn visit_seq<A: SeqAccess<'de>>(
        mut self,
        mut access: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let mut out = Vec::new();
        while let Some(value) = access.next_element::<&'de RawValue>()? {
            self.reserve(out.len())?;
            out.push(value);
        }
        Ok(Parts::Array(out))
    }
    fn visit_map<A: MapAccess<'de>>(
        mut self,
        mut access: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let (mut out, mut keys) = (Vec::new(), BTreeSet::new());
        while let Some((key, value)) = access.next_entry::<String, &'de RawValue>()? {
            self.reserve(out.len())?;
            if key.len() > MAX_POINTER || !keys.insert(key.clone()) {
                return Err(serde::de::Error::custom("oversized/duplicate object key"));
            }
            out.push((key, value));
        }
        Ok(Parts::Object(out))
    }
}
fn pointer(parent: &str, key: &str) -> String {
    format!("{parent}/{}", key.replace('~', "~0").replace('/', "~1"))
}
fn node<'a>(raw: &'a RawValue, depth: usize, count: &mut usize, path: &str) -> Result<Node<'a>> {
    if depth > MAX_DEPTH || path.len() > MAX_POINTER {
        return Err(format!("depth/pointer limit at {path}"));
    }
    let input = raw.get();
    let first = input.as_bytes()[0];
    Ok(match first {
        b'{' | b'[' => {
            let mut d = serde_json::Deserializer::from_str(input);
            let visitor = Children(count);
            let parts = if first == b'{' {
                d.deserialize_map(visitor)
            } else {
                d.deserialize_seq(visitor)
            }
            .map_err(|e| format!("{path}: {e}"))?;
            match parts {
                Parts::Array(rows) => Node::Array(
                    rows.into_iter()
                        .enumerate()
                        .map(|(i, v)| node(v, depth + 1, count, &pointer(path, &i.to_string())))
                        .collect::<Result<_>>()?,
                ),
                Parts::Object(rows) => Node::Object(
                    rows.into_iter()
                        .map(|(key, v)| {
                            let child = node(v, depth + 1, count, &pointer(path, &key))?;
                            Ok((key, child))
                        })
                        .collect::<Result<_>>()?,
                ),
            }
        }
        b'"' => Node::Text(serde_json::from_str(input).map_err(|e| format!("{path}: {e}"))?),
        b't' => Node::Boolean(true),
        b'f' => Node::Boolean(false),
        b'n' => Node::Null,
        _ => Node::Number(input),
    })
}
fn parse(input: &[u8]) -> Result<Node<'_>> {
    if input.len() > MAX_INPUT {
        return Err("input byte limit".into());
    }
    let raw: &RawValue = serde_json::from_slice(input).map_err(|e| e.to_string())?;
    node(raw, 1, &mut 1, "")
}
fn clip(text: &str) -> String {
    let mut chars = text.chars();
    let mut out: String = chars.by_ref().take(MAX_PREVIEW).collect();
    if chars.next().is_some() {
        out.push('…');
    }
    out
}
fn preview(node: Option<&Node<'_>>) -> String {
    match node {
        None => "<absent>".into(),
        Some(Node::Null) => "null".into(),
        Some(Node::Boolean(v)) => v.to_string(),
        Some(Node::Number(v)) => clip(v),
        Some(Node::Text(v)) => format!("{:?}", clip(v)),
        Some(Node::Array(v)) => format!("[{} items]", v.len()),
        Some(Node::Object(v)) => format!("{{{} ordered members}}", v.len()),
    }
}
#[derive(Debug)]
pub struct Difference {
    pub left_pointer: String,
    pub right_pointer: String,
    pub kind: &'static str,
    pub left_preview: String,
    pub right_preview: String,
}
#[derive(Debug)]
pub struct Report {
    differences: Vec<Difference>,
    complete: bool,
    left_bytes: usize,
    right_bytes: usize,
}
impl Report {
    pub fn differences(&self) -> &[Difference] {
        &self.differences
    }
    pub fn is_complete(&self) -> bool {
        self.complete
    }
    pub fn is_match(&self) -> bool {
        self.complete && self.differences.is_empty()
    }
    pub fn into_json(self) -> Value {
        let status = if self.is_match() { "MATCH" } else { "MISMATCH" };
        let diffs: Vec<_> = self
            .differences
            .into_iter()
            .map(|d| {
                json!({"kind":d.kind,
            "left_pointer":d.left_pointer,"right_pointer":d.right_pointer,
            "left_preview":d.left_preview,"right_preview":d.right_preview})
            })
            .collect();
        json!({"schema":"C02a-ordered-json-diff/1","status":status,"complete":self.complete,
            "left_bytes":self.left_bytes,"right_bytes":self.right_bytes,"differences":diffs,
            "scope":"Supplied observations only; no identity attestation or feature acceptance.",
            "comparison":"Full ordered tree; exact number lexemes. JSON whitespace/string escape spelling excluded."})
    }
    fn add(
        &mut self,
        kind: &'static str,
        lp: &str,
        rp: &str,
        left: Option<&Node<'_>>,
        right: Option<&Node<'_>>,
    ) {
        if self.differences.len() == MAX_DIFFS {
            self.complete = false;
            return;
        }
        self.differences.push(Difference {
            kind,
            left_pointer: lp.into(),
            right_pointer: rp.into(),
            left_preview: preview(left),
            right_preview: preview(right),
        });
    }
}
fn walk(
    left: Option<&Node<'_>>,
    right: Option<&Node<'_>>,
    lp: &str,
    rp: &str,
    report: &mut Report,
) -> Result<()> {
    check_pointers(lp, rp)?;
    if !report.complete {
        return Ok(());
    }
    match (left, right) {
        (None, _) => report.add("MISSING_LEFT", lp, rp, left, right),
        (_, None) => report.add("MISSING_RIGHT", lp, rp, left, right),
        (Some(Node::Array(a)), Some(Node::Array(b))) => {
            for i in 0..a.len().max(b.len()) {
                walk(
                    a.get(i),
                    b.get(i),
                    &pointer(lp, &i.to_string()),
                    &pointer(rp, &i.to_string()),
                    report,
                )?;
                if !report.complete {
                    break;
                }
            }
        }
        (Some(Node::Object(a)), Some(Node::Object(b))) => {
            for i in 0..a.len().max(b.len()) {
                let l = a.get(i);
                let r = b.get(i);
                let key = l.or(r).unwrap().0.as_str();
                let lpath = pointer(lp, l.map_or(key, |v| v.0.as_str()));
                let rpath = pointer(rp, r.map_or(key, |v| v.0.as_str()));
                check_pointers(&lpath, &rpath)?;
                if let (Some(l), Some(r)) = (l, r)
                    && l.0 != r.0
                {
                    report.add(
                        "OBJECT_MEMBER_ORDER_OR_KEY",
                        &lpath,
                        &rpath,
                        Some(&Node::Text(l.0.clone())),
                        Some(&Node::Text(r.0.clone())),
                    );
                }
                walk(l.map(|v| &v.1), r.map(|v| &v.1), &lpath, &rpath, report)?;
                if !report.complete {
                    break;
                }
            }
        }
        (Some(Node::Null), Some(Node::Null)) => {}
        (Some(Node::Boolean(a)), Some(Node::Boolean(b))) => {
            if a != b {
                report.add("BOOLEAN", lp, rp, left, right);
            }
        }
        (Some(Node::Number(a)), Some(Node::Number(b))) => {
            if a != b {
                report.add("NUMBER_LEXEME", lp, rp, left, right);
            }
        }
        (Some(Node::Text(a)), Some(Node::Text(b))) => {
            if a != b {
                report.add("STRING", lp, rp, left, right);
            }
        }
        _ => report.add("TYPE", lp, rp, left, right),
    }
    Ok(())
}
fn check_pointers(left: &str, right: &str) -> Result<()> {
    if left.len() > MAX_POINTER || right.len() > MAX_POINTER {
        return Err("comparison pointer limit; no report emitted".into());
    }
    Ok(())
}
/// Refusal is separate from comparison: both entire inputs must admit first.
/// Duplicate object keys refuse, so each emitted JSONPointer is unambiguous.
/// Capped reporting always produces incomplete MISMATCH, never MATCH.
pub fn compare(left: &[u8], right: &[u8]) -> Result<Report> {
    let a = parse(left).map_err(|e| format!("left input refused: {e}"))?;
    let b = parse(right).map_err(|e| format!("right input refused: {e}"))?;
    let mut report = Report {
        differences: Vec::new(),
        complete: true,
        left_bytes: left.len(),
        right_bytes: right.len(),
    };
    walk(Some(&a), Some(&b), "", "", &mut report)?;
    Ok(report)
}

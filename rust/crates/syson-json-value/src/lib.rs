// SPDX-License-Identifier: EPL-2.0
//! Bounded literal JSON, without generic Value deserialization or marker filters.
#![forbid(unsafe_code)]
use serde::Deserialize;
use serde_json::{Map, Number, Value, value::RawValue};
use std::collections::{BTreeMap, btree_map::Entry};

pub const HARD_BYTES: usize = 32 * 1024 * 1024;
pub const HARD_DEPTH: usize = 256;
pub const HARD_ITEMS: usize = 2_000_000;
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub bytes: usize,
    /// Root value has depth one; object keys do not add nesting depth.
    pub depth: usize,
    pub values: usize,
    pub keys: usize,
    /// Conservative UTF8 copy charge: twice raw key-body bytes, plus raw string
    /// body and numeric-token bytes. Quotes are excluded, escapes are not decoded.
    pub scalar_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            bytes: 4 * 1024 * 1024,
            depth: 256,
            values: 2_000_000,
            keys: 2_000_000,
            scalar_bytes: 8 * 1024 * 1024,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    Bytes,
    Depth,
    Values,
    Keys,
    ScalarBytes,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidLimits,
    Limit(Limit),
    Cancelled,
    DuplicateKey,
    StringEncoding,
    Syntax { line: usize, column: usize },
    Allocation,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
/// Duplicate policy is explicit at the caller boundary. Existing `parse` stays strict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DuplicateKeys {
    Reject,
    /// Keep the final value at the first decoded-key insertion position. All
    /// physical occurrences still consume admission budgets; raw bytes survive.
    LastWins,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub values: usize,
    pub keys: usize,
    pub scalar_bytes: usize,
}
/// Objects retain source member order; keys/strings are decoded JSON strings.
/// Number spelling is a borrowed, syntactically validated source token.
#[derive(Debug, PartialEq, Eq)]
pub enum Node<'a> {
    Null,
    Boolean(bool),
    Number(&'a str),
    String(String),
    Array(Vec<Node<'a>>),
    Object(Vec<(String, Node<'a>)>),
}
pub struct Literal<'a> {
    raw: &'a str,
    root: Node<'a>,
    statistics: Statistics,
}
impl<'a> Literal<'a> {
    pub fn raw(&self) -> &'a str {
        self.raw
    }
    pub fn root(&self) -> &Node<'a> {
        &self.root
    }
    pub fn statistics(&self) -> Statistics {
        self.statistics
    }
    /// Consumes only an admitted tree. Object construction is literal; callers
    /// retain raw bytes separately because Value member order is nonauthoritative.
    pub fn into_value(
        self,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<(Value, &'a str), Error> {
        let value = materialize(self.root, &mut cancelled)?;
        check(&mut cancelled)?;
        Ok((value, self.raw))
    }
}
fn check(cancelled: &mut impl FnMut() -> bool) -> Result<(), Error> {
    if cancelled() {
        Err(Error::Cancelled)
    } else {
        Ok(())
    }
}
fn poll(cancelled: &mut impl FnMut() -> bool, offset: usize) -> Result<(), Error> {
    if offset.is_multiple_of(1024) {
        check(cancelled)?;
    }
    Ok(())
}
fn syntax(e: serde_json::Error) -> Error {
    Error::Syntax {
        line: e.line(),
        column: e.column(),
    }
}
fn space(b: u8) -> bool {
    matches!(b, b' ' | b'\n' | b'\r' | b'\t')
}
fn boundary(b: u8) -> bool {
    space(b) || matches!(b, b'[' | b']' | b'{' | b'}' | b',' | b':' | b'"')
}
fn token<'a>(
    text: &'a str,
    cursor: &mut usize,
    cancel: &mut impl FnMut() -> bool,
) -> Result<&'a str, Error> {
    let bytes = text.as_bytes();
    let start = *cursor;
    if bytes[start] == b'"' {
        *cursor += 1;
        let mut escaped = false;
        while *cursor < bytes.len() {
            poll(cancel, *cursor)?;
            let b = bytes[*cursor];
            *cursor += 1;
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                break;
            }
        }
    } else {
        while *cursor < bytes.len() && !boundary(bytes[*cursor]) {
            poll(cancel, *cursor)?;
            *cursor += 1;
        }
    }
    Ok(&text[start..*cursor])
}
fn charge(used: &mut usize, amount: usize, maximum: usize, kind: Limit) -> Result<(), Error> {
    *used = used
        .checked_add(amount)
        .filter(|n| *n <= maximum)
        .ok_or(Error::Limit(kind))?;
    Ok(())
}
// No owned strings/tree are built before this entire lexical resource preflight.
fn preflight(
    text: &str,
    limits: Limits,
    cancel: &mut impl FnMut() -> bool,
) -> Result<Statistics, Error> {
    let (mut i, mut depth) = (0usize, 0usize);
    let mut stats = Statistics::default();
    while i < text.len() {
        poll(cancel, i)?;
        let b = text.as_bytes()[i];
        match b {
            b'{' | b'[' => {
                depth += 1;
                if depth > limits.depth {
                    return Err(Error::Limit(Limit::Depth));
                }
                charge(&mut stats.values, 1, limits.values, Limit::Values)?;
                i += 1;
            }
            b'}' | b']' => {
                depth = depth.saturating_sub(1);
                i += 1;
            }
            b':' | b',' => i += 1,
            b if space(b) => i += 1,
            _ => {
                let raw = token(text, &mut i, cancel)?;
                let mut next = i;
                while next < text.len() && space(text.as_bytes()[next]) {
                    poll(cancel, next)?;
                    next += 1;
                }
                let key = b == b'"' && text.as_bytes().get(next) == Some(&b':');
                if key {
                    charge(&mut stats.keys, 1, limits.keys, Limit::Keys)?;
                } else {
                    if depth + 1 > limits.depth {
                        return Err(Error::Limit(Limit::Depth));
                    }
                    charge(&mut stats.values, 1, limits.values, Limit::Values)?;
                }
                let bytes = if b == b'"' {
                    raw.len().saturating_sub(2) * if key { 2 } else { 1 }
                } else if b == b'-' || b.is_ascii_digit() {
                    raw.len()
                } else {
                    0
                };
                charge(
                    &mut stats.scalar_bytes,
                    bytes,
                    limits.scalar_bytes,
                    Limit::ScalarBytes,
                )?;
            }
        }
    }
    check(cancel)?;
    Ok(stats)
}
fn push<T>(values: &mut Vec<T>, value: T) -> Result<(), Error> {
    values.try_reserve(1).map_err(|_| Error::Allocation)?;
    values.push(value);
    Ok(())
}
struct Builder<'a, 'c, F> {
    text: &'a str,
    cursor: usize,
    cancel: &'c mut F,
    duplicates: DuplicateKeys,
}
impl<'a, F: FnMut() -> bool> Builder<'a, '_, F> {
    fn whitespace(&mut self) -> Result<(), Error> {
        while self.cursor < self.text.len() && space(self.text.as_bytes()[self.cursor]) {
            poll(self.cancel, self.cursor)?;
            self.cursor += 1;
        }
        Ok(())
    }
    fn string(&mut self) -> Result<String, Error> {
        let raw = token(self.text, &mut self.cursor, self.cancel)?;
        check(self.cancel)?;
        let decoded = serde_json::from_str::<String>(raw).map_err(|_| Error::StringEncoding)?;
        check(self.cancel)?;
        Ok(decoded)
    }
    fn node(&mut self) -> Result<Node<'a>, Error> {
        check(self.cancel)?;
        self.whitespace()?;
        // Full immutable input syntax was validated before this builder exists.
        let b = self.text.as_bytes()[self.cursor];
        match b {
            b'{' => {
                self.cursor += 1;
                self.whitespace()?;
                let mut result: Vec<(String, Node<'a>)> = Vec::new();
                let mut keys: BTreeMap<String, usize> = BTreeMap::new();
                if self.text.as_bytes()[self.cursor] == b'}' {
                    self.cursor += 1;
                    return Ok(Node::Object(result));
                }
                loop {
                    self.whitespace()?;
                    let key = self.string()?;
                    // Search once; moving the decoded key avoids duplicate-key clones.
                    let entry = keys.entry(key);
                    if self.duplicates == DuplicateKeys::Reject
                        && matches!(&entry, Entry::Occupied(_))
                    {
                        return Err(Error::DuplicateKey);
                    }
                    self.whitespace()?;
                    self.cursor += 1;
                    let child = self.node()?;
                    match entry {
                        Entry::Occupied(entry) => result[*entry.get()].1 = child,
                        Entry::Vacant(entry) => {
                            let key = entry.key().clone();
                            entry.insert(result.len());
                            push(&mut result, (key, child))?;
                        }
                    }
                    self.whitespace()?;
                    let last = self.text.as_bytes()[self.cursor] == b'}';
                    self.cursor += 1;
                    if last {
                        break;
                    }
                }
                Ok(Node::Object(result))
            }
            b'[' => {
                self.cursor += 1;
                self.whitespace()?;
                let mut result = Vec::new();
                if self.text.as_bytes()[self.cursor] == b']' {
                    self.cursor += 1;
                    return Ok(Node::Array(result));
                }
                loop {
                    let child = self.node()?;
                    push(&mut result, child)?;
                    self.whitespace()?;
                    let last = self.text.as_bytes()[self.cursor] == b']';
                    self.cursor += 1;
                    if last {
                        break;
                    }
                }
                Ok(Node::Array(result))
            }
            b'"' => self.string().map(Node::String),
            _ => {
                let raw = token(self.text, &mut self.cursor, self.cancel)?;
                Ok(match raw {
                    "null" => Node::Null,
                    "true" => Node::Boolean(true),
                    "false" => Node::Boolean(false),
                    _ => Node::Number(raw),
                })
            }
        }
    }
}
/// Fixed policies: decoded duplicate keys refuse; ordered tree/raw bytes retain
/// source order; materialized Value keys are sorted lexically, not wire authority.
/// All resource limits run before owned tree construction; duplicate detection
/// decodes/copies bounded keys privately. No partial tree/value escapes refusal.
pub fn parse<'a>(
    text: &'a str,
    limits: Limits,
    cancelled: impl FnMut() -> bool,
) -> Result<Literal<'a>, Error> {
    parse_with_duplicate_keys(text, limits, DuplicateKeys::Reject, cancelled)
}
/// Opt-in compatibility for native last-value callers. The shared preflight
/// charges every physical key/value/scalar before any duplicate is collapsed.
/// The ordered tree retains first key positions and final values; `raw()` still
/// retains all original bytes. No generic Value deserialization is introduced.
pub fn parse_with_duplicate_keys<'a>(
    text: &'a str,
    limits: Limits,
    duplicates: DuplicateKeys,
    mut cancelled: impl FnMut() -> bool,
) -> Result<Literal<'a>, Error> {
    if limits.bytes > HARD_BYTES
        || limits.depth == 0
        || limits.depth > HARD_DEPTH
        || limits.values > HARD_ITEMS
        || limits.keys > HARD_ITEMS
        || limits.scalar_bytes > HARD_BYTES * 2
    {
        return Err(Error::InvalidLimits);
    }
    check(&mut cancelled)?;
    if text.len() > limits.bytes {
        return Err(Error::Limit(Limit::Bytes));
    }
    let statistics = preflight(text, limits, &mut cancelled)?;
    let mut d = serde_json::Deserializer::from_str(text);
    // Borrowed RawValue validates syntax without generic Value/number decoding.
    let _: &RawValue = Deserialize::deserialize(&mut d).map_err(syntax)?;
    d.end().map_err(syntax)?;
    check(&mut cancelled)?;
    let root = Builder {
        text,
        cursor: 0,
        cancel: &mut cancelled,
        duplicates,
    }
    .node()?;
    check(&mut cancelled)?;
    Ok(Literal {
        raw: text,
        root,
        statistics,
    })
}
fn materialize(node: Node<'_>, cancel: &mut impl FnMut() -> bool) -> Result<Value, Error> {
    check(cancel)?;
    let value = match node {
        Node::Null => Value::Null,
        Node::Boolean(v) => Value::Bool(v),
        Node::String(v) => Value::String(v),
        // Pinned serde_json1.0.151 constructor: tokens came from complete strict
        // syntax validation. Its generic Number parser would normalize -0.
        Node::Number(raw) => Value::Number(Number::from_string_unchecked(raw.to_owned())),
        Node::Array(nodes) => {
            let mut result = Vec::new();
            result
                .try_reserve_exact(nodes.len())
                .map_err(|_| Error::Allocation)?;
            for node in nodes {
                result.push(materialize(node, cancel)?);
            }
            Value::Array(result)
        }
        Node::Object(nodes) => {
            let mut result = Map::new();
            for (key, node) in nodes {
                result.insert(key, materialize(node, cancel)?);
            }
            // Explicit even if downstream feature unification enables preserve_order.
            check(cancel)?;
            result.sort_keys();
            check(cancel)?;
            Value::Object(result)
        }
    };
    check(cancel)?;
    Ok(value)
}

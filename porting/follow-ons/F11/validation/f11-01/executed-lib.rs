// Copyright (c) 2022-2025 Sensmetry UAB and others.
// SPDX-License-Identifier: EPL-2.0 OR GPL-2.0 WITH Classpath-exception-2.0
//! Finite source-derived text construction, not a complete SysML parser or importer.
#![forbid(unsafe_code)]
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use syson_ast::{
    Arena, Assignment, Builder, Datum, DefaultValue, Lexeme, NodeId, Reflection, Value,
};
use syson_ast_values::{Control, Dispatch, Presence, Primitive, Rule};
use syson_reflection::{MandatoryKind, OrderingQualification, ReflectionDb};
include!("reserved.rs");

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub source_bytes: usize,
    pub tokens: usize,
    pub declarations: usize,
    /// At most 64, a proposed Rust admission policy, not a source grammar limit.
    pub depth: usize,
    pub kernel: syson_ast::Limits,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            source_bytes: 65_536,
            tokens: 8192,
            declarations: 256,
            depth: 32,
            kernel: syson_ast::Limits::default(),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Malformed {
        offset: usize,
        expected: &'static str,
    },
    /// Outside the finite family; does not assert that full SysML is invalid.
    Unsupported {
        offset: usize,
    },
    Limit,
    InvalidLimits,
    Cancelled,
    Kernel(syson_ast::Error),
    Metadata,
    Conversion,
}
fn malformed(offset: usize, expected: &'static str) -> Error {
    Error::Malformed { offset, expected }
}
impl From<syson_ast::Error> for Error {
    fn from(error: syson_ast::Error) -> Self {
        Self::Kernel(error)
    }
}
#[derive(Clone, Copy)]
struct Token<'a> {
    text: &'a str,
    start: usize,
    end: usize,
    name: bool,
}
struct Declaration<'a> {
    kind: &'static str,
    start: usize,
    end: usize,
    name: Option<Token<'a>>,
    children: Vec<Declaration<'a>>,
    typing: Vec<Token<'a>>,
}
fn check(cancel: &AtomicBool) -> Result<(), Error> {
    if cancel.load(Ordering::Relaxed) {
        Err(Error::Cancelled)
    } else {
        Ok(())
    }
}
fn lex<'a>(source: &'a str, maximum: usize, cancel: &AtomicBool) -> Result<Vec<Token<'a>>, Error> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < source.len() {
        check(cancel)?;
        let byte = source.as_bytes()[i];
        if byte.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if tokens.len() == maximum {
            return Err(Error::Limit);
        }
        let start = i;
        let name;
        if byte.is_ascii_alphabetic() || byte == b'_' {
            i += 1;
            while i < source.len()
                && (source.as_bytes()[i].is_ascii_alphanumeric() || source.as_bytes()[i] == b'_')
            {
                i += 1;
            }
            name = RESERVED.binary_search(&&source[start..i]).is_err();
        } else if byte == b'\'' {
            i += 1;
            loop {
                check(cancel)?;
                let Some(c) = source[i..].chars().next() else {
                    return Err(malformed(i, "closing name quote"));
                };
                i += c.len_utf8();
                if c == '\'' {
                    break;
                }
                if c == '\\' {
                    let Some(escape) = source[i..].chars().next() else {
                        return Err(malformed(i, "name escape"));
                    };
                    if !"'\"bftnr\\".contains(escape) {
                        return Err(malformed(i, "admitted name escape"));
                    }
                    i += escape.len_utf8();
                }
            }
            name = true;
        } else if b"{}:;".contains(&byte) {
            i += 1;
            if byte == b':' && source.as_bytes().get(i) == Some(&b':') {
                i += 1;
            }
            name = false;
        } else {
            return Err(Error::Unsupported { offset: i });
        }
        tokens.push(Token {
            text: &source[start..i],
            start,
            end: i,
            name,
        });
    }
    Ok(tokens)
}
struct Parser<'a, 'b> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
    at: usize,
    count: usize,
    limits: Limits,
    cancel: &'b AtomicBool,
}
impl<'a> Parser<'a, '_> {
    fn offset(&self) -> usize {
        self.tokens
            .get(self.at)
            .map_or(self.source.len(), |t| t.start)
    }
    fn take(&mut self, text: &str) -> bool {
        if self.tokens.get(self.at).is_some_and(|t| t.text == text) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, text: &'static str) -> Result<(), Error> {
        if self.take(text) {
            return Ok(());
        }
        if self
            .tokens
            .get(self.at)
            .is_some_and(|t| !t.name && t.text.as_bytes()[0].is_ascii_alphabetic())
        {
            return Err(Error::Unsupported {
                offset: self.offset(),
            });
        }
        Err(malformed(self.offset(), text))
    }
    fn name(&mut self) -> Result<Token<'a>, Error> {
        let Some(token) = self.tokens.get(self.at).copied() else {
            return Err(malformed(self.offset(), "name"));
        };
        if !token.name {
            return Err(if token.text.as_bytes()[0].is_ascii_alphabetic() {
                Error::Unsupported {
                    offset: token.start,
                }
            } else {
                malformed(token.start, "name")
            });
        }
        self.at += 1;
        Ok(token)
    }
    fn list(&mut self, nested: bool, depth: usize) -> Result<Vec<Declaration<'a>>, Error> {
        let mut children = Vec::new();
        while self.at < self.tokens.len() && !self.tokens[self.at].text.eq("}") {
            children.push(self.declaration(depth)?);
        }
        if nested {
            self.expect("}")?;
        } else if self.at != self.tokens.len() {
            return Err(malformed(self.offset(), "declaration"));
        }
        Ok(children)
    }
    fn declaration(&mut self, depth: usize) -> Result<Declaration<'a>, Error> {
        check(self.cancel)?;
        if depth >= self.limits.depth || self.count == self.limits.declarations {
            return Err(Error::Limit);
        }
        self.count += 1;
        let start = self.offset();
        let kind = if self.take("package") {
            "Package"
        } else if self.take("part") {
            if self.take("def") {
                "PartDefinition"
            } else {
                "PartUsage"
            }
        } else {
            return Err(Error::Unsupported { offset: start });
        };
        let name = if kind == "PartUsage" || self.tokens.get(self.at).is_some_and(|t| t.name) {
            Some(self.name()?)
        } else {
            None
        };
        let mut typing = Vec::new();
        let children;
        if kind == "PartUsage" {
            if self
                .tokens
                .get(self.at)
                .is_some_and(|t| matches!(t.text, ";" | "{"))
            {
                return Err(Error::Unsupported {
                    offset: self.offset(),
                });
            }
            self.expect(":")?;
            typing.push(self.name()?);
            while self.take("::") {
                typing.push(self.name()?);
            }
            if self.tokens.get(self.at).is_some_and(|t| t.text == "{") {
                return Err(Error::Unsupported {
                    offset: self.offset(),
                });
            }
            self.expect(";")?;
            children = Vec::new();
        } else if self.take(";") {
            children = Vec::new();
        } else {
            self.expect("{")?;
            children = self.list(true, depth + 1)?;
        }
        Ok(Declaration {
            kind,
            start,
            end: self.tokens[self.at - 1].end,
            name,
            children,
            typing,
        })
    }
}

/// Reuses F08 mandatory metadata and F10 primitive conversion, with F08's
/// source-derived candidate ordering qualification unchanged.
pub struct Adapter {
    reflection: Reflection,
    db: ReflectionDb,
}
impl Adapter {
    pub fn pinned() -> Result<Self, Error> {
        let db = ReflectionDb::pinned();
        let mut reflection = Reflection::default();
        for kind in [
            "Namespace",
            "Package",
            "PartDefinition",
            "PartUsage",
            "OwningMembership",
            "FeatureMembership",
            "FeatureTyping",
            "TypeReference",
        ] {
            let ty = db.resolve_type(kind).map_err(|_| Error::Metadata)?;
            let props = db
                .effective_mandatory(&ty)
                .map_err(|_| Error::Metadata)?
                .iter()
                .map(|p| match p.kind {
                    MandatoryKind::Array => DefaultValue::Array(p.name.into()),
                    MandatoryKind::Boolean => DefaultValue::Boolean(p.name.into()),
                })
                .collect();
            reflection.declare(kind.into(), props)?;
        }
        Ok(Self { reflection, db })
    }
    pub fn ordering_qualification(&self) -> OrderingQualification {
        self.db.ordering_qualification()
    }
    /// Constructs the finite text whitelist into an immutable kernel arena.
    /// This does not link references, run postprocessors, validate semantics,
    /// update native resources, recover malformed input or implement all grammar.
    pub fn construct(
        &self,
        source: &str,
        limits: Limits,
        cancel: Arc<AtomicBool>,
    ) -> Result<Arena, Error> {
        check(&cancel)?;
        if limits.depth == 0 || limits.depth > 64 {
            return Err(Error::InvalidLimits);
        }
        if source.len() > limits.source_bytes {
            return Err(Error::Limit);
        }
        let tokens = lex(source, limits.tokens, &cancel)?;
        let mut parser = Parser {
            source,
            tokens,
            at: 0,
            count: 0,
            limits,
            cancel: &cancel,
        };
        let children = parser.list(false, 0)?;
        let root = Declaration {
            kind: "Namespace",
            start: 0,
            end: source.len(),
            name: None,
            children,
            typing: Vec::new(),
        };
        let mut builder = Builder::new(self.reflection.clone(), limits.kernel, cancel.clone())?;
        let root_id = emit(
            &root,
            source,
            &mut builder,
            &self.db,
            &cancel,
            limits.source_bytes,
        )?;
        Ok(builder.finish(root_id)?)
    }
}
fn emit(
    decl: &Declaration<'_>,
    source: &str,
    builder: &mut Builder,
    db: &ReflectionDb,
    cancel: &AtomicBool,
    maximum: usize,
) -> Result<NodeId, Error> {
    check(cancel)?;
    let id = builder.allocate(
        Some(decl.kind),
        Some(Lexeme::from_source(source, decl.start, decl.end)?),
    )?;
    if let Some(token) = decl.name {
        let control = Control {
            max_utf16_units: maximum,
            is_cancelled: &|| cancel.load(Ordering::Relaxed),
        };
        let input = syson_ast_values::Lexeme {
            text: token.text,
            span: syson_ast_values::Span {
                start: token.start,
                end: token.end,
            },
        };
        let result = syson_ast_values::convert(
            Dispatch::Rule(Rule {
                name: "Name",
                declared_type: Some("string"),
            }),
            Presence::Value(input),
            &control,
        )
        .map_err(|_| Error::Conversion)?;
        let Presence::Value(converted) = result else {
            return Err(Error::Conversion);
        };
        let Primitive::String(value) = converted.value else {
            return Err(Error::Conversion);
        };
        builder.assign(
            id,
            "declaredName",
            Assignment::Set,
            Datum {
                value: Value::String(value.to_utf8().map_err(|_| Error::Conversion)?),
                lexeme: Some(Lexeme::from_source(source, token.start, token.end)?),
            },
        )?;
    }
    for child in &decl.children {
        let wrapper_kind = if decl.kind == "PartDefinition" && child.kind == "PartUsage" {
            "FeatureMembership"
        } else {
            "OwningMembership"
        };
        let wrapper = builder.allocate(
            Some(wrapper_kind),
            Some(Lexeme::from_source(source, child.start, child.end)?),
        )?;
        let target = emit(child, source, builder, db, cancel, maximum)?;
        builder.assign(wrapper, "target", Assignment::Set, Datum::node(target))?;
        builder.assign(id, "children", Assignment::Append, Datum::node(wrapper))?;
    }
    if let (Some(first), Some(last)) = (decl.typing.first(), decl.typing.last()) {
        let origin = Lexeme::from_source(source, first.start, last.end)?;
        let typing = builder.allocate(Some("FeatureTyping"), Some(origin.clone()))?;
        let reference = builder.allocate(Some("TypeReference"), Some(origin))?;
        let reference_type = db
            .resolve_type("TypeReference")
            .map_err(|_| Error::Metadata)?;
        for (index, token) in decl.typing.iter().enumerate() {
            let position = db
                .reference_position(&reference_type, index, decl.typing.len())
                .map_err(|_| Error::Metadata)?;
            let target = db
                .reference_target_type(&position)
                .map_err(|_| Error::Metadata)?;
            let target_name = db.name(&target).map_err(|_| Error::Metadata)?;
            builder.reference(
                reference,
                "parts",
                Assignment::Append,
                Lexeme::from_source(source, token.start, token.end)?,
                target_name,
            )?;
        }
        builder.assign(typing, "targetRef", Assignment::Set, Datum::node(reference))?;
        builder.assign(id, "heritage", Assignment::Append, Datum::node(typing))?;
    }
    Ok(id)
}

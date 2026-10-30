// SPDX-License-Identifier: EPL-2.0
//! Finite, atomic selected-model import; no native Syside/EMF interchange claim.
#![forbid(unsafe_code)]
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use syson_ast::{Arena, NodeId, Slot, Value};
use syson_ast_adapter::{Adapter, Limits};
use syson_linker::{Binding, Snapshot};
use syson_model::{Document, Editor, ModelError, NodeIds, Transaction};
use syson_references::{FragmentIdentity, Registry, Resource};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Parse(syson_ast_adapter::Error),
    Link(syson_linker::Error),
    Model(ModelError),
    Unsupported(&'static str),
    IdentitiesExhausted,
    Cancelled,
}
impl From<ModelError> for Error {
    fn from(e: ModelError) -> Self {
        Self::Model(e)
    }
}
impl From<syson_linker::Error> for Error {
    fn from(e: syson_linker::Error) -> Self {
        Self::Link(e)
    }
}
fn check(cancel: &AtomicBool) -> Result<(), Error> {
    if cancel.load(Ordering::Relaxed) {
        Err(Error::Cancelled)
    } else {
        Ok(())
    }
}
fn slot<'a>(arena: &'a Arena, id: NodeId, key: &str) -> Result<&'a Value, Error> {
    match arena.node(id).map(|n| n.slot(key)) {
        Some(Slot::Present(d)) => Ok(&d.value),
        _ => Err(Error::Unsupported("missing finite AST slot")),
    }
}
fn children(arena: &Arena, id: NodeId) -> Result<Vec<NodeId>, Error> {
    let Value::Array(rows) = slot(arena, id, "children")? else {
        return Err(Error::Unsupported("children"));
    };
    rows.iter()
        .map(|row| {
            let Value::Node(wrapper) = row.value else {
                return Err(Error::Unsupported("membership"));
            };
            let Value::Node(child) = slot(arena, wrapper, "target")? else {
                return Err(Error::Unsupported("target"));
            };
            Ok(*child)
        })
        .collect()
}
/// Holds the same owned model transaction until one commit or cancellation.
#[derive(Debug)]
pub struct Staged {
    transaction: Transaction,
    roots: Vec<String>,
    allocations: usize,
    cancel: Arc<AtomicBool>,
}
impl Staged {
    pub fn document(&self) -> &Document {
        self.transaction.document()
    }
    pub fn roots(&self) -> &[String] {
        &self.roots
    }
    pub fn allocations(&self) -> usize {
        self.allocations
    }
    pub fn cancel(self) {
        self.transaction.cancel();
    }
    pub fn commit(self, editor: &mut Editor) -> Result<bool, Error> {
        check(&self.cancel)?;
        Ok(editor.commit(self.transaction)?)
    }
}
pub struct Importer {
    parser: Adapter,
}
impl Importer {
    pub fn pinned() -> Result<Self, Error> {
        Ok(Self {
            parser: Adapter::pinned().map_err(Error::Parse)?,
        })
    }
    /// New declarations are the only lexical lookup targets. A parent selects
    /// attachment context; it does not inject existing declarations into F13.
    pub fn stage(
        &self,
        transaction: Transaction,
        source: &str,
        parent: Option<&str>,
        identities: impl IntoIterator<Item = NodeIds>,
        cancel: Arc<AtomicBool>,
    ) -> Result<Staged, Error> {
        check(&cancel)?;
        let arena = self
            .parser
            .construct(source, Limits::default(), cancel.clone())
            .map_err(|e| {
                if e == syson_ast_adapter::Error::Cancelled {
                    Error::Cancelled
                } else {
                    Error::Parse(e)
                }
            })?;
        if let Some(parent) = parent {
            let node = transaction
                .document()
                .element(parent)
                .ok_or_else(|| ModelError::MissingElement(parent.into()))?;
            if !matches!(
                node["eClass"].as_str(),
                Some("sysml:Package" | "sysml:PartDefinition" | "sysml:PartUsage")
            ) {
                return Err(Error::Unsupported(
                    "attachment parent outside package/part family",
                ));
            }
        }
        let mut build = Build {
            transaction,
            identities: identities.into_iter(),
            cancel: cancel.clone(),
            bindings: vec![],
            typing: vec![],
            allocations: 0,
            roots: vec![],
        };
        for child in children(&arena, arena.root())? {
            let semantic = build.create(&arena, child, parent)?;
            build.roots.push(semantic);
        }
        check(&cancel)?;
        let registry = Registry::new(vec![Resource {
            key: "staged-import".into(),
            document: build.transaction.document().clone(),
            fragments: FragmentIdentity::JsonNodeId,
        }])
        .map_err(|e| Error::Link(syson_linker::Error::Reference(e)))?;
        let snapshot = Snapshot::new(
            arena,
            registry,
            "staged-import".into(),
            vec![],
            build.bindings,
        )?;
        // Creation finishes first, so forward references use the actual linker.
        for (usage, reference) in build.typing {
            check(&cancel)?;
            let linked = snapshot.link(reference)?;
            let target = snapshot.semantic(&linked)?;
            let ids = allocate(&mut build.identities, &cancel, &mut build.allocations)?;
            // Preserve the existing selected-model API's semantic-ID encoding.
            build
                .transaction
                .type_part_usage(&usage, target.element_id(), ids)?;
        }
        check(&cancel)?;
        Ok(Staged {
            transaction: build.transaction,
            roots: build.roots,
            allocations: build.allocations,
            cancel,
        })
    }
}
fn allocate(
    ids: &mut impl Iterator<Item = NodeIds>,
    cancel: &AtomicBool,
    count: &mut usize,
) -> Result<NodeIds, Error> {
    check(cancel)?;
    let id = ids.next().ok_or(Error::IdentitiesExhausted)?;
    *count += 1;
    check(cancel)?;
    Ok(id)
}
struct Build<I> {
    transaction: Transaction,
    identities: I,
    cancel: Arc<AtomicBool>,
    bindings: Vec<Binding>,
    typing: Vec<(String, NodeId)>,
    allocations: usize,
    roots: Vec<String>,
}
impl<I: Iterator<Item = NodeIds>> Build<I> {
    fn create(
        &mut self,
        arena: &Arena,
        ast: NodeId,
        parent: Option<&str>,
    ) -> Result<String, Error> {
        check(&self.cancel)?;
        let Value::String(raw) = slot(arena, ast, "declaredName")? else {
            return Err(Error::Unsupported("named declarations only"));
        };
        // Exact current EAttributeTranslator.asCleanedText policy; F13 keeps
        // independent raw AST names for lookup, including its own sanitization.
        let name = raw.replace('\'', "");
        let ids = allocate(&mut self.identities, &self.cancel, &mut self.allocations)?;
        let semantic = ids.element.clone();
        let wire = ids.resource.clone();
        let membership = if parent.is_some() {
            Some(allocate(
                &mut self.identities,
                &self.cancel,
                &mut self.allocations,
            )?)
        } else {
            None
        };
        match arena.node(ast).and_then(|n| n.kind()) {
            Some("Package") => self
                .transaction
                .create_package(parent, ids, membership, &name)?,
            Some("PartDefinition") => self.transaction.create_part_definition(
                parent.ok_or(Error::Unsupported("top-level part needs explicit parent"))?,
                ids,
                membership.ok_or(Error::Unsupported("membership"))?,
                &name,
            )?,
            Some("PartUsage") => self.transaction.create_part_usage(
                parent.ok_or(Error::Unsupported("top-level part needs explicit parent"))?,
                ids,
                membership.ok_or(Error::Unsupported("membership"))?,
                &name,
            )?,
            _ => return Err(Error::Unsupported("declaration kind")),
        }
        self.bindings.push(Binding {
            declaration: ast,
            wire: serde_json::Value::String(wire),
        });
        if let Ok(Value::Array(rows)) = slot(arena, ast, "heritage") {
            for row in rows {
                let Value::Node(typing) = row.value else {
                    return Err(Error::Unsupported("heritage"));
                };
                if arena.node(typing).and_then(|n| n.kind()) != Some("FeatureTyping") {
                    return Err(Error::Unsupported("typing kind"));
                }
                let Value::Node(reference) = slot(arena, typing, "targetRef")? else {
                    return Err(Error::Unsupported("reference"));
                };
                self.typing.push((semantic.clone(), *reference));
            }
        }
        for child in children(arena, ast)? {
            self.create(arena, child, Some(&semantic))?;
        }
        Ok(semantic)
    }
}

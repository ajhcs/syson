// Copyright (c) 2022-2025 Sensmetry UAB and others.
// SPDX-License-Identifier: EPL-2.0 OR GPL-2.0 WITH Classpath-exception-2.0
//! Structural fixOperatorExpression port for synthetic kernel nodes.
//! Native child-cache aliases are not represented; parent links finalize at finish.
use std::sync::{Arc, atomic::AtomicBool};
use syson_ast::{Assignment, Builder, Datum, DefaultValue, NodeId, Reflection, Slot, Value};
use syson_ast_dispatch::{ConstructInput, DispatchError, Postprocessor, Registry, parser_registry};
use syson_reflection::{MandatoryKind, OrderingQualification, ReflectionDb, ReflectionError};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Kernel(syson_ast::Error),
    Dispatch(DispatchError),
    Reflection(ReflectionError),
    /// A narrower input domain than JavaScript's arbitrary truthy .length values.
    UnsupportedOperands,
}
impl From<syson_ast::Error> for Error {
    fn from(value: syson_ast::Error) -> Self {
        Self::Kernel(value)
    }
}
impl From<DispatchError> for Error {
    fn from(value: DispatchError) -> Self {
        Self::Dispatch(value)
    }
}
impl From<ReflectionError> for Error {
    fn from(value: ReflectionError) -> Self {
        Self::Reflection(value)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// No other postprocessor is invoked by this module.
    NotSelected,
    Unchanged {
        normalized_operands: bool,
    },
    Inserted {
        expression: NodeId,
        membership: NodeId,
        feature: NodeId,
        normalized_operands: bool,
    },
}

/// Immutable F08 reflection and F09 dispatcher; not a text-expression parser.
pub struct OperatorPostprocessor {
    db: Arc<ReflectionDb>,
    registry: Registry<Postprocessor>,
}
impl OperatorPostprocessor {
    pub fn pinned() -> Result<Self, Error> {
        let db = Arc::new(ReflectionDb::pinned());
        let registry = parser_registry(db.clone())?;
        Ok(Self { db, registry })
    }
    pub fn ordering_qualification(&self) -> OrderingQualification {
        self.db.ordering_qualification()
    }
    /// Exposes existing F08 metadata to the operation kernel, not native
    /// constructor acceptance for every declared kind. No parser callbacks run.
    pub fn builder(
        &self,
        limits: syson_ast::Limits,
        cancelled: Arc<AtomicBool>,
    ) -> Result<Builder, Error> {
        let mut reflection = Reflection::default();
        for ty in self.db.all_generated_types() {
            let defaults = self
                .db
                .effective_mandatory(ty)?
                .iter()
                .map(|p| match p.kind {
                    MandatoryKind::Array => DefaultValue::Array(p.name.into()),
                    MandatoryKind::Boolean => DefaultValue::Boolean(p.name.into()),
                })
                .collect();
            reflection.declare(self.db.name(ty)?.into(), defaults)?;
        }
        Ok(Builder::new(reflection, limits, cancelled)?)
    }
    /// Apply only the selected fixOperatorExpression structural writes.
    /// Falsey operands become []; an empty array and one of the four exact
    /// classification operators inserts a three-node placeholder chain. Nonempty
    /// arrays skip the operator condition. Arbitrary truthy non-arrays are outside
    /// this adapter's domain, rather than silently normalized or treated as [] .
    ///
    /// One kernel transaction rolls back all node/field mutations on an error;
    /// spent IDs and work retain the kernel's non-recycling semantics. Immediate
    /// native $container fields and $children/$childIndex aliases are excluded:
    /// call Builder::finish to establish immutable structural parenting.
    pub fn apply(&self, builder: &mut Builder, node: NodeId) -> Result<Report, Error> {
        let current = builder.node(node)?;
        let kind = current.kind().ok_or(syson_ast::Error::MissingType(node))?;
        if self.registry.construct_plan(ConstructInput::String(kind))?
            != Some(&Postprocessor::FixOperatorExpression)
        {
            return Ok(Report::NotSelected);
        }
        let (normalize, empty) = match current.slot("operands") {
            Slot::Absent => (true, true),
            Slot::Present(datum) => match &datum.value {
                Value::Null | Value::Boolean(false) => (true, true),
                Value::Number(n) if *n == 0.0 || n.is_nan() => (true, true),
                Value::String(s) if s.is_empty() => (true, true),
                Value::Array(values) => (false, values.is_empty()),
                _ => return Err(Error::UnsupportedOperands),
            },
        };
        let insert = empty
            && matches!(current.slot("operator"),
            Slot::Present(Datum { value: Value::String(value), .. })
                if ["istype", "hastype", "@", "as"].contains(&value.as_str()));
        Ok(builder.transaction(|builder| {
            if normalize {
                builder.assign(
                    node,
                    "operands",
                    Assignment::Set,
                    Datum::new(Value::Array(vec![])),
                )?;
            }
            if !insert {
                return Ok(Report::Unchanged {
                    normalized_operands: normalize,
                });
            }
            // Source createNode receives no CST for any of these implicit nodes.
            let expression = builder.allocate(Some("FeatureReferenceExpression"), None)?;
            builder.defaults(expression)?;
            builder.assign(
                node,
                "operands",
                Assignment::Append,
                Datum::node(expression),
            )?;
            let membership = builder.allocate(Some("ReturnParameterMembership"), None)?;
            builder.defaults(membership)?;
            builder.assign(
                expression,
                "expression",
                Assignment::Set,
                Datum::node(membership),
            )?;
            let feature = builder.allocate(Some("Feature"), None)?;
            builder.defaults(feature)?;
            builder.assign(membership, "target", Assignment::Set, Datum::node(feature))?;
            Ok(Report::Inserted {
                expression,
                membership,
                feature,
                normalized_operands: normalize,
            })
        })?)
    }
}

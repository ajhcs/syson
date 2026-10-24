/*******************************************************************************
 * Copyright (c) 2025, 2026 Obeo.
 * This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License v2.0
 * which accompanies this distribution, and is available at
 * https://www.eclipse.org/legal/epl-2.0/
 *
 * SPDX-License-Identifier: EPL-2.0
 *
 * Contributors:
 *     Obeo - initial API and implementation
 *******************************************************************************/
//! Explicit opt-in construction over the existing selected JSON projection.
//! Helpers retain their native-state, reference and serializer limitations.
//! Rust adapter contributors - composition and owned transaction protocol.
use std::{cell::RefCell, rc::Rc};
use syson_initializer_graph::{
    AcceptActionHelperError, AcceptActionHelperReport, ConnectionHelperError,
    ConnectionHelperReport, InterfaceHelperError, InterfaceHelperReport, PortDefinitionHelperError,
    PortDefinitionHelperReport, initialize_accept_action, initialize_connection_definition,
    initialize_interface_definition, initialize_port_definition,
};
use syson_metamodel::{self as registry, ClassifierId};
use syson_model::{
    ContainmentFeature, ModelError, NodeIds, RawAdmissionError, RawAdmissionReport, Transaction,
    attach_raw_selected,
    factory::{FactoryError, RawFactory},
};

/// The existing helper's exact result, including its unrepresented native state.
#[derive(Debug)]
pub enum CreationInitialization {
    Connection(ConnectionHelperReport),
    Interface(InterfaceHelperReport),
    AcceptAction(AcceptActionHelperReport),
    Port(PortDefinitionHelperReport),
}

#[derive(Debug)]
pub struct InitializedCreationReport {
    pub element_id: String,
    pub classifier: ClassifierId,
    pub membership_class: ClassifierId,
    pub membership: RawAdmissionReport,
    pub root: RawAdmissionReport,
    pub initialization: CreationInitialization,
    pub identities_consumed: usize,
}

#[derive(Debug)]
pub enum InitializedCreationError {
    UnsupportedClassifier(ClassifierId),
    UnsupportedParent(String),
    InvalidName,
    Cancelled,
    Factory(FactoryError),
    Admission(RawAdmissionError),
    Model(ModelError),
    Connection(ConnectionHelperError),
    Interface(InterfaceHelperError),
    AcceptAction(AcceptActionHelperError),
    Port(PortDefinitionHelperError),
}
impl std::fmt::Display for InitializedCreationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl std::error::Error for InitializedCreationError {}

struct IdentityPool<I> {
    remaining: I,
    issued: Vec<NodeIds>,
}

/// Stage a complete supported constructor in the caller's owned transaction.
///
/// Each supplied pair binds one fresh-v4 semantic identity to its explicit
/// resource identity. This adapter allocates membership first, then the root,
/// followed by the selected helper's source-ordered allocations. This outer
/// allocation protocol does not establish native factory identity order.
/// The caller must provide
/// genuine fresh entropy; UUID spelling does not establish randomness.
///
/// `None` preserves the initializer's default name. An explicit nonempty name
/// applies after initialization, in the same transaction. The membership rule
/// is pinned MetamodelMutationElementService.addChildInParent: Feature inside
/// Type uses FeatureMembership; other supported pairs use OwningMembership.
///
/// No Editor commit occurs here. Failure or cancellation drops the entire
/// owned transaction; consumed identities are not refunded. Success can be
/// cancelled by dropping the returned transaction, or committed once by the
/// caller. No generic factory, complete EMF serialization or runtime parity is
/// implied by exposing these four existing helpers.
pub fn stage_initialized_creation<I, K>(
    transaction: Transaction,
    classifier: ClassifierId,
    parent: &str,
    name: Option<&str>,
    identities: I,
    mut cancelled: K,
) -> Result<(Transaction, InitializedCreationReport), InitializedCreationError>
where
    I: IntoIterator<Item = NodeIds>,
    K: FnMut() -> bool,
{
    use ClassifierId as C;
    use InitializedCreationError as E;
    if cancelled() {
        return Err(E::Cancelled);
    }
    if !matches!(
        classifier,
        C::ConnectionDefinition | C::InterfaceDefinition | C::AcceptActionUsage | C::PortDefinition
    ) {
        return Err(E::UnsupportedClassifier(classifier));
    }
    if name.is_some_and(str::is_empty) {
        return Err(E::InvalidName);
    }
    if !transaction.document().can_create_in(parent) {
        return Err(E::UnsupportedParent(parent.into()));
    }
    let membership_class = if registry::is_subtype(classifier, C::Feature)
        && transaction.document().is_instance_of(parent, C::Type)
    {
        C::FeatureMembership
    } else {
        C::OwningMembership
    };
    let pool = Rc::new(RefCell::new(IdentityPool {
        remaining: identities.into_iter(),
        issued: Vec::new(),
    }));
    let semantic_pool = Rc::clone(&pool);
    let mut factory = RawFactory::new(move || {
        let mut pool = semantic_pool.borrow_mut();
        let ids = pool
            .remaining
            .next()
            .ok_or_else(|| "Creation identity pairs exhausted".to_owned())?;
        // Public NodeIds fields may have been changed since construction.
        NodeIds::new(&ids.resource, &ids.element).map_err(|error| error.to_string())?;
        let semantic = ids.element.clone();
        pool.issued.push(ids);
        Ok(semantic)
    });
    let resource_pool = Rc::clone(&pool);
    let mut resources = move |_: C, semantic: &str| {
        resource_pool
            .borrow()
            .issued
            .iter()
            .find(|ids| ids.element == semantic)
            .map(|ids| ids.resource.clone())
            .ok_or_else(|| "No resource identity was supplied for this allocation".to_owned())
    };
    if cancelled() {
        return Err(E::Cancelled);
    }
    let membership = factory.allocate(membership_class).map_err(E::Factory)?;
    let membership_resource = resources(membership_class, membership.semantic_id())
        .map_err(|message| E::Factory(FactoryError::IdentityProvider(message)))?;
    let (transaction, membership_report) = attach_raw_selected(
        transaction,
        parent,
        ContainmentFeature::OwnedRelationship,
        &membership,
        &membership_resource,
        &[],
    )
    .map_err(E::Admission)?;
    if cancelled() {
        return Err(E::Cancelled);
    }
    let raw_root = factory.allocate(classifier).map_err(E::Factory)?;
    let root_resource = resources(classifier, raw_root.semantic_id())
        .map_err(|message| E::Factory(FactoryError::IdentityProvider(message)))?;
    let element_id = raw_root.semantic_id().to_owned();
    let (transaction, root_report) = attach_raw_selected(
        transaction,
        membership.semantic_id(),
        ContainmentFeature::OwnedRelatedElement,
        &raw_root,
        &root_resource,
        &[],
    )
    .map_err(E::Admission)?;
    let (mut transaction, initialization) = match classifier {
        C::ConnectionDefinition => {
            let (transaction, report) = initialize_connection_definition(
                transaction,
                &element_id,
                &mut factory,
                &mut resources,
                &mut cancelled,
            )
            .map_err(E::Connection)?;
            (transaction, CreationInitialization::Connection(report))
        }
        C::InterfaceDefinition => {
            let (transaction, report) = initialize_interface_definition(
                transaction,
                &element_id,
                &mut factory,
                &mut resources,
                &mut cancelled,
            )
            .map_err(E::Interface)?;
            (transaction, CreationInitialization::Interface(report))
        }
        C::AcceptActionUsage => {
            let (transaction, report) = initialize_accept_action(
                transaction,
                &element_id,
                &mut factory,
                &mut resources,
                &mut cancelled,
            )
            .map_err(E::AcceptAction)?;
            (transaction, CreationInitialization::AcceptAction(report))
        }
        C::PortDefinition => {
            let (transaction, report) = initialize_port_definition(
                transaction,
                &element_id,
                &mut factory,
                &mut resources,
                &mut cancelled,
            )
            .map_err(E::Port)?;
            (transaction, CreationInitialization::Port(report))
        }
        _ => unreachable!("exact classifier guard above"),
    };
    if cancelled() {
        return Err(E::Cancelled);
    }
    if let Some(name) = name {
        transaction
            .set_attribute(&element_id, "declaredName", name.into())
            .map_err(E::Model)?;
    }
    let identities_consumed = pool.borrow().issued.len();
    Ok((
        transaction,
        InitializedCreationReport {
            element_id,
            classifier,
            membership_class,
            membership: membership_report,
            root: root_report,
            initialization,
            identities_consumed,
        },
    ))
}

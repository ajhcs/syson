// SPDX-License-Identifier: EPL-2.0
//! Immutable dispatch planning for the single SysMLMoveElementService category.
//! Caller order and capability observations are inputs, not Spring discovery or
//! native-model predicates. No provider code or semantic move executes here.
#![forbid(unsafe_code)]

use std::sync::Arc;

/// Diagnostic label, not a native provider key. Duplicate labels remain allowed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderId(String);

impl ProviderId {
    /// Rust-only admission policy: 1..=128 UTF-8 bytes, not whitespace-only.
    pub fn new(value: impl Into<String>) -> Result<Self, RegistryError> {
        let value = value.into();
        if value.trim().is_empty() || value.len() > 128 {
            return Err(RegistryError::InvalidLabel);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryError {
    InvalidLabel,
    MissingDefault,
    ExcessObservations { registered: usize, supplied: usize },
    MissingCapability { custom_index: usize },
}

/// Result of an external canHandle(element, newParent) observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    Observed(bool),
    Unavailable,
}

/// Caller-supplied injection order is preserved exactly, including duplicates.
#[derive(Clone, Debug)]
pub struct MoveRegistry {
    default: ProviderId,
    custom: Arc<[ProviderId]>,
}

impl MoveRegistry {
    /// A missing default is rejected, matching the upstream constructor's
    /// non-null default requirement. Labels use the separate Rust policy above.
    pub fn new(
        default: Option<ProviderId>,
        custom: Vec<ProviderId>,
    ) -> Result<Self, RegistryError> {
        Ok(Self {
            default: default.ok_or(RegistryError::MissingDefault)?,
            custom: custom.into(),
        })
    }

    pub fn custom_providers(&self) -> &[ProviderId] {
        &self.custom
    }

    /// One observation per custom slot. A short packet is allowed: unread
    /// trailing slots can be unavailable when an earlier provider matches.
    /// Native request identity/freshness remains the observing caller's duty.
    pub fn snapshot(&self, observations: Vec<Capability>) -> Result<MoveSnapshot, RegistryError> {
        if observations.len() > self.custom.len() {
            return Err(RegistryError::ExcessObservations {
                registered: self.custom.len(),
                supplied: observations.len(),
            });
        }
        Ok(MoveSnapshot {
            registry: self.clone(),
            observations: observations.into(),
        })
    }
}

/// Registry and observations are owned immutable data in the same packet.
#[derive(Clone, Debug)]
pub struct MoveSnapshot {
    registry: MoveRegistry,
    observations: Arc<[Capability]>,
}

impl MoveSnapshot {
    /// First true wins. An unavailable observation is an error only if reached;
    /// it is never silently interpreted as false or skipped to the fallback.
    pub fn select(&self) -> Result<MovePlan, RegistryError> {
        for (index, provider) in self.registry.custom.iter().enumerate() {
            match self.observations.get(index) {
                Some(Capability::Observed(true)) => {
                    return Ok(MovePlan {
                        selected: ProviderSlot::Custom {
                            index,
                            id: provider.clone(),
                        },
                        default: self.registry.default.clone(),
                    });
                }
                Some(Capability::Observed(false)) => {}
                Some(Capability::Unavailable) | None => {
                    return Err(RegistryError::MissingCapability {
                        custom_index: index,
                    });
                }
            }
        }
        Ok(MovePlan {
            selected: ProviderSlot::Default(self.registry.default.clone()),
            default: self.registry.default.clone(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderSlot {
    Custom { index: usize, id: ProviderId },
    Default(ProviderId),
}

/// The Java record has a success flag and message. A failure is still a non-null
/// status and therefore does not request the default or a later delegate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoveStatus {
    pub is_success: bool,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Completion {
    Returned(Option<MoveStatus>),
    InvokeDefault(ProviderId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MovePlan {
    selected: ProviderSlot,
    default: ProviderId,
}

impl MovePlan {
    pub fn selected(&self) -> &ProviderSlot {
        &self.selected
    }

    /// Observe the selected provider's result without executing it. Upstream
    /// Optional.map converts a null custom result into a default invocation;
    /// the default's own null result is returned without a further fallback.
    pub fn complete(&self, result: Option<MoveStatus>) -> Completion {
        match (&self.selected, result) {
            (ProviderSlot::Custom { .. }, None) => Completion::InvokeDefault(self.default.clone()),
            (_, result) => Completion::Returned(result),
        }
    }
}

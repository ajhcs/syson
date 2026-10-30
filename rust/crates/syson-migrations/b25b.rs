// Copyright (c) 2025 Obeo.
// SPDX-License-Identifier: EPL-2.0
//! ConnectorAsUsage source/target participant, version 2025.4.0-202502270000.
//! Pure bounded JSON preparation; history selection and atomic persistence are
//! caller responsibilities. The shared traversal preserves the pinned Java
//! helper's object-child selection, including non-EObject vendor objects.
use crate::{Error, Prepared, Removal};

pub const VERSION: &str = "2025.4.0-202502270000";

pub fn prepare(input: &str) -> Result<Prepared, Error> {
    crate::prepare(
        input,
        VERSION,
        &[Removal {
            class: "ConnectorAsUsage",
            features: &["source", "target"],
        }],
    )
}

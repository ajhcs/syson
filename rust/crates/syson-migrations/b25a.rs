// SPDX-License-Identifier: EPL-2.0
//! Annotation reference migration before native deserialization.
use crate::{Error, Prepared, Removal};

pub const VERSION: &str = "2025.4.0-202503140000";

/// Prepare the pinned Annotation/AnnotatingElement participant in native pass
/// order. The shared service preserves its first-namespace-entry and typed-
/// parent/object-child traversal behavior. No persistence or migration-history
/// dispatch is performed; callers admit the complete output before publishing.
pub fn prepare(input: &str) -> Result<Prepared, Error> {
    crate::prepare(
        input,
        VERSION,
        &[
            Removal {
                class: "Annotation",
                features: &["annotatingElement"],
            },
            Removal {
                class: "AnnotatingElement",
                features: &["annotation"],
            },
        ],
    )
}

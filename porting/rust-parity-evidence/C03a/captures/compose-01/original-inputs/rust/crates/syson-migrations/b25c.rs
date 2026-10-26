// SPDX-License-Identifier: EPL-2.0
//! Selected ViewUsage.exposedElement pre-deserialization migration.
//! Persistence and native global migration history remain the caller's concern.
use crate::{Error, Prepared, Removal};

pub const VERSION: &str = "2025.6.0-202506110000";

/// Preserve the pinned participant's exact class, feature spelling and version.
/// The shared helper applies its resource predicate and child-object traversal;
/// it does not rewrite exposedElement on unrelated or foreign classes.
pub fn prepare(input: &str) -> Result<Prepared, Error> {
    crate::prepare(
        input,
        VERSION,
        &[Removal {
            class: "ViewUsage",
            features: &["exposedElement"],
        }],
    )
}

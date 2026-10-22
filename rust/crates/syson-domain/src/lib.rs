// Copyright (c) 2024, 2026 Obeo.
// This program and accompanying materials are made available under the terms
// of the Eclipse Public License v2.0, https://www.eclipse.org/legal/epl-2.0/.
// SPDX-License-Identifier: EPL-2.0
// Contributors: Obeo - initial API and implementation
// Rust translation and compatibility tests: SysON Rust port contributors.

//! Bounded compatibility primitives for pinned SysON 6ba2ac64.
//! See porting/domain/README.md for supported methods and deliberate boundaries.
//! This is not a SysML parser, complete metamodel, server, or persistence adapter.
#![forbid(unsafe_code)]
pub mod model;
pub mod names;
pub mod uuid;

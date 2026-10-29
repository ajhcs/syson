// SPDX-License-Identifier: EPL-2.0
#![forbid(unsafe_code)]
mod admission;
pub use admission::{
    AdmittedRouting, Edge, Error, Limit, Limits, Point, Route, Routing, Side, Unsupported, admit,
};

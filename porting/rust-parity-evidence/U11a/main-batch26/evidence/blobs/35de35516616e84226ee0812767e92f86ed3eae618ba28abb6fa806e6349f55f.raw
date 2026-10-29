// SPDX-License-Identifier: EPL-2.0
//! Borrowed flat-layout inputs and typed finite admission failures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasuredNode<'a> {
    pub id: &'a str,
    pub width: f64,
    pub height: f64,
    pub parent: Option<&'a str>,
    pub port_count: usize,
    pub pinned: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectedEdge<'a> {
    pub id: &'a str,
    pub source: &'a str,
    pub target: &'a str,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Right,
    Down,
}
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub direction: Direction,
    pub layer_gap: f64,
    pub node_gap: f64,
    pub component_gap: f64,
    pub padding: f64,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            direction: Direction::Right,
            layer_gap: 40.0,
            node_gap: 20.0,
            component_gap: 60.0,
            padding: 16.0,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_nodes: usize,
    pub max_edges: usize,
    pub max_work: usize,
    /// Logical conservative buffer estimate, not allocator overhead or RSS.
    pub max_workspace_bytes: usize,
    pub max_id_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_nodes: 4096,
            max_edges: 16384,
            max_work: 100_000_000,
            max_workspace_bytes: 16 * 1024 * 1024,
            max_id_bytes: 1024,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    Nodes,
    Edges,
    Work,
    Memory,
    IdBytes,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unsupported {
    Container,
    Ports,
    Pinned,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutError {
    EmptyId,
    DuplicateNode,
    DuplicateEdge,
    UnknownEndpoint,
    InvalidNumber,
    ArithmeticOverflow,
    PrecisionLoss,
    Allocation,
    Cancelled,
    Limit(Limit),
    Unsupported(Unsupported),
}

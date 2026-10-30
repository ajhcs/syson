// SPDX-License-Identifier: EPL-2.0
//! Proposed finite Rust router, not ELK behavior. Positive integer rectangles,
//! side-center anchors and exact half-unit points; touching boundaries is allowed.
#![forbid(unsafe_code)]
use syson_layout::Placement;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    /// Twice the input coordinate, allowing exact half-unit side centers.
    pub x2: i64,
    pub y2: i64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}
#[derive(Clone, Copy, Debug)]
pub struct Edge<'a> {
    pub id: &'a str,
    pub source: &'a str,
    pub target: &'a str,
    pub source_side: Side,
    pub target_side: Side,
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_nodes: usize,
    pub max_edges: usize,
    pub max_grid_vertices: usize,
    pub max_output_points: usize,
    pub max_work: usize,
    pub max_id_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_nodes: 128,
            max_edges: 256,
            max_grid_vertices: 16_384,
            max_output_points: 65_536,
            max_work: 20_000_000,
            max_id_bytes: 1024,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    Nodes,
    Edges,
    Grid,
    Output,
    Work,
    IdBytes,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unsupported {
    Geometry,
    OverlappingInteriors,
    CoincidentAnchors,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    EmptyId,
    DuplicateNode,
    DuplicateEdge,
    UnknownEndpoint,
    Unsupported(Unsupported),
    Limit(Limit),
    Allocation,
    Cancelled,
    Unroutable,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route<'a> {
    pub id: &'a str,
    pub points: Vec<Point>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Routing<'a> {
    /// Canonical edge-ID order, independent of input permutations.
    pub edges: Vec<Route<'a>>,
    pub charged_work: usize,
    pub grid_vertices: usize,
}
struct Guard<F> {
    cancel: F,
    used: usize,
    max: usize,
}
impl<F: FnMut() -> bool> Guard<F> {
    fn tick(&mut self, n: usize) -> Result<(), Error> {
        if (self.cancel)() {
            return Err(Error::Cancelled);
        }
        self.used = self
            .used
            .checked_add(n)
            .filter(|n| *n <= self.max)
            .ok_or(Error::Limit(Limit::Work))?;
        Ok(())
    }
    fn sort_cost(&mut self, n: usize, bytes: usize) -> Result<(), Error> {
        self.tick(
            n.checked_mul(n)
                .and_then(|v| v.checked_mul(bytes.max(1)))
                .ok_or(Error::Limit(Limit::Work))?,
        )
    }
}
fn reserved<T>(n: usize) -> Result<Vec<T>, Error> {
    let mut v = Vec::new();
    v.try_reserve_exact(n).map_err(|_| Error::Allocation)?;
    Ok(v)
}
fn push<T>(v: &mut Vec<T>, value: T) -> Result<(), Error> {
    v.try_reserve(1).map_err(|_| Error::Allocation)?;
    v.push(value);
    Ok(())
}
#[derive(Clone, Copy)]
struct Rect<'a> {
    id: &'a str,
    l: i64,
    r: i64,
    t: i64,
    b: i64,
}
impl<'a> Rect<'a> {
    fn from(p: &Placement<'a>) -> Result<Self, Error> {
        let integer = |v: f64| v.is_finite() && v.fract() == 0.0 && v.abs() <= 2_000_000.0;
        if ![p.x, p.y, p.width, p.height].into_iter().all(integer)
            || p.width <= 0.0
            || p.height <= 0.0
        {
            return Err(Error::Unsupported(Unsupported::Geometry));
        }
        let r = Self {
            id: p.id,
            l: p.x as i64 * 2,
            r: (p.x as i64 + p.width as i64) * 2,
            t: p.y as i64 * 2,
            b: (p.y as i64 + p.height as i64) * 2,
        };
        if [r.l, r.r, r.t, r.b]
            .into_iter()
            .any(|v| v.abs() > 2_000_000)
        {
            return Err(Error::Unsupported(Unsupported::Geometry));
        }
        Ok(r)
    }
    fn anchor(self, side: Side) -> Point {
        match side {
            Side::Left => Point {
                x2: self.l,
                y2: (self.t + self.b) / 2,
            },
            Side::Right => Point {
                x2: self.r,
                y2: (self.t + self.b) / 2,
            },
            Side::Top => Point {
                x2: (self.l + self.r) / 2,
                y2: self.t,
            },
            Side::Bottom => Point {
                x2: (self.l + self.r) / 2,
                y2: self.b,
            },
        }
    }
    fn contains(self, p: Point) -> bool {
        self.l < p.x2 && p.x2 < self.r && self.t < p.y2 && p.y2 < self.b
    }
    fn intersects(self, a: Point, b: Point) -> bool {
        if a.x2 == b.x2 {
            self.l < a.x2 && a.x2 < self.r && a.y2.max(b.y2) > self.t && a.y2.min(b.y2) < self.b
        } else {
            self.t < a.y2 && a.y2 < self.b && a.x2.max(b.x2) > self.l && a.x2.min(b.x2) < self.r
        }
    }
}
fn identity(id: &str, limit: usize) -> Result<(), Error> {
    if id.is_empty() {
        return Err(Error::EmptyId);
    }
    if id.len() > limit {
        return Err(Error::Limit(Limit::IdBytes));
    }
    Ok(())
}
/// Route a flat finite geometry snapshot without mutation or partial results.
/// Paths minimize visibility-grid hops, not metric length/bends. Boundary and
/// corner contact is allowed (zero clearance). Parallel routes may coincide.
/// Integer x/y/width/height are required; every rectangle edge must be within
/// ±1e6 (positive dimensions can reach 2e6). Sides constrain anchor position,
/// not departure/arrival heading; routes can follow the endpoint node boundary.
/// Work is charged logical operations, not time; allocations/sorts poll between
/// operations. Limits are admission guards, not a process memory/RSS guarantee.
pub fn route<'a>(
    nodes: &[Placement<'a>],
    edges: &[Edge<'a>],
    limits: Limits,
    cancelled: impl FnMut() -> bool,
) -> Result<Routing<'a>, Error> {
    let mut guard = Guard {
        cancel: cancelled,
        used: 0,
        max: limits.max_work,
    };
    guard.tick(0)?;
    if nodes.len() > limits.max_nodes {
        return Err(Error::Limit(Limit::Nodes));
    }
    if edges.len() > limits.max_edges {
        return Err(Error::Limit(Limit::Edges));
    }
    let mut rects = reserved(nodes.len())?;
    for node in nodes {
        guard.tick(node.id.len().max(1))?;
        identity(node.id, limits.max_id_bytes)?;
        rects.push(Rect::from(node)?);
    }
    guard.sort_cost(rects.len(), limits.max_id_bytes)?;
    rects.sort_unstable_by_key(|r| r.id);
    for pair in rects.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(Error::DuplicateNode);
        }
    }
    for (i, a) in rects.iter().enumerate() {
        for b in &rects[i + 1..] {
            guard.tick(1)?;
            if a.l.max(b.l) < a.r.min(b.r) && a.t.max(b.t) < a.b.min(b.b) {
                return Err(Error::Unsupported(Unsupported::OverlappingInteriors));
            }
        }
    }
    let mut ordered = reserved(edges.len())?;
    for edge in edges {
        guard.tick(edge.id.len().max(1))?;
        identity(edge.id, limits.max_id_bytes)?;
        ordered.push(edge);
    }
    guard.sort_cost(ordered.len(), limits.max_id_bytes)?;
    ordered.sort_unstable_by_key(|e| e.id);
    for pair in ordered.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(Error::DuplicateEdge);
        }
    }
    let mut terminals = reserved(edges.len())?;
    for edge in ordered {
        let mut endpoint = |id: &str, side| -> Result<Point, Error> {
            guard.tick(rects.len().max(1))?;
            let i = rects
                .binary_search_by_key(&id, |r| r.id)
                .map_err(|_| Error::UnknownEndpoint)?;
            Ok(rects[i].anchor(side))
        };
        let a = endpoint(edge.source, edge.source_side)?;
        let b = endpoint(edge.target, edge.target_side)?;
        if a == b {
            return Err(Error::Unsupported(Unsupported::CoincidentAnchors));
        }
        terminals.push((edge.id, a, b));
    }
    if edges.is_empty() {
        return Ok(Routing {
            edges: Vec::new(),
            charged_work: guard.used,
            grid_vertices: 0,
        });
    }
    let bound = nodes
        .len()
        .checked_mul(2)
        .and_then(|v| edges.len().checked_mul(2).and_then(|e| v.checked_add(e)))
        .and_then(|v| v.checked_add(2))
        .ok_or(Error::Limit(Limit::Grid))?;
    let (mut xs, mut ys) = (reserved(bound)?, reserved(bound)?);
    for r in &rects {
        xs.extend([r.l, r.r]);
        ys.extend([r.t, r.b]);
    }
    for (_, a, b) in &terminals {
        xs.extend([a.x2, b.x2]);
        ys.extend([a.y2, b.y2]);
    }
    let xmin = *xs.iter().min().ok_or(Error::Unroutable)?;
    let xmax = *xs.iter().max().ok_or(Error::Unroutable)?;
    let ymin = *ys.iter().min().ok_or(Error::Unroutable)?;
    let ymax = *ys.iter().max().ok_or(Error::Unroutable)?;
    xs.extend([xmin - 1, xmax + 1]);
    ys.extend([ymin - 1, ymax + 1]);
    guard.sort_cost(xs.len(), 1)?;
    xs.sort_unstable();
    xs.dedup();
    guard.sort_cost(ys.len(), 1)?;
    ys.sort_unstable();
    ys.dedup();
    let count = xs
        .len()
        .checked_mul(ys.len())
        .filter(|v| *v <= limits.max_grid_vertices)
        .ok_or(Error::Limit(Limit::Grid))?;
    guard.tick(count)?;
    let point = |i: usize| Point {
        x2: xs[i % xs.len()],
        y2: ys[i / xs.len()],
    };
    let index =
        |p: Point| ys.binary_search(&p.y2).unwrap() * xs.len() + xs.binary_search(&p.x2).unwrap();
    let mut blocked = reserved(count)?;
    for i in 0..count {
        let mut inside = false;
        for r in &rects {
            guard.tick(1)?;
            inside |= r.contains(point(i));
        }
        blocked.push(inside);
    }
    let (mut parents, mut queue) = (reserved(count)?, reserved(count)?);
    parents.resize(count, usize::MAX);
    let mut routes = reserved(terminals.len())?;
    let mut output_points = 0usize;
    for (id, a, b) in terminals {
        guard.tick(count)?;
        parents.fill(usize::MAX);
        queue.clear();
        let (start, end) = (index(a), index(b));
        if blocked[start] || blocked[end] {
            return Err(Error::Unroutable);
        }
        parents[start] = start;
        queue.push(start);
        let mut head = 0;
        while head < queue.len() && parents[end] == usize::MAX {
            guard.tick(1)?;
            let from = queue[head];
            head += 1;
            let (x, y) = (from % xs.len(), from / xs.len());
            let adjacent = [
                (x + 1 < xs.len()).then_some(from + 1),
                (y + 1 < ys.len()).then(|| from + xs.len()),
                (x > 0).then(|| from - 1),
                (y > 0).then(|| from - xs.len()),
            ];
            for to in adjacent.into_iter().flatten() {
                guard.tick(1)?;
                if blocked[to] || parents[to] != usize::MAX {
                    continue;
                }
                let mut clear = true;
                for r in &rects {
                    guard.tick(1)?;
                    clear &= !r.intersects(point(from), point(to));
                }
                if clear {
                    parents[to] = from;
                    queue.push(to);
                }
            }
        }
        if parents[end] == usize::MAX {
            return Err(Error::Unroutable);
        }
        let mut reverse = reserved(count)?;
        let mut cursor = end;
        while cursor != start {
            guard.tick(1)?;
            reverse.push(point(cursor));
            cursor = parents[cursor];
        }
        reverse.push(a);
        reverse.reverse();
        let mut points: Vec<Point> = Vec::new();
        for p in reverse {
            guard.tick(1)?;
            if points.len() >= 2 {
                let (a, b) = (points[points.len() - 2], points[points.len() - 1]);
                if (a.x2 == b.x2 && b.x2 == p.x2) || (a.y2 == b.y2 && b.y2 == p.y2) {
                    points.pop();
                }
            }
            push(&mut points, p)?;
        }
        output_points = output_points
            .checked_add(points.len())
            .filter(|v| *v <= limits.max_output_points)
            .ok_or(Error::Limit(Limit::Output))?;
        routes.push(Route { id, points });
    }
    guard.tick(0)?;
    Ok(Routing {
        edges: routes,
        charged_work: guard.used,
        grid_vertices: count,
    })
}

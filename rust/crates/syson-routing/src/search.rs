// SPDX-License-Identifier: EPL-2.0
//! Consuming finite integer search; fractional/native/UI acceptance remains open.
use crate::admission::{Rect, reserved};
use crate::{AdmittedRouting, Error, Limit, Point, Route, Routing};

fn push<T>(v: &mut Vec<T>, value: T) -> Result<(), Error> {
    v.try_reserve(1).map_err(|_| Error::Allocation)?;
    v.push(value);
    Ok(())
}

impl Rect<'_> {
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

/// Consume canonical admission once, retaining its original work and callback.
/// Paths minimize visibility-grid hops, not metric length or bends. Boundary
/// contact has zero clearance; parallel routes may coincide. Failure publishes
/// no partial routes. Logical charges/reservations do not bound RSS or time.
pub fn route_admitted<'a, F: FnMut() -> bool>(
    admitted: AdmittedRouting<'a, F>,
) -> Result<Routing<'a>, Error> {
    let AdmittedRouting {
        rects,
        terminals,
        limits,
        mut guard,
    } = admitted;
    // Added zero-charge fences poll the original callback between phases and
    // at empty/final publication, without waiving any donor positive charge.
    guard.tick(0)?;
    if terminals.is_empty() {
        guard.tick(0)?;
        return Ok(Routing {
            edges: Vec::new(),
            charged_work: guard.used(),
            grid_vertices: 0,
        });
    }
    let bound = rects
        .len()
        .checked_mul(2)
        .and_then(|v| {
            terminals
                .len()
                .checked_mul(2)
                .and_then(|e| v.checked_add(e))
        })
        .and_then(|v| v.checked_add(2))
        .ok_or(Error::Limit(Limit::Grid))?;
    guard.tick(0)?;
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
    let extent = |value: i64, delta: i64| value.checked_add(delta).ok_or(Error::Limit(Limit::Grid));
    xs.extend([extent(xmin, -1)?, extent(xmax, 1)?]);
    ys.extend([extent(ymin, -1)?, extent(ymax, 1)?]);
    guard.sort_cost(xs.len(), 1)?;
    xs.sort_unstable();
    xs.dedup();
    guard.tick(0)?;
    guard.sort_cost(ys.len(), 1)?;
    ys.sort_unstable();
    ys.dedup();
    guard.tick(0)?;
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
    guard.tick(0)?;
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
        guard.tick(0)?;
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
        charged_work: guard.used(),
        grid_vertices: count,
    })
}

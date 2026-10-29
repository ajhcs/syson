// SPDX-License-Identifier: EPL-2.0
//! Actual integer routing admission only; search/native/UI/capacity remain OPEN.
//! Allocator/counters/VmHWM block copied unchanged from published u10a_profile.rs,
//! SHA-256 022cba2e1bf55c22a55e4ca85820793c73e191797c155ad02c2a039bf75b0cdd.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};
use std::time::Instant;
use syson_layout::Placement;
use syson_routing::{Edge, Error, Limit, Limits, Point, Side, admit};

struct Allocator;
static REQUESTS: AtomicU64 = AtomicU64::new(0);
static REQUESTED: AtomicU64 = AtomicU64::new(0);
static DEALLOCS: AtomicU64 = AtomicU64::new(0);
static FREED: AtomicU64 = AtomicU64::new(0);
static LIVE: AtomicU64 = AtomicU64::new(0);
static PEAK: AtomicU64 = AtomicU64::new(0);
static PHASE_PEAK: AtomicU64 = AtomicU64::new(0);

fn added(size: usize) {
    let live = LIVE.fetch_add(size as u64, SeqCst) + size as u64;
    PEAK.fetch_max(live, SeqCst);
    PHASE_PEAK.fetch_max(live, SeqCst);
}
fn removed(size: usize) {
    FREED.fetch_add(size as u64, SeqCst);
    DEALLOCS.fetch_add(1, SeqCst);
    LIVE.fetch_sub(size as u64, SeqCst);
}
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        REQUESTS.fetch_add(1, SeqCst);
        REQUESTED.fetch_add(layout.size() as u64, SeqCst);
        let result = unsafe { System.alloc(layout) };
        if !result.is_null() {
            added(layout.size());
        }
        result
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        removed(layout.size());
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        REQUESTS.fetch_add(1, SeqCst);
        REQUESTED.fetch_add(size as u64, SeqCst);
        let result = unsafe { System.realloc(ptr, layout, size) };
        if !result.is_null() {
            removed(layout.size());
            added(size);
        }
        result
    }
}
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

#[derive(Clone, Copy)]
struct Counters {
    requests: u64,
    requested: u64,
    deallocs: u64,
    freed: u64,
    live: u64,
    peak: u64,
}
fn counters() -> Counters {
    Counters {
        requests: REQUESTS.load(SeqCst),
        requested: REQUESTED.load(SeqCst),
        deallocs: DEALLOCS.load(SeqCst),
        freed: FREED.load(SeqCst),
        live: LIVE.load(SeqCst),
        peak: PEAK.load(SeqCst),
    }
}
fn delta(a: Counters, b: Counters, phase_peak: u64) -> serde_json::Value {
    serde_json::json!({"allocation_requests": b.requests-a.requests,
        "requested_bytes": b.requested-a.requested, "deallocation_events": b.deallocs-a.deallocs,
        "freed_bytes": b.freed-a.freed, "live_bytes_before": a.live, "live_bytes_after": b.live,
        "global_peak_before": a.peak, "global_peak_after": b.peak,
        "phase_peak_live_bytes": phase_peak,
        "phase_peak_above_initial_live": phase_peak.saturating_sub(a.live)})
}
fn rss_kib() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find(|line| line.starts_with("VmHWM:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}
fn limit_values(limits: Limits) -> [usize; 6] {
    [
        limits.max_nodes,
        limits.max_edges,
        limits.max_grid_vertices,
        limits.max_output_points,
        limits.max_work,
        limits.max_id_bytes,
    ]
}

fn anchor(node: &Placement<'_>, side: Side) -> Point {
    let (left, top) = (node.x as i64 * 2, node.y as i64 * 2);
    let (width, height) = (node.width as i64, node.height as i64);
    let (x2, y2) = match side {
        Side::Left => (left, top + height),
        Side::Right => (left + width * 2, top + height),
        Side::Top => (left + width, top),
        Side::Bottom => (left + width, top + height * 2),
    };
    Point { x2, y2 }
}

fn run(count: usize, density: usize, cancel_before: bool) -> Result<serde_json::Value, String> {
    let limits = Limits::default();
    if limit_values(limits) != [128, 256, 16_384, 65_536, 20_000_000, 1024] {
        return Err("default limits changed; review workload before execution".into());
    }
    let ids: Vec<_> = (0..count)
        .map(|i| format!("node/{i:08}-fixed-fixture"))
        .collect();
    let edge_ids: Vec<_> = (0..count * density)
        .map(|i| format!("edge/{i:08}-fixed-fixture"))
        .collect();
    let nodes: Vec<_> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| Placement {
            id,
            x: (i % 32 * 100) as f64,
            y: (i / 32 * 80) as f64,
            width: (41 + i % 7 * 2) as f64,
            height: (21 + i % 5 * 2) as f64,
            strong_component: i,
            weak_component: 0,
            rank: i,
        })
        .collect();
    let edges: Vec<_> = edge_ids
        .iter()
        .enumerate()
        .map(|(i, id)| Edge {
            id,
            source: &ids[i / density],
            target: &ids[(i / density + [1, 7][i % density]) % count],
            source_side: if i % density == 0 {
                Side::Right
            } else {
                Side::Bottom
            },
            target_side: if i % density == 0 {
                Side::Left
            } else {
                Side::Top
            },
        })
        .collect();
    let original_nodes = nodes.clone();
    let original_edges: Vec<_> = edges
        .iter()
        .map(|edge| {
            (
                edge.id,
                edge.source,
                edge.target,
                edge.source_side,
                edge.target_side,
            )
        })
        .collect();
    let polls = Cell::new(0usize);
    let before = counters();
    PHASE_PEAK.store(before.live, SeqCst);
    let started = Instant::now();
    let outcome = admit(&nodes, &edges, limits, || {
        polls.set(polls.get() + 1);
        cancel_before
    });
    let api_ns = started.elapsed().as_nanos();
    let after = counters();
    let phase_peak = PHASE_PEAK.load(SeqCst);
    // Read counters first: /proc allocation, verification and JSON are not API costs.
    // The prepared token, inputs and pre-call snapshots remain alive throughout.
    let process_peak_rss_kib = rss_kib();
    let unchanged = nodes == original_nodes
        && edges.len() == original_edges.len()
        && edges.iter().zip(&original_edges).all(|(edge, old)| {
            (
                edge.id,
                edge.source,
                edge.target,
                edge.source_side,
                edge.target_side,
            ) == *old
        });
    let canonical_valid = outcome.as_ref().ok().is_some_and(|token| {
        let expected_work = count * 27
            + count * count * 1024
            + count * (count - 1) / 2
            + edges.len() * 27
            + edges.len() * edges.len() * 1024
            + 2 * count * edges.len();
        token.rectangles().len() == count
            && token.terminals().len() == edges.len()
            && limit_values(token.limits()) == limit_values(limits)
            && token.charged_work() == expected_work
            && token.rectangles().enumerate().all(|(i, (id, bounds))| {
                let node = &nodes[i];
                id == node.id
                    && id.as_ptr() == node.id.as_ptr()
                    && bounds
                        == [
                            node.x as i64 * 2,
                            (node.x + node.width) as i64 * 2,
                            node.y as i64 * 2,
                            (node.y + node.height) as i64 * 2,
                        ]
            })
            && token.terminals().iter().zip(&edges).enumerate().all(
                |(index, ((id, source, target), edge))| {
                    let source_node = &nodes[index / density];
                    let target_node = &nodes[(index / density + [1, 7][index % density]) % count];
                    *id == edge.id
                        && id.as_ptr() == edge.id.as_ptr()
                        && *source == anchor(source_node, edge.source_side)
                        && *target == anchor(target_node, edge.target_side)
                },
            )
    });
    let expected_error = if cancel_before {
        Some(Error::Cancelled)
    } else if count > limits.max_nodes {
        Some(Error::Limit(Limit::Nodes))
    } else if count == 128 || (count == 64 && density == 2) {
        Some(Error::Limit(Limit::Work))
    } else {
        None
    };
    let zero_initial_requests =
        after.requests == before.requests && after.requested == before.requested;
    let expected = unchanged
        && match &outcome {
            Ok(_) => expected_error.is_none() && canonical_valid && polls.get() > 0,
            Err(error) => {
                Some(*error) == expected_error
                    && polls.get() > 0
                    && (!(cancel_before || count > limits.max_nodes)
                        || (zero_initial_requests && polls.get() == 1))
            }
        };
    let status = match &outcome {
        Ok(_) => "ADMITTED_CANONICAL_INTEGER_GEOMETRY",
        Err(Error::Cancelled) => "CANCELLED_BEFORE_ADMISSION",
        Err(Error::Limit(Limit::Work)) => "DEFAULT_WORK_REFUSAL",
        Err(Error::Limit(Limit::Nodes)) => "DEFAULT_NODE_REFUSAL",
        Err(_) => "UNEXPECTED_REFUSAL",
    };
    let report = serde_json::json!({
        "status": status, "expected_outcome": expected,
        "input_nodes": count, "edges_per_node": density, "input_edges": edges.len(),
        "input_id_bytes": ids[0].len(), "cancel_before": cancel_before, "callback_polls": polls.get(),
        "inputs_unchanged": unchanged, "canonical_token_valid": canonical_valid,
        "admitted_nodes": outcome.as_ref().ok().map(|token| token.rectangles().len()),
        "admitted_edges": outcome.as_ref().ok().map(|token| token.terminals().len()),
        "charged_work": outcome.as_ref().ok().map(|token| token.charged_work()),
        "error": outcome.as_ref().err().map(|error| format!("{error:?}")), "api_ns": api_ns,
        "allocations": delta(before, after, phase_peak),
        "live_delta_bytes": after.live as i128 - before.live as i128,
        "process_peak_rss_kib": process_peak_rss_kib,
        "limits": {"max_nodes": limits.max_nodes, "max_edges": limits.max_edges,
            "max_grid_vertices": limits.max_grid_vertices, "max_output_points": limits.max_output_points,
            "max_work": limits.max_work, "max_id_bytes": limits.max_id_bytes},
        "recipe": "Ascending 27-byte IDs; nonoverlapping 32-column integer placements with odd dimensions. Ring +1 edges use Right/Left; density2 adds +7 Bottom/Top edges. Fixtures and snapshots precede timing.",
        "scope": "One fresh process; actual syson_routing::admit only. No search, route correctness, B continuation, interior cancellation, native, UI, worker, deadline, p95 or successful-capacity claim. Work/Nodes refusal latency is not capacity; error outcomes expose no spent-work counter.",
        "accounting": "Exact published counting-System allocator block. Requested traffic includes failed allocation requests. Phase peak resets to initial live; signed live delta, absolute/incremental peak and process VmHWM differ. Counters precede /proc and all verification/JSON. Inputs/snapshots and successful owned prepared token remain live through observations and reporting; setup and observer/report allocations are excluded from API counters."
    });
    std::hint::black_box((&outcome, &nodes, &edges, &original_nodes, &original_edges));
    Ok(report)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count = args.get(1).and_then(|value| value.parse::<usize>().ok());
    let density = args.get(2).and_then(|value| value.parse::<usize>().ok());
    let flags = args.get(3..).unwrap_or(&[]);
    let cancel = flags.first().is_some_and(|flag| flag == "--cancel-before");
    if !(3..=4).contains(&args.len())
        || !count.is_some_and(|n| [32, 64, 128, 256, 1024].contains(&n))
        || !density.is_some_and(|n| [1, 2].contains(&n))
        || flags.iter().any(|flag| flag != "--cancel-before")
    {
        eprintln!(
            "usage: u11a-profile NODES(32|64|128|256|1024) EDGES_PER_NODE(1|2) [--cancel-before]"
        );
        println!("{}", serde_json::json!({"status": "INVALID_ARGUMENTS"}));
        std::process::exit(2);
    }
    match run(count.unwrap(), density.unwrap(), cancel) {
        Ok(report) => {
            let expected = report["expected_outcome"] == true;
            println!("{report}");
            if !expected {
                std::process::exit(1);
            }
        }
        Err(error) => {
            println!(
                "{}",
                serde_json::json!({"status": "SETUP_FAILURE", "error": error})
            );
            std::process::exit(1);
        }
    }
}

// SPDX-License-Identifier: EPL-2.0
//! Actual flat layout and separate continuation controls; no native/UI/capacity claim.
//! Adapted from published u10a_profile.rs SHA-256
//! 022cba2e1bf55c22a55e4ca85820793c73e191797c155ad02c2a039bf75b0cdd.
//! Allocator/counters/VmHWM body remains byte-identical to that source.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};
use std::time::Instant;
use syson_layout::{
    DirectedEdge, Layout as GraphLayout, LayoutError, Limit, Limits, MeasuredNode, Options, admit,
    layout, place,
};

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Layout,
    CancelBefore,
    Continuation,
    CancelBetween,
    BudgetContinuation,
}

fn geometry(
    result: &GraphLayout<'_>,
    nodes: &[MeasuredNode<'_>],
    edges: &[DirectedEdge<'_>],
    limits: Limits,
) -> bool {
    if result.nodes.len() != nodes.len()
        || result.edges.len() != edges.len()
        || !result.width.is_finite()
        || !result.height.is_finite()
        || result.charged_work == 0
        || result.charged_work > limits.max_work
        || result.estimated_workspace_bytes != nodes.len() * 1024 + edges.len() * 640
    {
        return false;
    }
    let rectangles = result.nodes.iter().zip(nodes).all(|(placed, input)| {
        placed.id == input.id
            && std::ptr::eq(placed.id, input.id)
            && placed.width == input.width
            && placed.height == input.height
            && placed.x == 16.0
            && placed.y.is_finite()
            && placed.y >= 16.0
            && placed.x + placed.width <= result.width
            && placed.y + placed.height <= result.height
            && placed.strong_component == 0
            && placed.weak_component == 0
            && placed.rank == 0
    });
    let classified = result.edges.iter().zip(edges).all(|(placed, input)| {
        placed.edge == *input
            && placed.internal_strong_component
            && std::ptr::eq(placed.edge.id, input.id)
            && std::ptr::eq(placed.edge.source, input.source)
            && std::ptr::eq(placed.edge.target, input.target)
    });
    rectangles
        && classified
        && result.nodes.first().is_some_and(|node| node.y == 16.0)
        && result
            .nodes
            .windows(2)
            .all(|pair| pair[1].y == pair[0].y + pair[0].height + 20.0)
        && result.width == nodes.iter().map(|node| node.width).fold(0.0, f64::max) + 32.0
        && result.height
            == nodes.iter().map(|node| node.height).sum::<f64>()
                + (nodes.len() - 1) as f64 * 20.0
                + 32.0
}

fn run(count: usize, density: usize, mode: Mode) -> Result<serde_json::Value, String> {
    let defaults = Limits::default();
    if defaults.max_nodes != 4096
        || defaults.max_edges != 16384
        || defaults.max_work != 100_000_000
        || defaults.max_workspace_bytes != 16 * 1024 * 1024
        || defaults.max_id_bytes != 1024
    {
        return Err("default limits changed; review workload before execution".into());
    }
    let mut limits = defaults;
    let ids: Vec<_> = (0..count)
        .map(|i| format!("node/{i:08}-fixed-fixture"))
        .collect();
    let edge_ids: Vec<_> = (0..count * density)
        .map(|i| format!("edge/{i:08}-fixed-fixture"))
        .collect();
    let nodes: Vec<_> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| MeasuredNode {
            id,
            width: 40.25 + (i % 7) as f64 * 0.5,
            height: 20.5 + (i % 5) as f64 * 0.25,
            parent: None,
            port_count: 0,
            pinned: false,
        })
        .collect();
    let edges: Vec<_> = edge_ids
        .iter()
        .enumerate()
        .map(|(i, id)| DirectedEdge {
            id,
            source: &ids[i / density],
            target: &ids[(i / density + [1, 7][i % density]) % count],
        })
        .collect();
    let original_nodes = nodes.clone();
    let original_edges = edges.clone();
    let direct = matches!(
        mode,
        Mode::Continuation | Mode::CancelBetween | Mode::BudgetContinuation
    );
    let cancelled = Cell::new(mode == Mode::CancelBefore);
    if mode == Mode::BudgetContinuation {
        let probe = admit(&nodes, &edges, Options::default(), defaults, || false)
            .map_err(|error| format!("budget control admission: {error:?}"))?;
        limits.max_work = probe.charged_work();
    }
    let prepared = if direct {
        Some(
            admit(&nodes, &edges, Options::default(), limits, || {
                cancelled.get()
            })
            .map_err(|error| format!("direct control admission: {error:?}"))?,
        )
    } else {
        None
    };
    let admission_work = prepared.as_ref().map(|graph| graph.charged_work());
    if mode == Mode::CancelBetween {
        cancelled.set(true);
    }
    let before = counters();
    PHASE_PEAK.store(before.live, SeqCst);
    let started = Instant::now();
    let outcome = match prepared {
        Some(graph) => place(graph),
        None => layout(&nodes, &edges, Options::default(), limits, || {
            cancelled.get()
        }),
    };
    let api_ns = started.elapsed().as_nanos();
    let after = counters();
    let phase_peak = PHASE_PEAK.load(SeqCst);
    // Snapshot counters before /proc allocation; HWM includes fixture/setup activity.
    // Verification, reference API calls and JSON below are outside observations.
    let process_peak_rss_kib = rss_kib();
    let unchanged = nodes == original_nodes && edges == original_edges;
    let geometry_valid = outcome
        .as_ref()
        .ok()
        .is_some_and(|value| geometry(value, &nodes, &edges, limits));
    let reference = if matches!(mode, Mode::Continuation | Mode::BudgetContinuation) {
        Some(
            layout(&nodes, &edges, Options::default(), defaults, || false)
                .map_err(|error| format!("post-snapshot reference layout: {error:?}"))?,
        )
    } else {
        None
    };
    let reference_work = reference.as_ref().map(|value| value.charged_work);
    let placement_work = reference_work
        .zip(admission_work)
        .and_then(|(total, spent)| total.checked_sub(spent));
    let continuation_valid = reference.as_ref().is_some_and(|value| {
        geometry(value, &nodes, &edges, defaults)
            && placement_work.is_some_and(|increment| increment > 0)
            && if mode == Mode::Continuation {
                outcome.as_ref().ok() == Some(value)
            } else {
                admission_work == Some(limits.max_work)
                    && placement_work.is_some_and(|increment| increment <= limits.max_work)
            }
    });
    let zero_requests = after.requests == before.requests && after.requested == before.requested;
    let (status, expected) = match outcome.as_ref() {
        Ok(_) if mode == Mode::Layout && unchanged && geometry_valid => ("LAYOUT_COMPUTED", true),
        Ok(_)
            if mode == Mode::Continuation && unchanged && geometry_valid && continuation_valid =>
        {
            ("EXPECTED_CONTINUED_LAYOUT_MATCHES_WRAPPER", true)
        }
        Err(LayoutError::Cancelled) if mode == Mode::CancelBefore && unchanged && zero_requests => {
            ("EXPECTED_CANCELLED_BEFORE_ADMISSION", true)
        }
        Err(LayoutError::Cancelled)
            if mode == Mode::CancelBetween && unchanged && zero_requests =>
        {
            ("EXPECTED_CANCELLED_BEFORE_PLACEMENT_ALLOCATION", true)
        }
        Err(LayoutError::Limit(Limit::Work))
            if mode == Mode::BudgetContinuation && unchanged && continuation_valid =>
        {
            ("EXPECTED_CONTINUED_WORK_REFUSAL", true)
        }
        Err(LayoutError::Limit(Limit::Work))
            if mode == Mode::Layout && count == 1024 && unchanged =>
        {
            ("EXPECTED_DEFAULT_WORK_REFUSAL", true)
        }
        _ => ("UNEXPECTED_OUTCOME", false),
    };
    let report = serde_json::json!({
        "status": status, "expected_outcome": expected, "mode": format!("{mode:?}"),
        "timed_api": if direct { "place_after_admission" } else { "layout" },
        "input_nodes": count, "edges_per_node": density, "input_edges": edges.len(),
        "input_id_bytes": ids[0].len(), "inputs_unchanged": unchanged,
        "geometry_and_borrowed_ids_valid": geometry_valid, "continuation_valid": continuation_valid,
        "output_nodes": outcome.as_ref().ok().map(|value| value.nodes.len()),
        "output_edges": outcome.as_ref().ok().map(|value| value.edges.len()),
        "charged_work": outcome.as_ref().ok().map(|value| value.charged_work),
        "admission_charged_work": admission_work, "reference_wrapper_work": reference_work,
        "placement_work_increment": placement_work,
        "width": outcome.as_ref().ok().map(|value| value.width),
        "height": outcome.as_ref().ok().map(|value| value.height),
        "logical_workspace_bytes": outcome.as_ref().ok().map(|value| value.estimated_workspace_bytes),
        "api_ns": api_ns, "error": outcome.as_ref().err().map(|error| format!("{error:?}")),
        "allocations": delta(before, after, phase_peak),
        "live_increment_bytes": after.live as i128 - before.live as i128,
        "process_peak_rss_kib": process_peak_rss_kib,
        "limits": {"max_nodes": limits.max_nodes, "max_edges": limits.max_edges,
            "max_work": limits.max_work, "max_workspace_bytes": limits.max_workspace_bytes,
            "max_id_bytes": limits.max_id_bytes},
        "default_limits_unchanged": mode != Mode::BudgetContinuation,
        "recipe": "Ascending 27-byte IDs; fractional rectangles; ring +1, and +7 at density2. Each graph is one strong/weak component, rank0. Fixtures and snapshots precede timing.",
        "scope": "One fresh process. Default API layout; separate 64-node/density1 direct-place controls. Actual default Work refusal is refusal latency. No ELK/native/UI/worker, all-shape geometry quality, p95 or supported-capacity claim.",
        "control_scope": "Continuation compares actual wrapper after snapshots. Budget control tightens Work to actual admission spent; requires a positive reference placement increment fitting that quota, yet actual continued place refuses. This checks carry-over; no runtime proof of absence of sorting/copies. Between-phase cancellation is explicit, not interior cancellation.",
        "accounting": "Published SeqCst System allocator unchanged. Requested traffic includes failed requests. Phase peak resets to initial live; signed live increment/absolute peak differ from process VmHWM. Counters precede /proc; HWM includes setup but precedes verification/reference calls/JSON. Inputs and any successful output remain alive through observations and report construction. Direct controls include prepared graph storage in initial live; cancellation may free it without allocating."
    });
    std::hint::black_box((
        &outcome,
        &nodes,
        &edges,
        &original_nodes,
        &original_edges,
        &reference,
    ));
    Ok(report)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count = args.get(1).and_then(|value| value.parse::<usize>().ok());
    let density = args.get(2).and_then(|value| value.parse::<usize>().ok());
    let mode = match args.get(3).map(String::as_str) {
        None => Some(Mode::Layout),
        Some("--cancel-before") => Some(Mode::CancelBefore),
        Some("--continuation") => Some(Mode::Continuation),
        Some("--cancel-between") => Some(Mode::CancelBetween),
        Some("--budget-continuation") => Some(Mode::BudgetContinuation),
        _ => None,
    };
    if !(3..=4).contains(&args.len())
        || mode.is_none()
        || !count.is_some_and(|n| [64, 128, 256, 1024].contains(&n))
        || !density.is_some_and(|n| [1, 2].contains(&n))
        || matches!(
            mode,
            Some(Mode::Continuation | Mode::CancelBetween | Mode::BudgetContinuation)
        ) && (count != Some(64) || density != Some(1))
    {
        eprintln!(
            "usage: u10b-profile NODES(64|128|256|1024) EDGES_PER_NODE(1|2) [--cancel-before|--continuation|--cancel-between|--budget-continuation]; direct controls require 64/1"
        );
        println!("{}", serde_json::json!({"status": "INVALID_ARGUMENTS"}));
        std::process::exit(2);
    }
    match run(count.unwrap(), density.unwrap(), mode.unwrap()) {
        Ok(value) => {
            let expected = value["expected_outcome"] == true;
            println!("{value}");
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

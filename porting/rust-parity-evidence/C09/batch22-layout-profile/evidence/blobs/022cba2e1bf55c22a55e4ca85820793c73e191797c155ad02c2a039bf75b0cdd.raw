//! Actual flat-graph admission API only; no placement/native/UI/worker/capacity claim.
//! Allocator/counters/VmHWM body is unchanged from p01_profile.rs,
//! SHA-256 56515398ae0cb8be6f9c66a2fdfb843c8a8e21d98033201e27e75cccb3722a05.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};
use std::time::Instant;
use syson_layout::{DirectedEdge, LayoutError, Limit, Limits, MeasuredNode, Options, admit};

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
fn run(count: usize, density: usize, cancel_before: bool) -> Result<serde_json::Value, String> {
    let limits = Limits::default();
    if limits.max_nodes != 4096
        || limits.max_edges != 16384
        || limits.max_work != 100_000_000
        || limits.max_workspace_bytes != 16 * 1024 * 1024
        || limits.max_id_bytes != 1024
    {
        return Err("default limits changed; review the workload before execution".into());
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
    let before = counters();
    PHASE_PEAK.store(before.live, SeqCst);
    let started = Instant::now();
    let outcome = admit(&nodes, &edges, Options::default(), limits, || cancel_before);
    let api_ns = started.elapsed().as_nanos();
    let after = counters();
    let phase_peak = PHASE_PEAK.load(SeqCst);
    // Counter reads precede /proc allocation; HWM includes fixture/setup activity.
    // Verification and JSON formatting below are outside these API observations.
    let process_peak_rss_kib = rss_kib();
    let unchanged = nodes == original_nodes && edges == original_edges;
    let graph_valid = outcome.as_ref().ok().is_some_and(|graph| {
        graph.nodes().len() == count
            && graph.edges().len() == count * density
            && std::ptr::eq(graph.nodes(), nodes.as_slice())
            && std::ptr::eq(graph.edges(), edges.as_slice())
            && graph.node_order().iter().copied().eq(0..count)
            && graph.maximum_id_bytes() == ids[0].len()
            && graph.charged_work() > 0
            && graph.charged_work() <= limits.max_work
            && graph.estimated_workspace_bytes() == count * 1024 + count * density * 640
            && graph.indexed_edges().len() == edges.len()
            && graph
                .indexed_edges()
                .iter()
                .enumerate()
                .all(|(i, &(s, t, original))| {
                    original == i
                        && s == i / density
                        && t == (s + [1, 7][i % density]) % count
                        && nodes[s].id == edges[original].source
                        && nodes[t].id == edges[original].target
                })
            && graph.forward().len() == count
            && graph.reverse().len() == count
            && (0..count).all(|i| {
                graph.forward()[i].iter().copied().eq([1, 7]
                    .into_iter()
                    .take(density)
                    .map(|step| (i + step) % count))
                    && graph.reverse()[i].iter().copied().eq(edges
                        .iter()
                        .enumerate()
                        .filter(|(_, edge)| edge.target == nodes[i].id)
                        .map(|(index, _)| index / density))
            })
    });
    let zero_cancel_requests =
        after.requests == before.requests && after.requested == before.requested;
    let (status, expected) = match outcome.as_ref() {
        Ok(_) if !cancel_before && unchanged && graph_valid => {
            ("ADMITTED_CANONICAL_FLAT_GRAPH", true)
        }
        Err(LayoutError::Cancelled) if cancel_before && unchanged && zero_cancel_requests => {
            ("EXPECTED_CANCELLED_BEFORE_ADMISSION", true)
        }
        Err(LayoutError::Limit(Limit::Work)) if !cancel_before && count == 1024 && unchanged => {
            ("EXPECTED_DEFAULT_WORK_REFUSAL", true)
        }
        _ => ("UNEXPECTED_OUTCOME", false),
    };
    let report = serde_json::json!({
        "status": status, "expected_outcome": expected,
        "input_nodes": count, "edges_per_node": density, "input_edges": edges.len(),
        "input_id_bytes": ids[0].len(), "cancel_before": cancel_before,
        "inputs_unchanged": unchanged, "canonical_borrowed_graph_valid": graph_valid,
        "admitted_nodes": outcome.as_ref().ok().map(|graph| graph.nodes().len()),
        "admitted_edges": outcome.as_ref().ok().map(|graph| graph.edges().len()),
        "charged_work": outcome.as_ref().ok().map(|graph| graph.charged_work()),
        "maximum_id_bytes": outcome.as_ref().ok().map(|graph| graph.maximum_id_bytes()),
        "logical_workspace_bytes": outcome.as_ref().ok().map(|graph| graph.estimated_workspace_bytes()),
        "api_ns": api_ns, "error": outcome.as_ref().err().map(|error| format!("{error:?}")),
        "allocations": delta(before, after, phase_peak),
        "process_peak_rss_kib": process_peak_rss_kib,
        "limits": {"max_nodes": limits.max_nodes, "max_edges": limits.max_edges,
            "max_work": limits.max_work, "max_workspace_bytes": limits.max_workspace_bytes,
            "max_id_bytes": limits.max_id_bytes},
        "recipe": "Ascending node/edge IDs with eight-digit index and -fixed-fixture suffix; fractional rectangles; outgoing ring +1, and +7 when density=2. All fixtures/snapshots precede timing.",
        "scope": "One fresh process; actual syson_layout::admit API only. Count ceilings are proposed admission guards. No placement, native/UI/worker, interior-cancellation, p95 or supported-capacity claim.",
        "accounting": "Unchanged SeqCst counting System allocator. Requested traffic includes failed requests. Phase peak resets to initial live; live increment and absolute peak differ from process VmHWM. Counters precede /proc read; VmHWM includes setup/earlier activity but precedes subsequent verification/JSON. Input vectors and any admitted graph remain alive through observations and report construction."
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
        || !count.is_some_and(|n| [64, 128, 256, 1024].contains(&n))
        || !density.is_some_and(|n| [1, 2].contains(&n))
        || flags.iter().any(|flag| flag != "--cancel-before")
    {
        eprintln!(
            "usage: u10a-profile NODES(64|128|256|1024) EDGES_PER_NODE(1|2) [--cancel-before]"
        );
        println!("{}", serde_json::json!({"status": "INVALID_ARGUMENTS"}));
        std::process::exit(2);
    }
    match run(count.unwrap(), density.unwrap(), cancel) {
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

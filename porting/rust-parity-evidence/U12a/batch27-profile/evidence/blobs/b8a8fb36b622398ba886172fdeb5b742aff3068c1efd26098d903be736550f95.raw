// SPDX-License-Identifier: EPL-2.0
//! Forest admission only; placement/native/UI/continuation/capacity remain OPEN.
//! Exact published counting-System allocator block SHA-256 1f927e4d0a780666081a13b4bdc4d2f3ffb94f220ac9bf39cdea8162734e3eb3.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};
use std::time::Instant;
use syson_layout::containers::{Error, PackageNode, Position, Size, admit_packages};
use syson_layout::{DirectedEdge, Direction, LayoutError, Limit, Limits, Options};

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

fn limits_key(limits: Limits) -> [usize; 5] {
    [
        limits.max_nodes,
        limits.max_edges,
        limits.max_work,
        limits.max_workspace_bytes,
        limits.max_id_bytes,
    ]
}
fn node_key<'a>(node: &PackageNode<'a>) -> (&'a str, Option<&'a str>, [Option<u64>; 10]) {
    (
        node.id,
        node.parent,
        [
            Some(node.leaf_size.width.to_bits()),
            Some(node.leaf_size.height.to_bits()),
            Some(node.header_height_footprint.to_bits()),
            Some(node.border_width.to_bits()),
            node.default_width.map(f64::to_bits),
            node.default_height.map(f64::to_bits),
            node.user_minimum.map(|s| s.width.to_bits()),
            node.user_minimum.map(|s| s.height.to_bits()),
            node.retained_position.map(|p| p.x.to_bits()),
            node.retained_position.map(|p| p.y.to_bits()),
        ],
    )
}
fn options_key(options: Options) -> (Direction, [u64; 4]) {
    (
        options.direction,
        [
            options.layer_gap.to_bits(),
            options.node_gap.to_bits(),
            options.component_gap.to_bits(),
            options.padding.to_bits(),
        ],
    )
}

fn run(count: usize, star: bool, cancel_before: bool) -> Result<serde_json::Value, String> {
    let limits = Limits::default();
    let options = Options::default();
    if limits_key(limits) != [4096, 16384, 100_000_000, 16 * 1024 * 1024, 1024] {
        return Err("default limits changed; review before execution".into());
    }
    let ids: Vec<_> = (0..count)
        .map(|i| format!("node/{i:08}-fixed-fixture"))
        .collect();
    let edge_count = count - usize::from(star);
    let edge_ids: Vec<_> = (0..edge_count)
        .map(|i| format!("edge/{i:08}-fixed-fixture"))
        .collect();
    let nodes: Vec<_> = ids
        .iter()
        .enumerate()
        .rev()
        .map(|(i, id)| PackageNode {
            id,
            parent: (star && i != 0).then_some(ids[0].as_str()),
            leaf_size: Size {
                width: 97.125,
                height: 41.75,
            },
            header_height_footprint: 13.625,
            border_width: 1.25,
            default_width: Some(120.5),
            default_height: Some(80.25),
            user_minimum: Some(Size {
                width: 140.875,
                height: 95.375,
            }),
            retained_position: Some(Position {
                x: -0.125 * i as f64,
                y: 0.375 * i as f64,
            }),
        })
        .collect();
    let edges: Vec<_> = edge_ids
        .iter()
        .enumerate()
        .rev()
        .map(|(i, id)| DirectedEdge {
            id,
            source: &ids[i + usize::from(star)],
            target: &ids[(i + 1) % edge_count + usize::from(star)],
        })
        .collect();
    let original_nodes: Vec<_> = nodes.iter().map(node_key).collect();
    let original_edges = edges.clone();
    let polls = Cell::new(0usize);
    let before = counters();
    PHASE_PEAK.store(before.live, SeqCst);
    let started = Instant::now();
    let outcome = admit_packages(&nodes, &edges, options, limits, || {
        polls.set(polls.get() + 1);
        cancel_before
    });
    let api_ns = started.elapsed().as_nanos();
    let after = counters();
    let phase_peak = PHASE_PEAK.load(SeqCst);
    // Counters precede /proc, verification and JSON; retained token stays live.
    let process_peak_rss_kib = rss_kib();
    let unchanged = nodes
        .iter()
        .map(node_key)
        .eq(original_nodes.iter().copied())
        && edges == original_edges;
    let expected_work = 32 * count
        + 8 * edge_count
        + 27 * count
        + 2 * count * count * 1024
        + usize::from(star) * (count - 1)
        + 2 * count * edge_count * 1024
        + edge_count * edge_count * 1024
        + count * count;
    let logical_bytes = 4096 * count + 1024 * edge_count;
    let canonical_valid =
        outcome.as_ref().ok().is_some_and(|token| {
            token.nodes().len() == count
                && token.parent_indices().len() == count
                && token.bottom_up().len() == count
                && token.sibling_edges().len() == edge_count
                && limits_key(token.limits()) == limits_key(limits)
                && options_key(token.options()) == options_key(options)
                && token.charged_work() == expected_work
                && token.estimated_workspace_bytes() == logical_bytes
                && token.nodes().iter().enumerate().all(|(i, node)| {
                    let source = &nodes[count - 1 - i];
                    node_key(node) == node_key(source)
                        && node.id.as_ptr() == source.id.as_ptr()
                        && node.parent.map(str::as_ptr) == source.parent.map(str::as_ptr)
                        && token.parent_indices()[i] == (star && i != 0).then_some(0)
                        && token.bottom_up()[i] == if star { (i + 1) % count } else { i }
                })
                && token.sibling_edges().iter().zip(edges.iter().rev()).all(
                    |((edge, parent), old)| {
                        *edge == *old
                            && edge.id.as_ptr() == old.id.as_ptr()
                            && edge.source.as_ptr() == old.source.as_ptr()
                            && edge.target.as_ptr() == old.target.as_ptr()
                            && *parent == star.then_some(0)
                    },
                )
        });
    let expected_error = if cancel_before {
        Some(Error::Layout(LayoutError::Cancelled))
    } else if count >= 256 {
        Some(Error::Layout(LayoutError::Limit(Limit::Work)))
    } else {
        None
    };
    let expected = unchanged
        && match &outcome {
            Ok(_) => expected_error.is_none() && canonical_valid && polls.get() > 0,
            Err(error) => {
                Some(*error) == expected_error
                    && polls.get() > 0
                    && (!cancel_before
                        || (polls.get() == 1
                            && after.requests == before.requests
                            && after.requested == before.requested))
            }
        };
    let status = match &outcome {
        Ok(_) => "ADMITTED_CANONICAL_FOREST",
        Err(Error::Layout(LayoutError::Cancelled)) => "CANCELLED_BEFORE_ADMISSION",
        Err(Error::Layout(LayoutError::Limit(Limit::Work))) => "DEFAULT_WORK_REFUSAL",
        Err(_) => "UNEXPECTED_REFUSAL",
    };
    let report = serde_json::json!({
        "status": status, "expected_outcome": expected,
        "input_nodes": count, "topology": if star { "one_root_star" } else { "flat" },
        "input_edges": edge_count, "input_id_bytes": ids[0].len(),
        "cancel_before": cancel_before, "callback_polls": polls.get(),
        "inputs_unchanged": unchanged, "canonical_token_valid": canonical_valid,
        "admitted_nodes": outcome.as_ref().ok().map(|token| token.nodes().len()),
        "admitted_edges": outcome.as_ref().ok().map(|token| token.sibling_edges().len()),
        "charged_work": outcome.as_ref().ok().map(|token| token.charged_work()),
        "logical_workspace_bytes": outcome.as_ref().ok().map(|token| token.estimated_workspace_bytes()),
        "complete_success_charge_formula": expected_work, "logical_input_allowance": logical_bytes,
        "error": outcome.as_ref().err().map(|error| format!("{error:?}")), "api_ns": api_ns,
        "allocations": delta(before, after, phase_peak),
        "live_delta_bytes": after.live as i128 - before.live as i128,
        "process_peak_rss_kib": process_peak_rss_kib, "limits": limits_key(limits),
        "recipe": "Reverse input order; 27-byte borrowed IDs; flat ring or one root with sibling-leaf ring. Fractional measured leaf/header/border/default/user-minimum/retained geometry. All fixtures and immutable bit snapshots precede timing.",
        "scope": "One fresh process; actual public admit_packages only. No placement, B continuation, interior cancellation, native, UI, worker, p95 or successful-capacity claim. Work refusal latency is not capacity; errors expose no spent-work counter.",
        "accounting": "Exact published counting-System allocator block. Requested traffic includes failed allocation requests. Signed live delta, absolute/incremental phase peak and process VmHWM differ. Counters precede /proc and verification/JSON; inputs, snapshots and owned token remain live through reporting. Logical allowance is not measured heap/RSS."
    });
    std::hint::black_box((&outcome, &nodes, &edges, &original_nodes, &original_edges));
    Ok(report)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count = args.get(1).and_then(|value| value.parse::<usize>().ok());
    let topology = args.get(2).map(String::as_str);
    let flags = args.get(3..).unwrap_or(&[]);
    let cancel = flags.first().is_some_and(|flag| flag == "--cancel-before");
    if !(3..=4).contains(&args.len())
        || !count.is_some_and(|n| [32, 64, 128, 256, 1024].contains(&n))
        || !topology.is_some_and(|value| ["flat", "star"].contains(&value))
        || flags.iter().any(|flag| flag != "--cancel-before")
    {
        eprintln!(
            "usage: u12a-profile NODES(32|64|128|256|1024) TOPOLOGY(flat|star) [--cancel-before]"
        );
        println!("{}", serde_json::json!({"status": "INVALID_ARGUMENTS"}));
        std::process::exit(2);
    }
    match run(count.unwrap(), topology == Some("star"), cancel) {
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

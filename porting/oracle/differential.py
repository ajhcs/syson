#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Run every corpus case through the reference SysON and the Rust importer, then diff.

  differential.py [--record | --golden] [--golden-dir DIR] [--rust-bin PATH]
                  [--only ID_SUBSTRING] [--out DIR]

Live and record modes need a running oracle (oracle.py up). Golden replay
only needs Python and the Rust example binary, built with:
  cargo build --offline --locked --example oracle_import \
      --manifest-path rust/crates/syson-text-import/Cargo.toml
Writes one result file per case plus summary.json and SUMMARY.md to --out
(default: <oracle run dir>/differential-<UTC time>).
"""
import argparse
import contextlib
import hashlib
import io
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import compare  # noqa: E402
import golden  # noqa: E402
import reference  # noqa: E402

REPO = HERE.parents[1]
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", "/workspace/build/builderbob/syson-oracle/target"))
EXAMPLE = "oracle_import.exe" if os.name == "nt" else "oracle_import"
RUST_BIN = TARGET / "debug/examples" / EXAMPLE
GOLDEN_TARGET = Path(os.environ.get("CARGO_TARGET_DIR", REPO / "rust/crates/syson-text-import/target"))
GOLDEN_RUST_BIN = GOLDEN_TARGET / "debug/examples" / EXAMPLE


def cases():
    corpus = json.loads((HERE / "corpus.json").read_text(encoding="utf-8"))
    for c in corpus["inline"]:
        yield c["id"], c["source"], "inline"
    for p in corpus["paths"]:
        rel = Path(p)
        yield f"{rel.parent.name}-{rel.stem}", (REPO / rel).read_text(encoding="utf-8"), p


def run_rust(source, rust_bin=RUST_BIN):
    try:
        process = subprocess.run([str(rust_bin)], input=source.encode("utf-8"),
                                 capture_output=True, timeout=60)
        if process.returncode != 0:
            return {"status": "crash", "error": process.stderr[-600:].decode("utf-8", errors="replace")}
        result = json.loads(process.stdout.decode("utf-8"))
        if isinstance(result, dict):
            if result.get("status") == "ok" and isinstance(result.get("document"), dict):
                return result
            if result.get("status") == "error" and isinstance(result.get("error"), str):
                return result
        return {"status": "crash", "error": "invalid Rust importer result envelope"}
    except (OSError, subprocess.TimeoutExpired, ValueError) as error:
        return {"status": "crash", "error": f"{type(error).__name__}: {error}"[:600]}


def run_reference(base, case_id, source):
    try:
        out = reference.import_text(base, case_id, source)
        docs = list(out["documents"].values())
        if len(docs) != 1:
            raise reference.ReferenceProtocolError(f"expected 1 document, got {len(docs)}")
        content = docs[0].get("content")
        if not isinstance(content, list):
            raise reference.ReferenceProtocolError("downloaded document lacks a content list")
        return {"status": "ok" if content else "empty", "report": out["report"], "document": docs[0]}
    except reference.ReferenceError as error:
        return {"status": "error", "error": str(error)[:600]}
    except Exception as error:
        return {"status": "harness_error", "error": f"{type(error).__name__}: {error}"[:600]}


def classify(ref, rust, cmp):
    if ref["status"] == "harness_error" or rust["status"] == "crash":
        return "HARNESS_ERROR"
    if ref["status"] != "ok" and rust["status"] != "ok":
        return "BOTH_REFUSE"
    if ref["status"] != "ok":
        return "REFERENCE_ONLY_REFUSES"
    if rust["status"] != "ok":
        return "RUST_REFUSES"
    return "MATCH" if cmp["structural_match"] else "DIFFER"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--only")
    ap.add_argument("--out", type=Path)
    mode = ap.add_mutually_exclusive_group()
    mode.add_argument("--record", action="store_true")
    mode.add_argument("--golden", action="store_true")
    ap.add_argument("--golden-dir", type=Path, default=HERE / "golden")
    ap.add_argument("--rust-bin", type=Path)
    ap.add_argument("--restart-every", type=int, default=15,
                    help="restart the reference app after this many cases (0 = never); "
                         "one JVM accumulates memory across many imported projects")
    args = ap.parse_args()
    rust_bin = (args.rust_bin or (GOLDEN_RUST_BIN if args.golden else RUST_BIN)).resolve()
    selected = [(case_id, source, origin) for case_id, source, origin in cases()
                if not args.only or args.only in case_id]
    if not selected:
        ap.error("no corpus cases match")
    try:
        names = {golden.case_name(case_id) for case_id, _, _ in selected}
        if len(names) != len(selected):
            ap.error("duplicate corpus case IDs")
        if args.record and (args.only or args.golden_dir.exists()):
            ap.error("--record requires the full corpus and a fresh golden directory")
        replay = ({case_id: golden.load(args.golden_dir, case_id, source)
                   for case_id, source, _ in selected} if args.golden else {})
    except golden.GoldenError as error:
        ap.error(str(error))
    if not rust_bin.is_file():
        ap.error(f"missing {rust_bin}; build the oracle_import example first")
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    if args.golden:
        state = golden.REFERENCE
        default_out = rust_bin.parent / f"differential-golden-{stamp}"
    else:
        import oracle

        if not oracle.STATE.exists():
            ap.error("no running oracle: run oracle.py up first")
        state = json.loads(oracle.STATE.read_text(encoding="utf-8"))
        if args.record and any(state.get(key) != pin for key, pin in golden.REFERENCE.items()):
            ap.error("live reference does not match the pinned release")
        default_out = Path(state["run"]) / f"differential-{stamp}"
    out_dir = args.out or default_out
    (out_dir / "cases").mkdir(parents=True, exist_ok=True)
    results = []
    captures = []
    since_restart = 0
    for case_id, source, origin in selected:
        if not args.golden and args.restart_every and since_restart >= args.restart_every:
            with contextlib.redirect_stdout(io.StringIO()):
                oracle.down()
                oracle.up()
            state = json.loads(oracle.STATE.read_text(encoding="utf-8"))
            since_restart = 0
        since_restart += 1
        t0 = time.monotonic()
        ref = replay[case_id] if args.golden else run_reference(state["url"], case_id, source)
        rust = run_rust(source, rust_bin)
        if args.record and ref["status"] != "harness_error":
            try:
                captures.append(golden.capture(case_id, source, state, ref))
            except golden.GoldenError as error:
                ap.error(f"{case_id}: {error}")
        cmp = (compare.compare(ref["document"], rust["document"])
               if ref["status"] == "ok" and rust["status"] == "ok" else None)
        verdict = classify(ref, rust, cmp or {"structural_match": False})
        row = {"id": case_id, "origin": origin, "verdict": verdict,
               "source_sha256": hashlib.sha256(source.encode()).hexdigest(),
               "reference_status": ref["status"], "reference_report": ref.get("report"),
               "reference_error": ref.get("error"), "rust_status": rust["status"],
               "rust_error": rust.get("error"), "comparison": cmp,
               "seconds": round(time.monotonic() - t0, 2)}
        (out_dir / "cases" / f"{case_id}.json").write_text(json.dumps(
            dict(row, source=source, reference_document=ref.get("document"),
                 rust_document=rust.get("document")), indent=1) + "\n")
        results.append(row)
        print(f"{verdict:24} {case_id}", flush=True)
    counts = {}
    for r in results:
        counts[r["verdict"]] = counts.get(r["verdict"], 0) + 1
    wire = sorted({w for r in results if r["comparison"] for w in r["comparison"]["wire_diffs"]})
    rust_head, rust_dirty = rust_revision()
    summary = {"generated_utc": stamp,
               "reference_mode": "golden" if args.golden else "live",
               "oracle": ({k: state[k] for k in golden.REFERENCE} if args.golden else
                          {k: state[k] for k in ("url", "run", *golden.REFERENCE)}),
               "rust_head": rust_head, "rust_dirty": rust_dirty,
               "counts": counts, "wire_findings": wire, "cases": results}
    (out_dir / "summary.json").write_text(json.dumps(summary, indent=1) + "\n")
    (out_dir / "SUMMARY.md").write_text(render(summary))
    if (args.record or args.golden) and counts.get("HARNESS_ERROR"):
        ap.error(f"{counts['HARNESS_ERROR']} harness errors; see {out_dir / 'SUMMARY.md'}")
    if args.record:
        try:
            golden.write(args.golden_dir, captures)
        except (OSError, golden.GoldenError) as error:
            ap.error(f"could not write golden records: {error}")
    print(json.dumps({"counts": counts, "wire_findings": wire, "out": str(out_dir)}, indent=1))


def rust_revision():
    try:
        head = subprocess.check_output(["git", "-C", str(REPO), "rev-parse", "HEAD"],
                                       text=True, stderr=subprocess.DEVNULL).strip()
        dirty = subprocess.check_output(["git", "-C", str(REPO), "status", "--porcelain", "rust"],
                                        text=True, stderr=subprocess.DEVNULL).strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown", None
    return head, bool(dirty)


def render(s):
    lines = [f"# Differential run {s['generated_utc']}", "",
             f"Reference: SysON release jar `{s['oracle']['jar_sha256'][:12]}` (source {s['oracle']['jar_source_revision'][:9]}). "
             f"Rust: `{s['rust_head'][:9]}`{' + uncommitted changes' if s['rust_dirty'] else ''}.", "",
             "| Verdict | Count |", "|---|---|"]
    lines += [f"| {k} | {v} |" for k, v in sorted(s["counts"].items())]
    lines += ["", "Wire-format findings (all cases):", ""] + [f"- {w}" for w in s["wire_findings"] or ["none"]]
    lines += ["", "| Case | Verdict | Reference | Rust | First structural difference |", "|---|---|---|---|---|"]
    for r in s["cases"]:
        first = ""
        if r["comparison"] and r["comparison"]["structural_diffs"]:
            d = r["comparison"]["structural_diffs"][0]
            first = f"`{d['path']}`: {d['reference']} vs {d['rust']}"
        elif r["rust_error"]:
            first = str(r["rust_error"])[:100]
        elif r["reference_error"]:
            first = str(r["reference_error"])[:100]
        lines.append(f"| {r['id']} | {r['verdict']} | {r['reference_status']} | {r['rust_status']} | {first.replace('|', '/')} |")
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    main()

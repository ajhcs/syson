#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Run every corpus case through the reference SysON and the Rust importer, then diff.

  differential.py [--only ID_SUBSTRING] [--out DIR]

Needs a running oracle (oracle.py up) and the Rust example binary, built with:
  cargo build --offline --locked --example oracle_import \
      --manifest-path rust/crates/syson-text-import/Cargo.toml
Writes one result file per case plus summary.json and SUMMARY.md to --out
(default: <oracle run dir>/differential-<UTC time>).
"""
import argparse
import hashlib
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
import oracle  # noqa: E402
import reference  # noqa: E402

REPO = HERE.parents[1]
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", "/workspace/build/builderbob/syson-oracle/target"))
RUST_BIN = TARGET / "debug/examples/oracle_import"


def cases():
    corpus = json.loads((HERE / "corpus.json").read_text())
    for c in corpus["inline"]:
        yield c["id"], c["source"], "inline"
    for p in corpus["paths"]:
        rel = Path(p)
        yield f"{rel.parent.name}-{rel.stem}", (REPO / rel).read_text(), p


def run_rust(source):
    p = subprocess.run([str(RUST_BIN)], input=source, capture_output=True, text=True, timeout=60)
    if p.returncode != 0:
        return {"status": "crash", "error": (p.stderr or "")[-600:]}
    return json.loads(p.stdout)


def run_reference(base, case_id, source):
    try:
        out = reference.import_text(base, case_id, source)
    except Exception as e:  # report, never hide
        return {"status": "error", "error": str(e)[:600]}
    docs = list(out["documents"].values())
    if len(docs) != 1:
        return {"status": "error", "error": f"expected 1 document, got {len(docs)}", "report": out["report"]}
    content = docs[0].get("content", [])
    if not content:
        return {"status": "empty", "report": out["report"], "document": docs[0]}
    return {"status": "ok", "report": out["report"], "document": docs[0]}


def classify(ref, rust, cmp):
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
    ap.add_argument("--out")
    args = ap.parse_args()
    if not oracle.STATE.exists():
        sys.exit("no running oracle: run `oracle.py up` first")
    state = json.loads(oracle.STATE.read_text())
    if not RUST_BIN.exists():
        sys.exit(f"missing {RUST_BIN}; build the oracle_import example first")
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out_dir = Path(args.out) if args.out else Path(state["run"]) / f"differential-{stamp}"
    (out_dir / "cases").mkdir(parents=True, exist_ok=True)
    results = []
    for case_id, source, origin in cases():
        if args.only and args.only not in case_id:
            continue
        t0 = time.monotonic()
        ref = run_reference(state["url"], case_id, source)
        rust = run_rust(source)
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
    summary = {"generated_utc": stamp, "oracle": {k: state[k] for k in ("url", "run", "jar_sha256", "jar_source_revision")},
               "rust_head": subprocess.run(["git", "-C", str(REPO), "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip(),
               "rust_dirty": bool(subprocess.run(["git", "-C", str(REPO), "status", "--porcelain", "rust"], capture_output=True, text=True).stdout.strip()),
               "counts": counts, "wire_findings": wire, "cases": results}
    (out_dir / "summary.json").write_text(json.dumps(summary, indent=1) + "\n")
    (out_dir / "SUMMARY.md").write_text(render(summary))
    print(json.dumps({"counts": counts, "wire_findings": wire, "out": str(out_dir)}, indent=1))


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

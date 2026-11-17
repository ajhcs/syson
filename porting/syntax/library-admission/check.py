#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Compare default syntax admission for the exact shipped Syside library files."""
import argparse
import csv
import hashlib
import io
import json
import re
import subprocess
from collections import Counter
from pathlib import Path
from zipfile import ZipFile

HERE = Path(__file__).resolve().parent
VSIX_SHA = "4b6717a384e6e348ac74f004bbfbdafe25c376acacea71b0d58232e6a1742534"
JAR_SHA = "85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1"
BUNDLE_SHA = "47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd"
COLUMNS = ["member", "bytes", "sha256", "syside_0.9.0", "rust_main", "rust_290"]
PREFIX = "extension/sysml.library/"
STATUS = {"accepted", "chart-state-budget"}

# Use the release's bundled services directly to distinguish syntax admission
# from recovery or the document/linking pipeline used by the CLI's dump command.
PROBE = """
const services = import_syside_languageserver.createSysMLServices(import_node.SysMLNodeFileSystem);
const input = JSON.parse(require("node:fs").readFileSync(0, "utf8"));
const results = input.map(({language, text}) => {
  const service = language === "sysml" ? services.SysML : services.KerML;
  const result = service.parser.LangiumParser.parse(text);
  return result.lexerErrors.length === 0 && result.parserErrors.length === 0;
});
process.stdout.write(JSON.stringify(results));
"""


def require(condition, message):
    if not condition:
        raise ValueError(message)


def verified_bytes(path, digest):
    data = path.read_bytes()
    require(hashlib.sha256(data).hexdigest() == digest, f"SHA-256 mismatch: {path}")
    return data


def sources(vsix):
    with (HERE / "baseline.tsv").open(newline="", encoding="utf-8") as ledger:
        reader = csv.DictReader(ledger, delimiter="\t")
        require(reader.fieldnames == COLUMNS, "unexpected inventory fields")
        rows = list(reader)
    require(all(None not in row and all(row.values()) for row in rows), "incomplete inventory row")
    names = [row["member"] for row in rows]
    require(len(names) == 94 and names == sorted(set(names)), "invalid inventory order or member count")
    with ZipFile(io.BytesIO(verified_bytes(vsix, VSIX_SHA))) as archive:
        members = sorted(name for name in archive.namelist() if name.endswith((".sysml", ".kerml")))
        require(members == names, "unexpected library member list")
        for row in rows:
            name = row["member"]
            require(
                name.startswith(PREFIX)
                and re.fullmatch(r"[1-9][0-9]*", row["bytes"])
                and re.fullmatch(r"[0-9a-f]{64}", row["sha256"])
                and row["syside_0.9.0"] == "accepted"
                and row["rust_main"] in STATUS and row["rust_290"] in STATUS,
                f"invalid inventory row: {name}",
            )
            data = archive.read(name)
            require(
                len(data) == int(row["bytes"]) and hashlib.sha256(data).hexdigest() == row["sha256"],
                f"member length or SHA-256 mismatch: {name}",
            )
            yield row, data


def reference_results(inputs, jar, node, launcher):
    with ZipFile(io.BytesIO(verified_bytes(jar, JAR_SHA))) as release:
        nested = release.read("BOOT-INF/lib/syson-sysml-import-2026.9.3.jar")
    with ZipFile(io.BytesIO(nested)) as importer:
        bundle = importer.read("syside-cli.js")
    require(hashlib.sha256(bundle).hexdigest() == BUNDLE_SHA, "embedded Syside bundle mismatch")
    body, separator, end = bundle.rpartition(b"\nsrc_default();")
    require(separator and not end.strip(), "unexpected bundled CLI entry point")
    with launcher.open("xb") as file:
        file.write(body + b"\n" + PROBE.encode())
    payload = [{"language": Path(row["member"]).suffix[1:], "text": data.decode()} for row, data in inputs]
    result = subprocess.run(
        [node.resolve(), "--max-old-space-size=768", launcher.resolve()],
        input=json.dumps(payload).encode(), capture_output=True, check=True, timeout=90,
    )
    require(not result.stderr, result.stderr.decode(errors="replace"))
    verdicts = json.loads(result.stdout)
    require(
        isinstance(verdicts, list) and len(verdicts) == len(inputs)
        and all(verdict is True for verdict in verdicts),
        f"release admission differs from the all-accepted baseline: {verdicts}",
    )


def unique_object(pairs):
    result = dict(pairs)
    require(len(result) == len(pairs), "duplicate key in Rust probe result")
    return result


def observe(probe, language, data):
    result = subprocess.run(
        [probe, language], input=data, capture_output=True, check=True, timeout=20,
    )
    require(not result.stderr, result.stderr.decode(errors="replace"))
    outcome = json.loads(result.stdout, object_pairs_hook=unique_object)
    require(isinstance(outcome, dict), "invalid Rust probe result")
    if outcome == {"status": "accepted"}:
        return "accepted", outcome
    status = outcome.get("status")
    if status == "budget":
        require(
            outcome.keys() == {"status", "resource", "limit"}
            and isinstance(outcome["resource"], str) and bool(outcome["resource"].strip())
            and type(outcome["limit"]) is int and outcome["limit"] > 0,
            "invalid Rust probe budget result",
        )
        if outcome["resource"] == "chart states" and outcome["limit"] == 250000:
            return "chart-state-budget", outcome
        return "other-refusal", outcome
    if status == "refused":
        require(
            outcome.keys() == {"status", "diagnostic"}
            and isinstance(outcome["diagnostic"], str) and bool(outcome["diagnostic"].strip()),
            "invalid Rust probe refusal result",
        )
        return "other-refusal", outcome
    raise ValueError("invalid Rust probe result")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vsix", type=Path, required=True)
    parser.add_argument("--rust-probe", type=Path, required=True)
    parser.add_argument("--expect", choices=("rust_main", "rust_290"))
    parser.add_argument("--reference-jar", type=Path)
    parser.add_argument("--node", type=Path)
    parser.add_argument("--launcher", type=Path, help="vacant workspace path for optional upstream launcher")
    args = parser.parse_args()
    if any((args.reference_jar, args.node, args.launcher)) and not all(
        (args.reference_jar, args.node, args.launcher)
    ):
        parser.error("--reference-jar, --node and --launcher must be used together")
    inputs = list(sources(args.vsix))
    if args.reference_jar:
        reference_results(inputs, args.reference_jar, args.node, args.launcher)
    probe = args.rust_probe.resolve()
    probe_sha256 = hashlib.sha256(probe.read_bytes()).hexdigest()
    mismatches = []
    totals = Counter()
    for row, data in inputs:
        status, outcome = observe(probe, Path(row["member"]).suffix[1:], data)
        totals[status] += 1
        if args.expect and row[args.expect] != status:
            mismatches.append(row["member"])
        print(json.dumps(dict(row, current=status, outcome=outcome), ensure_ascii=False))
    print(json.dumps({"totals": dict(totals), "rust_probe_sha256": probe_sha256,
                      "reference_replayed": bool(args.reference_jar),
                      "expected": args.expect, "mismatches": mismatches}))
    return bool(mismatches)


if __name__ == "__main__":
    raise SystemExit(main())

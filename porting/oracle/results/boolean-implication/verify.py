#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Check captured Boolean implication models without launching either importer."""

import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent.parent))
from compare import compare  # Repository's existing EMF normalization.

WIRE = [
    "import root Namespace wrapper: reference=True rust=False",
    "cross-references point at: reference=['id'] rust=['elementId']",
]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def verify_source(row):
    data = row["source"].encode("utf-8")
    require(digest(data) == row["source_sha256"], f"{row['case']}: source digest")
    return data


def verify_lexemes(row, data):
    entries = row["lexemes"]
    require(sum(x["feature"] == "literal" for x in entries) >= 2, f"{row['case']}: missing literal spans")
    require(any(x["feature"] == "operator" and x["raw"] == "implies" for x in entries),
            f"{row['case']}: missing implication span")
    previous = 0
    for lexeme in row["lexemes"]:
        start, end = lexeme["byte"]
        raw = lexeme["raw"]
        require(previous <= start < end <= len(data), f"{row['case']}: invalid byte span")
        previous = end
        require(data[start:end].decode("utf-8") == raw, f"{row['case']}: source spelling")
        offset = len(data[:start].decode("utf-8").encode("utf-16-le")) // 2
        units = len(raw.encode("utf-16-le")) // 2
        require(lexeme["utf16"] == [offset, offset + units], f"{row['case']}: UTF-16 span")


def verify_match(row, data):
    case = row["case"]
    snapshot = row["rust_snapshot"].encode("utf-8")
    require(digest(snapshot) == row["snapshot_sha256"], f"{case}: snapshot digest")
    require(digest(snapshot + b"\n") == row["reopened_stdout_sha256"], f"{case}: reopen digest")
    require(row["native_report"] == "", f"{case}: native reported diagnostics")
    actual = compare(row["native_document"], json.loads(snapshot))
    require(actual == row["comparison"], f"{case}: recorded comparison differs")
    require(actual["structural_match"] is True, f"{case}: structure differs")
    require(actual["wire_diffs"] == WIRE, f"{case}: wire differences changed")
    verify_lexemes(row, data)


def verify_refusal(row):
    case = row["case"]
    require(row["snapshot_absent"] is True and "rust_snapshot" not in row, f"{case}: snapshot recorded")
    raw = row["refusal_stdout"].encode("utf-8")
    require(digest(raw) == row["refusal_stdout_sha256"], f"{case}: refusal digest")
    result = row["rust_refusal"]
    require(json.loads(raw) == result, f"{case}: refusal record differs from stdout")
    require(set(result) == {"status", "error", "identities_requested"}, f"{case}: refusal fields")
    require(result["status"] == "error", f"{case}: refusal status")
    require(isinstance(result["error"], str) and bool(result["error"].strip()), f"{case}: diagnostic")
    ids = result["identities_requested"]
    require(type(ids) is int and ids >= 0, f"{case}: identity request count")


def verify(receipt, encoded):
    require(receipt["format"] == 1, "unknown receipt format")
    normalizer = HERE.parent.parent / "compare.py"
    require(digest(normalizer.read_bytes()) == receipt["normalizer_sha256"], "oracle normalizer digest")
    require(digest(encoded) == receipt["cases_sha256"], "case inventory digest")
    matches, refusals, seen = [], [], set()
    for entry in encoded.splitlines():
        row = json.loads(entry)
        case = row["case"]
        require(isinstance(case, str) and case and case not in seen, "missing or repeated case")
        seen.add(case)
        data = verify_source(row)
        if "comparison" in row:
            require("rust_refusal" not in row, f"{case}: ambiguous result")
            verify_match(row, data)
            matches.append(case)
        else:
            verify_refusal(row)
            refusals.append(case)
    require(matches and matches == receipt["stored_matches"], "match inventory differs")
    require(refusals and refusals == receipt["stored_refusals"], "refusal inventory differs")
    return len(matches), len(refusals)


def main():
    try:
        receipt = json.loads((HERE / "receipt.json").read_bytes())
        matches, refusals = verify(receipt, (HERE / "cases.jsonl").read_bytes())
    except (KeyError, OSError, TypeError, ValueError) as error:
        raise SystemExit(f"Boolean implication receipt failed: {error}") from None
    print(f"Verified {matches} stored model comparisons and {refusals} refusal records.")


if __name__ == "__main__":
    main()

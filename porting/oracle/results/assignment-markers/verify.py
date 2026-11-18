#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Check captured feature assignment marker models without launching either importer."""

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
    require(sum(x["feature"] == "literal" for x in entries) == 1, f"{row['case']}: missing integer span")
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


def elements(node, kind):
    if isinstance(node, dict):
        if node.get("eClass") == "sysml:" + kind:
            yield node
        for child in node.values():
            yield from elements(child, kind)
    elif isinstance(node, list):
        for item in node:
            yield from elements(item, kind)


def verify_payload(document, name, flags, value, case, native=False):
    usage, = elements(document, "AttributeUsage")
    require(usage["data"]["declaredName"] == name, f"{case}: attribute name")
    relation, typing = usage["data"]["ownedRelationship"]
    require(typing["eClass"] == "sysml:FeatureTyping", f"{case}: typed attribute")
    require(relation["eClass"] == "sysml:FeatureValue", f"{case}: feature value")
    definition, = elements(document, "AttributeDefinition")
    require(definition["data"]["declaredName"] == "N", f"{case}: definition name")
    typed = typing["data"]
    if native:
        require(typed["type"] == definition["id"] and typed["typedFeature"] == usage["id"],
                f"{case}: resource typing references")
    else:
        require(typed["type"] == definition["data"]["elementId"]
                and typed["typedFeature"] == usage["data"]["elementId"], f"{case}: semantic typing references")
    fields = relation["data"]
    for flag, enabled in zip(("isInitial", "isDefault"), flags, strict=True):
        require((flag in fields) is enabled and fields.get(flag, False) is enabled, f"{case}: {flag}")
    literal, = fields["ownedRelatedElement"]
    require(literal["eClass"] == "sysml:LiteralInteger", f"{case}: integer classifier")
    fields = literal["data"]
    integer = fields.get("value", 0)
    require(type(value) is int and type(integer) is int and integer == value, f"{case}: integer value")
    require(("value" in fields) is (value != 0), f"{case}: integer zero field")


def verify_snapshot(row, data):
    case = row["case"]
    snapshot = row["rust_snapshot"].encode("utf-8")
    require(digest(snapshot) == row["snapshot_sha256"], f"{case}: snapshot digest")
    require(digest(snapshot + b"\n") == row["reopened_stdout_sha256"], f"{case}: reopen digest")
    verify_lexemes(row, data)
    return json.loads(snapshot)


def verify_match(row, data):
    case = row["case"]
    flags = {"=": [False, False], ":=": [True, False],
             "default =": [False, True], "default :=": [True, True]}[row["marker"]]
    require(row["flags"] == flags, f"{case}: marker flags")
    expected = [("isDefault", "default")] * flags[1] + [("isInitial", ":=")] * flags[0]
    lexical = [(item["feature"], item["raw"]) for item in row["lexemes"]
               if item["feature"] in {"isDefault", "isInitial"}]
    require(lexical == expected, f"{case}: marker spans")
    document = verify_snapshot(row, data)
    require(row["native_report"] == "", f"{case}: native reported diagnostics")
    verify_payload(document, "n", flags, row["value"], case)
    verify_payload(row["native_document"], "n", flags, row["value"], case, native=True)
    actual = compare(row["native_document"], document)
    require(actual == row["comparison"], f"{case}: recorded comparison differs")
    require(actual["structural_match"] is True, f"{case}: structure differs")
    require(actual["wire_diffs"] == WIRE, f"{case}: wire differences changed")


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
    if case == "late-wrong-type":
        prefix = dict(row["valid_prefix"], case="valid-prefix")
        require(ids == 17 and prefix["identities_requested"] == 12, f"{case}: late typing sequence")
        require(result["error"].startswith("Model(WrongType"), f"{case}: wrong type status")
        document = verify_snapshot(prefix, verify_source(prefix))
        verify_payload(document, "good", [True, False], 0, "valid-prefix")
    if case == "duplicate-marker":
        require(ids == 0 and result["error"].startswith("Unexpected {"), f"{case}: syntax status")
        require("syntax error" in row["native_report"].lower(), f"{case}: native recovery report")


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
        raise SystemExit(f"Assignment marker receipt failed: {error}") from None
    print(f"Verified {matches} stored model comparisons and {refusals} refusal records.")


if __name__ == "__main__":
    main()

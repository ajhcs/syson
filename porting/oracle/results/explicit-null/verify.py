#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Check captured explicit null feature values without launching either importer."""

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
    spelling = {"explicit-null": ["null"], "empty-expression": ["()"], "default-null": ["null"],
                "unassigned": [], "quoted-null": ['"null"']}[row["case"]]
    require([x["raw"] for x in entries if x["feature"] == "literal"] == spelling,
            f"{row['case']}: null or string source spelling")
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


def verify_payload(document, row, native=False):
    case = row["case"]
    usage, = elements(document, "AttributeUsage")
    definition, = elements(document, "AttributeDefinition")
    require(usage["data"]["declaredName"] == "n" and definition["data"]["declaredName"] == "N",
            f"{case}: typed attribute")
    relationships = usage["data"]["ownedRelationship"]
    kinds = [item["eClass"] for item in relationships]
    classifier = row["classifier"]
    expected = (["sysml:FeatureValue"] if classifier else []) + ["sysml:FeatureTyping"]
    require(kinds == expected, f"{case}: feature value presence or order")
    typing = relationships[-1]["data"]
    typed_id = usage["id"] if native else usage["data"]["elementId"]
    type_id = definition["id"] if native else definition["data"]["elementId"]
    require(typing["type"] == type_id and typing["typedFeature"] == typed_id, f"{case}: typing references")
    if classifier:
        fields = relationships[0]["data"]
        for flag, enabled in [("isInitial", row["initial"]), ("isDefault", row["default"])]:
            require((flag in fields) is enabled and fields.get(flag, False) is enabled, f"{case}: {flag}")
        target, = fields["ownedRelatedElement"]
        require(target["eClass"] == "sysml:" + classifier, f"{case}: null or string classifier")
        if classifier == "NullExpression":
            require(set(target["data"]) == {"elementId"}, f"{case}: unexpected scalar data on NullExpression")
        else:
            require(isinstance(row["scalar"], str) and target["data"]["value"] == row["scalar"],
                    f"{case}: string payload")


def verify_snapshot(row, data):
    case = row["case"]
    snapshot = row["rust_snapshot"].encode("utf-8")
    require(digest(snapshot) == row["snapshot_sha256"], f"{case}: snapshot digest")
    require(digest(snapshot + b"\n") == row["reopened_stdout_sha256"], f"{case}: reopen digest")
    verify_lexemes(row, data)
    return json.loads(snapshot)


def verify_match(row, data):
    case = row["case"]
    expected = None if case == "unassigned" else "LiteralString" if case == "quoted-null" else "NullExpression"
    require(row["classifier"] == expected, f"{case}: expected classifier")
    require(row["default"] is (case == "default-null") and row["initial"] is (case == "default-null"),
            f"{case}: expected assignment flags")
    document = verify_snapshot(row, data)
    require(row["native_report"] == "", f"{case}: native reported diagnostics")
    verify_payload(document, row)
    verify_payload(row["native_document"], row, native=True)
    actual = compare(row["native_document"], document)
    require(actual["structural_match"] is True, f"{case}: structure differs")
    require(actual["wire_diffs"] == WIRE, f"{case}: wire differences changed")
    require(actual["reference_nodes"] == actual["rust_nodes"] == row["native_nodes"],
            f"{case}: model node count")
    return row["snapshot_sha256"]


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
    require(case == "late-unsupported" and ids == 11, f"{case}: atomic refusal control")
    require(result["error"] == 'Unsupported("null expression operands")', f"{case}: refusal kind")
    require(row["native_report"] == "", f"{case}: native observation reported diagnostics")
    operator, = elements(row["native_document"], "OperatorExpression")
    require(operator["data"]["operator"] == "+", f"{case}: native operator")
    operands = []
    for parameter in operator["data"]["ownedRelationship"]:
        require(parameter["eClass"] == "sysml:ParameterMembership", f"{case}: native parameter")
        feature, = parameter["data"]["ownedRelatedElement"]
        require(feature["eClass"] == "sysml:Feature" and feature["data"]["direction"] == "in",
                f"{case}: native input feature")
        value, = feature["data"]["ownedRelationship"]
        require(value["eClass"] == "sysml:FeatureValue", f"{case}: native operand value")
        target, = value["data"]["ownedRelatedElement"]
        operands.append(target["eClass"])
    require(operands == ["sysml:NullExpression", "sysml:LiteralInteger"],
            f"{case}: native null and integer operands")


def verify(receipt, encoded):
    require(receipt["format"] == 1, "unknown receipt format")
    normalizer = HERE.parent.parent / "compare.py"
    require(digest(normalizer.read_bytes()) == receipt["normalizer_sha256"], "oracle normalizer digest")
    require(digest(encoded) == receipt["cases_sha256"], "case inventory digest")
    matches, refusals, seen = {}, [], set()
    for entry in encoded.splitlines():
        row = json.loads(entry)
        case = row["case"]
        require(isinstance(case, str) and case and case not in seen, "missing or repeated case")
        seen.add(case)
        data = verify_source(row)
        if "rust_snapshot" in row:
            require("rust_refusal" not in row, f"{case}: ambiguous result")
            matches[case] = verify_match(row, data)
        else:
            verify_refusal(row)
            refusals.append(case)
    require(matches and list(matches) == receipt["stored_matches"], "match inventory differs")
    require(matches["explicit-null"] == matches["empty-expression"], "null spellings changed the saved model")
    require(len(set(matches.values())) == 4, "distinct null, default, absent or string models collapsed")
    require(refusals and refusals == receipt["stored_refusals"], "refusal inventory differs")
    return len(matches), len(refusals)


def main():
    try:
        receipt = json.loads((HERE / "receipt.json").read_bytes())
        matches, refusals = verify(receipt, (HERE / "cases.jsonl").read_bytes())
    except (KeyError, OSError, TypeError, ValueError) as error:
        raise SystemExit(f"Explicit-null receipt failed: {error}") from None
    print(f"Verified {matches} stored model comparisons and {refusals} refusal record.")


if __name__ == "__main__":
    main()

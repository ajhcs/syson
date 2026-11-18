# SPDX-License-Identifier: EPL-2.0
"""Verify the captured native archives and editor responses without a live app."""

import argparse
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from capture import JAR, RELEASE
from shape import node, payload, require, restore_targets, verify_readback, wires, without_targets, zip_document

OUTCOMES = {
    "nested-private": (["Inner", "Outer"], "DataType", True),
    "public-sibling": (["Peer"], None, False),
    "nested-shadow": (["Inner"], "Package", False),
}
SCALAR_SHA256 = "9c81e0f3bd67c8945cd843c8bfdff2d2eb8217b9ebddf9e502a6d78c16564f3d"


def checked_document(out, name, input_bytes, observed):
    zipped = (out / f"{name}.zip").read_bytes()
    require(sha256(input_bytes).hexdigest() == observed["upload_sha256"], f"Wrong upload: {name}")
    require(sha256(zipped).hexdigest() == observed["zip_sha256"], f"Wrong ZIP: {name}")
    raw, document = zip_document(zipped)
    require(sha256(raw).hexdigest() == observed["document_sha256"], f"Wrong document: {name}")
    return document


def check_editor(document, linked, results):
    expected = {
        "membership_class": ("sysml:MembershipImport", "aql:self.importedMembership.eClass().name",
                             "StringExpressionResult", "OwningMembership"),
        "membership_name": ("sysml:MembershipImport",
                            "aql:self.importedMembership.ownedRelatedElement->first().name",
                            "StringExpressionResult", "Integer"),
        "type_absent": ("sysml:FeatureTyping", "aql:self.type = null",
                        "BooleanExpressionResult", not linked),
    }
    if linked:
        expected["type_name"] = ("sysml:FeatureTyping", "aql:self.type.name",
                                 "StringExpressionResult", "Integer")
    require(set(results) == set(expected), "Missing or extra editor observation")
    for key, (kind, expression, result_type, value) in expected.items():
        field = "booleanValue" if result_type == "BooleanExpressionResult" else "stringValue"
        row = {"selected_raw_id": node(document, kind)["id"], "expression": expression,
               "response": {"__typename": "EvaluateExpressionSuccessPayload",
                            "result": {"__typename": result_type, field: value}}}
        require(payload(results[key]) == payload(row), f"Incorrect editor observation: {key}")


def check_case(out, name, source, rust, receipt):
    chain, selected, linked = OUTCOMES[name]
    require((rust["scope_chain"], rust["selected_class"], rust["type"] is not None)
            == (chain, selected, linked), f"Wrong lexical result: {name}")
    require(rust["query_graph_scoped_unqualified"] is False
            and rust["query_graph_global_unqualified"] is True, f"Unexpected raw search result: {name}")
    encoded = source.encode("utf-8")
    start, end = rust["name_span"]
    require(encoded[start:end].decode() == rust["parsed_name"] == "Integer", f"Wrong type span: {name}")
    start, end = rust["import"]["span"]
    prefix = rust["import"]["visibility"].lower()
    reference = rust["import"]["qualified_name"]
    require(encoded[start:end].decode().strip() == f"{prefix} import {reference};",
            f"Wrong import span: {name}")
    original = checked_document(out, name, encoded, receipt["text"])
    report = receipt["text"]["report"]
    require(report == "" if linked else "Unable to resolve name 'Integer'" in report,
            f"Wrong native report: {name}")
    imported = node(original, "sysml:MembershipImport")["data"]
    require(imported.get("visibility", "private") == prefix, f"Wrong native visibility: {name}")
    scaffold = without_targets(original)
    require(sha256(payload(scaffold)).hexdigest() == receipt["scaffold_sha256"], f"Wrong scaffold: {name}")
    restored = restore_targets(scaffold, rust)
    require(json.dumps(restored, sort_keys=True) == json.dumps(original, sort_keys=True),
            f"Restored document changes another field: {name}")
    readback = checked_document(out, f"{name}-reloaded", payload(restored), receipt["reloaded"])
    require(verify_readback(original, readback) == receipt["stable_ids"], f"Wrong stable ID count: {name}")
    require(wires(original) == wires(readback), f"Re-export changed references: {name}")
    if not linked:
        require("type" not in node(readback, "sysml:FeatureTyping")["data"], f"Unexpected type: {name}")
    check_editor(readback, linked, receipt["editor"])
    return original, scaffold, readback


def rejected(action, description):
    try:
        action()
    except ValueError:
        return
    raise ValueError(f"Expected refusal: {description}")


def controls(original, scaffold, readback, rust):
    rejected(lambda: restore_targets(original, rust), "scaffold with original references")
    duplicate_role = deepcopy(scaffold)
    duplicate_role["content"].append(deepcopy(node(scaffold, "sysml:MembershipImport")))
    rejected(lambda: restore_targets(duplicate_role, rust), "scaffold with duplicate role")
    renamed = deepcopy(readback)
    node(renamed, "sysml:AttributeUsage")["data"]["declaredName"] = "changed"
    rejected(lambda: verify_readback(original, renamed), "non-ID change outside the allowlist")


def check_packaged_provenance(capture):
    manifest = json.loads((capture / "provenance.json").read_bytes())
    for root, key in ((HERE, "source_sha256"), (capture, "evidence_sha256")):
        for name, expected in manifest[key].items():
            require(sha256((root / name).read_bytes()).hexdigest() == expected,
                    f"Packaged file hash mismatch: {name}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture", type=Path, default=HERE / "evidence")
    parser.add_argument("--rust-results", type=Path, default=HERE / "evidence" / "rust-results.json")
    parser.add_argument("--probe", type=Path)
    parser.add_argument("--scalar", type=Path, help="Exact ScalarValues.json from the fragment replay")
    args = parser.parse_args()
    require((args.probe is None) == (args.scalar is None), "Supply --probe and --scalar together")
    if args.capture.resolve() == (HERE / "evidence").resolve():
        check_packaged_provenance(args.capture)
    cases = json.loads((HERE / "cases.json").read_bytes())
    rust = json.loads(args.rust_results.read_bytes())
    receipt = json.loads((args.capture / "observations.json").read_bytes())
    require(receipt["release"] == {"jar_sha256": JAR, "jar_source_revision": RELEASE}, "Wrong reference release")
    require(set(receipt["cases"]) == set(cases) == set(rust) == set(OUTCOMES), "Missing or extra fixture")
    if args.probe:
        require(sha256(args.scalar.read_bytes()).hexdigest() == SCALAR_SHA256, "Wrong ScalarValues.json")
        replay = subprocess.check_output([args.probe.resolve(), HERE / "cases.json", args.scalar.resolve()])
        require(json.loads(replay) == rust, "Regenerated Rust result differs")
    for name, source in cases.items():
        control_documents = check_case(args.capture, name, source, rust[name], receipt["cases"][name])
        controls(*control_documents, rust[name])
        print(f"PASS {name}")


if __name__ == "__main__":
    main()

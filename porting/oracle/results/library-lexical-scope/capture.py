# SPDX-License-Identifier: EPL-2.0
"""Run the three lexical scope cases against an isolated SysON v2026.9.3."""

import argparse
from hashlib import sha256
import json
from pathlib import Path
import sys
from urllib.request import urlopen
from uuid import uuid4

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from shape import node, payload, require, restore_targets, verify_readback, wires, without_targets, zip_document

JAR = "85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1"
RELEASE = "b380dada415ce05155150f23cb3b74ae21cc45e4"
EVALUATE = """mutation($input: EvaluateExpressionInput!) {
  evaluateExpression(input:$input) { __typename
    ... on EvaluateExpressionSuccessPayload { result { __typename
      ... on StringExpressionResult { stringValue: value }
      ... on BooleanExpressionResult { booleanValue: value }
    } }
    ... on ErrorPayload { messages { body level } }
  }
}"""


def upload(reference, base, out, case, extension, body):
    created = reference._payload(reference.gql(base, reference.CREATE_PROJECT, {"i": {
        "id": str(uuid4()), "name": f"Lexical {case}", "templateId": "blank-project", "libraryIds": [],
    }}), "createProject")
    project = created["project"]["id"]
    context = reference.gql(base, reference.EDITING_CONTEXT, {"p": project})
    context_id = context["viewer"]["project"]["currentEditingContext"]["id"]
    receipt = reference._upload(base, context_id, f"{case}.{extension}", body)
    require(receipt["__typename"] == "UploadDocumentSuccessPayload", f"Upload failed: {case}")
    with urlopen(base + "/api/projects/" + project, timeout=120) as response:
        require(response.headers.get_content_type() == "application/zip", "Expected native ZIP")
        zipped = response.read(262_145)
    raw, document = zip_document(zipped)
    (out / f"{case}.zip").write_bytes(zipped)
    return context_id, document, {
        "report": receipt.get("report"), "upload_sha256": sha256(body).hexdigest(),
        "zip_sha256": sha256(zipped).hexdigest(), "document_sha256": sha256(raw).hexdigest(),
    }


def evaluate(reference, base, context, document, linked):
    checks = [
        ("membership_class", "sysml:MembershipImport", "aql:self.importedMembership.eClass().name",
         "StringExpressionResult", "OwningMembership"),
        ("membership_name", "sysml:MembershipImport",
         "aql:self.importedMembership.ownedRelatedElement->first().name", "StringExpressionResult", "Integer"),
        ("type_absent", "sysml:FeatureTyping", "aql:self.type = null", "BooleanExpressionResult", not linked),
    ]
    if linked:
        checks.append(("type_name", "sysml:FeatureTyping", "aql:self.type.name",
                       "StringExpressionResult", "Integer"))
    results = {}
    for role, kind, expression, result_type, expected in checks:
        selection = node(document, kind)["id"]
        response = reference.gql(base, EVALUATE, {"input": {
            "id": str(uuid4()), "editingContextId": context, "expression": expression,
            "selectedObjectIds": [selection],
        }})["evaluateExpression"]
        field = "booleanValue" if result_type == "BooleanExpressionResult" else "stringValue"
        require(response == {"__typename": "EvaluateExpressionSuccessPayload",
                             "result": {"__typename": result_type, field: expected}},
                f"Incorrect editor result for {role}: {response}")
        results[role] = {"selected_raw_id": selection, "expression": expression, "response": response}
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--rust-results", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--helpers", type=Path, default=HERE.parents[1])
    args = parser.parse_args()
    sys.path.insert(0, str(args.helpers.resolve()))
    import reference

    state = json.loads(args.state.read_bytes())
    require(state["jar_sha256"] == JAR and state["jar_source_revision"] == RELEASE, "Wrong SysON release")
    base = state["url"]
    require(base == f"http://127.0.0.1:{state['app_port']}", "Expected an isolated loopback oracle")
    rust = json.loads(args.rust_results.read_bytes())
    cases = json.loads((HERE / "cases.json").read_bytes())
    require(set(cases) == set(rust), "Missing or extra fixture result")
    args.out.mkdir(mode=0o700)
    results = {}
    for case, source in cases.items():
        row = rust[case]
        _, original, text = upload(reference, base, args.out, case, "sysml", source.encode("utf-8"))
        linked = row["type"] is not None
        require(wires(original) == {key: row[key] for key in ("type", "importedMembership")},
                f"Native text import and Rust references differ: {case}")
        report = text["report"]
        require(report == "" if linked else "Unable to resolve name 'Integer'" in report,
                f"Unexpected native text-import report: {case}")
        imported = node(original, "sysml:MembershipImport")["data"]
        require(imported.get("visibility", "private") == row["import"]["visibility"].lower(),
                f"Native import visibility differs: {case}")
        scaffold = without_targets(original)
        restored = restore_targets(scaffold, row)
        require(json.dumps(restored, sort_keys=True) == json.dumps(original, sort_keys=True),
                f"Restored model differs before upload: {case}")
        context, reloaded, loaded = upload(reference, base, args.out, f"{case}-reloaded",
                                           "json", payload(restored))
        stable_ids = verify_readback(original, reloaded)
        require(wires(original) == wires(reloaded), f"Native re-export changed wire: {case}")
        if not linked:
            require("type" not in node(reloaded, "sysml:FeatureTyping")["data"],
                    f"Unexpected type in reloaded document: {case}")
        results[case] = {
            "text": text, "reloaded": loaded, "stable_ids": stable_ids,
            "scaffold_sha256": sha256(payload(scaffold)).hexdigest(),
            "editor": evaluate(reference, base, context, reloaded, linked),
        }
        print(f"PASS {case}", flush=True)
    (args.out / "observations.json").write_bytes(payload({"release": {"jar_sha256": JAR,
        "jar_source_revision": RELEASE}, "cases": results}))


if __name__ == "__main__":
    main()

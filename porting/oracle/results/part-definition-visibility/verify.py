# SPDX-License-Identifier: EPL-2.0
"""Verify the archived visibility evidence without invoking either importer."""

from hashlib import sha256
import importlib.util
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
SEAL = "430bd0ccdf2f03fc0a2cdf97c60b309041551234d0b4c465112f76d0b34738a0"
SOURCE = "package P { private part def A; public part def B; }"
EXPECTED_REFUSALS = {
    "abstract-definition", "attribute-definition", "body-definition", "duplicate-modifier",
    "individual-definition", "item-definition", "mistyped-modifier", "namespace-import",
    "namespace-root-definition", "nested-feature", "nested-nonpackage-owner", "part-usage",
    "private-package", "protected-definition", "specialized-definition", "supplementary-broader-native",
    "unnamed-definition",
}


def require(value, detail):
    if not value:
        raise ValueError(detail)


def sha(value):
    return sha256(value if isinstance(value, bytes) else value.encode()).hexdigest()


def checked(row, field, checksum):
    value = row[field]
    require(sha(value) == row[checksum], field)
    return value


def shape(package, expected):
    require(package["eClass"] == "sysml:Package" and package["data"]["declaredName"] == "P", "Package P")
    members = package["data"]["ownedRelationship"]
    nodes = [package]
    for member, (name, visibility) in zip(members, expected, strict=True):
        part, = member["data"]["ownedRelatedElement"]
        require(member["eClass"] == "sysml:OwningMembership" and part["eClass"] == "sysml:PartDefinition"
                and part["data"]["declaredName"] == name and "visibility" not in part["data"], "owning membership target")
        require(("visibility" in member["data"]) == (visibility is not None)
                and member["data"].get("visibility") == visibility, "stored membership visibility")
        nodes.extend((member, part))
    ids = {node["id"] for node in nodes}
    elements = {node["data"]["elementId"] for node in nodes}
    require(len(ids) == len(elements) == len(nodes) and ids.isdisjoint(elements), "distinct identities")


def positive(row, expected):
    source = checked(row, "source", "source_sha256")
    stage = json.loads(checked(row, "stage", "stage_sha256"))
    snapshot = checked(row, "snapshot", "snapshot_sha256")
    doc = json.loads(snapshot)
    require(row["fresh_reader"] is True and sha(snapshot + "\n") == row["reopen_sha256"]
            and stage["status"] == "ok" and stage["document"] == doc
            and stage["identities_requested"] == 1 + 2 * len(expected), "fresh reader and exact staged document")
    package, = doc["content"]
    shape(package, expected)
    for token in stage["lexemes"]:
        start, end = token["byte"]
        require(token["byte"] == token["utf16"] and source[start:end] == token["raw"], "source token span")
    if source == SOURCE:
        markers = [(token["raw"], token["byte"]) for token in stage["lexemes"] if token["feature"] == "visibility"]
        require(markers == [("private", [12, 19]), ("public", [32, 38])], "visibility marker spans")
    return doc


def native_comparison(receipt, rust):
    native = receipt["native"]["visibility-member"]
    require(checked(native, "source", "source_sha256") == SOURCE and native["warnings"] == "", "native source")
    capture = json.loads(checked(native, "primary_record", "golden_sha256"))
    require(capture["id"] == "visibility-member" and capture["reference"] == receipt["release_identity"]
            and capture["result"]["status"] == "ok" and capture["result"]["report"] == "", "native release evidence")
    document = capture["result"]["document"]
    root, = document["content"]
    membership, = root["data"]["ownedRelationship"]
    package, = membership["data"]["ownedRelatedElement"]
    shape(package, [("A", "private"), ("B", None)])
    spec = importlib.util.spec_from_file_location("oracle_compare", HERE.parents[1] / "compare.py")
    comparator = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(comparator)
    require(comparator.normalize(document) == native["normalized"] == capture["normalized"], "normalized release capture")
    expected = dict(structural_match=True, structural_diffs=[], reference_nodes=5, rust_nodes=5,
                    wire_diffs=["import root Namespace wrapper: reference=True rust=False"])
    require(comparator.compare(document, rust) == receipt["comparison"] == expected, "native/Rust comparison")
    supplemental = receipt["native"]["convertVisibilityTest-visibility"]
    checked(supplemental, "source", "source_sha256")
    members = supplemental["normalized"]["roots"][0]["data"]["ownedRelationship"]
    require([member["data"].get("visibility") for member in members] == [None, None, "private"], "supplemental ownership")
    for member, visibility in zip(members[1:], ("protected", "private"), strict=True):
        target, = member["data"]["ownedRelatedElement"]
        feature, = target["data"]["ownedRelationship"]
        require(feature["eClass"] == "sysml:FeatureMembership" and feature["data"]["visibility"] == visibility,
                "supplemental feature membership")


def verify():
    raw = (HERE / "receipt.json").read_bytes()
    require(sha(raw) == SEAL, "receipt seal")
    receipt = json.loads(raw)
    replay = (HERE / "replay.jsonl").read_bytes()
    require(sha(replay) == receipt["replay"]["sha256"], "replay seal")
    rows = [json.loads(line) for line in replay.splitlines()]
    require(len(rows) == len({row["case"] for row in rows}) == receipt["replay"]["count"] == 260, "replay count")
    require([row for row in rows if row["source_sha256"] == sha(SOURCE)] ==
            [dict(case="default-visibility-member", source_sha256=sha(SOURCE),
                  stdout_sha256=receipt["predecessor_default"]["sha256"])],
            "exact source remains an incumbent on the default path")
    for row in rows:
        require(all(re.fullmatch("[0-9a-f]{64}", row[key]) for key in ("source_sha256", "stdout_sha256")),
                "incumbent hashes")
    positives = receipt["positives"]
    require(positives["visibility-member"]["source"] == SOURCE, "exact Rust source")
    rust = positive(positives["visibility-member"], [("A", "private"), ("B", None)])
    require(positive(positives["explicit-public"], [("A", None)])
            == positive(positives["implicit-public"], [("A", None)]), "public default omission")
    native_comparison(receipt, rust)

    refusals = receipt["refusals"]
    require(set(refusals) == EXPECTED_REFUSALS | {"late-wrong-type"}, "refusal scope")
    for name, row in refusals.items():
        checked(row, "source", "source_sha256")
        parsed = json.loads(checked(row, "stage", "stage_sha256"))
        require(row["snapshot_present"] is False and parsed["status"] == "error", name + " atomic refusal")
        if name == "late-wrong-type":
            require(parsed["identities_requested"] == 10 and parsed["error"].startswith("Model(WrongType "), name)
        else:
            require(parsed["identities_requested"] == 0 and row["stage_sha256"] == row["predecessor_sha256"], name)
    old = receipt["predecessor_probe"]
    require(json.loads(checked(old, "stdout", "sha256")) == dict(status="error", identities_requested=0,
        error="UnsupportedModelFeature(OwningMembership.visibility)"), "intended prototype change")
    default = receipt["predecessor_default"]
    require(default["agrees"] is True and json.loads(checked(default, "stdout", "sha256"))
            == dict(status="error", error="Parse(Unsupported { offset: 12 })"), "default refusal unchanged")
    require(receipt["cancellation"] == dict(status="error", error="Cancelled", identities_requested=0), "cancellation")
    scope = receipt["scope"]
    require(scope["default_adopted"] is scope["lookup_access_control_qualified"]
            is scope["abstract_definition_qualified"] is False, "encoding-only scope")
    print("PASS: five-node Package, three fresh reopens, 260 replay records and 17 unchanged refusals")


if __name__ == "__main__":
    verify()

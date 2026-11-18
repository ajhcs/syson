# SPDX-License-Identifier: EPL-2.0
"""Check the captured evidence packet without running an importer."""

from collections import Counter
from hashlib import sha256
import importlib.util
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
SOURCE = "package P { part def W; part def C { part w : W[*..4]; } }"
RECEIPT = "398f09498c52dee4611b1c852c2773139871455d841bbd363efe1a669cb4f8c8"
NEGATIVES = {
    "both-stars", "feature-range", "feature-sequence", "nameless-typed-part", "nameless-untyped-part",
    "open-lower", "open-upper", "other-owner-attribute", "other-owner-connection", "other-owner-item",
    "other-owner-port", "overflow", "pre-type-placement", "rational", "same-owner-feature-value",
    "signed", "specialization-in-place-of-typing", "untyped-part",
}


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def digest(raw):
    return sha256(raw if isinstance(raw, bytes) else raw.encode()).hexdigest()


def text_at(row, field, checksum):
    text = row[field]
    require(digest(text) == row[checksum], field)
    return text


def nodes(value):
    if isinstance(value, dict):
        if "eClass" in value:
            yield value
        for child in value.values():
            yield from nodes(child)
    if isinstance(value, list):
        for child in value:
            yield from nodes(child)


def named(doc, classifier, name):
    node, = (node for node in nodes(doc)
             if node["eClass"] == "sysml:" + classifier and node["data"].get("declaredName") == name)
    return node


def owned(member):
    require(member["eClass"] == "sysml:OwningMembership", "owning membership")
    element, = member["data"]["ownedRelatedElement"]
    return element


def shape(doc, name, count, native=False):
    all_nodes = list(nodes(doc))
    resources = {node["id"] for node in all_nodes}
    elements = {node["data"]["elementId"] for node in all_nodes}
    require(len(all_nodes) == len(resources) == len(elements) == count
            and resources.isdisjoint(elements), "distinct resource and semantic identities")
    part = named(doc, "PartUsage", name)
    target = named(doc, "PartDefinition", "W")
    membership, typing = part["data"]["ownedRelationship"]
    multiplicity = owned(membership)
    require(multiplicity["eClass"] == "sysml:MultiplicityRange"
            and set(multiplicity["data"]) == {"elementId", "ownedRelationship"}, "stored range")
    lower_member, upper_member = multiplicity["data"]["ownedRelationship"]
    lower, upper = owned(lower_member), owned(upper_member)
    require(lower["eClass"] == "sysml:LiteralInfinity" and set(lower["data"]) == {"elementId"},
            "lower infinity without implicit zero")
    data = upper["data"]
    require(upper["eClass"] == "sysml:LiteralInteger" and set(data) == {"elementId", "value"}
            and type(data["value"]) is int and data["value"] == 4, "stored upper integer")
    require(all(node["eClass"] != "sysml:OperatorExpression" for node in all_nodes), "no persisted AST wrapper")
    part_id = part["id"] if native else part["data"]["elementId"]
    target_id = target["id"] if native else target["data"]["elementId"]
    require(typing["eClass"] == "sysml:FeatureTyping" and
            (typing["data"]["typedFeature"], typing["data"]["type"]) == (part_id, target_id),
            "resolved typing after range")
    return lower, upper


def check_success(row, count):
    source = text_at(row, "source", "source_sha256").encode()
    stage = json.loads(text_at(row, "stage_stdout", "stage_sha256"))
    snapshot = text_at(row, "snapshot", "snapshot_sha256")
    model = json.loads(snapshot)
    require(row["fresh_reader"] is True and digest(snapshot + "\n") == row["reopen_sha256"]
            and stage["document"] == model, "fresh reader and exact snapshot")
    require(stage["status"] == "ok" and stage["identities_requested"] == count and stage["infix_actions"] == 1
            and stage["targets"] == [[{"raw": "W", "target": "Type"}]], "staged identities and reference")
    shape(model, row["usage"], count)
    tokens = [item for item in stage["lexemes"] if item["feature"] in {"literal", "operator"}]
    first = source.index(b"*..4")
    require([item["raw"] for item in tokens] == ["*", "..", "4"]
            and [item["byte"] for item in tokens] == [[first, first+1], [first+1, first+3], [first+3, first+4]]
            and tokens[0]["kind"] == "expression", "Rust operand order and byte spans")
    for token in stage["lexemes"]:
        start, end = token["byte"]
        span = source[start:end].decode()
        offset = len(source[:start].decode().encode("utf-16-le")) // 2
        require(span == token["raw"] and token["utf16"] == [offset, offset + len(span.encode("utf-16-le")) // 2],
                "Rust UTF-16 span")
    return model


def check_native(receipt, rust):
    native = receipt["native"]
    model = json.loads(text_at(native, "model", "model_sha256"))
    facts = [line.split("\t") for line in text_at(native, "facts", "facts_sha256").splitlines()]
    require(native["converter_terminal"] == native["transformation_messages"] == 0
            and native["library_resources"] == 94
            and facts[:3] == [["libraries", "94"], ["model_nodes", "16"], ["messages", "0"]], "native scope")
    lower, upper = shape(model, "w", 16, native=True)
    for label, node, spelling in [("lower", lower, "*"), ("upper", upper, "4")]:
        getter, = (line for line in facts if line[0] == label + "_getter")
        require(getter == [label + "_getter", node["eClass"].split(":")[1], spelling,
                           "true", node["id"], node["data"]["elementId"]], "getter returns stored node")
    require(native["operator_byte"] == [49, 51] and SOURCE[49:51] == ".." and native["operands"] == [
        dict(role="left", byte=[48, 49], lexeme="*", classifier="LiteralInfinity"),
        dict(role="right", byte=[51, 52], lexeme="4", classifier="LiteralNumber", literal=4),
    ], "release parser spans")
    namespace, = model["content"]
    require(namespace["eClass"] == "sysml:Namespace"
            and set(namespace["data"]) == {"elementId", "ownedRelationship"}, "native root wrapper")
    membership, = namespace["data"]["ownedRelationship"]
    require(set(membership["data"]) == {"elementId", "ownedRelatedElement"}, "wrapper membership")
    package = owned(membership)
    require(package["eClass"] == "sysml:Package" and package["data"]["declaredName"] == "P", "native Package")
    spec = importlib.util.spec_from_file_location("oracle_compare", HERE.parents[1] / "compare.py")
    comparator = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(comparator)
    raw = comparator.compare(model, rust)
    projected = comparator.compare(dict(model, content=[package]), rust)
    require(raw == receipt["comparison"]["raw"] and not raw["structural_match"]
            and projected == receipt["comparison"]["projected"] and projected["structural_match"]
            and projected["reference_nodes"] == projected["rust_nodes"] == 14, "raw and projected comparison")


def run():
    raw_receipt = (HERE / "receipt.json").read_bytes()
    require(digest(raw_receipt) == RECEIPT, "receipt seal")
    receipt = json.loads(raw_receipt)
    raw_cases = (HERE / "cases.jsonl").read_bytes()
    require(digest(raw_cases) == receipt["cases_sha256"], "case packet seal")
    rows = [json.loads(line) for line in raw_cases.splitlines()]
    cases = {row["case"]: row for row in rows}
    require(len(cases) == len(rows) == 257, "case identities")
    require({name for name, row in cases.items() if row["kind"] == "success"}
            == {"native-mixed-lower", "valid-prefix"}, "positive scope")
    require({name for name, row in cases.items() if row["kind"] == "refusal"}
            == NEGATIVES | {"late-wrong-type"}, "refusal scope")
    replay = {name: row for name, row in cases.items() if row["kind"] == "replay"}
    counts = Counter(row["family"] for row in replay.values())
    require(counts == receipt["replay"]["counts"] and counts["default"] == 62
            and len(replay) == receipt["replay"]["unchanged"] == 236, "incumbent campaign counts")
    omitted = receipt["replay"]["missing_intended_case"]
    require(omitted == "finite-range-mixed-lower" and omitted not in replay, "one excluded case")
    for row in replay.values():
        require(row["source_sha256"] != digest(SOURCE)
                and all(re.fullmatch("[0-9a-f]{64}", row[key]) for key in ("source_sha256", "stdout_sha256")),
                "incumbent record binding")

    rust = check_success(cases["native-mixed-lower"], 14)
    prefix = check_success(cases["valid-prefix"], 16)
    for name in NEGATIVES | {"late-wrong-type"}:
        row = cases[name]
        text_at(row, "source", "source_sha256")
        stdout = text_at(row, "stage_stdout", "stage_sha256")
        refusal = json.loads(stdout)
        require(row["snapshot_present"] is False and refusal["status"] == "error", "atomic refusal")
        if name == "late-wrong-type":
            wrong = named(prefix, "AttributeDefinition", "Wrong")["data"]["elementId"]
            require(refusal == dict(status="error", identities_requested=19,
                                    error=f'Model(WrongType {{ element: "{wrong}", expected: "PartDefinition" }})'),
                    "late refusal after provisional allocations")
        else:
            require(refusal["identities_requested"] == 0
                    and stdout == text_at(row, "predecessor_stdout", "predecessor_sha256"), name + " unchanged")

    intended = receipt["intended_delta"]
    require(cases["native-mixed-lower"]["source"] == SOURCE and digest(SOURCE) == intended["source_sha256"],
            "exact intended source")
    require(json.loads(text_at(intended, "predecessor_stdout", "predecessor_sha256")) ==
            dict(status="error", error="UnsupportedInfixOperator(..)", identities_requested=0), "archived refusal")
    default = receipt["default_path"]
    require(default["predecessor_byte_match"] is True and json.loads(text_at(default, "stdout", "sha256")) ==
            dict(status="error", error="Parse(Unsupported { offset: 47 })"), "default importer unchanged")
    require(receipt["cancellation"] == dict(status="error", error="Cancelled", identities_requested=0),
            "cancellation before allocations")
    check_native(receipt, rust)
    scope = receipt["scope"]
    require(scope["app_http_and_database"] is scope["default_adoption"] is False, "evidence-only scope")
    print("PASS: lower-infinity packet, 14-node Package comparison, 236 replay records, 18 unchanged refusals")


if __name__ == "__main__":
    run()

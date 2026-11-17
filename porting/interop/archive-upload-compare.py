# SPDX-License-Identifier: EPL-2.0
"""Verify the two SysON ArchiveDemo ZIPs and report their identity remapping."""

import argparse
from hashlib import sha256
from io import BytesIO
import json
from pathlib import Path
from uuid import UUID
from zipfile import ZipFile

ROOT = "ArchiveDemo/"
FIXTURE_HASH = "96846d20a44dee2cf87e3643a7c6804ce0db0a13731aee013a1abaf40ab28012"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def identifier(value):
    require(str(UUID(value)) == value, f"Expected a canonical UUID: {value}")
    return value


def only(items):
    only_item, = items
    return only_item


def archive(path):
    with path.open("rb") as source:
        blob = source.read(16_385)
    require(len(blob) <= 16_384, "Expected a small ArchiveDemo ZIP")
    with ZipFile(BytesIO(blob)) as zipped:
        entries = zipped.infolist()
        require(len(entries) == 3 and sum(i.file_size for i in entries) <= 16_384,
                "Expected three small ArchiveDemo archive members")
        require(zipped.testzip() is None, "Invalid member CRC")
        bodies = {info.filename: zipped.read(info) for info in entries}
        require(len(bodies) == 3, "Duplicate archive member")
        manifest = json.loads(bodies[ROOT + "manifest.json"])
        doc_id = identifier(only(manifest["documentIdsToName"]))
        representation_id = identifier(only(manifest["representations"]))
        doc = json.loads(bodies[f"{ROOT}documents/{doc_id}.json"])
        rep = json.loads(bodies[f"{ROOT}representations/{representation_id}.json"])
        receipt = {
            "bytes": len(blob),
            "sha256": sha256(blob).hexdigest(),
            "members": [{
                "path": info.filename,
                "bytes": info.file_size,
                "crc32": f"{info.CRC:08x}",
                "sha256": sha256(bodies[info.filename]).hexdigest(),
            } for info in entries],
        }
    return manifest, doc, rep, receipt


def descendants(elements):
    for element in elements:
        yield element
        for key in ("ownedRelationship", "ownedRelatedElement"):
            yield from descendants(element["data"].get(key, []))


def ids(elements, getter):
    entries = {identifier(getter(element)): element for element in elements}
    require(len(entries) == len(elements), "Duplicate semantic ID")
    return entries


def find_named(elements, cls, name):
    return only(element for element in elements.values() if element["eClass"] == f"sysml:{cls}"
                and element["data"].get("declaredName") == name)


def typing(owner):
    return only(rel["data"] for rel in owner["data"]["ownedRelationship"]
                if rel["eClass"] == "sysml:FeatureTyping")


def snapshot(path):
    manifest, doc, rep, receipt = archive(path)
    doc_id, doc_name = only(manifest["documentIdsToName"].items())
    rep_id, rep_meta = only(manifest["representations"].items())
    require(doc_name == "ArchiveDemoSource.sysml", "Manifest document name changed")
    elements = list(descendants(doc["content"]))
    by_id = ids(elements, lambda element: element["id"])
    by_stable_id = ids(elements, lambda element: element["data"]["elementId"])
    package = find_named(by_id, "Package", "ArchiveDemo")
    wheel_definition = find_named(by_id, "PartDefinition", "Wheel")
    wheel_usage = find_named(by_id, "PartUsage", "wheel")
    view = find_named(by_id, "ViewUsage", "Archive overview")
    view_typing = typing(view)
    wheel_typing = typing(wheel_usage)
    view_expose = only(rel["data"] for rel in view["data"]["ownedRelationship"]
                       if rel["eClass"] == "sysml:MembershipExpose")
    owning_membership = by_id[view_expose["importedMembership"]]
    require(owning_membership["eClass"] == "sysml:OwningMembership"
            and only(owning_membership["data"]["ownedRelatedElement"])["id"] == package["id"],
            "View membership expose does not resolve to the membership owning the package")
    require(wheel_typing["type"] == wheel_definition["id"]
            and wheel_typing["typedFeature"] == wheel_usage["id"],
            "FeatureTyping does not resolve wheel to its Wheel definition")
    require(view_typing["typedFeature"] == view["id"] and view_typing["type"].startswith(
        "sysml:ViewDefinition sysmllibrary:///"), "View typing is not linked to its bundled definition")

    target_uri = f"sirius:///{doc_id}#{view['id']}"
    diagram = rep["representation"]
    require(rep["id"] == diagram["id"] == rep_id, "Representation IDs disagree")
    require(rep["targetObjectId"] == diagram["targetObjectId"] == view["id"]
            and rep_meta["targetObjectURI"] == target_uri, "Representation targets the wrong ViewUsage")
    require(rep["kind"] == diagram["kind"] == rep_meta["type"]
            and rep["descriptionId"] == diagram["descriptionId"] == rep_meta["descriptionURI"]
            and rep["label"] == "Archive overview", "Representation metadata disagrees")

    node = only(diagram["nodes"])
    label = node["insideLabel"]
    require(not diagram["edges"] and node["targetObjectId"] == package["id"]
            and node["targetObjectLabel"] == label["text"] == "ArchiveDemo",
            "Expected one package node with a matching label and zero diagram edges")
    layout = diagram["layoutData"]
    for key, obj in (("nodeLayoutData", node), ("labelLayoutData", label)):
        layout_id, layout_item = only(layout[key].items())
        require(layout_id == layout_item["id"] == obj["id"], f"{key} points to the wrong object")

    generated = {
        "project": identifier(rep["projectId"]),
        "document": doc_id,
        "representation": rep_id,
        "packageNode": identifier(node["id"]),
        "packageNodeLabel": identifier(label["id"]),
    }
    generated.update({stable_id: element["id"] for stable_id, element in by_stable_id.items()})
    object_ids = set(generated.values())
    require(len(object_ids) == len(generated) and not object_ids.intersection(by_stable_id),
            "Stable semantic IDs and generated IDs must be distinct")
    bindings = {value: key for key, value in generated.items()}
    bindings[target_uri] = f"sirius:///document#{bindings[view['id']]}"

    def bind(value):
        # These bindings cover complete JSON tokens. The ViewDefinition URI and other
        # descriptive strings remain literal, as do the stable elementId fields.
        if isinstance(value, dict):
            return {bindings.get(k, k): bind(v) for k, v in value.items()}
        if isinstance(value, list):
            return [bind(v) for v in value]
        if isinstance(value, str):
            return bindings.get(value, value)
        return value

    receipt["identities"] = generated
    return bind({"manifest": manifest, "document": doc, "representation": rep}), receipt


def first_difference(left, right, path="$"):
    if type(left) is not type(right):
        return path
    if isinstance(left, dict):
        if left.keys() != right.keys():
            return f"{path} keys differ: {sorted(left.keys() ^ right.keys())}"
        pairs = ((f"{path}.{key}", value, right[key]) for key, value in left.items())
    elif isinstance(left, list):
        if len(left) != len(right):
            return f"{path} length differs"
        pairs = ((f"{path}[{i}]", value, right[i]) for i, value in enumerate(left))
    else:
        return None if left == right else f"{path}: {left!r} != {right!r}"
    for child, left_value, right_value in pairs:
        difference = first_difference(left_value, right_value, child)
        if difference is not None:
            return difference
    return None


def compare(original_path, reexport_path, capture):
    original, original_receipt = snapshot(original_path)
    reexport, reexport_receipt = snapshot(reexport_path)
    require(original_receipt["sha256"] == FIXTURE_HASH, "Wrong input fixture hash")
    require(capture["input"] == {k: original_receipt[k] for k in ("bytes", "sha256")},
            "Capture points to another input fixture")
    require(all(capture["output"][k] == reexport_receipt[k] for k in ("bytes", "sha256")),
            "Capture points to another output fixture")
    original_ids = original_receipt["identities"]
    new_ids = reexport_receipt["identities"]
    require(capture["output"]["project"]["id"] == new_ids["project"],
            "Re-exported representation projectId differs from the project upload response")
    diff = first_difference(original, reexport)
    require(diff is None, f"Archive differs after identity resolution: {diff}")
    require(not set(original_ids.values()).intersection(new_ids.values()),
            "A generated identity was reused across different projects")
    return {
        "result": "equal_after_identity_mapping",
        "archive_members": {
            "input": original_receipt["members"],
            "output": reexport_receipt["members"],
        },
        "identity_mapping": {key: [original_ids[key], new_ids[key]] for key in original_ids},
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("original", type=Path)
    parser.add_argument("reexport", type=Path)
    parser.add_argument("capture", type=Path)
    args = parser.parse_args()
    result = compare(args.original, args.reexport, json.loads(args.capture.read_text(encoding="utf-8")))
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()

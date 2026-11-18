# SPDX-License-Identifier: EPL-2.0
"""The two fields admitted for the finite standard-library replay."""

from copy import deepcopy
from io import BytesIO
import json
from zipfile import ZipFile

ROLES = {"sysml:MembershipImport": "importedMembership", "sysml:FeatureTyping": "type"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def walk(value):
    if isinstance(value, dict):
        if "eClass" in value:
            yield value
        for part in value.values():
            yield from walk(part)
    elif isinstance(value, list):
        for part in value:
            yield from walk(part)


def node(document, kind):
    matches = [part for part in walk(document) if part["eClass"] == kind]
    require(len(matches) == 1, f"Expected exactly one {kind}, got {len(matches)}")
    return matches[0]


def wires(document):
    return {key: node(document, kind)["data"].get(key) for kind, key in ROLES.items()}


def without_targets(document):
    scaffold = deepcopy(document)
    for kind, key in ROLES.items():
        node(scaffold, kind)["data"].pop(key, None)
    return scaffold


def restore_targets(scaffold, references):
    restored = deepcopy(scaffold)
    for kind, key in ROLES.items():
        target = node(restored, kind)["data"]
        require(key not in target, f"Scaffold already has {kind}.{key}")
        wire = references[key]
        require(wire is None or isinstance(wire, str), f"Invalid wire for {kind}.{key}")
        if wire is not None:
            target[key] = wire
    return restored


def payload(document):
    return (json.dumps(document, indent=2, ensure_ascii=False) + "\n").encode("utf-8")


def zip_document(body):
    require(len(body) <= 262_144, "Expected a small synthetic ZIP")
    with ZipFile(BytesIO(body)) as archive:
        require(sum(i.file_size for i in archive.infolist()) <= 262_144, "ZIP exceeds fixture bound")
        require(archive.testzip() is None, "ZIP member CRC mismatch")
        names = [name for name in archive.namelist() if "/documents/" in name]
        require(len(names) == 1, "Expected exactly one native model document")
        raw = archive.read(names[0])
    return raw, json.loads(raw)


def normalized(document):
    copy = deepcopy(document)
    objects = list(walk(copy))
    raw_to_semantic = {obj["id"]: obj["data"]["elementId"] for obj in objects}
    require(len(raw_to_semantic) == len(set(raw_to_semantic.values())) == len(objects),
            "Expected unique identifiers in both identity domains")
    for obj in objects:
        del obj["id"]
        if obj["eClass"] == "sysml:FeatureTyping":
            feature = obj["data"]["typedFeature"]
            require(feature in raw_to_semantic, "typedFeature must resolve through a raw object ID")
            obj["data"]["typedFeature"] = raw_to_semantic[feature]
    return copy, raw_to_semantic


def verify_readback(before, after):
    lhs, old = normalized(before)
    rhs, new = normalized(after)
    require(json.dumps(lhs, sort_keys=True) == json.dumps(rhs, sort_keys=True),
            "Project model changed after normalizing only raw IDs and the local typedFeature")
    require(not old.keys() & new.keys(), "Raw object ID was reused")
    return len(old)

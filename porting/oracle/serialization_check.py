# SPDX-License-Identifier: EPL-2.0
"""Check whether the reference SysON release can reopen finite Rust EMF JSON."""

import argparse
import copy
import hashlib
import json
import re
import sys
import uuid
from pathlib import Path
from urllib.parse import urlsplit

sys.path.insert(0, str(Path(__file__).resolve().parent))
import oracle  # noqa: E402
import reference  # noqa: E402

TYPING = "sysml:FeatureTyping"
LINKS = ("type", "typedFeature")
DECLARATION = {"elementId", "declaredName", "ownedRelationship"}
MEMBERSHIP = {"elementId", "ownedRelatedElement"}
FINITE_FIELDS = {
    "sysml:Package": DECLARATION,
    "sysml:PartDefinition": DECLARATION,
    "sysml:PartUsage": DECLARATION | {"isComposite"},
    "sysml:OwningMembership": MEMBERSHIP,
    "sysml:FeatureMembership": MEMBERSHIP,
    TYPING: {"elementId", *LINKS},
}
CONTROL_IDS = {"control-no-links", "control-element-id", "control-object-id", "control-dangling"}


class ModelMismatch(ValueError):
    pass


def admit_finite_input(document):
    indexed, _, _ = index_links(document)
    if not indexed:
        raise ValueError("no finite model elements")
    for path, node in indexed.items():
        fields = FINITE_FIELDS.get(node["eClass"], set())
        if not fields or set(node["data"]) - fields:
            raise ValueError(f"{path}: outside the finite importer wire check")


def objects(value, path="$"):
    if isinstance(value, dict):
        if "eClass" in value:
            if not isinstance(value.get("id"), str) or not isinstance(value.get("data"), dict):
                raise ModelMismatch(f"{path}: object lacks an EMF id or data")
            yield path, value
        for key, child in value.items():
            yield from objects(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from objects(child, f"{path}[{index}]")


def index_links(document):
    if not isinstance(document, dict) or not isinstance(document.get("content"), list):
        raise ModelMismatch("document has no content list")
    indexed = dict(objects(document["content"]))
    ids = {node["id"]: path for path, node in indexed.items()}
    if len(ids) != len(indexed):
        raise ModelMismatch("duplicate EMF object id")
    semantic = {}
    for path, node in indexed.items():
        alias = node["data"].get("elementId")
        if isinstance(alias, str):
            if alias in semantic or (alias in ids and ids[alias] != path):
                raise ModelMismatch("ambiguous semantic element id")
            semantic[alias] = path
    return indexed, ids, semantic


def resolve(shape, paths, alias, path, key):
    value = shape.get(key)
    if not isinstance(value, str) or (value not in paths and value not in alias):
        raise ModelMismatch(f"{path}.data.{key}: no resolved local EMF object id")
    return paths[value] if value in paths else alias[value]


def shape(value):
    if isinstance(value, dict):
        if "eClass" in value:
            omitted = set(LINKS) if value["eClass"] == TYPING else set()
            data = {key: shape(child) for key, child in value["data"].items()
                    if key not in omitted and key != "elementId"}
            return {**{k: shape(v) for k, v in value.items() if k != "id" and k != "data"},
                    "data": data}
        return {k: shape(v) for k, v in value.items()}
    if isinstance(value, list):
        return [shape(x) for x in value]
    return value


def first_difference(a, b, path="$"):
    if type(a) != type(b):
        return f"{path}: value type changed"
    if isinstance(a, dict):
        for key in sorted(set(a) | set(b)):
            if key not in b or key not in a:
                action = "missing" if key not in b else "added"
                return f"{path}.{key}: field {action}"
            reason = first_difference(a[key], b[key], f"{path}.{key}")
            if reason:
                return reason
    elif isinstance(a, list):
        if len(a) != len(b):
            return f"{path}: list length changed from {len(a)} to {len(b)}"
        for i, (x, y) in enumerate(zip(a, b)):
            reason = first_difference(x, y, f"{path}[{i}]")
            if reason:
                return reason
    elif a != b:
        return f"{path}: value changed"
    return None


def check_readback(expected, actual):
    before, ids_before, semantic_before = index_links(expected)
    after, ids_after, _ = index_links(actual)
    for path, node in before.items():
        found = after.get(path)
        if found is None or found["eClass"] != node["eClass"]:
            raise ModelMismatch(f"{path}: expected {node['eClass']} after upload")
        if found["data"].get("elementId") != node["data"].get("elementId"):
            raise ModelMismatch(f"{path}.data.elementId: semantic identity changed")
        if node["eClass"] == TYPING:
            for key in LINKS:
                desired = resolve(node["data"], ids_before, semantic_before, path, key)
                resolved = resolve(found["data"], ids_after, {}, path, key)
                if resolved != desired:
                    raise ModelMismatch(f"{path}.data.{key}: resolved to a different object")
    reason = first_difference(shape(expected["content"]), shape(actual["content"]))
    if reason:
        raise ModelMismatch(reason)


def id_control(document):
    """Convert only the fixture's two finite reference roles for a positive control."""
    control = copy.deepcopy(document)
    rows, ids, aliases = index_links(control)
    for path, node in rows.items():
        if node["eClass"] == TYPING:
            for key in LINKS:
                target = resolve(node["data"], ids, aliases, path, key)
                node["data"][key] = rows[target]["id"]
    return control


def upload_and_download(base, case_id, document, result):
    created = reference._payload(reference.gql(base, reference.CREATE_PROJECT, {"i": {
        "id": str(uuid.uuid4()), "name": f"wire-{case_id}"[:60],
        "templateId": "blank-project", "libraryIds": [],
    }}), "createProject")
    project = created["project"]["id"]
    try:
        context = reference.gql(base, reference.EDITING_CONTEXT, {"p": project})
        context = context["viewer"]["project"]["currentEditingContext"]["id"]
        payload = json.dumps(document, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        result["upload_sha256"] = hashlib.sha256(payload).hexdigest()
        try:
            report = reference._upload(base, context, f"{case_id}.json", payload)
        except reference.ReferenceError as error:
            if str(error).startswith("uploadDocument:"):
                raise ModelMismatch(str(error)) from error
            raise
        result["upload_report"] = report.get("report")
        documents = reference._project_documents(base, project)
        if len(documents) != 1:
            raise ModelMismatch(f"upload produced {len(documents)} documents; expected one")
        return next(iter(documents.values()))
    finally:
        deletion = reference.gql(base, reference.DELETE_PROJECT, {"i": {
            "id": str(uuid.uuid4()), "projectId": project,
        }})
        if deletion["deleteProject"]["__typename"] == "ErrorPayload":
            raise RuntimeError(f"could not delete owned wire check project {project}")


def run_case(base, case_id, document, expected=None):
    row = {"id": case_id, "opens_in_syson": "n/a", "reason": "Rust did not import"}
    if document is None:
        return row
    row["input"] = document
    try:
        admit_finite_input(document)
    except (ValueError, AttributeError, TypeError, KeyError) as error:
        row.update(reason=f"unsupported check input: {error}")
        return row
    try:
        row["readback"] = upload_and_download(base, case_id, document, row)
        check_readback(document if expected is None else expected, row["readback"])
    except ModelMismatch as error:
        row.update(opens_in_syson="no", reason=str(error))
    except (OSError, ValueError, KeyError, TypeError, RuntimeError, reference.ReferenceError,
            getattr(reference, "ReferenceProtocolError", reference.ReferenceError)) as error:
        row.update(reason=f"reference harness error: {type(error).__name__}: {error}")
    else:
        row.update(opens_in_syson="yes", reason="all elements and finite typed links survived")
    return row


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--cases", required=True, type=Path, help="existing differential cases/ directory")
    ap.add_argument("--out", required=True, type=Path, help="fresh output directory")
    ap.add_argument("--controls", action="store_true")
    args = ap.parse_args()
    inputs = {}
    for path in sorted(args.cases.glob("*.json")):
        entry = json.loads(path.read_text(encoding="utf-8"))
        name = path.stem
        if name == "summary" or name in CONTROL_IDS or name != entry["id"] or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,180}", name):
            ap.error(f"invalid case ID in {path}")
        inputs[name] = entry["rust_document"] if entry["rust_status"] == "ok" else None
    if not inputs:
        ap.error("no differential case inputs found")
    expected = {}
    if args.controls:
        typed = inputs["p01-basic-typed-part"]
        dangling = copy.deepcopy(typed)
        next(node["data"] for _, node in objects(dangling["content"])
             if node["eClass"] == TYPING)["type"] = "ffffffff-ffff-4fff-8fff-ffffffffffff"
        expected["control-dangling"] = typed
        inputs.update({
            "control-no-links": inputs["s01-empty-package"],
            "control-element-id": typed,
            "control-object-id": id_control(typed),
            "control-dangling": dangling,
        })
    state = json.loads(oracle.STATE.read_text(encoding="utf-8"))
    source_revision = getattr(oracle, "JAR_SOURCE_REVISION", "b380dada415ce05155150f23cb3b74ae21cc45e4")
    if (state["jar_sha256"] != oracle.PINNED[oracle.JAR] or
            state["jar_source_revision"] != source_revision):
        ap.error("reference state does not match the qualification release")
    base = state["url"]
    url = urlsplit(base)
    if (url.scheme != "http" or url.hostname != "127.0.0.1" or not url.port or
            url.username or url.password or url.path or url.query or url.fragment):
        ap.error("reference oracle must use an owned HTTP loopback port")
    args.out.mkdir()
    rows = []
    for name, model in inputs.items():
        row = run_case(base, name, model, expected.get(name))
        with (args.out / f"{name}.json").open("x", encoding="utf-8") as stream:
            json.dump(row, stream, indent=1, ensure_ascii=False)
            stream.write("\n")
        rows.append({k: row[k] for k in ("id", "opens_in_syson", "reason")})
        print(f"{name}: {row['opens_in_syson']} ({row['reason']})", flush=True)
    summary = {"reference": {k: state[k] for k in ("jar_sha256", "jar_source_revision")},
               "cases": rows}
    with (args.out / "summary.json").open("x", encoding="utf-8") as stream:
        json.dump(summary, stream, indent=1, ensure_ascii=False)
        stream.write("\n")
    if any(row["opens_in_syson"] == "n/a" and inputs[row["id"]] is not None for row in rows):
        raise SystemExit(2)


if __name__ == "__main__":
    main()

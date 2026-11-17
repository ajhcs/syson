# SPDX-License-Identifier: EPL-2.0
"""Bind the two SysON reference URIs to release bytes and run the Rust probe."""

import argparse
from hashlib import file_digest, md5, sha256
from io import BytesIO
import json
from pathlib import Path
import subprocess
from uuid import UUID
from zipfile import ZipFile

SOURCE = "backend/application/syson-application-configuration/src/main/resources"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    with path.open("rb") as source:
        return file_digest(source, "sha256").hexdigest()


def exact(path, expected):
    require(digest(path) == expected, f"Input hash mismatch: {path}")


def nodes(value, names=()):
    if isinstance(value, dict):
        data = value.get("data")
        if isinstance(data, dict):
            name = data.get("declaredName")
            if name:
                names = (*names, name)
            yield value, names
        for part in value.values():
            yield from nodes(part, names)
    elif isinstance(value, list):
        for part in value:
            yield from nodes(part, names)


def observe(document, eclass):
    matching = []
    for node, _ in nodes(document):
        data = node["data"]
        if node["eClass"] != "sysml:FeatureTyping":
            continue
        wire = data.get("type")
        if isinstance(wire, str) and wire.startswith(f"{eclass} ") and "library:///" in wire:
            matching.append(wire)
    (wire,) = matching
    return wire


def archive_wire(path, baseline):
    exact(path, baseline["zip_sha256"])
    with ZipFile(path) as zipped:
        require(zipped.testzip() is None, f"Invalid archive CRC: {path}")
        (document,) = [name for name in zipped.namelist() if "/documents/" in name]
        require(document == baseline["entry"], f"Wrong archive document: {path}")
        wire = observe(json.loads(zipped.read(document)), "sysml:ViewDefinition")
    require(wire == baseline["wire"], f"Archive reference changed: {path}")
    return wire


def text_wire(path, evidence):
    sample = evidence["baseline"]["text_case"]
    actual = digest(path)
    require(actual in (evidence["golden_case_sha256"], sample["hash"]),
            f"Text capture hash mismatch: {path}")
    packet = json.loads(path.read_bytes())
    require(packet["id"] == sample["case"], "Expected the saved assignment1 result")
    if actual == evidence["golden_case_sha256"]:
        require(packet["reference"] == {"jar_source_revision": evidence["release_revision"],
                                        "jar_sha256": evidence["app_jar_sha256"]}, "Golden release mismatch")
        result = packet["result"]
        require(result["status"] == "ok" and result["report"] == "", "Golden outcome changed")
        document = result["document"]
    else:
        require(packet["reference_status"] == "ok" and packet["reference_report"] == "",
                "Reference text-import outcome changed")
        document = packet["reference_document"]
    return observe(document, "sysml:DataType")


def resource(archive, baseline, wire, evidence):
    path = baseline["path"]
    raw = archive.read(path)
    require(len(raw) == baseline["bytes"] and sha256(raw).hexdigest() == baseline["sha256"],
            f"Unexpected library resource: {path}")
    for revision in (evidence["release_revision"], evidence["source_pin"]):
        source = subprocess.check_output(["git", "show", f"{revision}:{SOURCE}/{path}"])
        require(source == raw, f"Library resource differs from {revision}: {path}")
    require(wire == baseline["wire"], f"Unexpected observed wire: {path}")
    eclass, address = wire.split(" ")
    uri, fragment = address.split("#")
    scheme, _ = uri.split(":///")
    resource_id = UUID(bytes=md5(path.encode("ascii"), usedforsecurity=False).digest(), version=3)
    require(uri == baseline["resource"] == f"{scheme}:///{resource_id}", f"Resource UUID differs: {path}")
    document = json.loads(raw)
    (root,) = document["content"]
    require(root["id"] == baseline["root_node_id"]
            and root["data"]["elementId"] == baseline["root_element_id"], f"Resource root differs: {path}")
    ((node, names),) = [(node, names) for node, names in nodes(document)
                        if node["data"].get("elementId") == fragment]
    require(fragment == baseline["element_id"] and node["id"] == baseline["node_id"]
            and node["id"] != fragment and node["eClass"] == eclass
            and "::".join(names) == baseline["declaration"], f"Wrong fragment target: {path}")
    return raw


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", required=True, type=Path)
    parser.add_argument("--assignment", required=True, type=Path)
    parser.add_argument("--original", required=True, type=Path)
    parser.add_argument("--reexport", required=True, type=Path)
    parser.add_argument("--writer-outputs", nargs=2, type=Path, metavar=("EDITED", "OPAQUE"))
    parser.add_argument("--probe", required=True, type=Path, help="compiled Rust probe")
    parser.add_argument("--output", required=True, type=Path, help="new external evidence directory")
    args = parser.parse_args()
    local = Path(__file__).resolve().parent
    evidence = json.loads((local / "evidence.json").read_bytes())
    exact(local / "probe.rs", evidence["probe_source_sha256"])
    exact(args.jar, evidence["app_jar_sha256"])
    with ZipFile(args.jar) as jar:
        body = jar.read("BOOT-INF/lib/syson-application-configuration-2026.9.3.jar")
    require(sha256(body).hexdigest() == evidence["configuration_jar_sha256"], "Configuration JAR differs")

    original, reexport, edited, opaque = evidence["baseline"]["archive_observations"]
    view_wire = archive_wire(args.original, original)
    require(archive_wire(args.reexport, reexport) == view_wire, "Re-export changed the library wire")
    if args.writer_outputs:
        for path, expected in zip(args.writer_outputs, (edited, opaque)):
            require(archive_wire(path, expected) == view_wire, "Rust writer changed the library wire")
    wires = (text_wire(args.assignment, evidence), view_wire)
    subjects = evidence["baseline"]["subjects"]
    with ZipFile(BytesIO(body)) as archive:
        payloads = [resource(archive, item, wire, evidence) for item, wire in zip(subjects, wires)]

    args.output.mkdir(mode=0o700)
    for record, payload, wire in zip(subjects, payloads, wires):
        name = Path(record["path"]).name
        (args.output / name).write_bytes(payload)
        (args.output / f"{name}.wire").write_text(wire + "\n", encoding="utf-8")
    output = subprocess.check_output([args.probe.resolve(), args.output.resolve()])
    require(json.loads(output) == evidence["rust_results"], "The Rust probe result differs")
    (args.output / "rust-results.json").write_bytes(output)
    print("PASS both external fragment identities and six refusal controls per target")


if __name__ == "__main__":
    main()

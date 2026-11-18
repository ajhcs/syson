# SPDX-License-Identifier: EPL-2.0
"""Replay the new namespace wire evidence from the two exact archive captures."""

import argparse
from hashlib import sha256
import json
from pathlib import Path
from zipfile import ZipFile


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


def normalize(document, *, allow_semantic_references):
    nodes = list(walk(document["content"]))
    by_object = {node["id"]: node["data"]["elementId"] for node in nodes}
    semantic = set(by_object.values())
    require(len(by_object) == len(semantic) == len(nodes), "Duplicate document IDs")
    require(not semantic.intersection(by_object), "Semantic and EMF ID aliases overlap")

    def transform(value):
        if isinstance(value, dict):
            if "eClass" in value:
                node = {"eClass": value["eClass"], "data": transform(value["data"])}
                if value["eClass"] == "sysml:FeatureTyping":
                    for key in ("type", "typedFeature"):
                        ref = node["data"][key]
                        if allow_semantic_references and ref in semantic:
                            continue
                        require(ref in by_object, f"Unresolved {key}: {ref}")
                        node["data"][key] = by_object[ref]
                return node
            return {key: transform(part) for key, part in value.items()}
        if isinstance(value, list):
            return [transform(part) for part in value]
        return value

    return transform(document), set(by_object)


def envelope(document):
    root, = document["content"]
    require(root["eClass"] == "sysml:Namespace", "Missing root namespace")
    require(root["data"]["eAnnotations"] == [{"source": "org.eclipse.syson.sysml.imported"}],
            "Missing exact imported annotation")
    names = []
    for membership in root["data"]["ownedRelationship"]:
        require(membership["eClass"] == "sysml:OwningMembership", "Wrong root membership")
        child, = membership["data"]["ownedRelatedElement"]
        require(child["eClass"] == "sysml:Package", "Unexpected package root")
        names.append(child["data"]["declaredName"])
    return names


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--screenshots", type=Path, help="directory containing the optional original screenshots")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent
    evidence = json.loads((root / "evidence.json").read_bytes())
    for name, expected in evidence["file_sha256"].items():
        require(sha256((root / name).read_bytes()).hexdigest() == expected, f"Changed file: {name}")

    if args.screenshots is not None:
        for name, expected in evidence["browser"]["external_screenshot_sha256"].items():
            screenshot = (args.screenshots / name).read_bytes()
            require(sha256(screenshot).hexdigest() == expected, f"Changed screenshot: {name}")
            print(f"PASS external screenshot {name}")

    for case, names in (("s02-two-packages", ["A", "B"]), ("p01-basic-typed-part", ["Vehicle"])):
        row = evidence["native"][case]
        upload = (root / f"{case}-enveloped.json").read_bytes()
        require(sha256(upload).hexdigest() == row["upload_sha256"], f"{case}: wrong upload")
        archive_path = root / f"{case}-reexport.zip"
        require(sha256(archive_path.read_bytes()).hexdigest() == row["zip_sha256"], f"{case}: wrong ZIP")
        with ZipFile(archive_path) as archive:
            require(archive.testzip() is None, f"{case}: invalid ZIP CRC")
            document, = [name for name in archive.namelist() if "/documents/" in name]
            require(document == row["downloaded_document"], f"{case}: wrong ZIP member")
            returned = archive.read(document)
        require(sha256(returned).hexdigest() == row["document_sha256"], f"{case}: wrong document")
        expected = json.loads(upload)
        actual = json.loads(returned)
        require(envelope(expected) == envelope(actual) == names, f"{case}: root layout changed")
        before, old_ids = normalize(expected, allow_semantic_references=True)
        after, new_ids = normalize(actual, allow_semantic_references=False)
        require(before == after, f"{case}: normalized semantic content changed")
        require(len(new_ids) == row["raw_object_ids_regenerated"], f"{case}: ID count changed")
        require(not new_ids.intersection(old_ids), f"{case}: native EMF ID reused")
        print(f"PASS {case}")


if __name__ == "__main__":
    main()

"""Verify retained multi-document Rust archive evidence without a runtime launch."""
from copy import deepcopy
from hashlib import sha256
from io import BytesIO
import json
from pathlib import Path
from zipfile import ZIP_STORED, ZipFile, ZipInfo

HERE = Path(__file__).resolve().parent
OBSERVATION = "693f7752ea5848202e0281521aa189ae1a5fe3fbf65aa59a759fbce7a5909fd8"
SERVICE = "170a7f78cfad60268420e9e2aa73cb104da813b50c1a5860e79998826314165c"
PARENT = "f6ad3d6861a9ecf2e33f5068d2dd9faf9e1764bc"
NATIVE = "04aa94d7ee6f003004067d41a98ea12f8449fbf5d110f250859fad532f5af68b"
SCOPE = "87be18361c9c396b56e297bf0fd6ea2a2bf4534100beb66f481022a3d8041d5b"
RECEIPT = "a39f667c1fbbe6f851bc1fc34072a8f2da0ad2085d5f91b557c3c31e73e495b8"
READER = "24dafe44e73d56f15c4447773f921012b70c207876fb91f15b91af37563c9460"
WITNESS = "PASS network and process-group changes denied by kernel\n"
REFUSALS = {"missing-document": "manifest-declared member missing\n",
            "wrong-representation-owner": "diagram target EMF object absent\n"}


def require(value, reason):
    if not value:
        raise ValueError(reason)


def digest(data):
    return sha256(data).hexdigest()


def entries(items):
    value = dict(items)
    require(len(value) == len(items), "duplicate JSON key")
    return value


def decode(data):
    return json.loads(data, object_pairs_hook=entries)


def pinned_json(path, expected):
    raw = path.read_bytes()
    require(digest(raw) == expected, f"changed file {path.name}")
    return decode(raw)


def load_members(raw):
    with ZipFile(BytesIO(raw)) as archive:
        members = archive.infolist()
        names = [item.filename for item in members]
        require(len(names) == len(set(names)) and not archive.comment, "wrong ZIP frame")
        return {item.filename: archive.read(item) for item in members}, {
            item.filename: {"name": item.filename, "bytes": item.file_size,
                            "compressed_bytes": item.compress_size, "crc32": f"{item.CRC:08x}"}
            for item in members
        }


def prepared_zip(members):
    result = BytesIO()
    with ZipFile(result, "w", compression=ZIP_STORED, allowZip64=False) as writer:
        for name, data in members.items():
            row = ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            row.create_system = 3
            row.external_attr = 0o100600 << 16
            writer.writestr(row, data)
    return result.getvalue()


def objects(elements):
    for element in elements:
        yield element
        for role in ("ownedRelationship", "ownedRelatedElement"):
            yield from objects(element["data"].get(role, []))


def document_report(raw, resource, title):
    rows = []
    for node in objects(decode(raw)["content"]):
        data = node["data"]
        rows.append({"emf_id": node["id"], "element_id": data["elementId"], "class": node["eClass"],
                     "name": data.get("declaredName"), "type": data.get("type"),
                     "typed_feature": data.get("typedFeature")})
    require(len(rows) == len({row["emf_id"] for row in rows}) == len({row["element_id"] for row in rows}),
            "conflicting object identities")
    return {"document_id": resource, "name": title, "objects": sorted(rows, key=lambda row: row["emf_id"])}


def diagram_report(raw, declaration, source):
    wire = decode(raw)
    representation = wire["representation"]
    keys = {row["emf_id"]: row for row in source["objects"]}
    uri = declaration["targetObjectURI"]
    prefix = f"sirius:///{source['document_id']}#"
    require(uri.startswith(prefix), "changed representation document")
    target = uri.removeprefix(prefix)
    require(wire["targetObjectId"] == representation["targetObjectId"] == target
            and keys[target]["class"] == "sysml:ViewUsage", "changed ViewUsage target")
    require(wire["id"] == representation["id"] and wire["descriptionId"] == declaration["descriptionURI"],
            "changed representation envelope")
    links = []
    for node in representation["nodes"]:
        bound = keys[node["targetObjectId"]]
        links.append({"node_id": node["id"], "label_id": node["insideLabel"]["id"],
                      "target_emf_id": bound["emf_id"], "target_element_id": bound["element_id"],
                      "target_class": bound["class"]})
    return {"representation_id": wire["id"], "project_id": wire["projectId"], "label": wire["label"],
            "target_uri": uri, "target_element_id": keys[target]["element_id"], "node_links": links}


def expected_report(archive, info, manifest, fixture, documents, rep):
    labels = {fixture["manifest"]: "manifest", fixture["native_document"]: "parsed-emf-document",
              fixture["native_representation"]: "opaque-diagram-with-verified-links",
              fixture["new_document"]: "parsed-emf-document"}
    members = [{**info[path], "status": labels[path]} for path in sorted(info)]
    return {"archive_bytes": len(archive), **{key: manifest[key] for key in ("dependencies", "metamodels", "natures")},
            "root": fixture["root"], "members": members, "observed_diagrams": [rep],
            "documents": sorted(documents, key=lambda doc: doc["document_id"])}


def verify_streams(record, positive, negatives):
    require(record["kernel_stdout"] == WITNESS, "changed kernel witness")
    process = record["process"]
    require(process["complete"] is True and process["frozen_inputs_unchanged"] is True, "changed completion")
    proof = {"empty_live_group_samples": 2, "group_sigkill_attempted": True,
             "waitable_leader_retained_until_proof": True}
    expected = []
    for label, data in positive.items():
        expected.append({"step": label, "stdout_sha256": digest(data), "stderr_sha256": digest(b""),
                         "exit": 0, "guard": None, "owned_group_quiescent": True, "teardown_proof": proof})
    require(set(record["refusals"]) == set(REFUSALS), "changed control set")
    for label, stderr in REFUSALS.items():
        require(record["refusals"][label] == {"archive_sha256": digest(negatives[label]), "stderr": stderr},
                f"changed refusal {label}")
        expected.append({"step": label, "stdout_sha256": digest(b""), "stderr_sha256": digest(stderr.encode()),
                         "exit": 2, "guard": None, "owned_group_quiescent": True, "teardown_proof": proof})
    require(process["records"] == expected, "changed process/stream custody")


def check(record, service, native, combined):
    require((record["service_commit"], record["service_observation_sha256"]) == (PARENT, SERVICE), "service custody")
    require((record["scope_sha256"], record["terminal_receipt_sha256"], record["reader_binary_sha256"])
            == (SCOPE, RECEIPT, READER), "reader custody")
    require(record["native_archive_pr"] == 296 and digest(native) == record["native_archive_sha256"] == NATIVE
            and digest(combined) == record["combined_archive_sha256"], "archive custody")
    fixture = record["fixture"]
    original, original_info = load_members(native)
    staged, staged_info = load_members(combined)
    manifest_path, native_doc, native_rep, new_doc = (
        fixture[key] for key in ("manifest", "native_document", "native_representation", "new_document")
    )
    require(set(original) == {manifest_path, native_doc, native_rep}
            and set(staged) == set(original) | {new_doc}, "wrong complete inventory")
    for path in (native_doc, native_rep):
        require(staged[path] == original[path], "native document or representation bytes changed")
    initial_manifest, combined_manifest = decode(original[manifest_path]), decode(staged[manifest_path])
    expected_manifest = deepcopy(initial_manifest)
    new_id, new_name = fixture["new_document_id"], fixture["new_document_name"]
    require(new_id not in expected_manifest["documentIdsToName"], "new document identity already used")
    expected_manifest["documentIdsToName"][new_id] = new_name
    require(combined_manifest == expected_manifest and new_doc == f"{fixture['root']}/documents/{new_id}.json",
            "positive manifest changed beyond one additional declared document")
    official_model = service["projects"]["alpha"]["revision"]
    require(staged[new_doc] == official_model["data"].encode()
            and digest(staged[new_doc]) == official_model["sha256"], "changed service document bytes")
    for label, members in (("native_sha256", original), ("combined_sha256", staged)):
        require(fixture[label] == {name: digest(data) for name, data in members.items()}, "member hashes changed")

    native_id, = initial_manifest["documentIdsToName"]
    representation_id, = initial_manifest["representations"]
    source = document_report(original[native_doc], native_id, initial_manifest["documentIdsToName"][native_id])
    added = document_report(staged[new_doc], new_id, new_name)
    rep = diagram_report(original[native_rep], initial_manifest["representations"][representation_id], source)
    by_id = {row["emf_id"]: row for row in added["objects"]}
    edges = [row for row in by_id.values() if row["class"] == "sysml:FeatureTyping"]
    require(len(edges) == 2, "wrong typing edge count")
    typed = set()
    for edge in edges:
        target, feature = by_id[edge["type"]], by_id[edge["typed_feature"]]
        require(target["class"] == "sysml:PartDefinition" and target["name"] == "RestartedA"
                and target["emf_id"] != target["element_id"] and feature["emf_id"] != feature["element_id"],
                "wrong typing endpoint or identity role")
        typed.add(feature["class"])
    require(typed == {"sysml:PartUsage", "sysml:InvocationExpression"}, "wrong typed feature classes")
    require(decode(record["native_report"]) == expected_report(
        native, original_info, initial_manifest, fixture, [source], rep
    ), "native reader report changed")
    require(decode(record["combined_report"]) == expected_report(
        combined, staged_info, combined_manifest, fixture, [source, added], rep
    ), "combined reader report changed")

    missing = {name: data for name, data in staged.items() if name != new_doc}
    wrong_manifest = deepcopy(combined_manifest)
    target = rep["target_uri"].split("#", 1)[1]
    require(target not in by_id, "wrong-owner control no longer isolates the binding")
    wrong_manifest["representations"][representation_id]["targetObjectURI"] = f"sirius:///{new_id}#{target}"
    wrong = {**staged, manifest_path: (json.dumps(wrong_manifest, ensure_ascii=False, indent=2) + "\n").encode()}
    positive = {"kernel": WITNESS.encode(), "native": record["native_report"].encode(),
                "combined": record["combined_report"].encode(), "combined-original": combined,
                "manifest": staged[manifest_path], "native-document": staged[native_doc],
                "invocation-document": staged[new_doc], "representation": staged[native_rep]}
    verify_streams(record, positive, {"missing-document": prepared_zip(missing),
                                      "wrong-representation-owner": prepared_zip(wrong)})


def reject_mutations(record, service, native, combined):
    def altered_report(r):
        graph = decode(r["combined_report"])
        graph["documents"][0]["objects"][0]["element_id"] = graph["documents"][0]["objects"][0]["emf_id"]
        r["combined_report"] = json.dumps(graph)
        step = next(row for row in r["process"]["records"] if row["step"] == "combined")
        step["stdout_sha256"] = digest(r["combined_report"].encode())

    def change_zip(r, previous, member, data):
        changed, _ = load_members(previous)
        changed[member] = data
        r["fixture"]["combined_sha256"][member] = digest(data)
        raw = prepared_zip(changed)
        r["combined_archive_sha256"] = digest(raw)
        return raw

    controls = (
        lambda r: altered_report(r),
        lambda r: r["refusals"]["missing-document"].update(stderr=REFUSALS["wrong-representation-owner"]),
        lambda r: r["refusals"]["wrong-representation-owner"].update(archive_sha256="different"),
        lambda r: r["process"]["records"][2]["teardown_proof"].update(empty_live_group_samples=1),
        lambda r: r["process"]["records"][-1].update(stdout_sha256=digest(b"fabricated output")),
    )
    for mutate in controls:
        modified = deepcopy(record)
        mutate(modified)
        refuse(modified, service, native, combined)
    members, _ = load_members(combined)
    for member in (record["fixture"]["native_representation"], record["fixture"]["new_document"]):
        modified = deepcopy(record)
        rebuilt = change_zip(modified, combined, member, members[member] + b"\n")
        refuse(modified, service, native, rebuilt)
    return len(controls) + 2


def refuse(record, service, native, combined):
    try:
        check(record, service, native, combined)
    except ValueError:
        return
    raise ValueError("accepted altered archive evidence")


def main():
    record = pinned_json(HERE / "observation.json", OBSERVATION)
    service = pinned_json(HERE.parent / "invocation-service/observation.json", SERVICE)
    native = (HERE / "native-reference.syson.zip").read_bytes()
    combined = (HERE / "combined.syson.zip").read_bytes()
    check(record, service, native, combined)
    print(f"Verified 4 exact archive members, 2 reader refusals and {reject_mutations(record, service, native, combined)} altered records.")


if __name__ == "__main__":
    main()

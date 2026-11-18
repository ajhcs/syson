"""Verify the sanitized Rust HTTP restart packet using only retained files."""
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
EDIT_DIR = HERE.parent / "invocation-edit"
OBSERVATION = "170a7f78cfad60268420e9e2aa73cb104da813b50c1a5860e79998826314165c"
EDIT = "f190822b7ab0c3dbb3b4ec9f1fe9fd027de9f0cb5030c35e32a10257c589ff07"
EDIT_VERIFIER = "2962d3a56f5bb663fc8d98fd2f5b64759e64cbc1dcd0400cf2e51188389e8c60"
EARLIER = "0ecdf06b7a675b8905518fcd1e969537c7bbc056513bc0b474cb3f904b32ef0c"
PARENT = "32f801c81f3a5a143cc73ded465a7b4b27be1e47"
SCOPE = "20fc3c6c0ae3514ee85906c859c620f55f3ddc258c78e00ddccea7293c059ab8"
RECEIPT = "96e16c55093769b87d85d77c354da520635b020ae5adc28c4958533326647380"
BINARY = "de3fb3fb492ffbf6a09fd8ecbf66eaded08d1f51ba64e73ce0bf6bcec3ef0265"
WITNESS = "PASS connected MSG_NOSIGNAL write; addressed sends and outbound connect denied\n"
PARTS = {"alpha": ("concrete-seed", "part-call", False),
         "beta": ("abstract-seed", "abstract-part-call", True)}
SAVED = "verified-revision"
LOADED = "Saved revision loaded."
IMMUTABLE = "Start a new revision to edit this saved model."
STALE = "The draft changed in another window. Review the current model before retrying."


def digest(raw):
    return sha256(raw).hexdigest()


def prior_helpers():
    source = EDIT_DIR / "verify.py"
    if digest(source.read_bytes()) != EDIT_VERIFIER:
        raise ValueError("changed edit verifier")
    return runpy.run_path(str(source))


helpers = prior_helpers()
require = helpers["require"]
decode = helpers["decode"]
artifact = helpers["artifact"]
pinned_json = helpers["pinned_json"]


def graph_hash(graph):
    return digest(json.dumps(graph, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode())


def route(project, key):
    return f"/projects/{project}/revisions/{key}"


def exchange(path, notice, graph, operation=None, location=None):
    return {"method": "GET" if operation is None else "POST", "path": path,
            "status": 303 if location else 200, "location": location, "operation": operation,
            "notice": notice, "model_sha256": None if graph is None else graph_hash(graph),
            "definition_selector": None if graph is None else "No definition loaded"}


def published_project(project, record, edit):
    seed, case, flag = PARTS[project]
    entry = record["projects"][project]
    require(set(entry) == {"source_case", "seed", "seed_sha256", "revision"}, "changed project projection")
    original = edit["candidate_snapshots"][f"{case}-candidate"]
    require((entry["source_case"], entry["seed"], entry["seed_sha256"]) == (case, seed, original["sha256"]),
            "seed custody differs")
    initial = decode(artifact(original))
    saved = decode(artifact(entry["revision"]))
    inspect = helpers["inspect_part"]
    require(inspect(initial, "RenamedA", flag) == inspect(saved, "RestartedA", flag), "changed typing identity")
    expected = deepcopy(initial)
    package = helpers["package"](expected)
    helpers["declaration"](package, "RenamedA", "PartDefinition")["data"]["declaredName"] = "RestartedA"
    require(saved == expected, "service revision differs beyond the single target declaredName")
    return initial, saved


def check(record, edit, earlier):
    require((record["source_commit"], record["edit_observation_sha256"]) == (PARENT, EDIT), "edit custody drift")
    require((record["scope_sha256"], record["receipt_sha256"], record["service_binary_sha256"])
            == (SCOPE, RECEIPT, BINARY), "service custody drift")
    require(artifact(record["kernel_witness"]) == WITNESS, "connected kernel witness changed")
    proof = {"empty_live_group_samples": 2, "group_sigkill_attempted": True,
             "waitable_leader_retained_until_proof": True}
    require(record["process"] == {
        "complete": True, "inputs_unchanged": True, "fresh_service_process": True,
        "same_numeric_loopback_endpoint": True,
        "teardown": {"kernel": {**proof, "exit": 0},
                     "before": {**proof, "exit": -9}, "after": {**proof, "exit": -9}},
    }, "owned lifetimes or teardown changed")
    require(set(record["projects"]) == set(PARTS), "changed isolated project set")
    expected = {"before": [], "after": []}
    for project, (seed, _, _) in PARTS.items():
        initial, saved = published_project(project, record, edit)
        root, revision = route(project, seed), route(project, SAVED)
        expected["before"].extend([
            exchange(root, LOADED, initial),
            exchange(root, IMMUTABLE, initial, "rename"),
            exchange(root, None, None, "fork", revision),
            exchange(revision, f"New draft from revision {seed}.", initial),
            exchange(revision, STALE, initial, "rename"),
            exchange(revision, "Renamed definition.", saved, "rename"),
            exchange(revision, "Revision saved.", saved, "save"),
        ])
        expected["after"].extend([exchange(revision, LOADED, saved), exchange(root, LOADED, initial)])

    original = earlier["attempts"][1]["snapshots"]["typed-calculation-call"]
    require(record["calculation_seed_sha256"] == original["sha256"]
            == edit["originals_sha256"]["typed-calculation-call-original"], "changed calculation seed")
    calculation = decode(artifact(original))
    helpers["inspect_calculation"](calculation)
    for phase in expected.values():
        phase.append(exchange(route("alpha", "calculation-seed"), LOADED, calculation))
    require(record["requests"] == expected, "HTTP response order, controls, or complete model changed")


def mutate_revision(record, change):
    item = record["projects"]["alpha"]["revision"]
    document = decode(item["data"])
    change(document)
    item["data"] = json.dumps(document, sort_keys=True)
    item["sha256"] = digest(item["data"].encode())


def reject_mutations(record, edit, earlier):
    def extra_edit(graph):
        helpers["package"](graph)["data"]["isAbstract"] = True

    def wrong_feature_typing(graph):
        owner = helpers["package"](graph)
        target = helpers["declaration"](owner, "RestartedA", "PartDefinition")
        usage = helpers["declaration"](owner, "x", "PartUsage")
        invocation = helpers["invocation"](usage)
        edge = helpers["child"](invocation, "ownedRelationship", "FeatureTyping")
        edge["data"]["type"] = target["data"]["elementId"]

    mutations = (
        lambda r: mutate_revision(r, extra_edit),
        lambda r: mutate_revision(r, wrong_feature_typing),
        lambda r: r["requests"]["before"][4].update(notice="Renamed definition."),
        lambda r: r["requests"]["after"][2].update(path=route("alpha", SAVED)),
        lambda r: r["requests"]["after"].pop(),
        lambda r: r["process"].update(fresh_service_process=False),
    )
    for index, mutate in enumerate(mutations, start=1):
        changed = deepcopy(record)
        mutate(changed)
        try:
            check(changed, edit, earlier)
        except ValueError:
            continue
        raise ValueError(f"accepted altered record {index}")
    return len(mutations)


def main():
    record = pinned_json(HERE / "observation.json", OBSERVATION)
    edit = pinned_json(EDIT_DIR / "observation.json", EDIT)
    earlier = pinned_json(HERE.parent / "invocation-rust/observation.json", EARLIER)
    check(record, edit, earlier)
    print(f"Verified 20 HTTP exchanges, two saved graphs, and {reject_mutations(record, edit, earlier)} altered records.")


if __name__ == "__main__":
    main()

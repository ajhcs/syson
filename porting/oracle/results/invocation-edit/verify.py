"""Verify retained edit and reopen evidence without running either product."""
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
PRIOR_DIR = HERE.parent / "invocation-rust"
OBSERVATION = "f190822b7ab0c3dbb3b4ec9f1fe9fd027de9f0cb5030c35e32a10257c589ff07"
PRIOR = "0ecdf06b7a675b8905518fcd1e969537c7bbc056513bc0b474cb3f904b32ef0c"
PARENT = "4a32c1bcadd0833144cd9775237f950d65e3cfe2"
SCOPE = "d3915d19eb14e9a625bbb0ab4bc91f0c98a19b639c5c20cbd5b0b1a7a050fa53"
RECEIPT = "50a6beb1860a712396a422e3c9a896619ad0d2dde4e9cc5e03fc654447b29ab1"
BINARY = "a9f0a55056b9abab56b38d9a966a85c867cdafa665af8b550bf4dd3a4effbda4"
PARTS = {"part-call": False, "abstract-part-call": True}
CALC = "typed-calculation-call"
CONTROLS = [
    "cancel_preserves_original", "cancel_creates_no_history", "stage_isolated_until_commit",
    "undo_restores_original", "redo_restores_candidate", "same_key_refuses_and_preserves_original",
]
KERNEL = (
    "PASS exact anonymous AF_UNIX seqpacket handshake; external socket and alternate shapes denied\n"
    "PASS setpgid and setsid denied by kernel\n"
)


def hash_bytes(data):
    return sha256(data).hexdigest()


def load_prior_helpers():
    source = PRIOR_DIR / "verify.py"
    if hash_bytes(source.read_bytes()) != "dfe0e595c5a50b25dbaf6820942e9fc350b621431a7fe274620b0d59579eac32":
        raise ValueError("changed prior verifier")
    return runpy.run_path(str(source))


helpers = load_prior_helpers()
require = helpers["require"]
decode = helpers["decode"]
artifact = helpers["artifact"]
pinned_json = helpers["file_json"]
child = helpers["child"]
only = helpers["only"]
declaration = helpers["declaration"]
identity = helpers["identifiers"]
endpoints = helpers["endpoints"]


def package(document):
    root = only(document["content"], "root namespace")
    require(root["eClass"] == "sysml:Namespace", "changed root class")
    member = child(root, "ownedRelationship", "OwningMembership")
    package_node = child(member, "ownedRelatedElement", "Package")
    require(package_node["data"]["declaredName"] == "P", "changed package")
    return package_node


def typing(feature, target):
    edge = child(feature, "ownedRelationship", "FeatureTyping")
    endpoints(edge, target, feature)
    return {"relationship": identity(edge), "type": identity(target), "typed_feature": identity(feature)}


def invocation(usage):
    value = child(usage, "ownedRelationship", "FeatureValue")
    return child(value, "ownedRelatedElement", "InvocationExpression")


def inspect_part(document, name, flag):
    owner = package(document)
    target = declaration(owner, name, "PartDefinition")
    require(target["data"].get("isAbstract", False) is flag, "changed abstract flag")
    usage = declaration(owner, "x", "PartUsage")
    return {"abstract": flag, "usage_typing": typing(usage, target),
            "invocation_typing": typing(invocation(usage), target)}


def inspect_calculation(document):
    owner = package(document)
    calculation = declaration(owner, "F", "CalculationDefinition")
    attribute = declaration(owner, "N", "AttributeDefinition")
    usage = declaration(owner, "x", "AttributeUsage")
    membership = child(calculation, "ownedRelationship", "ReturnParameterMembership")
    parameter = child(membership, "ownedRelatedElement", "ReferenceUsage")
    require((parameter["data"]["declaredName"], parameter["data"]["direction"]) == ("r", "out"),
            "changed calculation parameter")
    return {"case": CALC, "original": f"{CALC}-original",
            "invocation_typing": typing(invocation(usage), calculation),
            "return_typing": typing(parameter, attribute)}


def expected_cases(record, originals):
    original_ids = {f"{name}-original": data["sha256"] for name, data in originals.items()}
    require(record["originals_sha256"] == original_ids and set(originals) == {*PARTS, CALC},
            "original snapshots changed")
    candidates = {f"{case}-candidate" for case in PARTS}
    require(set(record["candidate_snapshots"]) == candidates
            and set(record["retained_conflicts_sha256"]) == {f"{case}-original" for case in PARTS},
            "changed edit/staged file set")
    rows = []
    for case, flag in PARTS.items():
        original_key, candidate_key = f"{case}-original", f"{case}-candidate"
        original = decode(artifact(originals[case]))
        candidate_row = record["candidate_snapshots"][candidate_key]
        candidate = decode(artifact(candidate_row))
        require(candidate_row["sha256"] == record["retained_conflicts_sha256"][original_key]
                and candidate_row["sha256"] != original_ids[original_key], "changed retained conflict")
        references = inspect_part(original, "A", flag)
        require(inspect_part(candidate, "RenamedA", flag) == references, "changed edit reference identities")
        restored = deepcopy(candidate)
        declaration(package(restored), "RenamedA", "PartDefinition")["data"]["declaredName"] = "A"
        require(restored == original, "candidate differs beyond the target declaredName")
        rows.append({"case": case, "original": original_key, "candidate": candidate_key,
                     "original_name": "A", "candidate_name": "RenamedA", "references": references})
    rows.append(inspect_calculation(decode(artifact(originals[CALC]))))
    return rows


def check(record, prior):
    require((record["source_commit"], record["prior_observation_sha256"]) == (PARENT, PRIOR), "prior custody differs")
    require((record["execution_scope_sha256"], record["terminal_receipt_sha256"], record["binary_sha256"])
            == (SCOPE, RECEIPT, BINARY), "execution custody differs")
    process = record["process"]
    require(process["blocked"] is False and process["frozen_inputs_unchanged"] is True, "run qualification changed")
    require(artifact(record["kernel_witness"]) == KERNEL, "kernel witness changed")
    proof = {"empty_live_group_samples": 2, "group_sigkill_attempted": True,
             "waitable_leader_retained_until_proof": True}
    require(process["records"] == [
        {"step": step, "exit": 0, "owned_group_quiescent": True, "teardown_proof": proof}
        for step in ("kernel", "build", "stage", "reopen")
    ], "step or teardown witness changed")

    expected = expected_cases(record, prior["attempts"][1]["snapshots"])
    require(set(record["readbacks"]) == {"stage", "reopen"}, "readback phase set changed")
    for phase in ("stage", "reopen"):
        output = decode(artifact(record["readbacks"][phase]))
        require(output == {"phase": phase, "cases": expected,
                           "edit_controls_per_part": CONTROLS if phase == "stage" else []},
                f"{phase} graph or control capture changed")


def mutate_data(value, modify):
    data = decode(value["data"])
    modify(data)
    value["data"] = json.dumps(data, sort_keys=True)
    value["sha256"] = hash_bytes(value["data"].encode())


def mutate_candidate(record, case, modify):
    candidate = record["candidate_snapshots"][f"{case}-candidate"]
    mutate_data(candidate, modify)
    record["retained_conflicts_sha256"][f"{case}-original"] = candidate["sha256"]


def reject_mutations(record, prior):
    def extra_edit(graph):
        package(graph)["data"]["isAbstract"] = True

    def bad_endpoint(graph):
        usage = declaration(package(graph), "x", "PartUsage")
        child(invocation(usage), "ownedRelationship", "FeatureTyping")["data"]["type"] = usage["id"]

    def extra_history(output):
        output["edit_controls_per_part"].append("history_persisted")

    mutations = (
        lambda r: mutate_candidate(r, "part-call", extra_edit),
        lambda r: mutate_candidate(r, "abstract-part-call", bad_endpoint),
        lambda r: r["originals_sha256"].update({"typed-calculation-call-original": "different"}),
        lambda r: r["retained_conflicts_sha256"].update({"part-call-original": "different"}),
        lambda r: mutate_data(r["readbacks"]["reopen"], extra_history),
        lambda r: r["process"]["records"][3]["teardown_proof"].update(empty_live_group_samples=1),
    )
    for index, mutation in enumerate(mutations, start=1):
        changed = deepcopy(record)
        mutation(changed)
        try:
            check(changed, prior)
        except ValueError:
            continue
        raise ValueError(f"accepted mutation {index}")
    return len(mutations)


def main():
    record = pinned_json(HERE / "observation.json", OBSERVATION)
    prior = pinned_json(PRIOR_DIR / "observation.json", PRIOR)
    for field, path in (("probe_lock_sha256", "probe/Cargo.lock"),
                        ("portable_probe_source_sha256", "probe/src/main.rs"),
                        ("portable_probe_manifest_sha256", "probe/Cargo.toml")):
        require(hash_bytes((HERE / path).read_bytes()) == record[field], f"changed probe file {path}")
    check(record, prior)
    print(f"Verified 5 snapshots, 2 retained conflict stages and {reject_mutations(record, prior)} altered records.")


if __name__ == "__main__":
    main()

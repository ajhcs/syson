"""Check the retained Rust readback without running Cargo or starting SysON."""
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
OBSERVATION = "0ecdf06b7a675b8905518fcd1e969537c7bbc056513bc0b474cb3f904b32ef0c"
NATIVE = "5cc9cb070cc32a45ed4a5a3bc89e83576106013f4e43a412a396a975df7430a6"
PARENT = "478ad5a43a2723fac7e87bb52e86649ac7f0026d"
ATTEMPTS = (
    ("26c9c5a861e74b6c2a90e669d0b68190022097ee1734489c28820af07cf11f51",
     "16412eb90794a1cf509b059df5d278530a781f56d8054a286c5379f5bc561a7b"),
    ("d371404f1bb93d54740d33309641ed336e83225bacd91bef9924b2b0b21523da",
     "c71268d08bced0bfcbe9613ca99dd525eac97c0f3d0faf5c5ac18e067fbd6a7d"),
)
BINARY = "fd3279190498388d3fa023cc1100db62296a72acfe5252824063491cc1853781"
CASES = {
    "part-call": ("A", "PartDefinition", False),
    "abstract-part-call": ("A", "PartDefinition", True),
    "typed-calculation-call": ("F", "CalculationDefinition", None),
}
WITNESSES = (
    "PASS socket(AF_INET, AF_INET6, AF_UNIX) denied by kernel\n",
    "PASS exact anonymous AF_UNIX seqpacket handshake; external socket and alternate shapes denied\n",
)
FILES = {
    "probe_lock_sha256": "probe/Cargo.lock",
    "portable_probe_source_sha256": "probe/src/main.rs",
    "portable_probe_manifest_sha256": "probe/Cargo.toml",
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def pairs(items):
    result = dict(items)
    require(len(result) == len(items), "duplicate JSON object key")
    return result


def decode(text):
    return json.loads(text, object_pairs_hook=pairs)


def file_json(path, expected):
    b = path.read_bytes()
    require(sha256(b).hexdigest() == expected, f"changed evidence file {path.name}")
    return decode(b)


def artifact(value):
    require(set(value) == {"data", "sha256"}, "invalid artifact record")
    data = value["data"]
    require(sha256(data.encode()).hexdigest() == value["sha256"], "artifact hash mismatch")
    return data


def only(items, detail):
    found = list(items)
    require(len(found) == 1, detail)
    return found[0]


def child(owner, feature, kind):
    return only((row for row in owner["data"].get(feature, []) if row["eClass"] == f"sysml:{kind}"),
                f"expected one {kind}")


def declaration(package, name, kind):
    return only(
        (node for relation in package["data"]["ownedRelationship"]
         for node in relation["data"].get("ownedRelatedElement", [])
         if (node["eClass"], node["data"].get("declaredName")) == (f"sysml:{kind}", name)),
        f"expected one {kind} {name}",
    )


def identifiers(node):
    ids = {"resource_id": node["id"], "element_id": node["data"]["elementId"]}
    require(ids["resource_id"] != ids["element_id"], "identity roles were collapsed")
    return ids


def endpoints(relation, target, typed):
    data = relation["data"]
    require(data["type"] == target["id"] and data["typedFeature"] == typed["id"], "serialized endpoints changed")


def expected_row(saved, key):
    target_name, target_class, abstract = CASES[key]
    root = only(saved["content"], "expected one captured root")
    require(root["eClass"] == "sysml:Namespace", "expected captured namespace")
    membership = child(root, "ownedRelationship", "OwningMembership")
    package = child(membership, "ownedRelatedElement", "Package")
    target = declaration(package, target_name, target_class)
    usage = declaration(package, "x", "AttributeUsage" if abstract is None else "PartUsage")
    value = child(usage, "ownedRelationship", "FeatureValue")
    invocation = child(value, "ownedRelatedElement", "InvocationExpression")
    endpoints(child(invocation, "ownedRelationship", "FeatureTyping"), target, invocation)

    if abstract is None:
        output = child(target, "ownedRelationship", "ReturnParameterMembership")
        parameter = child(output, "ownedRelatedElement", "ReferenceUsage")
        require(parameter["data"]["declaredName"] == "r" and parameter["data"]["direction"] == "out",
                "calculation return parameter changed")
        parameter_type = declaration(package, "N", "AttributeDefinition")
        endpoints(child(parameter, "ownedRelationship", "FeatureTyping"), parameter_type, parameter)
        ids = identifiers(parameter_type)
        additional = {"return_parameter": "r", "direction": "out", "return_type": "N",
                      "return_type_resource_id": ids["resource_id"], "return_type_element_id": ids["element_id"]}
    else:
        require(target["data"].get("isAbstract", False) is abstract, "part abstract flag changed")
        additional = {"is_abstract": abstract}

    return {"case": key, "target": target_name, "target_class": f"sysml:{target_class}",
            "type": identifiers(target), "typed_feature": identifiers(invocation), "additional": additional,
            "controls_rejected": 6}


def steps(attempt, expected):
    return {"blocked": attempt == 0, "frozen_inputs_unchanged": attempt == 1, "records": [
        {"step": label, "exit": code, "owned_group_quiescent": True} for label, code in expected
    ]}


def check(record, native):
    require(record["source_commit"] == PARENT and record["native_observation_sha256"] == NATIVE,
            "native custody differs")
    require(len(record["attempts"]) == 2, "expected failure and retry")
    failed, passed = record["attempts"]
    for attempt, pins, witness in zip(record["attempts"], ATTEMPTS, WITNESSES, strict=True):
        require((attempt["manifest_sha256"], attempt["terminal_receipt_sha256"]) == pins, "terminal custody differs")
        require(artifact(attempt["kernel_witness"]) == witness, "kernel witness differs")

    shared_keys = {"manifest_sha256", "terminal_receipt_sha256", "kernel_witness", "process"}
    require(set(failed) == shared_keys | {"outcome"}, "unexpected failed attempt artifact")
    require(set(passed) == shared_keys | {"binary_sha256", "readbacks", "snapshots"}, "unexpected retry artifact")
    require(passed["binary_sha256"] == BINARY, "unexpected diagnostic binary")
    require(failed["process"] == steps(0, (("kernel", 0), ("build", 101))), "failed attempt changed")
    require(failed["outcome"] == "Cargo exited 101 before rustc executed with Operation not permitted",
            "failed outcome changed")
    require(passed["process"] == steps(1, (("kernel", 0), ("build", 0), ("stage", 0), ("reopen", 0))),
            "retry receipt changed")

    snapshots = passed["snapshots"]
    require(set(snapshots) == set(CASES) == set(native["cases"]), "unexpected snapshot set")
    expected = []
    for key in CASES:
        live_graph = decode(artifact(native["cases"][key]["files"]["model.emf.json"]))
        rust_graph = decode(artifact(snapshots[key]))
        require(rust_graph == live_graph, f"Rust snapshot differs from released graph for {key}")
        expected.append(expected_row(rust_graph, key))

    readbacks = passed["readbacks"]
    require(set(readbacks) == {"stage", "reopen"}, "unexpected readback set")
    for phase, data in readbacks.items():
        require(decode(artifact(data)) == {"phase": phase, "cases": expected}, f"{phase} graph or controls changed")


def encoded(value):
    data = json.dumps(value, sort_keys=True)
    return {"data": data, "sha256": sha256(data.encode()).hexdigest()}


def reject(record, native, alter):
    changed = deepcopy(record)
    alter(changed["attempts"])
    try:
        check(changed, native)
    except ValueError:
        return
    raise ValueError(f"accepted mutation: {alter.__name__}")


def swap_snapshot_identity(attempts):
    snapshots = attempts[1]["snapshots"]
    key = "part-call"
    data = decode(snapshots[key]["data"])
    data["content"][0]["id"] = data["content"][0]["data"]["elementId"]
    snapshots[key] = encoded(data)


def change_results(attempts, phase, alter):
    readbacks = attempts[1]["readbacks"]
    data = decode(readbacks[phase]["data"])
    alter(data["cases"])
    readbacks[phase] = encoded(data)


def false_typed_feature(rows):
    rows[0]["typed_feature"]["resource_id"] = rows[0]["type"]["resource_id"]


def false_return_type(attempts):
    change_results(attempts, "reopen", lambda rows: rows[2]["additional"].update(return_type="F"))


def same_false_typed_feature(attempts):
    for phase in ("stage", "reopen"):
        change_results(attempts, phase, false_typed_feature)


def omitted_refusal(attempts):
    for phase in ("stage", "reopen"):
        change_results(attempts, phase, lambda rows: rows[0].update(controls_rejected=5))


def omitted_failure(attempts):
    del attempts[0]


def surviving_group(attempts):
    attempts[1]["process"]["records"][3]["owned_group_quiescent"] = False


def main():
    record = file_json(HERE / "observation.json", OBSERVATION)
    native = file_json(HERE.parent / "invocation-emf/observation.json", NATIVE)
    for key, path in FILES.items():
        require(sha256((HERE / path).read_bytes()).hexdigest() == record[key], f"changed probe file {path}")
    check(record, native)
    for alter in (swap_snapshot_identity, false_return_type, same_false_typed_feature, omitted_refusal,
                  omitted_failure, surviving_group):
        reject(record, native, alter)
    print("Verified 3 released invocation graphs, Rust save/reopen and 6 altered records.")


if __name__ == "__main__":
    main()

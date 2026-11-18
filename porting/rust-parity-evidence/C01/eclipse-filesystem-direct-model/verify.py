"""Verify the replay of 25 Filesystem POMs through direct Maven ModelBuilder."""
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
SEALS = {
    "observation.json": "79467ff3d54bceb01955245346240aae910f5d3c7c81cf8ad7d2c52c7e7d0f15",
    "FilesystemModelProbe.java": "af2ff9555746a68af486f82ca8b3482a9dcca4a629cdeda00079b12a36b94691",
    "../eclipse-filesystem-descriptor-passive-run/observation.json":
        "91f18d33d16b20aea6b1d7347a3154e3b18d2bfb23d9ca3d7ecb4d9f1e4bf25d",
}
GROUP = "org.eclipse.platform"
ARTIFACT = "org.eclipse.core.filesystem"
REJECTED = "1.9.500"
RANGE = "[3.2.0,4.0.0)"
PROBLEM = "'modelVersion' must be one of [4.0.0] but is '4.0'."
HEADER = [["java", "21.0.12.1+1-LTS"], ["maven-model-builder", "3.9.16"]]
TUPLE_FIELDS = ["group_id", "artifact_id", "version", "type_with_default", "classifier",
                "scope_with_default", "optional_literal", "optional", "exclusion_count"]


def require(condition, error):
    if not condition:
        raise ValueError(error)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read(name):
    raw = (HERE / name).read_bytes()
    require(digest(raw) == SEALS[name], f"byte seal changed: {name}")
    return raw


def trace_digest(rows):
    return digest("".join("\t".join(row) + "\n" for row in rows).encode("utf-8"))


def split_groups(trace):
    require(trace[:2] == HEADER and trace[-1] == ["terminal", "direct-model-only", "no-dependency-graph-or-binary"],
            "direct ModelBuilder trace header or terminal changed")
    groups = []
    for row in trace[2:-1]:
        if row[0] == "case":
            groups.append([])
        require(groups, "unexpected unbound model record")
        groups[-1].append(row)
    return groups


def read_raw_rejected(obs):
    expected = [[GROUP, artifact, RANGE, "jar", "", "compile", "false", "false", "0"]
                for artifact in ("org.eclipse.equinox.common", "org.eclipse.equinox.registry", "org.eclipse.osgi")]
    require(obs["rejected_raw_direct_tuple_fields"] == TUPLE_FIELDS and obs["rejected_raw_direct_tuples"] == expected,
            "rejected raw declarations changed")
    return expected


def check_model_cases(cases, capture):
    require(set(cases) == {"direct_model", "wrong_hash"}, "unexpected replay case roster")
    direct, wrong = cases["direct_model"], cases["wrong_hash"]
    require(wrong["records"] == HEADER + [["terminal", "wrong-hash-refused", "before-model-builder"]],
            "wrong hash did not stop before ModelBuilder")
    for case in cases.values():
        require(case["stdout_sha256"] == trace_digest(case["records"]), "trace hash changed")
    groups = split_groups(direct["records"])
    declarations = capture["raw_model_declarations"]
    require(len(groups) == len(declarations) == 25 and len(capture["request_rows"]) == 125,
            "expected one case for each captured POM")
    models, deps, ranges = 0, 0, 0
    for i, (records, (version, spelling, _)) in enumerate(zip(groups, declarations, strict=True)):
        code, _, input_hash = capture["request_rows"][i * 5]
        require(code == 200 and records[:3] == [
            ["case", version, input_hash],
            ["repository-ignored", "central", "https://repo.maven.apache.org/maven2", "true"],
            ["maven-raw-model", f"{GROUP}:{ARTIFACT}:{version}", spelling],
        ], f"source, repository guard or literal model version changed: {version}")
        if version == REJECTED:
            require(spelling == "4.0" and records[3:] == [["problem", "ERROR", PROBLEM], ["built", "false"]],
                    "rejected model or error diagnostic changed")
            continue
        require(spelling == "4.0.0" and len(records) >= 8
                and records[3] == ["effective-model", GROUP, ARTIFACT, version, spelling, "jar"]
                and records[-1] == ["built", "true"], f"unexpected model acceptance: {version}")
        models += 1
        for row in records[4:-1]:
            require(len(row) == 10 and row[0] == "direct", f"unexpected effective tuple: {version}")
            _, group, artifact, dep_version, kind, classifier, scope, optional, is_optional, exclusions = row
            require(group and artifact and (dep_version == RANGE or re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", dep_version))
                    and (kind, classifier, scope, is_optional, exclusions) == ("jar", "", "compile", "false", "0")
                    and optional in ("false", ""), f"unexpected direct dependency fields: {version}")
            deps += 1
            ranges += dep_version == RANGE
    return {"inputs": len(groups), "effective_models": models, "effective_direct": deps,
            "effective_fixed": deps - ranges, "effective_ranges": ranges}


def check_negatives(cases, capture):
    def mutate_input(rows):
        rows[2][2] = "0" * 64

    def mutate_rejection(rows):
        next(row for row in rows if row[0] == "problem")[2] = "unexpected validation error"

    candidates = (
        ("input hash", "direct_model", mutate_input),
        ("diagnostic", "direct_model", mutate_rejection),
        ("model lookup", "direct_model", lambda rows: rows.insert(4, ["lookup-refused", "parent", "x", "y", "1"])),
        ("dependency scope", "direct_model", lambda rows: next(row for row in rows if row[0] == "direct").__setitem__(6, "test")),
        ("graph terminal", "direct_model", lambda rows: rows[-1].__setitem__(1, "graph-complete")),
        ("wrong hash bypass", "wrong_hash", lambda rows: rows[-1].__setitem__(2, "after-model-builder")),
    )
    for label, arm, mutate in candidates:
        changed = deepcopy(cases)
        rows = changed[arm]["records"]
        mutate(rows)
        changed[arm]["stdout_sha256"] = trace_digest(rows)
        try:
            check_model_cases(changed, capture)
        except ValueError:
            continue
        raise ValueError(f"accepted resealed negative: {label}")
    return len(candidates)


def verify():
    read("FilesystemModelProbe.java")
    capture = json.loads(read("../eclipse-filesystem-descriptor-passive-run/observation.json"))
    obs = json.loads(read("observation.json"))
    require(obs["schema"] == "SYSON-C01-ECLIPSE-FILESYSTEM-DIRECT-MODEL/1"
            and obs["status"] == "OBSERVED_PAIRED_DATA_ONLY_REPLAY"
            and obs["capture_commit"] == "20aa3eda9bd7a641d1af2c39381cffd41efe3da0"
            and obs["probe_sha256"] == SEALS["FilesystemModelProbe.java"]
            and obs["api"] == "org.apache.maven.model.building.ModelBuilder"
            and obs["validation"] == {"constant": "VALIDATION_LEVEL_MAVEN_3_0", "level": 30},
            "probe identity or validation mode changed")
    cases = obs["cases"]
    counts = check_model_cases(cases, capture)
    rejected = read_raw_rejected(obs)
    counts["raw_direct"] = counts["effective_direct"] + len(rejected)
    require(counts == obs["counts"] == {
        "inputs": 25, "effective_models": 24, "raw_direct": 93,
        "effective_direct": 90, "effective_fixed": 51, "effective_ranges": 39,
    }, "model or dependency totals changed")
    require(obs["process_exits"] == {"kernel": 0, "compiler": 0, "direct_model": 0, "wrong_hash": 41},
            "unexpected process outcomes")
    kernel = b"PASS socket(AF_INET, AF_INET6, AF_UNIX) and io_uring_setup denied by kernel\n"
    require(obs["controls"] == {
        "inputs_match_before_and_after": True, "controller_exit": 0,
        "owned_scope_confirmed_empty_or_absent": True, "kernel_witness_sha256": digest(kernel),
        "network": False, "model_lookup": False, "descriptor_reader": False,
        "dependency_graph": False, "final_version_selected": False, "binary_resolution": False,
        "cache_installation": False, "project_execution": False,
    }, "offline, qualification, or teardown controls changed")
    negatives = check_negatives(cases, capture)
    print(f"PASS 25 exact inputs, 24 effective models, 90 direct tuples, wrong hash refused, {negatives} negative checks")


if __name__ == "__main__":
    verify()

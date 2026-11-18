"""Verify the retained JDT direct descriptor traces and the guard stop."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SEALS = {
    "observation.json": "b12ec467f728d8c1137ad8adb0f7f16c39be0cba8d664fa476196d4bf5f848f7",
    "../eclipse-jdt-core-model-replay/models.tsv": "4b5c8939545d3ac28e0c19e2e03ed17c43653bc498cdcf26fec7029a76fb7a28",
    "../eclipse-jdt-core-model-replay/observation.json":
        "e56df30cf4fca41d805ddbbc59278b0c88fb367b9797fd05287c304851a999e0",
    "../eclipse-jdt-core-passive-run/observation.json":
        "bb7e2818adf6fe8a62b43293e1d7f5e0ae1b2bc109dc0432a95696366dd22896",
}
JDT = "org.eclipse.jdt:org.eclipse.jdt.core"
VERSIONS = ["3.30.0", "3.31.0", "3.32.0", "3.47.0"]
REJECTED = "3.31.0"
MODES = ("diagnostic", "native-policy")
PROBLEM = "'modelVersion' must be one of [4.0.0] but is '4.0'."
ROOT = "org.eclipse.emf:org.eclipse.emf.codegen:2.28.0"
FAILED_TASK = "diagnostic_codegen-ancestor"


def require(condition, error):
    if not condition:
        raise ValueError(error)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read(name):
    raw = (HERE / name).read_bytes()
    require(digest(raw) == SEALS[name], f"unexpected byte seal: {name}")
    return raw


def baseline_edges(models, anomaly):
    result = {}
    current = None
    for line in models.decode("utf-8", "strict").splitlines():
        row = line.split("\t")
        if row[0] == "case":
            current = row[1]
            result[current] = []
        elif row[0] == "direct" and current in VERSIONS:
            group, artifact, version, kind, classifier, scope, _, optional, exclusions = row[1:]
            require(exclusions == "0", f"unhandled excluded dependency in {current}")
            result[current].append([f"{group}:{artifact}:{version}", kind, classifier, scope, optional])
    require(anomaly["rejected_version"] == REJECTED and not result[REJECTED], "unexpected #371 outcome")
    result[REJECTED] = [[f"{g}:{a}:{v}", "jar", "", "compile", str(optional).lower()]
                        for g, a, v, optional in anomaly["raw_direct_in_rejected_model"]]
    return [result[version] for version in VERSIONS]


def expected_direct_trace(mode, expected):
    trace = [["case", mode, "direct-controls"], ["tools", "21.0.12.1+1-LTS", "3.9.16", "1.9.27"]]
    source = mode + ":direct-controls"
    for version, edges in zip(VERSIONS, expected):
        coordinate = f"{JDT}:{version}"
        request = coordinate + ":jar:"
        trace.append(["descriptor", request, "pinned"])
        if mode == "diagnostic" and version == REJECTED:
            detail = "1 problem was encountered while building the effective model for " + coordinate
            detail += f"\\n[ERROR] {PROBLEM} @ line 17, column 17\\n"
            trace.extend([
                ["descriptor-outcome", source, "project", coordinate, "", "diagnostic-model-rejected", "0", detail],
                ["descriptor-problem", request, "ERROR", PROBLEM],
            ])
            continue
        count = str(len(edges))
        if mode == "diagnostic":
            trace.append(["descriptor-model", request, count, "0", "1"])
            status = "diagnostic-built"
        else:
            trace.append(["repository-aggregation", "inherited-super-pom-central", "1"])
            status = "native-built"
        trace.append(["descriptor-outcome", source, "project", coordinate, "", status, count, ""])
        trace.extend(["descriptor-child", source, "project", coordinate, *edge, ""] for edge in edges)
        if mode == "native-policy":
            trace.append(["native-declared-existing-repository", coordinate, "central", "https://repo.maven.apache.org/maven2"])
    return trace


def check_stop(evidence):
    tasks = ["kernel", "compiler"] + [
        f"{mode}_{case}" for case in ("metadata-only", "withheld-metadata", "direct-controls",
                                      "codegen-ancestor", "ecore-ancestor", "target", "withheld-highest")
        for mode in MODES
    ] + ["diagnostic_guard-" + case + "-control" for case in
         ("drift", "repository", "fixed-spelling", "snapshot", "malformed-range")]
    stop = tasks.index(FAILED_TASK)
    exits = {**dict.fromkeys(tasks[:stop], 0), FAILED_TASK: 1}
    require(evidence["planned_task_order"] == tasks and evidence["observed_exits"] == exits
            and evidence["unrun_task_count"] == len(tasks[stop + 1:]) == 12, "task stop boundary changed")
    require(evidence["failure"] == {
        "task": FAILED_TASK,
        "process_exit": 1,
        "wrapper_failure": "ValueError: unexpected child exit: " + FAILED_TASK,
        "root_descriptor_outcome": ["descriptor-outcome", "diagnostic:codegen-ancestor", "project", ROOT,
                                    "collect:", "diagnostic-built", "7", ""],
        "observed_root_direct_count": 7,
        "guard_terminal": ["guard-violation", "diagnostic:codegen-ancestor", "repository aggregation outside scope"],
        "traversal_records": 0,
    }, "unexpected guard terminal or traversal")


def verify():
    evidence = json.loads(read("observation.json"))
    captured = json.loads(read("../eclipse-jdt-core-passive-run/observation.json"))
    model_trace = read("../eclipse-jdt-core-model-replay/models.tsv")
    anomaly = json.loads(read("../eclipse-jdt-core-model-replay/observation.json"))
    require(evidence["status"] == "RETAINED_PARTIAL_OR_UNQUALIFIED" and
            evidence["model_baseline_commit"] == "8487ccd2f2b64a4d4671c32406767111ebde6934",
            "unexpected baseline or receipt status")
    require([row["version"] for row in evidence["pom_inputs"]] == VERSIONS, "unbound input roster")
    for row in evidence["pom_inputs"]:
        index = int(row["version"].split(".")[1]) - 26
        require(captured["request_rows"][5 * index] == [200, row["bytes"], row["sha256"]], "passive byte pin changed")
        spelling = "4.0" if row["version"] == REJECTED else "4.0.0"
        require(captured["model_versions"][index] == [row["version"], spelling], "original spelling changed")
    require(list(evidence["direct_controls"]) == list(MODES), "arm roster changed")
    edges = baseline_edges(model_trace, anomaly)
    require([len(group) for group in edges] == [5, 5, 5, 6], "unexpected baseline edge counts")
    for mode, result in evidence["direct_controls"].items():
        records = result["records"]
        require(records == expected_direct_trace(mode, edges), f"unexpected direct comparison: {mode}")
        raw = "".join("\t".join(row) + "\n" for row in records).encode("utf-8")
        require(digest(raw) == result["stdout_sha256"], f"direct trace digest changed: {mode}")
    check_stop(evidence)
    controls = evidence["controls"]
    kernel = "PASS socket(AF_INET, AF_INET6, AF_UNIX) and io_uring_setup denied by kernel"
    require(controls == {
        "inputs_match_before_and_after": True, "kernel_witness_lines": [kernel],
        "kernel_witness_sha256": digest((kernel + "\n").encode()),
        "owned_scope_confirmed_empty_or_absent": True, "network": False, "binary_resolution": False,
        "cache_installation": False, "project_execution": False,
    }, "unexpected offline guards or scope cleanup")
    print("PASS direct descriptor comparison, 3.31.0 native edges, and graph guard stop before traversal")


if __name__ == "__main__":
    verify()

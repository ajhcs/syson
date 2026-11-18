"""Verify the projected JDT collection observations and the retained frontier."""
from collections import Counter
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SEALS = {
    "observation.json": "ed7395ad79d0e055373e1a698d12e73adb5c5dc92692929f66761380c3df8f22",
    "../eclipse-jdt-descriptor-direct/observation.json":
        "b12ec467f728d8c1137ad8adb0f7f16c39be0cba8d664fa476196d4bf5f848f7",
}
PHASES = ("codegen-ancestor", "ecore-ancestor", "target", "withheld-highest")
MODES = ("diagnostic", "native-policy")
CASES = [f"{mode}_{phase}" for phase in PHASES for mode in MODES]
JDT = "org.eclipse.jdt:org.eclipse.jdt.core"
CODEGEN = "org.eclipse.emf:org.eclipse.emf.codegen:2.28.0"
ECORE = "org.eclipse.emf:org.eclipse.emf.codegen.ecore:2.46.0"
TARGET = "org.eclipse.syson:syson-sysml-metamodel:2026.9.3"
FILESYSTEM = "org.eclipse.platform:org.eclipse.core.filesystem"
JDT_NODE = JDT + ":3.47.0:jar:"
NOOP_COUNTS = (23, 24, 49, 51, 83, 85, 81, 83)
NODE_COUNTS = (10, 10, 14, 14, 60, 60, 58, 58)
HOLD_COUNTS = (198, 198, 198, 198, 219, 219, 218, 218)
ROLES = ("prior-range", "prior-range,observed-jdt-fixed-coordinate", "observed-jdt-fixed-coordinate",
         "prior-target-fixed", "prior-range,withheld-highest-control")
GUARDS = {
    "drift": "range outside the exact admitted allowlist: " + JDT + " [3.27.0,4.0.0)",
    "repository": "repository aggregation outside scope",
    "fixed-spelling": "fixed-version scope drift: " + JDT + ":not-a-release:jar:",
    "snapshot": "fixed-version scope drift: " + JDT + ":3.47.0-SNAPSHOT:jar:",
    "malformed-range": "malformed version: " + JDT + ":[3.26.0,4.0.0:jar:",
}


def require(condition, error):
    if not condition:
        raise ValueError(error)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def load(name):
    data = (HERE / name).read_bytes()
    require(digest(data) == SEALS[name], f"byte seal changed: {name}")
    return json.loads(data)


def check_guard_cases(observation):
    outcomes = observation["guard_cases"]
    tasks = {f"diagnostic_guard-{control}-control": marker for control, marker in GUARDS.items()}
    expected_exits = {"kernel": 0, "compiler": 0, **dict.fromkeys(tasks, 1), **dict.fromkeys(CASES, 0)}
    require(list(outcomes) == list(tasks) and list(observation["observed_process_exits"].items()) ==
            list(expected_exits.items()), "guard or invocation roster changed")
    for label, guard in outcomes.items():
        name = label.removeprefix("diagnostic_")
        rows = guard["records"]
        require(guard["process_exit"] == 1 and
                rows[:2] == [["case", "diagnostic", name], ["tools", *observation["tools"]]] and
                rows[-1] == ["guard-violation", f"diagnostic:{name}", tasks[label]],
                f"unexpected guard result: {name}")
        require(all(row[0] == "version" and row[-1] in ("malformed", "not-admitted") for row in rows[2:-1]),
                f"guard returned an unexpected event: {name}")
        raw = "".join("\t".join(row) + "\n" for row in rows).encode("utf-8")
        require(digest(raw) == guard["stdout_sha256"], f"guard transcript changed: {name}")


def check_frontier(observation):
    frontier = observation["frontier"]
    descriptors = frontier["descriptor_holds"]
    require(frontier["descriptor_row_fields"] == ["request_context", "coordinate", "hold_roles", "hits_by_case_order"]
            and len(descriptors) == len({row[1] for row in descriptors}) == 220
            and all(context == "project" and roles in ROLES and len(hits) == len(CASES) and
                    any(hits) and all(type(h) is int and h in (0, 1) for h in hits)
                    for context, _, roles, hits in descriptors),
            "descriptor frontier id, context, roles or hits changed")
    for i, case in enumerate(CASES):
        counts = Counter(roles for _, _, roles, hits in descriptors if hits[i])
        expected = [130, 41, 25 if i >= 6 else 27, 21 if i >= 4 else 0, 1 if i >= 6 else 0]
        require([counts[role] for role in ROLES] == expected and
                sum(counts.values()) == HOLD_COUNTS[i] == observation["graph_cases"][case]["callback_count"]["descriptor-held"],
                f"descriptor frontier counts changed: {case}")
    require(["project", JDT + ":3.47.0", ROLES[-1], [0, 0, 0, 0, 0, 0, 1, 1]] in descriptors,
            "withheld-highest hold changed")
    require(frontier["range_row_fields"] == ["coordinate", "constraint", "hold_reason", "hits_by_case_order"]
            and frontier["range_holds"] ==
            [[FILESYSTEM, "[1.7.0,2.0.0)", "metadata-not-admitted", [5, 6] * 4]],
            "held metadata range changed")
    require(frontier["model_row_fields"] == ["owner", "request_role", "coordinate", "result", "hits_by_case_order"]
            and frontier["model_holds"] ==
            [["org.apache.commons:commons-text:1.10.0", "coordinate", "org.junit:junit-bom:5.9.1", "absent", [0] * 4 + [1] * 4]],
            "held model request changed")


def check_graphs(observation, earlier):
    cases = observation["graph_cases"]
    require(observation["case_order"] == list(cases) == CASES, "graph case roster changed")
    require(observation["common_jdt_offer"] == {
        "coordinate": JDT, "constraint": "[3.26.0,4.0.0)", "ordered_versions": [f"3.{i}.0" for i in range(26, 48)],
    }, "JDT offer roster changed")
    for index, (label, case) in enumerate(cases.items()):
        mode, phase = label.split("_", 1)
        direct = earlier["direct_controls"][mode]["records"]
        root = (CODEGEN, ECORE, TARGET, TARGET)[index // 2]
        pivot = CODEGEN if root == CODEGEN else ECORE
        depth = 2 if root == TARGET else 1
        lineage = "CollectStepDataImpl <= collect:" + (root if root == TARGET else "")
        require((case["root"], case["jdt_lineage_anchor"], case["collector_trace_context"]) == (root, pivot, lineage),
                f"unexpected root/context: {label}")
        require(case["terminal"] == ["partial", "no-binary-resolution"] and
                (case["collection_exceptions"], case["collection_cycles"]) == (50, 0),
                f"unexpected collection terminal: {label}")
        require(case["repository_noop_raw_flag_counts"] == {"true": NOOP_COUNTS[index]},
                f"unexpected no-op repository callback: {label}")
        require(set(case["selector_outcome_counts"]) == {"selected", "excluded"} and
                sum(case["selector_outcome_counts"].values()) == case["callback_count"]["selection"]
                and case["callback_count"]["observed-node"] == NODE_COUNTS[index]
                and case["callback_count"]["range-held"] == [5, 6][index % 2],
                f"unexpected selector, node, or range counts: {label}")
        expected = []
        for row in direct:
            if row[0] != "descriptor-outcome" or row[3] not in (JDT + ":3.31.0", JDT + ":3.47.0"):
                continue
            if phase == "withheld-highest" and row[3] == JDT + ":3.47.0":
                continue
            expected.append([f"{mode}:{phase}", "project", row[3], lineage, *row[5:]])
        require(case["descriptor_outcomes"] == expected, f"unexpected descriptor stage: {label}")
        require(case["partial_jdt_nodes"] == [[str(depth), pivot + ":jar:", JDT_NODE, "compile"]],
                f"unexpected JDT partial node: {label}")
        children = [[str(depth + 1), JDT_NODE, child, "compile"] for child in (
            FILESYSTEM + ":1.11.500:jar:", "org.eclipse.jdt:ecj:3.46.100:jar:")]
        require(case["partial_jdt_children"] == ([] if phase == "withheld-highest" else children),
                f"unexpected JDT partial-node children: {label}")


def verify():
    observation = load("observation.json")
    earlier = load("../eclipse-jdt-descriptor-direct/observation.json")
    require(observation["status"] == "OBSERVED_PAIRED_DATA_ONLY_REPLAY" and earlier["unrun_task_count"] == 12
            and observation["prior_direct_commit"] == "48a4b4a8acc98ddcb19b57704823c51bf1721429"
            and observation["tools"] == ["21.0.12.1+1-LTS", "3.9.16", "1.9.27"], "attempt lineage changed")
    check_guard_cases(observation)
    check_graphs(observation, earlier)
    check_frontier(observation)
    kernel = "PASS socket(AF_INET, AF_INET6, AF_UNIX) and io_uring_setup denied by kernel"
    require(observation["controls"] == {
        "launcher_exit": 0, "inputs_match_before_and_after": True, "kernel_witness": kernel,
        "kernel_witness_sha256": digest((kernel + "\n").encode()), "owned_scope_confirmed_empty_or_absent": True,
        "network": False, "binary_resolution": False, "cache_installation": False, "project_execution": False,
    }, "offline or teardown controls changed")
    print("PASS eight partial graph cases, five exact guard negatives, and 220 held descriptor coordinates")


if __name__ == "__main__":
    verify()

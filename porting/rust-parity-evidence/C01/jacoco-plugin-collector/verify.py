"""Verify the sanitized output of the bounded JaCoCo plugin collector."""
import hashlib
import json
from pathlib import Path
import re

SITE = Path(__file__).resolve().parent
PARENT_SHA = "54ae5eecbff32bd079c33709a93e466065178c4c48bcdf50432cf2465afd5bed"
OBSERVATION_SHA = "157c24646c72c580b15cdc0ea3d529edd38bbd160c76b12d8bbc5125e90eb3d4"
ROOT = "org.jacoco:jacoco-maven-plugin:0.8.14:jar:"
WITHHELD = "org.apache.maven.shared:file-management:3.2.0:jar:"
RANGE = "org.apache.maven.shared:file-management:[3.2.0,3.3.0):jar:"
LOSER = "org.codehaus.plexus:plexus-utils:4.0.2:jar:"
WINNER = "org.codehaus.plexus:plexus-utils:3.0.24:jar:"
AGENT = "org.jacoco:org.jacoco.agent:0.8.14:jar:runtime"


def require(condition, error):
    if not condition:
        raise ValueError(error)


def load(path, expected):
    require(path.stat().st_size <= 65536, "input bound exceeded")
    data = path.read_bytes()
    require(hashlib.sha256(data).hexdigest() == expected, f"digest changed: {path.name}")
    return json.loads(data)


def verify(observed, parent):
    require(observed["schema"] == "C01-JACOCO-POM-COLLECTION-OBSERVATION/1" and
            observed["parent_observation_sha256"] == PARENT_SHA, "unexpected protocol or parent")
    for pin in ("runtime_receipt_sha256", "scope_spec_sha256"):
        require(re.fullmatch(r"[0-9a-f]{64}", observed[pin]), f"unsealed {pin}")
    expected = {"compiler", "kernel", "baseline", "ci-property", "range-control", "withheld-pom"}
    require(set(observed["exits"]) == expected and
            all(type(v) is int and v == 0 for v in observed["exits"].values()), "nonzero probe exit")
    require(observed["runtime"] == {
        "maven_model_builder": "3.9.16", "resolver": "1.9.27", "jdk": "21.0.12.1+1-LTS",
        "offline": True, "kernel_socket_families_denied": ["AF_INET", "AF_INET6", "AF_UNIX"],
        "plugin_goals_run": False, "dependency_binaries_resolved": False, "native_build": False,
    }, "incorrect execution boundary")

    direct = []
    for line in parent["cases"]["baseline"]["stdout"].splitlines():
        if line.startswith("direct\t"):
            _, group, artifact, version, extension, classifier, scope = line.split("\t")
            direct.append([":".join((group, artifact, version, extension, classifier)), scope,
                           "false", "excluded" if scope == "provided" else "selected"])
    require(len(direct) == 9 and observed["root_selectors"] == direct, "plugin scope or classifier drift")
    require(observed["selector_row_count"] == 37, "selector count changed")
    provided = {row[0] for row in direct if row[1] == "provided"}
    require(len(provided) == 3, "provided declarations missing")

    graph = observed["baseline_ci_graph"]
    require(len(graph) == 13 and graph[0] == ["0", "", ROOT, "", "would-resolve"],
            "wrong mediated graph root")
    seen = set()
    lineage = []
    for depth, owner, artifact, scope, action in graph:
        require(depth in {"0", "1", "2"} and artifact not in seen and action == "would-resolve",
                "invalid graph row")
        if depth != "0":
            index = int(depth) - 1
            require(index < len(lineage) and owner == lineage[index] and scope == "compile",
                    "invalid graph ownership or scope")
            lineage[index + 1:] = [artifact]
        else:
            lineage.append(artifact)
        seen.add(artifact)
    require({r[0] for r in direct if r[1] == "compile"} == {r[2] for r in graph if r[0] == "1"},
            "direct selected dependencies changed")
    descriptors = observed["descriptor_inputs"]
    require(len(descriptors) == len(set(descriptors)) == 14 and set(descriptors) == seen | {LOSER} and
            not provided & set(descriptors), "descriptor membership drift")
    require({WINNER, AGENT} <= seen and LOSER not in seen, "conflict selection or classifier changed")

    cases = observed["cases"]
    require(set(cases) == expected - {"compiler", "kernel"}, "case set changed")
    for name, case in cases.items():
        require(re.fullmatch(r"[0-9a-f]{64}", case["stdout_sha256"]) and not case["missing_models"],
                "unsealed observation or missing model")
        if name in ("baseline", "ci-property"):
            require(case["terminal"] == ["collected", "binary-boundary-blocked"] and
                    case["descriptor_count"] == 14 and not case["missing_descriptors"] and
                    not case["denied_versions"], "positive case drift")
        else:
            expected_missing = [WITHHELD] if name == "withheld-pom" else []
            expected_range = [RANGE] if name == "range-control" else []
            expected_count = 11 if name == "withheld-pom" else 10
            require(case["terminal"] == ["partial", "binary-boundary-not-reached"] and
                    case["missing_descriptors"] == expected_missing and
                    case["denied_versions"] == expected_range and case["descriptor_count"] == expected_count,
                    "negative control drift")
    inventory = observed["binary_path_inventory"]
    counts = inventory["matching_regular_file_counts"]
    require(set(counts) == seen and all(type(n) is int and 0 <= n <= 2 for n in counts.values()) and
            sum(n > 0 for n in counts.values()) == 3 and inventory["trees_checked"] == 16 and
            inventory["binaries_checked_or_admitted_for_execution"] is False, "artifact inventory drift")


def main():
    parent = load(SITE.parent / "jacoco-plugin-own-model" / "observation.json", PARENT_SHA)
    observed = load(SITE / "observation.json", OBSERVATION_SHA)
    verify(observed, parent)
    print("PASS: 13 mediated nodes, 14 descriptor visits, two fail-closed controls")


if __name__ == "__main__":
    main()

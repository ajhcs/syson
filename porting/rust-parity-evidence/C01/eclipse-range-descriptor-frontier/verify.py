"""Check the sanitized, captured C01 Maven replay projection without subprocesses."""
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent
OBSERVATION_SHA256 = "459861665f2c3798672748b10ba2470c76f8c8931230d19eaaaae10dc7475dbe"
TARGET = "org.eclipse.syson:syson-sysml-metamodel:2026.9.3"
ECORE = "org.eclipse.emf:org.eclipse.emf.codegen.ecore:2.46.0"
CODEGEN = "org.eclipse.emf:org.eclipse.emf.codegen:2.28.0"
RANGES = (
    ("org.eclipse.jdt:org.eclipse.jdt.core", "[3.26.0,4.0.0)", 43, 22),
    ("org.eclipse.jdt:org.eclipse.jdt.launching", "[3.5.0,4.0.0)", 40, 40),
    ("org.eclipse.platform:org.eclipse.core.resources", "[3.8.0,4.0.0)", 35, 35),
    ("org.eclipse.platform:org.eclipse.core.runtime", "[3.9.0,4.0.0)", 33, 33),
    ("org.eclipse.platform:org.eclipse.text", "[3.5.0,4.0.0)", 29, 29),
    ("org.eclipse.platform:org.eclipse.debug.core", "[3.6.0,4.0.0)", 34, 34),
)
AUTHORITY = {
    "network", "dependency_binaries_resolved", "plugin_goals", "full_native_build",
    "final_target_versions_selected", "full_target_repository_policy_replayed",
}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def is_digest(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def canonical_digest(value):
    return sha(json.dumps(value, separators=(",", ":"), ensure_ascii=True).encode("ascii"))


def unique(fields):
    result = {}
    for key, value in fields:
        require(key not in result, "duplicate observation key")
        result[key] = value
    return result


def numeric(value):
    require(isinstance(value, str) and re.fullmatch(r"(0|[1-9][0-9]*)(\.(0|[1-9][0-9]*))+", value),
            "unexpected captured version spelling")
    return tuple(map(int, value.split(".")))


def pom_path(coordinate):
    require(isinstance(coordinate, str) and re.fullmatch(r"[A-Za-z0-9][\w.-]*:[A-Za-z0-9][\w.-]*:[0-9][\w.-]*",
                                                       coordinate, re.ASCII), "invalid POM coordinate")
    group, artifact, version = coordinate.split(":")
    require(all(group.split(".")) and not artifact.endswith(".") and not version.endswith("."),
            "noncanonical POM coordinate")
    return f"{group.replace('.', '/')}/{artifact}/{version}/{artifact}-{version}.pom"


def validate(data):
    require(data["schema"] == "c01-eclipse-range-descriptor-frontier-v1", "wrong schema")
    require(data["tools"] == {"maven": "3.9.16", "resolver": "1.9.27", "jdk": "21.0.12.1+1-LTS"}
            and data["target_revision"] == "6ba2ac64e6b16ee2b05752ad55395eebb87d4910", "tool or target drift")
    require(is_digest(data["scope_sha256"]) and is_digest(data["receipt_sha256"]), "missing run evidence binding")
    require(set(data["authority"]) == AUTHORITY and all(v is False for v in data["authority"].values()),
            "replay claims more than captured")
    require(data["declared_target_project_repositories"] ==
            ["Central", "github-sirius-emfjson", "github-sirius-web", "central"]
            and data["replayed_metadata_origins"] == ["empty-local-repository", "captured-maven-central"],
            "repository scope drift")
    families = data["metadata_families"]
    require(len(families) == len(RANGES), "wrong range count")
    offered_gavs = []
    for row, (ga, interval, raw_count, offered_count) in zip(families, RANGES, strict=True):
        require((row["ga"], row["interval"]) == (ga, interval) and is_digest(row["metadata_sha256"]), "range input drift")
        raw = row["raw_versions"]
        require(len(raw) == len(set(raw)) == raw_count, "wrong raw version count")
        lo, hi = map(numeric, interval[1:-1].split(","))
        offered = [v for v in sorted(raw, key=numeric) if lo <= numeric(v) < hi]
        require(len(offered) == offered_count and offered == row["offered_versions"], "observed offers drift")
        offered_gavs.extend(f"{ga}:{v}" for v in offered)
    offered_gavs.sort()
    require(len(offered_gavs) == 193 and data["missing_range_descriptor_gavs"] == offered_gavs, "range frontier drift")

    graph_shapes = {
        "codegen-ancestor": (CODEGEN, 195, 7),
        "ecore-ancestor": (ECORE, 199, 8),
        "target": (TARGET, 245, 9),
    }
    require(set(data["cases"]) == {*graph_shapes, "metadata-only", "withheld-metadata"}, "wrong case partition")
    for name, row in data["cases"].items():
        require(is_digest(row["stdout_sha256"]), "missing trace binding")
        counters = (
            row["metadata_request_count"], row["missing_local_metadata_requests"],
            row["pinned_central_metadata_requests"], row["missing_central_metadata_requests"],
            row["offered_version_count"], row["descriptor_request_rows"], row["missing_range_descriptor_count"],
        )
        require(all(type(value) is int for value in counters), "invalid counter type")
        if name in graph_shapes:
            root, count, direct_count = graph_shapes[name]
            require(row["terminal"] == ["partial", "no-binary-resolution"] and row["root"] == root
                    and counters == (12, 6, 6, 0, 193, count, 193), "partial graph counters or terminal drift")
            require(row["attempted_range_gavs_sha256"] == canonical_digest(offered_gavs), "attempted descriptors differ")
            expected_direct = row["effective_direct_dependencies"]
            require(len(expected_direct) == direct_count, "effective direct set changed")
            for group, artifact, version, kind, classifier, role in expected_direct:
                require(kind == "jar" and classifier == "" and role in ("compile", "test"), "direct dependency role changed")
                ga = f"{group}:{artifact}"
                if version.startswith("["):
                    require((ga, version) in {(g, v) for g, v, _, _ in RANGES}, "unexpected direct range")
                else:
                    pom_path(f"{ga}:{version}")
            if name != "target":
                require(not row["other_missing_descriptor_gavs"] and not row["missing_model_requests"],
                        "unexpected non-range ancestor gap")
        else:
            expected = (12, 6, 6, 0, 193, 0, 0) if name == "metadata-only" else (2, 1, 0, 1, 0, 0, 0)
            require(row["terminal"] == ["offers-only", "no-descriptor-collection"] and row["root"] is None
                    and counters == expected, "metadata-only case drift")
            require(not row["other_missing_descriptor_gavs"] and not row["missing_model_requests"]
                    and not row["effective_direct_dependencies"]
                    and row["attempted_range_gavs_sha256"] == canonical_digest([]), "unexpected metadata-only graph data")
    target = data["cases"]["target"]
    require([row[-1] for row in target["effective_direct_dependencies"]] == ["compile"] * 7 + ["test"] * 2,
            "target direct scope drift")
    fixed = target["other_missing_descriptor_gavs"]
    require(len(fixed) == len(set(fixed)) == 21 and fixed == sorted(fixed), "wrong fixed frontier")
    require(target["missing_model_requests"] ==
            [["org.apache.commons:commons-text:1.10.0", "coordinate", "org.junit:junit-bom:5.9.1"]],
            "target model import drift")
    frontier = [*offered_gavs, *fixed, target["missing_model_requests"][0][2]]
    paths = sorted(pom_path(gav) for gav in frontier)
    require(len(paths) == len(set(paths)) == 215, "frontier overlap")
    return paths


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--frontier", action="store_true", help="print observed repository-relative POM paths")
    args = parser.parse_args()
    raw = (ROOT / "observation.json").read_bytes()
    require(len(raw) <= 65536 and sha(raw) == OBSERVATION_SHA256, "exact observation drift")
    paths = validate(json.loads(raw, object_pairs_hook=unique))
    if args.frontier:
        print("\n".join(paths))
    else:
        print("PASS: 214 raw entries, 193 observed offers and absent range POMs, 215 unique observed POM requests")


if __name__ == "__main__":
    main()

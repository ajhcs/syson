"""Recheck the exact isolated ModelBuilder traces and their acquisition fixture."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
BOM = "org.junit:junit-bom:5.12.1"
ROOT = "org.apache.maven.shared:file-management:3.2.0"
PARENTS = ("org.apache.maven.shared:maven-shared-components:44",
           "org.apache.maven:maven-parent:44", "org.apache:apache:34")
CASES = ("baseline", "ci-property", "selector-control", "withheld-bom")
OBSERVATION_SHA = "d48fd9b57303a7e50e9e099776b02878b59244d62602a09356728921a98260fd"
ACQUISITION_SHA = "030238fc5164905fae81e2683572b8d8441a034d134ba0ef6341c6e77d112de6"
BOM_SHA = "7c826bc72beddc817dad9263027f9012a0f55a377d38df89c42932a2501c2bf0"
HARNESS_SHA = "2efde21c951c6d1302b7d0cbfb8f06294714697eab8501bd5abecc08a6995db4"


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def load(path, expected):
    data = path.read_bytes()
    require(digest(data) == expected, path.name + " byte seal changed")
    return json.loads(data)


def trace(name):
    success = name in ("baseline", "ci-property")
    selector = "org.junit:junit-bom:99.99.99-model-probe" if name == "selector-control" else BOM
    ci_profile = "format-check" if name == "ci-property" else "format"
    rows = [f"case\t{name}", "java\t21.0.12.1+1-LTS", "maven-model-builder\t3.9.16"]
    rows.extend(f"request\tparent\t{parent}\tpinned" for parent in PARENTS)
    rows.append(f"request\tcoordinate\t{selector}\t" + ("pinned" if success else "absent"))
    rows.extend([f"profiles\t{ROOT}\t", f"profiles\t{PARENTS[0]}\t",
                 f"profiles\t{PARENTS[1]}\tjava11+,{ci_profile}",
                 f"raw-selector\t{PARENTS[1]}\t5.12.1", f"profiles\t{PARENTS[2]}\tjdk9+"])
    if success:
        rows.extend([f"effective\t{ROOT}", "jupiter-managed\t5.12.1",
                     "jupiter-direct\t5.12.1\ttest", "built\ttrue"])
    else:
        rows.extend([f"problem\tERROR\tNon-resolvable import POM {selector}: "
                     "not available in the pinned offline model set",
                     "problem\tERROR\t'dependencies.dependency.version' for "
                     "org.junit.jupiter:junit-jupiter-api:jar is missing.", "built\tfalse"])
    return "\n".join(rows) + "\n"


def check(record, capture, harness):
    require(capture["status"] == "PASS_PASSIVE_ACQUISITION_ONLY", "acquisition did not pass")
    primary = capture["primary_model"]
    require(primary["sha256"] == record["bom_sha256"] == BOM_SHA and primary["bytes"] == 5649,
            "BOM identity changed")
    require(primary["coordinate"] == {"groupId": "org.junit", "artifactId": "junit-bom", "version": "5.12.1"}
            and primary["parent"] is None and primary["profiles"] == [] and primary["managed_import_count"] == 0,
            "BOM shape changed")
    require(primary["managed_jupiter_api"] == {"groupId": "org.junit.jupiter",
            "artifactId": "junit-jupiter-api", "version": "5.12.1"}, "BOM dependency changed")
    require(record["acquisition_observation_sha256"] == ACQUISITION_SHA, "wrong acquisition receipt")
    require(record["harness_sha256"] == digest(harness) == HARNESS_SHA, "probe source changed")
    require(record["schema"] == "C01-JACOCO-FILE-MANAGEMENT-MODEL/1", "wrong evidence schema")
    require(record["guard_witness"] == "PASS socket(AF_INET, AF_INET6, AF_UNIX) denied by kernel\n",
            "missing kernel witness")
    require(record["exits"] == dict.fromkeys((*CASES, "kernel", "compiler"), 0)
            and all(type(status) is int for status in record["exits"].values()), "process outcomes changed")
    require(record["file_management_model_built"] is True and all(
        record[key] is False for key in ("dependency_mediation_complete", "full_build", "network", "plugin_goals")),
        "model outcome or scope changed")
    require(set(record["cases"]) == set(CASES), "wrong observed cases")
    for name in CASES:
        observation = record["cases"][name]
        expected = trace(name)
        require(observation["stdout"] == expected and observation["stdout_sha256"] == digest(expected.encode()),
                name + " trace changed")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--acquisition", type=Path,
                        default=HERE.parent / "models-04-passive-run" / "observation.json")
    args = parser.parse_args()
    try:
        record = load(HERE / "observation.json", OBSERVATION_SHA)
        acquisition = load(args.acquisition, ACQUISITION_SHA)
        check(record, acquisition, (HERE / "ModelProbe.java").read_bytes())
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print("PASS: both effective models bound Jupiter 5.12.1; both controls failed closed")
    return 0


if __name__ == "__main__":
    sys.exit(main())

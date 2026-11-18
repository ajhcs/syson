"""Verify the checked-in JDT model-only evidence using only Python's standard library."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
SEALS = {
    "../eclipse-jdt-core-passive-run/observation.json":
        "bb7e2818adf6fe8a62b43293e1d7f5e0ae1b2bc109dc0432a95696366dd22896",
    "observation.json": "e56df30cf4fca41d805ddbbc59278b0c88fb367b9797fd05287c304851a999e0",
    "ModelProbe.java": "a11750ee1e6010b6bbbc53dfef99a0481c40902f7dd6dec76ae0edb6ee2d8687",
    "models.tsv": "4b5c8939545d3ac28e0c19e2e03ed17c43653bc498cdcf26fec7029a76fb7a28",
}
JDT = "org.eclipse.jdt"
CORE = "org.eclipse.jdt.core"
PLATFORM = "org.eclipse.platform"
DIRECT_ORDER = (
    (PLATFORM, "org.eclipse.core.resources"),
    (PLATFORM, "org.eclipse.core.runtime"),
    (PLATFORM, "org.eclipse.core.filesystem"),
    (PLATFORM, "org.eclipse.text"),
    (PLATFORM, "org.eclipse.team.core"),
)
VERSIONS = [f"3.{minor}.0" for minor in range(26, 48)]
REJECTED = "3.31.0"
DIAGNOSTIC = "'modelVersion' must be one of [4.0.0] but is '4.0'."
VERSION = r"[0-9]+\.[0-9]+\.[0-9]+"
FIXED = re.compile(VERSION)
RANGE = re.compile(rf"\[{VERSION},{VERSION}\)")


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def read(name):
    data = (HERE / name).read_bytes()
    require(hashlib.sha256(data).hexdigest() == SEALS[name], f"byte seal changed: {name}")
    return data


def split_cases(trace):
    rows = [line.split("\t") for line in trace.decode("utf-8", "strict").splitlines()]
    require(rows[:2] == [["java", "21.0.12.1+1-LTS"], ["maven-model-builder", "3.9.16"]],
            "unexpected recorded tool identity")
    cases = []
    for row in rows[2:]:
        if row[0] == "case":
            cases.append([])
        require(cases, "unbound row")
        cases[-1].append(row)
    require(len(cases) == len(VERSIONS), "unexpected case count")
    return cases


def dependency_counts(rows):
    return len(rows), sum(row[3] for row in rows), sum(bool(RANGE.fullmatch(row[2])) for row in rows)


def verify():
    captured = json.loads(read("../eclipse-jdt-core-passive-run/observation.json"))
    evidence = json.loads(read("observation.json"))
    read("ModelProbe.java")
    cases = split_cases(read("models.tsv"))
    require(evidence["passive_capture_commit"] == "a616629f3854dc51ec8c240a3a3339b26bfe761f",
            "upstream evidence commit changed")
    require(evidence["model_trace_sha256"] == SEALS["models.tsv"], "trace identity changed")
    witness = b"PASS socket(AF_INET, AF_INET6, AF_UNIX) denied by kernel\n"
    require(evidence["kernel_witness_sha256"] == hashlib.sha256(witness).hexdigest(),
            "kernel witness changed")
    require(evidence["process_exits"] == {"kernel": 0, "compiler": 0, "models": 0}
            and all(value is False for value in evidence["outside_scope"].values()),
            "scope or process outcome changed")
    require(evidence["rejected_version"] == REJECTED and len(captured["request_rows"]) == 110,
            "unexpected capture or rejected model identity")

    effective = []
    built = 0
    optionals = Counter()
    for index, (version, case) in enumerate(zip(VERSIONS, cases)):
        accepted = version != REJECTED
        spelling = "4.0.0" if accepted else "4.0"
        status, size, digest = captured["request_rows"][5 * index]
        require(status == 200 and size > 0 and captured["model_versions"][index] == [version, spelling],
                f"unexpected capture row: {version}")
        require(case[:3] == [
            ["case", version, digest],
            ["repository-ignored", "central", "https://repo.maven.apache.org/maven2", "true"],
            ["maven-raw-model", f"{JDT}:{CORE}:{version}", spelling],
        ], f"unexpected input or repository behavior: {version}")

        if not accepted:
            require(case[3:] == [["problem", "ERROR", DIAGNOSTIC], ["built", "false"]],
                    "unexpected modelVersion diagnostic or effective output")
            continue

        require(case[3] == ["effective-model", JDT, CORE, version, spelling, "jar"]
                and case[-1] == ["built", "true"], f"unexpected model result: {version}")
        expected_order = DIRECT_ORDER if index < 7 else (*DIRECT_ORDER, (JDT, "ecj"))
        direct = case[4:-1]
        require(len(direct) == len(expected_order), f"direct dependency count changed: {version}")
        for row, coordinate in zip(direct, expected_order):
            require(len(row) == 10 and row[0] == "direct" and tuple(row[1:3]) == coordinate,
                    f"unexpected direct dependency: {version}")
            constraint, kind, classifier, scope, optional, is_optional, exclusions = row[3:]
            expected_constraint = RANGE if index < 5 else FIXED
            require(expected_constraint.fullmatch(constraint) and (kind, classifier, scope, exclusions) ==
                    ("jar", "", "compile", "0"), f"dependency version or Maven defaults changed: {version}")
            require(optional in ("", "true", "false") and is_optional == str(optional == "true").lower()
                    and (is_optional == "true") == (coordinate == DIRECT_ORDER[-1]),
                    f"optional flag changed: {version}")
            if coordinate == (JDT, "ecj"):
                expected_ecj = "3.46.100" if version == "3.47.0" else version
                require(constraint == expected_ecj, f"ECJ version changed: {version}")
            optionals[optional] += 1
            effective.append([*coordinate, constraint, is_optional == "true"])
        built += 1

    rejected_rows = evidence["raw_direct_in_rejected_model"]
    require([tuple(row[:2]) for row in rejected_rows] == list(DIRECT_ORDER)
            and all(len(row) == 4 and RANGE.fullmatch(row[2])
                    and row[3] is (index == 4) for index, row in enumerate(rejected_rows)),
            "rejected raw declarations changed")
    require(optionals == {"": 20, "false": 79, "true": 21}, "effective optional literals changed")
    counts = {"input_models": len(cases), "built_models": built}
    for prefix, rows in (("raw", effective + rejected_rows), ("effective", effective)):
        counts.update({f"{prefix}_{field}": count for field, count in
                       zip(("direct", "optional", "ranges"), dependency_counts(rows))})
    require(counts == evidence["counts"], "declared dependency or model totals changed")
    print("PASS 22 exact JDT inputs, 21 effective models, 3.31.0 modelVersion refusal, 120 effective direct dependencies")


if __name__ == "__main__":
    verify()

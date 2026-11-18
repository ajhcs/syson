"""Check the recorded Commons Text ModelBuilder outcomes without Java or a network."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SEALS = {
    "../commons-text-junit-bom-passive-run/observation.json":
        "3c7bc70b3d98fdf1ebfab5e66ecf95e1bcf1e996a0af2c2d1e0b9219d549b259",
    "observation.json": "7a49477bd252beae57fe6cb0c14bd893a9ca0fe047691756a277051e13330a0e",
    "ModelProbe.java": "ccdeafb2946677ec34bc47865419a929fe0007929addd69e6ac556a9d301b660",
}
TEXT = "org.apache.commons:commons-text:1.10.0"
COMMONS_PARENT = "org.apache.commons:commons-parent:54"
APACHE_PARENT = "org.apache:apache:27"
BOM = "org.junit:junit-bom:5.9.1"
PRIOR_INPUTS = [
    ("input", TEXT, "388dd52348ba1842aa38aeb897c91d270b14661eb875a20fd98031535ab27567"),
    ("input", COMMONS_PARENT, "000d8187952b223702fde296df799563f35de82ce72adb4e7bf157342378fbe3"),
    ("input", APACHE_PARENT, "b2b0fc69e22a650c3892f1c366d77076f29575c6738df4c7a70a44844484cdf9"),
]
CASES = ("baseline", "withheld-bom")
MANAGEMENT = {
    "jupiter": ("jupiter", "jupiter-api", "jupiter-engine", "jupiter-migrationsupport", "jupiter-params"),
    "platform": ("platform-commons", "platform-console", "platform-engine", "platform-jfr", "platform-launcher",
                 "platform-reporting", "platform-runner", "platform-suite", "platform-suite-api",
                 "platform-suite-commons", "platform-suite-engine", "platform-testkit"),
    "vintage": ("vintage-engine",),
}
KERNEL = (b"PASS socket(AF_INET, AF_INET6, AF_UNIX) denied by kernel\n"
          b"PASS process group changes denied by kernel\n")


def require(condition, error):
    if not condition:
        raise ValueError(error)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read(name):
    data = (HERE / name).read_bytes()
    require(digest(data) == SEALS[name], f"byte seal changed: {name}")
    return data


def expected_trace(label, bom_digest):
    accepted = label == "baseline"
    rows = [
        ("case", label), ("java", "21.0.12.1+1-LTS"), ("maven-model-builder", "3.9.16"),
        *PRIOR_INPUTS,
    ]
    if accepted:
        rows.append(("input", BOM, bom_digest))
    rows.extend([
        ("request", "parent", COMMONS_PARENT, "pinned"),
        ("request", "parent", APACHE_PARENT, "pinned"),
        ("request", "coordinate", BOM, "pinned" if accepted else "absent"),
        ("profiles", TEXT, "java9+"),
        ("raw-selector", TEXT, "5.9.1"),
        ("profiles", COMMONS_PARENT, "parse-target-version,svn-buildnumber"),
        ("raw-selector", COMMONS_PARENT, "5.9.0"),
        ("profiles", APACHE_PARENT, ""),
    ])
    if accepted:
        rows.append(("effective", TEXT, "jar"))
        for family, artifacts in MANAGEMENT.items():
            version = "1.9.1" if family == "platform" else "5.9.1"
            rows.extend(("junit-managed", "org.junit." + family, "junit-" + artifact, version, "jar", "", "")
                        for artifact in artifacts)
        rows.extend([
            ("direct", "org.apache.commons", "commons-lang3", "3.12.0", "jar", "", "compile"),
            ("direct", "org.junit.jupiter", "junit-jupiter", "5.9.1", "jar", "", "test"),
            ("built", "true"),
        ])
    else:
        rows.extend([
            ("problem", "ERROR", "Non-resolvable import POM " + BOM + ": not available in the pinned offline model set"),
            ("problem", "ERROR", "'dependencies.dependency.version' for org.junit.jupiter:junit-jupiter:jar is missing."),
            ("built", "false"),
        ])
    return rows


def verify():
    read("ModelProbe.java")
    capture = json.loads(read("../commons-text-junit-bom-passive-run/observation.json"))
    evidence = json.loads(read("observation.json"))
    require(evidence["status"] == "OBSERVED_PINNED_COMMONS_TEXT_MODELS_ONLY" and
            evidence["capture_commit"] == "eb51c56679eb07f0feaef545d42e9d981e710274", "capture anchor changed")
    status, length, digest_bom = capture["request_rows"][0]
    require(status == 200 and length == 5630 and capture["same_origin_checksum_matches"] == ["sha512", "sha256", "sha1"],
            "BOM capture changed")
    require(list(evidence["cases"]) == list(CASES), "unexpected replay cases")
    for label, case in evidence["cases"].items():
        trace = [tuple(row) for row in case["stdout_records"]]
        require(trace == expected_trace(label, digest_bom), f"unexpected ordered model trace: {label}")
        raw = "".join("\t".join(row) + "\n" for row in trace).encode("utf-8")
        require(case["stdout_sha256"] == digest(raw)
                and case["built"] is (label == "baseline"), f"unexpected trace digest or build outcome: {label}")
    require(evidence["kernel_stdout_sha256"] == digest(KERNEL), "kernel witness changed")
    commands = ("kernel", "compiler", *CASES)
    require(evidence["process_exits"] == dict.fromkeys(commands, 0)
            and set(evidence["teardown"]) == set(commands),
            "unexpected process exit or teardown record")
    for label, witness in evidence["teardown"].items():
        require(witness == {"group_sigkill_attempted": True, "empty_live_group_samples": 2,
                            "waitable_leader_retained_until_proof": True, "leader_exit": 0, "completion": "exited"},
                f"incomplete teardown: {label}")
    require(evidence["outside_scope"] == dict.fromkeys((
        "network", "maven_cli_or_goals", "acquired_payload_execution", "graph_collection", "range_selection",
        "binary_resolution", "dependency_download_or_cache_install", "native_compile_or_full_build",
    ), False), "replay scope changed")
    print("PASS Commons Text model, 18 managed JUnit entries, compile Lang3, test Jupiter, and withheld BOM refusal")


if __name__ == "__main__":
    verify()

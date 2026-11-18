"""Verify the exact stored JaCoCo plugin ModelBuilder observations."""
import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
PLUGIN = "org.jacoco:jacoco-maven-plugin:0.8.14"
PARENT = "org.jacoco:org.jacoco.build:0.8.14"
ASM_BOM = "org.ow2.asm:asm-bom:9.9"
OW2 = "org.ow2:ow2:1.5.1"
FILE_MANAGEMENT = "org.apache.maven.shared:file-management:3.2.0"
OBSERVATION_SHA = "54ae5eecbff32bd079c33709a93e466065178c4c48bcdf50432cf2465afd5bed"
FILE_MANAGEMENT_SHA = "d48fd9b57303a7e50e9e099776b02878b59244d62602a09356728921a98260fd"
HARNESS_SHA = "ee6f45bb368608da17c537628f4572fc18de853a057da79ab398b41faf1aeff2"
SCOPE_SHA = "5aad4787d11b9a81a8da66129e877febf4ed1224063e0cdf2272f8d60af6fcb9"
RECEIPT_SHA = "691f549b2545df3131c9ab056aed446dfa88c42d79fb3e097490e09ca9c6786d"
INPUTS = (
    (PLUGIN, 4230, "ba3a08fe93fef4d73b76dce547076510691f61aaf79c42ce9b7fb459b259a821"),
    (PARENT, 46112, "87d059b35849df97779148fd7e52f63f1fcd03a4e7e64c2a61a6c9cce61a8de7"),
    (ASM_BOM, 3254, "0fa27336fa7b62923ca9bdc69526ebe712a3bfb1f7ee8bf76c4a0b3f85f19d62"),
    (OW2, 11290, "321ddbb7ee6fe4f53dea6b4cd6db74154d6bfa42391c1f763b361b9f485acf05"),
)
SUCCESS = ("baseline", "ci-property")
CASES = (*SUCCESS, "selector-control", "withheld-bom")
MANAGED = ("asm", "asm-tree", "asm-analysis", "asm-util", "asm-commons")
DIRECT = (
    ("org.apache.maven", "maven-plugin-api", "3.0", "jar", "", "provided"),
    ("org.apache.maven", "maven-core", "3.0", "jar", "", "provided"),
    ("org.codehaus.plexus", "plexus-utils", "3.0.24", "jar", "", "compile"),
    ("org.apache.maven.shared", "file-management", "3.2.0", "jar", "", "compile"),
    ("org.apache.maven.reporting", "maven-reporting-api", "3.0", "jar", "", "compile"),
    ("org.jacoco", "org.jacoco.agent", "0.8.14", "jar", "runtime", "compile"),
    ("org.jacoco", "org.jacoco.core", "0.8.14", "jar", "", "compile"),
    ("org.jacoco", "org.jacoco.report", "0.8.14", "jar", "", "compile"),
    ("org.apache.maven.plugin-tools", "maven-plugin-annotations", "3.4", "jar", "", "provided"),
)


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def load(path, expected):
    data = path.read_bytes()
    require(digest(data) == expected, path.name + " byte seal changed")
    return json.loads(data)


def expected_trace(name):
    success = name in SUCCESS
    selector = "org.ow2.asm:asm-bom:99.99.99-model-probe" if name == "selector-control" else ASM_BOM
    lines = [
        f"case\t{name}",
        "java\t21.0.12.1+1-LTS",
        "maven-model-builder\t3.9.16",
        f"request\tparent\t{PARENT}\tpinned",
        f"request\tcoordinate\t{selector}\t" + ("pinned" if success else "absent"),
    ]
    if success:
        lines.append(f"request\tparent\t{OW2}\tpinned")
    lines.extend((
        f"profiles\t{PLUGIN}\t",
        f"profiles\t{PARENT}\tmaven-jdk20",
        f"raw-selector\t{PARENT}\t9.9",
    ))
    if success:
        lines.extend((f"effective\t{PLUGIN}", "packaging\tmaven-plugin"))
        lines.extend(f"managed-asm\torg.ow2.asm\t{artifact}\t9.9\tjar\t\t" for artifact in MANAGED)
        lines.extend("\t".join(("direct", *row)) for row in DIRECT)
        lines.append("built\ttrue")
    else:
        lines.append(f"problem\tERROR\tNon-resolvable import POM {selector}: "
                     "not available in the pinned offline model set")
        lines.append("built\tfalse")
    return "\n".join(lines) + "\n"


def check(record, file_management, harness):
    require(record["schema"] == "C01-JACOCO-PLUGIN-OWN-MODEL/1"
            and record["status"] == "OBSERVED_PINNED_MAVEN_MODELS_ONLY", "observation status changed")
    require(record["runtime_receipt_sha256"] == RECEIPT_SHA
            and record["scope_spec_sha256"] == SCOPE_SHA, "private receipt binding changed")
    require(record["harness_sha256"] == digest(harness) == HARNESS_SHA, "probe source changed")
    require(record["file_management_observation_sha256"] == FILE_MANAGEMENT_SHA
            and file_management["file_management_model_built"] is True, "file-management evidence changed")
    for name in SUCCESS:
        previous = file_management["cases"][name]
        require(digest(previous["stdout"].encode()) == previous["stdout_sha256"]
                and f"\neffective\t{FILE_MANAGEMENT}\n" in previous["stdout"], "direct dependency model changed")
    require(record["model_inputs"] == [
        {"coordinate": coordinate, "bytes": size, "sha256": sha} for coordinate, size, sha in INPUTS
    ], "model input changed")
    require(record["guard_witness"] == "PASS socket(AF_INET, AF_INET6, AF_UNIX) denied by kernel\n",
            "network guard witness changed")
    require(record["exits"] == dict.fromkeys((*CASES, "kernel", "compiler"), 0)
            and all(type(status) is int for status in record["exits"].values()), "process outcomes changed")
    require(record["jacoco_plugin_model_built"] is True and all(
        record[key] is False for key in ("dependency_mediation_complete", "full_build", "network", "plugin_goals")),
        "model outcome or scope changed")
    require(set(record["cases"]) == set(CASES), "case inventory changed")
    for name in CASES:
        case = record["cases"][name]
        expected = expected_trace(name)
        require(case["stdout"] == expected and case["stdout_sha256"] == digest(expected.encode()),
                name + " trace changed")


def main():
    try:
        record = load(HERE / "observation.json", OBSERVATION_SHA)
        file_management = load(HERE.parent / "jacoco-file-management-model" / "observation.json",
                               FILE_MANAGEMENT_SHA)
        check(record, file_management, (HERE / "ModelProbe.java").read_bytes())
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print("PASS: two JaCoCo models, five ASM managed dependencies, nine direct declarations, two refusals")
    return 0


if __name__ == "__main__":
    sys.exit(main())

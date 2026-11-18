"""Verify the retained, sanitized Maven model-building observation."""
from hashlib import sha256
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SOURCE = "2f7debee3fd53ad0f37ee154a3985dd7bf568c96c4af6794e929d048d0daac64"
PARENT = "2a3db744883bc3ee1f7933713a174bb70cce755080826dc1927d3bb54e2615ae"
MODELS = {
    "file-management-3.2.0.pom.partial": "04e3bdd6d1920d1cdba206bc35dd3c6e0646c5730969f8b5d5397e5f1c7b5edd",
    "maven-shared-components-44.pom.partial": "41fbab97912beb3a061b1e3845d2e06d2bb214cc71f92fa7f396480dc765a14e",
    "maven-parent-44.pom.partial": "1c0a088d7352d52ea8f3581d05778c0c4a679613fff2c58737fba55f1007fe24",
    "apache-34.pom.partial": "3671ae9d4d062ae3bb985731c76088bb2f6f7d7254e2d304ee9f690b97651328",
}
CASES = {
    "baseline": ("5.12.1", "format", "494d591f137244ea43af590bbc2a7ec5914f1db73c373a065eb2bf7d777d0f91"),
    "ci-property": ("5.12.1", "format-check", "63a4c4ebf7ec4a16b912ad87e85c6f243eb828c8306553604cb1b9fa421c91ca"),
    "selector-control": ("99.99.99-model-probe", "format", "d216c44cc67205177e2b984ebd11f7ad0077304d88aceb3c02ce63d45f48ace7"),
}


def require(condition, error):
    if not condition:
        raise ValueError(error)


def decode(data):
    def distinct(items):
        result = dict(items)
        require(len(result) == len(items), "duplicate JSON key")
        return result

    require(len(data) <= 32768, "observation size")
    return json.loads(data, object_pairs_hook=distinct)


def validate(observation, parent_data, source_data):
    require(sha256(source_data).hexdigest() == SOURCE == observation["source_sha256"], "Java source pin")
    require(sha256(parent_data).hexdigest() == PARENT, "passive acquisition parent pin")
    parent = decode(parent_data)
    apache = [model for model in parent["primary_model_observations"]
              if model["coordinate"] == {"groupId": "org.apache", "artifactId": "apache", "version": "34"}]
    require(len(apache) == 1 and apache[0]["sha256"] == MODELS["apache-34.pom.partial"] and
            apache[0]["parent"] is None and apache[0]["managed_import_count"] == 0, "Apache 34 parent binding")
    require(observation["scope"] == "offline_model_builder_only" and
            observation["parent_evidence_commit"] == "c0a74fc2e2ae61c9e4662c9cccb11b05449843c6" and
            observation["four_pom_sha256"] == MODELS, "model context")
    require(observation["private_scope_sha256"] == "e75dd8dc9a0a45018dbfe15574922379bd9c55b2b045578c54a0cd833d91ceab" and
            observation["private_receipt_sha256"] == "c7656c1c1d837397436add8655ed8bec0163190716b7b94b270354efd9f3265a",
            "private scope and receipt")
    require(observation["compiler_and_jvm_inputs"] == {"jdk_nodes": 454, "maven_library_jars": 9} and
            observation["kernel_socket_control"] == "PASS socket(AF_INET, AF_INET6, AF_UNIX) denied by kernel" and
            all(observation[key] is False for key in ("maven_cli_goal", "full_effective_model", "native_compile")),
            "runtime boundary")
    observations = observation["observations"]
    require(set(observations) == set(CASES), "incomplete case set")
    for case, (version, profile, expected_hash) in CASES.items():
        record = observations[case]
        trace = record["trace"]
        require(sha256(trace.encode("utf-8")).hexdigest() == record["sha256"] == expected_hash, f"trace pin: {case}")
        lines = trace.splitlines()
        requests = [line.split("\t")[1:] for line in lines if line.startswith("request\t")]
        require(requests == [
            ["parent", "org.apache.maven.shared:maven-shared-components:44", "pinned"],
            ["parent", "org.apache.maven:maven-parent:44", "pinned"],
            ["parent", "org.apache:apache:34", "pinned"],
            ["coordinate", f"org.junit:junit-bom:{version}", "absent"],
        ], f"resolver sequence: {case}")
        require(f"profiles\torg.apache.maven:maven-parent:44\tjava11+,{profile}" in lines and
                "profiles\torg.apache:apache:34\tjdk9+" in lines and lines[-1] == "built\tfalse",
                f"profiles or incomplete-model boundary: {case}")


if __name__ == "__main__":
    validate(decode((ROOT / "observation.json").read_bytes()),
             (ROOT.parent / "models-03-passive-run/observation.json").read_bytes(),
             (ROOT / "ModelProbe.java").read_bytes())
    print("PASS: baseline and CI select JUnit 5.12.1; synthetic override diverges")

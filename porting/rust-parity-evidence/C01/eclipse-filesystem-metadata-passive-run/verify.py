"""Verify the public Filesystem metadata capture projection offline."""
import hashlib
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
CORE = HERE.parents[2] / "rust-parity-plan-20261026" / "closure" / "core-compile"
EPOCH = "eclipse-filesystem-metadata-01"
CLIENT_SHA = "b376097f7c3901f6177096b0ae42cf3cda3fbd949349516d70b570059b422375"
MANIFEST_SHA = "72fbfd8426066a6a6434df5a72d49400232f4598ff4a73adf37007c173b997fc"
EVIDENCE_SHA = "3269cbf40e827dc12f0a0ec79e02e8c5fbbcc47759509a87d2dd0adc54a2e906"
BASE_URL = "https://repo.maven.apache.org/maven2/org/eclipse/platform/org.eclipse.core.filesystem/maven-metadata.xml"
RESPONSES = (
    ("", "version_metadata", 200, 1188),
    (".sha512", "sha512", 200, 128),
    (".sha256", "sha256", 200, 64),
    (".sha1", "sha1", 200, 40),
    (".asc", "signature", 404, 554),
)
DIGEST = re.compile(r"[0-9a-f]{64}")


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def read_sealed(path, expected):
    limit = 256 * 1024
    with path.open("rb") as file:
        source = file.read(limit + 1)
    require(len(source) <= limit and hashlib.sha256(source).hexdigest() == expected,
            f"byte seal changed: {path.name}")
    return source


def check_responses(manifest, evidence):
    requests = manifest["requests"]
    rows = evidence["request_rows"]
    require(len(requests) == len(rows) == len(RESPONSES) and len(manifest["new_primaries"]) == 1,
            "unexpected request or primary count")
    for request, row, (suffix, role, status, size) in zip(requests, rows, RESPONSES, strict=True):
        require(request["url"] == BASE_URL + suffix and request.get("algorithm", request["role"]) == role,
                f"request identity changed: {role}")
        require(row[:2] == [status, size] and len(row) == 3 and status in request["allowed_status"]
                and size <= request["maximum_bytes"] and DIGEST.fullmatch(row[2]),
                f"response row changed: {role}")
    require(sum(row[1] for row in rows) == evidence["body_bytes"] == 1974
            and evidence["body_bytes"] <= manifest["limits"]["payload_and_sidecar_total_bytes"],
            "aggregate byte accounting changed")
    require(evidence["same_origin_checksum_matches"] == ["sha512", "sha256", "sha1"]
            and evidence["signature_status"] == rows[-1][0] == 404, "checksum or signature disposition changed")


def check_declarations(evidence):
    declarations = evidence["metadata_declarations"]
    versions = declarations["versions"]
    require(declarations["metadata_sha256"] == evidence["request_rows"][0][2]
            and [declarations["group_id"], declarations["artifact_id"]] ==
            ["org.eclipse.platform", "org.eclipse.core.filesystem"], "metadata identity changed")
    require(len(versions) == len(set(versions)) == 26
            and all(re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version) for version in versions)
            and versions[0] == "1.6.1" and versions[1] == "1.7.0" and versions[-1] == "1.11.500"
            and declarations["latest"] == declarations["release"] == versions[-1]
            and declarations["last_updated"] == "20260907074538", "publisher declarations changed")


def verify():
    read_sealed(CORE / "acquire.py", CLIENT_SHA)
    manifest = json.loads(read_sealed(CORE / "requests" / f"{EPOCH}.json", MANIFEST_SHA))
    evidence = json.loads(read_sealed(HERE / "observation.json", EVIDENCE_SHA))
    require(evidence["schema"] == "SYSON-C01-ECLIPSE-FILESYSTEM-METADATA-PASSIVE/1"
            and evidence["status"] == "PASS_PASSIVE_ACQUISITION_ONLY"
            and evidence["epoch"] == manifest["epoch"] == EPOCH
            and evidence["proposal_head"] == "ded22f10d64f44173f67faa7fe759b931b3bef36"
            and evidence["manifest_sha256"] == MANIFEST_SHA and evidence["client_sha256"] == CLIENT_SHA,
            "scope identity changed")
    check_responses(manifest, evidence)
    check_declarations(evidence)
    custody = evidence["custody_sha256"]
    require(set(custody) == {"frozen_scope", "getter_receipt", "controller_receipt"}
            and all(DIGEST.fullmatch(seal) for seal in custody.values()), "private custody seals changed")
    require(evidence["frozen_input_count"] == 39
            and evidence["frozen_inputs_match_before_and_after"] is True
            and evidence["all_final_urls_match_roster"] is True
            and type(evidence["controller_exit"]) is int and evidence["controller_exit"] == 0
            and evidence["owned_scope_cleanup"] == "not-found"
            and evidence["configured_scope_limits"] == {
                "cpu.max": "100000 100000", "memory.high": "536870912", "memory.max": "1073741824",
                "memory.swap.max": "0", "pids.max": "32",
            }, "capture or cleanup facts changed")
    require(evidence["outside_scope"] == dict.fromkeys((
        "publisher_signature_verified", "resolver_offers_observed", "final_version_selected",
        "descriptor_graph_qualified", "cache_installation", "binary_execution", "offline_native_compile",
    ), False), "unsupported authority or scope claim")
    print("PASS five capture responses, three same-origin checksum matches, and 26 declared versions")


if __name__ == "__main__":
    verify()

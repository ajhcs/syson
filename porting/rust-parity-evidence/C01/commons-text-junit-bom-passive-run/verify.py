"""Verify the public Commons Text JUnit BOM capture metadata offline."""

from hashlib import sha256
import json
from pathlib import Path
import re


HERE = Path(__file__).resolve().parent
CORE = HERE.parents[2] / "rust-parity-plan-20261026" / "closure" / "core-compile"
EPOCH = "commons-text-junit-bom-01"
MANIFEST_SHA = "65626d1e844c0b780b13b359a058eca4a03e88d371c06bb236ecd9b0620555fd"
CLIENT_SHA = "30ea750c2bebbbef18e69909860bf3231699151eba69af6a0b4fcef58de26fc3"
OBSERVATION_SHA = "3c7bc70b3d98fdf1ebfab5e66ecf95e1bcf1e996a0af2c2d1e0b9219d549b259"
DIGEST = re.compile(r"[0-9a-f]{64}")
ROSTER = [("pom", 5630), ("sha512", 128), ("sha256", 64), ("sha1", 40), ("signature", 853)]
AUTHORITY = {
    "publisher_signature_verified", "effective_maven_model_verified", "cache_install_allowed",
    "acquired_code_execution_allowed", "closure_complete", "offline_native_compile_verified",
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read_sealed(path, expected):
    limit = 256 * 1024
    with path.open("rb") as data:
        raw = data.read(limit + 1)
    require(len(raw) <= limit and sha256(raw).hexdigest() == expected, f"seal changed: {path.name}")
    return raw


def verify(manifest, observed):
    require(observed["schema"] == "SYSON-C01-COMMONS-TEXT-JUNIT-BOM-PASSIVE-RUN/1"
            and observed["status"] == "PASS_PASSIVE_ACQUISITION_ONLY"
            and observed["epoch"] == manifest["epoch"] == EPOCH
            and observed["manifest_sha256"] == MANIFEST_SHA
            and observed["client_sha256"] == CLIENT_SHA, "capture binding changed")

    checksums = []
    require(len(manifest["new_primaries"]) == 1, "unexpected primary count")
    requests = manifest["requests"]
    rows = observed["request_rows"]
    require(len(rows) == len(requests) == len(ROSTER), "response count differs")
    for request, row, (expected_role, expected_bytes) in zip(requests, rows, ROSTER, strict=True):
        role = request.get("algorithm", request["role"])
        require(isinstance(row, list) and len(row) == 3, "invalid response row")
        status, length, digest = row
        require(role == expected_role and type(status) is int and status == 200 in request["allowed_status"]
                and type(length) is int and 0 < length == expected_bytes <= request["maximum_bytes"]
                and isinstance(digest, str) and DIGEST.fullmatch(digest), "response differs from roster")
        if request["role"] == "checksum":
            checksums.append(role)
    require(checksums == observed["same_origin_checksum_matches"] == ["sha512", "sha256", "sha1"],
            "same-origin checksum coverage changed")
    require(sum(row[1] for row in rows) == observed["body_bytes"] == 6715
            and observed["body_bytes"] <= manifest["limits"]["payload_and_sidecar_total_bytes"],
            "capture body byte accounting differs")

    custody = observed["custody_sha256"]
    require(set(custody) == {"frozen_spec", "getter_receipt", "scope_proof", "controller_receipt"}
            and all(isinstance(value, str) and DIGEST.fullmatch(value) for value in custody.values()),
            "custody bindings are incomplete")
    require(observed["frozen_input_count"] == 23
            and observed["frozen_inputs_match_before_and_after"] is True
            and observed["all_final_urls_match_roster"] is True and type(observed["controller_exit"]) is int
            and observed["controller_exit"] == 0 and observed["owned_scope_cleanup"] == "not-found"
            and observed["controls"] == {
                "cpu.max": "100000 100000", "memory.high": "536870912", "memory.max": "1073741824",
                "memory.swap.max": "0", "pids.max": "32",
            }, "capture or teardown evidence changed")
    authority = observed["authority"]
    require(set(authority) == AUTHORITY and all(value is False for value in authority.values()),
            "unsupported verification or authority claim")


def main():
    read_sealed(CORE / "acquire.py", CLIENT_SHA)
    manifest = json.loads(read_sealed(CORE / "requests" / f"{EPOCH}.json", MANIFEST_SHA))
    observed = json.loads(read_sealed(HERE / "observation.json", OBSERVATION_SHA))
    verify(manifest, observed)
    print("PASS: five passive responses and three recorded same-origin checksum matches")


if __name__ == "__main__":
    main()

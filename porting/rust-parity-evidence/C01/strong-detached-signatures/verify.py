"""Validate the public observation against the committed passive capture metadata."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
OBSERVATION = "41066ae1540f8741bc382615d6207a674ebf9530eda0eaf2ee6681a45bb0291d"
CAPTURES = {
    "passive_observation_sha256": "68528f34cdda6789e00cdeb653a2eb4f49cd8c4f28cf11c3c3bd9e0e12251b00",
    "direct_projected_receipt_sha256": "c5b6565f729f797cd9f60466c83251835b9a13406d7975741b8b9bfa658ec9c7",
    "direct_qualification_sha256": "34d2b4abae0c8c3b47a05a83c497c6eacd2a43ecd325785d2e56dedc7f14514c",
    "direct_private_receipt_sha256": "b0b1c91009b5cb964b4a6a277dbabd6389212c9bf8110c75ef09ecf0b3260ec8",
}
GROUPS = {
    "file-management": (
        "apache-maven-project-key-catalog", "SHA-256", "84789D24DF77A32433CE1F079EB80E92EB2135B1",
        {"org/apache/maven/shared/file-management/3.2.0/file-management-3.2.0.jar": 1744579165},
    ),
    "commons-io": (
        "apache-commons-project-key-catalog", "SHA-1", "2DB4F1EF0FA761ECC4EA935C86FDC7E2A11262CB",
        {"commons-io/commons-io/2.19.0/commons-io-2.19.0.jar": 1744149939},
    ),
    "jacoco": (
        "apache-groovy-downstream-pinned-keyring", "SHA-1", "A413F67D71BEEC23ADD0CE0ACB43338E060CF9FA",
        {
            "org/jacoco/jacoco-maven-plugin/0.8.14/jacoco-maven-plugin-0.8.14.jar": 1760142641,
            "org/jacoco/org.jacoco.agent/0.8.14/org.jacoco.agent-0.8.14-runtime.jar": 1760142605,
            "org/jacoco/org.jacoco.core/0.8.14/org.jacoco.core-0.8.14.jar": 1760142579,
            "org/jacoco/org.jacoco.report/0.8.14/org.jacoco.report-0.8.14.jar": 1760142593,
        },
    ),
}
DIRECT_ARTIFACT = "jacoco-maven-plugin-0.8.14"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def unique(pairs):
    data = {}
    for key, value in pairs:
        require(key not in data, "duplicate JSON key")
        data[key] = value
    return data


def read(path, expected, limit=65536):
    with path.open("rb") as stream:
        raw = stream.read(limit + 1)
    require(len(raw) <= limit and hashlib.sha256(raw).hexdigest() == expected,
            f"{path.name}: input digest mismatch")
    return json.loads(raw, object_pairs_hook=unique)


def validate(observation, passive, direct, qualification):
    require(observation["schema"] == "syson-c01-strong-detached-signatures-v1"
            and observation["capture_evidence"] == CAPTURES, "observation or capture identity differs")
    authority = observation["authority"]
    require(set(authority) == {"artifact_execution", "cache_installation", "native_compile_accepted",
                              "complete_runtime_closure", "current_key_authority_verified"}
            and all(value is False for value in authority.values()), "unexpected authority")
    require(qualification["network_private_receipt_sha256"] == CAPTURES["direct_private_receipt_sha256"],
            "direct capture receipt differs")
    require(passive["status"] == direct["status"] == "PASS_PASSIVE_ACQUISITION_ONLY",
            "inherited capture status differs")
    require([row["group"] for row in observation["groups"]] == list(GROUPS), "group roster differs")
    for group in observation["groups"]:
        source, self_certification, primary, expected = GROUPS[group["group"]]
        require(group["source_kind"] == source and group["key_primary_fingerprint"] == primary
                and group["key_self_certification_hash"] == self_certification, "key identity differs")
        rows = group["signatures"]
        require([row["central_coordinate"] for row in rows] == list(expected), "artifact roster differs")
        for row in rows:
            coordinate = row["central_coordinate"]
            name = Path(coordinate).stem
            require(row["name"] == name and row["signed_unix_second"] == expected[coordinate]
                    and row["hash_algorithm"] == "SHA-512"
                    and row["signing_fingerprint"] == row["primary_fingerprint"] == primary,
                    "artifact signature identity differs")
            from_direct = name == DIRECT_ARTIFACT
            captures = direct["requests"] if from_direct else passive["request_rows"]
            require(row["capture_source"] ==
                    ("batch34-direct-core" if from_direct else "jacoco-binaries-passive-run"),
                    "capture source differs")
            for suffix, role, kind in (("", "jar", "artifact"),
                                       (".asc", "signature", "signature")):
                if from_direct and suffix:
                    role = "checksum_or_unverified_signature"
                path = "downloads/" + coordinate + suffix + ".partial"
                matches = [entry for entry in captures if entry["relative_path"] == path]
                require(len(matches) == 1, f"{path}: capture must be unique")
                capture = matches[0]
                require(capture["role"] == role and capture["status"] == 200
                        and capture["bytes"] == row[kind + "_bytes"]
                        and capture["sha256"] == row[kind + "_sha256"], f"{path}: capture differs")
                if from_direct:
                    url = "https://repo.maven.apache.org/maven2/" + coordinate + suffix
                    require(capture["url"] == capture["final_url"] == url, f"{path}: source URL differs")


def main():
    direct = HERE.parent / "batch34-direct-core"
    validate(
        read(HERE / "observation.json", OBSERVATION),
        read(HERE.parent / "jacoco-binaries-passive-run/observation.json",
             CAPTURES["passive_observation_sha256"]),
        read(direct / "captures/network-projected/receipt.json",
             CAPTURES["direct_projected_receipt_sha256"], limit=1 << 19),
        read(direct / "qualification.json", CAPTURES["direct_qualification_sha256"], limit=1 << 19),
    )
    print("PASS six detached-signature observations match previously committed capture metadata")


if __name__ == "__main__":
    main()

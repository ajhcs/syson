"""Check the committed SLF4J packet observation and prior JAR inventory offline."""

import json
from pathlib import Path


EVIDENCE = Path(__file__).resolve().parent
ARTIFACT = "org/slf4j/slf4j-api/1.7.36/slf4j-api-1.7.36.jar"
PRIMARY = "475F3B8E59E6E63AA78067482C7B12F2A511E325"
KEY_SHA256 = "ccc266248ba920d074bba695551751f9ba145a46549c5ca6d40ed5d3cdcc4f56"
SIGNATURE_SHA256 = "7f60e6c3dec52d947b6cf8750b8d14c5b8ccf6b66ff79ab583d62010f7e2696d"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def validate(record, inventory):
    require(record["schema"] == "syson.c01.slf4j-historical-signature.v1", "unknown evidence schema")
    jar = record["artifact"]
    require(jar["coordinate"] == "org.slf4j:slf4j-api:1.7.36" and jar["retained_copies"] == 2,
            "wrong artifact binding")
    matches = [item for item in inventory["reused_existing_artifacts"] if item["relative_artifact"] == ARTIFACT]
    require(len(matches) == 1 and jar["bytes"] == matches[0]["bytes"] == 41125
            and jar["sha256"] == matches[0]["sha256"], "JAR differs from prior inventory")

    key = record["historical_key"]
    detached = record["detached_signature"]
    require(key["url"] == "https://www.slf4j.org/public-keys/ceki-public-key.pgp"
            and detached["url"] == "https://repo.maven.apache.org/maven2/" + ARTIFACT + ".asc",
            "unexpected evidence source")
    require(key["bytes"] == 1672 and key["sha256"] == KEY_SHA256, "historical key input differs")
    require(detached["bytes"] == 455 and detached["sha256"] == SIGNATURE_SHA256,
            "detached signature input differs")

    require(key["packet_primary_fingerprint"] == key["publisher_primary_fingerprint"] == PRIMARY
            and key["publisher_cutoff_date"] == "2022-08-08"
            and key["release_date"] == "2022-02-08", "wrong historical key binding")
    require(detached["packet_hash_algorithm_id"] == 2 and detached["packet_hash_algorithm"] == "SHA-1"
            and record["accepted_detached_hash_algorithms"] == ["SHA-256", "SHA-384", "SHA-512"],
            "unexpected signature algorithm or policy")
    require(detached["packet_version"] == 4 and detached["packet_tag"] == 2
            and detached["packet_public_key_algorithm_id"] == 1
            and detached["packet_public_key_algorithm"] == "RSA"
            and detached["packet_created_at_utc"] == "2022-02-08T13:33:50Z", "unexpected signature packet")
    require(detached["packet_claimed_issuer_key_id"] == PRIMARY[-16:]
            and detached["packet_claimed_issuer_key_id_location"] == "UNHASHED_SIGNATURE_SUBPACKET"
            and detached["packet_issuer_fingerprint_present"] is False, "unexpected issuer fields")
    require(record["disposition"] == "EXECUTABLE_TRUST_HELD", "executable trust hold missing")
    require(set(record["qualification"]) == {
        "artifact_signature_verified", "current_key_authority_verified", "key_imported",
        "cache_installed", "artifact_executed", "native_compile_accepted", "runtime_closure_verified",
    } and all(value is False for value in record["qualification"].values()), "unsupported qualification")


def main():
    record = json.loads((EVIDENCE / "observation.json").read_bytes())
    inventory = json.loads((EVIDENCE.parent / "batch33-passive-closure" / "requests.json").read_bytes())
    validate(record, inventory)
    print("PASS: SLF4J packet observation and prior JAR inventory agree; executable trust remains held")


if __name__ == "__main__":
    main()

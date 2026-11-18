"""Check the public signature observation without a network or GnuPG."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
OBSERVATION_SHA256 = "0f34b2c4da21ce5f69b31a84c1844e5fe1856092a159b1a8c87b9551c3b3dae3"
PARENT_SHA256 = "68528f34cdda6789e00cdeb653a2eb4f49cd8c4f28cf11c3c3bd9e0e12251b00"
PRIMARY = "A5BD02B93E7A40482EB1D66A5F69AD087600B22C"
KEY_PACKET = "7128d9276ee2da6de7a862c1f146974078c88ed1a3c4ddc471e3cb3afe33293d"
CORE_PARENT = HERE.parent / "batch33-passive-closure/captures/network/downloads/org/ow2/asm/asm/9.9"
CORE_CAPTURE = (
    ("", "jar", 126122, "b1739b74c01cb24f75571dd50b9ecf6dfa9137262a0e43d8f378434ed75530f0"),
    (".asc", "checksum_or_unverified_signature", 475,
     "f2671afff383cb068249dec57d4bdfb8f4860cec5aca784e8ff7d6497e8ee43f"),
)
TIMES = {"asm": 1759585309, "asm-commons": 1759585315, "asm-tree": 1759585320}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def unique(pairs):
    data = {}
    for key, value in pairs:
        require(key not in data, "duplicate JSON key")
        data[key] = value
    return data


def read(path, expected):
    with path.open("rb") as stream:
        raw = stream.read(65537)
    require(len(raw) <= 65536 and hashlib.sha256(raw).hexdigest() == expected, "input digest mismatch")
    return json.loads(raw, object_pairs_hook=unique)


def validate(observation, parent, core):
    require(observation["schema"] == "syson-asm-ow2-detached-signatures-v1", "schema mismatch")
    require(observation["parent_capture_sha256"] == PARENT_SHA256, "capture parent mismatch")
    require(observation["verification_receipt_sha256"] ==
            "7ba218ff9cd5ac7ac3d77d88f7ae3bd8db7b7c96c5e04a6b965265de323c6a37"
            and observation["key_capture_receipt_sha256"] ==
            "2dc77b8ba512bbb7e2cc4d8eb2d541e0876257ae01eeefc70ce198a9c3634547",
            "capture or verification receipt mismatch")
    key = observation["ow2_key"]
    require(key["url"] == "https://asm.ow2.io/KEY.txt" and type(key["http_status"]) is int
            and key["http_status"] == 200, "publisher key URL or status mismatch")
    require(key["raw_bytes"] == 2002
            and key["raw_sha256"] == "5809422ba83e33de621e8ecfde7d3669e85023136d1cf7281791bc400b642a65"
            and key["decoded_packet_bytes"] == 1184 and key["decoded_packet_sha256"] == KEY_PACKET
            and key["primary_fingerprint"] == PRIMARY, "publisher key byte or identity mismatch")
    authority = observation["authority"]
    require(set(authority) == {"artifact_execution", "cache_installation", "native_compile_accepted"}
            and all(value is False for value in authority.values()), "unexpected authority")
    method = observation["method"]
    require(set(method) == {"tool", "explicit_keyring", "empty_gnupg_home", "kernel_denied_socket_families"}
            and method["tool"] == "gpgv 2.4.4" and method["explicit_keyring"] is True
            and method["empty_gnupg_home"] is True
            and method["kernel_denied_socket_families"] == ["IPv4", "IPv6", "Unix"],
            "verification method mismatch")
    rows = observation["signatures"]
    require([row["name"] for row in rows] == list(TIMES), "artifact roster mismatch")
    captured = parent["request_rows"]
    for row in rows:
        name = row["name"]
        require(row["version"] == "9.9" and row["hash_algorithm"] == "SHA-512"
                and row["public_key_algorithm"] == "RSA" and row["signature_class"] == "binary"
                and row["signing_fingerprint"] == row["primary_fingerprint"] == PRIMARY
                and type(row["signed_unix_second"]) is int and row["signed_unix_second"] == TIMES[name],
                "signer, algorithm, version or timestamp mismatch")
        if name == "asm":
            require(row["capture_source"] == "earlier-passive-capture"
                    and row["capture_receipt_sha256"] ==
                    "e48815adff28f57ed8d0e2a7917a1a22a2329e59c108e45e0b63cfa801b18892"
                    and row["artifact_sha256"] ==
                    "03d99a74ad1ee5c71334ef67437f4ef4fe3488caa7c96d8645abc73c8e2017d4"
                    and row["signature_sha256"] ==
                    "6f1b215812e189dcae491a46e5803b123b18c94f1c65cdc48d0039e417b1523c",
                    "core ASM capture mismatch")
            for (suffix, role, size, _), entry, digest in zip(
                    CORE_CAPTURE, core, (row["artifact_sha256"], row["signature_sha256"]), strict=True):
                coordinate = "org/ow2/asm/asm/9.9/asm-9.9.jar" + suffix
                url = "https://repo.maven.apache.org/maven2/" + coordinate
                require(entry["url"] == entry["final_url"] == url and entry["role"] == role
                        and entry["relative_path"] == "downloads/" + coordinate + ".partial"
                        and entry["bytes"] == size and entry["status"] == 200
                        and entry["sha256"] == digest, "landed core body or signature mismatch")
            continue
        require(row["capture_source"] == "jacoco-binaries-passive-run"
                and row["capture_receipt_sha256"] ==
                "06af3ed0fdf005bea58ae9bf332f11e2d282090a53f6720fd526bc143409ea7e",
                "Commons or Tree capture receipt mismatch")
        path = f"downloads/org/ow2/asm/{name}/9.9/{name}-9.9.jar"
        for role, suffix, digest in (("jar", ".partial", row["artifact_sha256"]),
                                      ("signature", ".asc.partial", row["signature_sha256"])):
            candidates = [entry for entry in captured if entry["relative_path"] == path + suffix]
            require(len(candidates) == 1 and candidates[0]["role"] == role
                    and candidates[0]["status"] == 200 and candidates[0]["sha256"] == digest,
                    "inherited body or signature mismatch")


def main():
    core = [read(CORE_PARENT / ("asm-9.9.jar" + suffix + ".partial.response.json"), digest)
            for suffix, _, _, digest in CORE_CAPTURE]
    validate(read(HERE / "observation.json", OBSERVATION_SHA256),
             read(HERE.parent / "jacoco-binaries-passive-run/observation.json", PARENT_SHA256), core)
    print("PASS exact OW2 key and three ASM 9.9 RSA/SHA-512 signature observations")


if __name__ == "__main__":
    main()

"""Compare a Rust correction ZIP with the two retained native exports."""

import hashlib
import json
from pathlib import Path
import runpy


PACKET = Path(__file__).resolve().parent
SNAPSHOT = runpy.run_path(str(PACKET.parent / "archive-hierarchy" / "verify.py"))
EXPORTS = ("native-initial.zip", "native-first-browser.zip")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def archive_info(path):
    raw = path.read_bytes()
    return {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}


def verify(receipt):
    require(receipt["release"] == "v2026.9.3" and receipt["bad_manifest_projects_after"] == 0,
            "reference release or upload refusal differs")
    previous = PACKET.parent / "archive-corrected-publication"
    source = previous / "rust-corrected.zip"
    require(archive_info(source) == receipt["input"], "Rust input archive differs")
    source_receipts = json.loads((previous / "receipt.json").read_bytes())["source_receipt_sha256"]
    require(all(source_receipts[mode] == receipt["source_receipt_sha256"]["rust_browser"][mode]
                for mode in ("desktop", "touch")), "Rust browser receipts differ")
    snapshot, normalize = SNAPSHOT["snapshot"], SNAPSHOT["normalize"]
    before, source_ids, source_uri = snapshot(source)
    exports = []
    shape = receipt["native_shape"]
    for name in EXPORTS:
        path = PACKET / name
        require(archive_info(path) == receipt["exports"][name], f"{name} digest differs")
        after, native_ids, native_uri = snapshot(path)
        require(len(source_ids) == len(native_ids) == 452 and source_ids.keys() == native_ids.keys()
                and not set(source_ids.values()) & set(native_ids.values()), f"{name} ID roles")
        require(native_ids["project"] == shape["projectId"]
                and native_ids["representation"] == shape["representationId"], f"{name} identity")
        ids = {identity: native_ids[role] for role, identity in source_ids.items()}
        ids[source_uri] = native_uri
        require(tuple(normalize(item, ids) for item in before) == after, f"{name} normalized state")
        layout = after[2]["representation"]["layoutData"]["nodeLayoutData"][shape["target"]["id"]]
        require(layout["position"] == {"x": 9, "y": 46}
                and layout["size"] == {"width": 206, "height": 101}
                and layout["resizedByUser"] is True and layout["movedByUser"] is False,
                f"{name} corrected geometry or flags")
        exports.append(native_ids)
    require(exports[0] == exports[1], "native project identities changed between exports")
    stop = receipt["scope_stop"]
    require(stop["cgroupAbsent"] is True and stop["returncode"] == 143
            and not stop["sourcePinsChanged"] and stop["stopReason"]["pressureReasons"],
            "interrupted run cleanup receipt differs")


if __name__ == "__main__":
    verify(json.loads((PACKET / "receipt.json").read_bytes()))
    print("Both native exports match the Rust correction after identity normalization.")

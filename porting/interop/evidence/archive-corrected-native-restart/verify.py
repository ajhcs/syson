"""Verify that native startup and fresh-browser exports preserve the same project."""

import hashlib
import json
from pathlib import Path
import runpy


PACKET = Path(__file__).resolve().parent
HIERARCHY = runpy.run_path(str(PACKET.parent / "archive-hierarchy" / "verify.py"))
EXPORTS = ("native-after-restart.zip", "native-fresh-browser.zip")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def info(path):
    raw = path.read_bytes()
    return {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}


def verify_geometry(result, native_ids, before):
    require(result["projectId"] == native_ids["project"]
            and result["representationId"] == native_ids["representation"]
            and len(result["rendered"]) == 9 and "Drivetrain" in result["rendered"],
            "fresh Chromium project or diagram differs")
    diagram = before[2]["representation"]["layoutData"]["nodeLayoutData"]
    saved = diagram[result["nodeId"]]
    require(saved["position"] == {"x": 9, "y": 46}
            and saved["size"] == {"width": 206, "height": 101}
            and saved["resizedByUser"] is True and saved["movedByUser"] is False,
            "saved geometry or flags differ")
    raw = result["rawDom"]
    node, parent, scale = raw["nodeBounds"], raw["parentBounds"], raw["viewportScale"]
    require(scale > 0, "invalid viewport scale")
    for group, keys in (("position", ("x", "y")), ("size", ("width", "height"))):
        for key in keys:
            measured = (node[key] - parent[key]) / scale if group == "position" else node[key] / scale
            require(abs(measured - result[group][key]) < 1e-6
                    and abs(measured - saved[group][key]) < 0.01,
                    f"rendered {key} differs from saved layout")


def verify(receipt):
    source = PACKET.parent / "archive-corrected-publication" / "rust-corrected.zip"
    old_native = PACKET.parent / "archive-corrected-native" / "native-first-browser.zip"
    require(info(source) == receipt["rust_input"]
            and info(old_native) == receipt["old_native_archive"], "prior archive bytes differ")
    snapshot, normalize = HIERARCHY["snapshot"], HIERARCHY["normalize"]
    rust, rust_ids, rust_uri = snapshot(source)
    previous = snapshot(old_native)
    before, native_ids, native_uri = previous
    require(len(native_ids) == 452 and rust_ids.keys() == native_ids.keys(), "identity coverage")
    mapping = {identity: native_ids[role] for role, identity in rust_ids.items()}
    mapping[rust_uri] = native_uri
    require(tuple(normalize(item, mapping) for item in rust) == before, "Rust project differs")
    for name in EXPORTS:
        new = PACKET / name
        require(info(new) == receipt["exports"][name] and snapshot(new) == previous,
                f"{name} state or generated identities differ")
    browser = receipt["browser"]
    require(browser["status"] == "COMPLETE_SINGLE_FRESH_CONTEXT" and not browser["pageErrors"],
            "fresh browser observation missing")
    verify_geometry(browser["fresh"], native_ids, before)
    database, teardown = receipt["database"], receipt["teardown"]
    preserved = database["preservation"]
    require(database["files"] == 1030 and database["directories"] == 26
            and database["distinct_clone_inodes"] is True
            and preserved["unchanged"] is True
            and preserved["beforeSha256"] == preserved["afterSha256"],
            "source database preservation receipt differs")
    require(teardown == {"returncode": 0, "cgroupAbsent": True, "stopReason": None,
                         "sourcePinsChanged": []}, "owned scope cleanup receipt differs")


if __name__ == "__main__":
    verify(json.loads((PACKET / "receipt.json").read_bytes()))
    print("Native restart and fresh Chromium match the original imported project.")

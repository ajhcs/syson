import hashlib
import io
import json
import re
import sys
from pathlib import Path
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parent
OBSERVATION = "5df5dee9d963652d16cb4bc17d3b2b8559c12cafaedbac7e445840e1289001a0"
BASELINE = "63b88989e6fcbe0a9d61da61aa162170aa843b49d88a09517e8c71bcfc2f2c4d"
ARCHIVE = "e24d7f1f4fe5ddf4e5047dc62b89c2cca029e367efaefbbc915656c9b08b1abf"
RETRY = {
    "missingOrigin": 403,
    "confirm": {"intent": "confirm", "status": 303, "sameRouteRedirect": True, "unchanged": True},
    "save": {"intent": "save", "status": 409, "sameRouteRedirect": False, "unchanged": True},
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def sha256_string(text):
    return isinstance(text, str) and re.fullmatch(r"[0-9a-f]{64}", text) is not None


def nodes_in_tree(nodes):
    for node in nodes:
        yield node
        yield from nodes_in_tree(node["childNodes"])


def check_archive(data):
    require(digest(data) == ARCHIVE, "published corrected archive changed")
    with ZipFile(io.BytesIO(data)) as archive:
        require(len(archive.infolist()) == 3 and archive.testzip() is None, "invalid archive")
        representation, = [name for name in archive.namelist() if "/representations/" in name]
        diagram = json.loads(archive.read(representation))["representation"]
        size_rows = diagram["layoutData"]["nodeLayoutData"]
        require(len(size_rows) == 9, "wrong layout count")
        drivetrain, = [node for node in nodes_in_tree(diagram["nodes"])
                       if node["insideLabel"]["text"] == "Drivetrain"]
        layout = size_rows[drivetrain["id"]]
        require(layout["size"] == {"width": 206, "height": 101} and
                layout["resizedByUser"] is True, "wrong corrected resize")


def check_observation(observation):
    require(observation["archive_sha256"] == ARCHIVE and
            observation["archive_commit"] == "02223020295cd540b13144ad3a296b7fd8ac8a88" and
            observation["loss_baseline_commit"] == "fadc19ee6933fa8f1f943a1b7fddaad088c9b380", "wrong parents")
    require(observation["fixture"] == {"layout_nodes": 9, "model_objects": 431, "width": 206, "height": 101},
            "wrong fixture")
    implementations = observation["implementations"]
    require(implementations["subject"] == "ee11656a278a293dec806868c979dee2b0e8be9be129760ea9f591996ee98ecd" and
            implementations["run_harness"] == "14729b7e2346fc21aca95f279a84aff6ed55229ad204a97816bd693459094461",
            "wrong tool identities")
    cases = observation["cases"]
    require(set(cases) == {"confirm-delivered", "confirm-withheld"}, "incomplete response comparison")
    for case in cases.values():
        for key in ("pending_download_hidden_before_restart", "pending_download_hidden_after_restart",
                    "confirmation_only_new_artifact", "pending_archive_identity_preserved",
                    "retained_parent_and_discarded_revision", "scope_removed", "repeat_preserves_both_artifacts"):
            require(case[key] is True, "missing witness: " + key)
        require(sha256_string(case["raw_receipt_sha256"]) and case["distinct_rust_processes"] == 6,
                "missing exact process witness")
        require(case["pending_save_http"] == 503 and case["fresh_download_sha256"] == ARCHIVE and
                case["retry"] == RETRY, "wrong pending or replay outcome")
    require(cases["confirm-delivered"]["delivery"] == {"chromium_statuses": [303], "saved_view": True},
            "missing delivered control")
    require(cases["confirm-withheld"]["delivery"] == {
        "upstream_requests": 1, "server_status": 303, "same_route": True, "empty_response": True,
        "chromium_statuses": [], "chromium_failures": ["net::ERR_CONNECTION_CLOSED"],
    }, "wrong withheld response")
    require(cases["confirm-delivered"]["raw_receipt_sha256"] != cases["confirm-withheld"]["raw_receipt_sha256"],
            "same receipt reused for both cases")


def main():
    raw = (ROOT / "observation.json").read_bytes()
    require(digest(raw) == OBSERVATION, "observation changed")
    baseline = ROOT.parent / "archive-corrected-response-loss" / "observation.json"
    require(digest(baseline.read_bytes()) == BASELINE, "Save loss baseline changed")
    check_archive((ROOT.parent / "archive-corrected-publication" / "rust-corrected.zip").read_bytes())
    check_observation(json.loads(raw))
    print("PASS: pending 404, delivered and withheld Confirm, unchanged repeat and fresh exact archive")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError) as error:
        sys.exit("REFUSE: " + str(error))

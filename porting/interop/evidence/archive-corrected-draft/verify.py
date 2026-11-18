"""Verify the staged correction and fresh-process reopen receipt."""

import hashlib
import json
from pathlib import Path


PACKET = Path(__file__).resolve().parent
EXPECTED = {
    "corrected": [2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 2, 0],
    "ticket-only": [2, 2],
    "pair-sync-failure": [2, 0],
    "correction-draft-file-sync": [2, 2, 0],
    "correction-draft-directory-sync": [2, 2, 0, 0],
    "changed-archive-correction.grant.json": [0, 2, 2],
    "changed-archive-correction.ticket.json": [0, 2, 2],
    "pre-stage-current-grant-inode": [0],
    "byte-identical-checkpoint": [0, 0],
    "occupied-archive-correction.confirmed.json": [2, 2],
    "occupied-archive-correction.zip": [2, 2],
    "tampered-checkpoint": [0, 2],
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def verify(receipt):
    checkpoint, staged, opened = (receipt[key] for key in ("checkpoint", "staged", "opened"))
    require(staged["event"] == "correction-draft-staged"
            and opened["event"] == "correction-draft-opened", "stage or reopen event mismatch")
    for key in ("candidate_archive_sha256", "draft_inode", "draft_sha256",
                "revision_key", "width", "height"):
        require(staged[key] == opened[key], f"fresh reopen changed {key}")
    require(checkpoint["revision_key"] == staged["revision_key"] == "archive-correction"
            and checkpoint["version"] == 7, "checkpoint scope mismatch")
    require((checkpoint["source_width"], staged["width"], staged["height"]) == (200, 206, 101)
            and (checkpoint["width"], checkpoint["height"]) == (206, 101),
            "corrected layout mismatch")
    encoded = json.dumps(checkpoint, sort_keys=True, separators=(",", ":")).encode("utf-8")
    require(digest(encoded) == staged["draft_sha256"], "staged checkpoint hash mismatch")
    require(checkpoint["candidate_archive_sha256"] == staged["candidate_archive_sha256"]
            == receipt["ordinary_comparator_sha256"], "ordinary resize comparator mismatch")
    parent = PACKET.parent / "archive-resize" / "rust-parent.zip"
    source = PACKET.parent / "archive-hierarchy" / "native-source.zip"
    require(digest(parent.read_bytes()) == checkpoint["parent_archive_sha256"]
            and digest(source.read_bytes()) == receipt["source_inputs"]["archive"]["sha256"],
            "prior archive bytes mismatch")

    outcomes = {}
    for run in receipt["outcomes"]:
        outcomes.setdefault(run["case"], []).append(run["exit"])
    require(outcomes == EXPECTED, "unexpected CLI outcome")
    require(receipt["http_process_count"] == 14 and receipt["cli_process_count"] == 63,
            "unexpected process inventory")


if __name__ == "__main__":
    verify(json.loads((PACKET / "receipt.json").read_text()))
    print("Corrected draft and fresh-process receipt match.")

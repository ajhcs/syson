"""Verify captured correction authority bindings and publication outcomes."""

import hashlib
import json
from pathlib import Path


PACKET = Path(__file__).resolve().parent
TICKET = ".archive-correction.ticket.json"
GRANT = ".archive-correction.grant.json"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, separators=(",", ":"), sort_keys=True).encode("utf-8")


def verify(receipt):
    ticket, grant, issued = (receipt[key] for key in ("issued_ticket", "issued_grant", "normal_issue"))
    require(digest(canonical(ticket)) == grant["ticket_sha256"] == issued["ticket_sha256"],
            "ticket hash binding mismatch")
    require(digest(canonical(grant)) == issued["grant_sha256"], "grant hash binding mismatch")
    require(ticket["project"] == grant["project"]
            and ticket["revision_key"] == grant["revision_key"] == issued["revision_key"]
            == "archive-correction"
            and grant["permission"] == "stage-one-corrected-width-resize", "grant scope mismatch")
    require((ticket["original_width"], ticket["discarded_width"], ticket["height"]) == (200, 210, 101),
            "baseline geometry mismatch")
    expected_widths = [width for width in range(188, 213) if width not in (200, 210)]
    cold = receipt["pending_discard_synced_in_cli_without_prior_ui_confirmation"]
    require(ticket["allowed_widths"] == issued["allowed_widths"] == cold["allowed_widths"]
            == expected_widths, "width authorization mismatch")
    require(issued["event"] == cold["event"] == "correction-authorized", "issuance event mismatch")
    for role in ("archive", "grant", "ticket"):
        require(ticket[f"source_{role}_sha256"] == receipt["source_inputs"][role]["sha256"],
                f"source {role} mismatch")
    source = PACKET.parent / "archive-hierarchy" / "native-source.zip"
    parent = PACKET.parent / "archive-resize" / "rust-parent.zip"
    require(digest(source.read_bytes()) == ticket["source_archive_sha256"]
            and digest(parent.read_bytes()) == ticket["parent_archive_sha256"], "archive mismatch")

    outcomes = {}
    for run in receipt["correction_cli"]:
        outcomes.setdefault(run["case"], []).append(run["exit"])
    pending = "pending-discard-confirmed-by-cli"
    require(outcomes["issued"] == [2, 2, 0, 0] and outcomes[pending] == [2, 0],
            "normal issuance or cold sync recovery mismatch")
    partial = {run["fault"]: set(run["visible_records_after_refusal"]) for run in receipt["partial"]}
    expected_partial = {
        "correction-ticket-file-sync": set(),
        "correction-ticket-directory-sync": {TICKET},
        "correction-grant-file-sync": {TICKET},
        "correction-grant-directory-sync": {TICKET, GRANT},
    }
    require(partial == expected_partial, "partial publication inventory mismatch")
    with_refusal = ("mixed-axis", "discarded-move", "foreign-profile", "linked-save",
                    "conflicting-confirmation", "foreign-correction-ticket")
    require(all(outcomes[name] == [2] for name in with_refusal), "accepted unavailable input")
    with_recovery = ("replaced-parent", "replaced-checkpoint", "replaced-marker",
                     "exact-grant-missing-ticket")
    require(all(outcomes[name] == [0, 2] for name in with_recovery), "accepted replaced input")
    require(all(outcomes[name] == [2, 0] for name in expected_partial)
            and receipt["http_process_count"] == 15 and receipt["cli_process_count"] == 43,
            "unexpected partial failure or process inventory")


if __name__ == "__main__":
    verify(json.loads((PACKET / "receipt.json").read_text()))
    print("Correction authority receipt and prior archive bindings match.")

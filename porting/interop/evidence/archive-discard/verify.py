"""Check recorded discard outcomes and their archive custody."""

import hashlib
import json
from pathlib import Path


PACKET = Path(__file__).resolve().parent
DRAFT = ".archive-successor.draft.json"
MARKER = ".archive-successor.discarded.json"
ARCHIVE = "archive-successor.zip"
PARENT = "archive-rim.zip"
EXPECTED = {
    "normal": ([303, 409, 303], "reopen-discard", {MARKER}),
    "uncertain": ([503], "reopen-discard", {MARKER}),
    "linked": ([503, 409], "reopen-pending", {ARCHIVE}),
    "stale": ([409], "reopen-draft", set()),
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def archive_digest(name):
    return hashlib.sha256((PACKET.parent / "archive-resize" / name).read_bytes()).hexdigest()


def verify_report(case, report, parent, resized):
    staged, terminal = report["staged"], report["terminal"]
    codes, reopen_phase, additions = EXPECTED[case]
    require(set(terminal) == set(staged) | additions, f"{case}: unexpected publication")
    require(staged[PARENT] == terminal[PARENT] == report["parent"]
            and terminal[PARENT]["sha256"] == parent, f"{case}: parent changed")
    for name in staged:
        before, after = staged[name], terminal[name]
        if case == "stale" and name == DRAFT:
            require(before["sha256"] == after["sha256"] and before["inode"] != after["inode"],
                    "stale test did not substitute a byte-identical checkpoint")
        else:
            require(before == after, f"{case}: staged file changed: {name}")
    denials = report["route_denials"]
    require(len(denials) == 4 and set(denials.values()) == {404}, f"{case}: route exposed")

    decision, reopen = report["phases"]
    outcome = decision["result"]
    calls = outcome["calls"]
    require([call["code"] for call in calls] == codes and reopen["phase"] == reopen_phase,
            f"{case}: decision or reopen mismatch")
    journal = report["journal"]
    if MARKER in additions:
        marker = terminal[MARKER]
        require(journal["decision"] == "discarded" and journal["parent_archive_sha256"] == parent
                and journal["checkpoint_sha256"] == staged[DRAFT]["sha256"]
                and journal["checkpoint_inode"] == staged[DRAFT]["inode"],
                f"{case}: marker binding mismatch")
        for role in ("grant", "ticket"):
            require(journal[f"{role}_sha256"] == staged[f".archive-successor.{role}.json"]["sha256"],
                    f"{case}: marker {role} binding mismatch")
        require(calls[0]["body"] == "op=discard&receipt=" + marker["sha256"]
                and outcome["markerBeforeDuplicate"] == outcome["markerAfterDuplicate"],
                f"{case}: request binding or marker changed")
        require(outcome["state"]["controls"] == {"save": 0, "discard": 0, "archive": 0}
                and reopen["result"] == {"parentArchive": parent, "parentSize": [200, 101]},
                f"{case}: closed slot or parent mismatch")
    else:
        require(journal is None, f"{case}: unexpected marker")
    if case == "normal":
        require(calls[-1]["body"] == calls[0]["body"]
                and outcome["state"]["preview"] == [210, 101], "normal duplicate or preview")
    if case == "uncertain":
        require(calls[0]["trustedTouch"] is True and outcome["state"]["followUpStatus"] == 503
                and outcome["state"]["retryConfirmation"] is True,
                "fault path prematurely confirmed discard")
    if case == "linked":
        require(terminal[ARCHIVE]["sha256"] == resized and reopen["result"] == {"pending": True},
                "linked archive custody or pending state")
    if case == "stale":
        require(outcome["oldForm"]["receipt"] != outcome["freshForm"]["receipt"]
                and int(outcome["oldInode"]) == staged[DRAFT]["inode"]
                and int(outcome["newInode"]) == terminal[DRAFT]["inode"]
                and reopen["result"] == {"draft": True}, "stale form or reopen mismatch")


def verify(data):
    browser, http = data["browser"], data["http"]
    require(set(browser) == set(EXPECTED), "browser scenario coverage")
    parent, resized = (archive_digest(f"rust-{kind}.zip") for kind in ("parent", "resized"))
    for case, report in browser.items():
        require(report["subject"]["subject"] == http["binary_sha256"]
                and report["process_count"] == 4, f"{case}: subject or process count mismatch")
        verify_report(case, report, parent, resized)
    race = http["race_post_statuses"]
    winner = {"Save": 0, "Discard": 1}[http["race_winner"]]
    require(http["process_count"] == 9 and len(race) == 2
            and race[winner] == 303 and race[1 - winner] in (409, 503), "HTTP race outcome")


if __name__ == "__main__":
    verify(json.loads((PACKET / "runs.json").read_text()))
    print("Durable discard receipts and archive custody match.")

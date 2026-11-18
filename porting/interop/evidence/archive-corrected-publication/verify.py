"""Check the downloaded correction and recorded desktop and touch outcomes."""

import hashlib
import json
import zipfile
from pathlib import Path


PACKET = Path(__file__).resolve().parent
DESKTOP = {"correction-stage": [303, 303], "correction-save": [303, 409],
           "correction-reopen": []}
TOUCH = {"correction-stage": [303, 303], "correction-file-fault": [500, 500],
         "correction-link-fault": [503], "correction-confirm": [303],
         "correction-reopen": []}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def verify_archive(selected):
    parent = PACKET.parent / "archive-resize" / "rust-parent.zip"
    with (zipfile.ZipFile(parent) as original,
          zipfile.ZipFile(PACKET / "rust-corrected.zip") as corrected):
        require(corrected.testzip() is None and original.namelist() == corrected.namelist(),
                "corrected archive member mismatch")
        diagram, = [name for name in original.namelist() if "/representations/" in name]
        for member in original.namelist():
            if member != diagram:
                require(original.read(member) == corrected.read(member), f"unexpected change in {member}")
        expected = json.loads(original.read(diagram))
        node = expected["representation"]["layoutData"]["nodeLayoutData"][selected]
        require(node["position"] == {"x": 9, "y": 46}
                and node["size"] == {"width": 200, "height": 101}
                and node["resizedByUser"] is False, "parent layout mismatch")
        node["size"]["width"] = 206
        node["resizedByUser"] = True
        require(expected == json.loads(corrected.read(diagram)), "unexpected diagram change")


def verify_browser(mode, browser, expected, archive, binary):
    require(browser["subject_sha256"] == binary and browser["archive_sha256"] == archive,
            f"{mode} binary or archive mismatch")
    require(browser["objects"] == 431 and browser["diagram_nodes"] == 9, f"{mode} fixture size")
    phases = {phase["phase"]: phase for phase in browser["phases"]}
    require(set(phases) == set(expected), f"{mode} phase coverage")
    for name, codes in expected.items():
        require([write["http"] for write in phases[name]["writes"]] == codes, f"{mode} {name}")
    stage = phases["correction-stage"]
    require([run["postBody"] for run in stage["writes"]] == ["op=correction&width=206"] * 2,
            f"{mode} stage requests")
    chooser = stage["result"]["chooser"]
    require(chooser["value"] == 206 and chooser["height"] >= 44, f"{mode} width chooser")
    if mode == "desktop":
        require(stage["focus"] and all(item["outline"] == 3 for item in stage["focus"]),
                "desktop chooser focus")
    else:
        require(stage["touch"] and all(item["trusted"] and item["height"] >= 44
                                      for item in stage["touch"]),
                "missing trusted touch on chooser")
        require(phases["correction-confirm"]["writes"][0]["postBody"] == "op=confirm",
                "touch recovery operation mismatch")
    reopen = phases["correction-reopen"]["result"]
    require(reopen["sha256"] == archive and reopen["retainedOldGeometry"] is True,
            f"{mode} cold reopen")
    saved = browser["saved"]
    for name, before in browser["retained"].items():
        require(before == saved[name], f"{mode} prior file changed: {name}")
    require("archive-successor.zip" not in saved, f"{mode} discarded draft published")
    require(saved["archive-correction.zip"]["sha256"] == archive,
            f"{mode} archive bytes differ from download")


def verify(data):
    archive = hashlib.sha256((PACKET / "rust-corrected.zip").read_bytes()).hexdigest()
    previous = PACKET.parent / "archive-corrected-draft" / "receipt.json"
    imported = json.loads(previous.read_text())
    http = data["http"]
    require(archive == imported["ordinary_comparator_sha256"] == http["archive_sha256"],
            "candidate archive differs from staged comparator")
    require(http["reference_b1_receipt_sha256"] == imported["source_receipt_sha256"],
            "draft receipt reference differs")
    require(http["confirmation"]["archive_sha256"] == archive, "confirmation archive mismatch")
    require(set(http["faults"]) == {
        "correction-draft-file-sync", "correction-draft-directory-sync",
        "correction-file-sync", "correction-directory-sync", "correction-confirmation-directory-sync",
    }, "HTTP faults missing")
    desktop, touch = data["desktop"], data["touch"]
    verify_browser("desktop", desktop, DESKTOP, archive, http["binary_sha256"])
    verify_browser("touch", touch, TOUCH, archive, http["binary_sha256"])
    require(desktop["selected_node"] == touch["selected_node"], "edited node mismatch")
    verify_archive(desktop["selected_node"])


if __name__ == "__main__":
    verify(json.loads((PACKET / "receipt.json").read_text()))
    print("Corrected archive and recorded desktop and touch publication match.")

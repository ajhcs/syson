"""Replay the recorded Chromium observations and compare accepted archive bytes."""

import hashlib
import json
from pathlib import Path


PACKET = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def phase_named(report, name):
    phase, = (phase for phase in report["phases"] if phase["phase"] == name)
    return phase


def verify_move(report):
    scenarios = {
        "branch invalidates redo": ([11, 46], True, False, True),
        "cancel cleared history": ([9, 46], False, False, False),
        "held partial cannot stage": ([19, 50], False, False, False),
        "one undo returns clean": ([9, 46], False, True, False),
        "no-op gesture keeps redo": ([9, 46], False, True, False),
        "one redo restores gesture": ([19, 50], True, False, True),
    }
    for phase_name, gesture in (("draft", "mouse"), ("move-touch-preview", "touch")):
        preview = phase_named(report, phase_name)["move"]
        states = {point["name"]: point for point in preview["points"]}
        expected = scenarios
        posts = ["op=move&x=19&y=50"]
        if gesture == "touch":
            expected = scenarios | {
                "trusted pointercancel leaves clean history": ([9, 46], False, False, False),
                "reload history empty": ([9, 46], False, False, False),
            }
            posts = []
            require(preview["trusted"]["Undo"] > 0 and preview["trusted"]["Redo"] > 0,
                    "touch controls must have trusted input")
        require(preview["gesture"] == gesture and preview["postForms"] == posts,
                f"{gesture} move submitted an unexpected operation")
        for name, state in expected.items():
            received = tuple(states[name][key] for key in ("position", "undo", "redo", "stage"))
            require(received == state, f"{gesture} move: {name}")


def verify_resize(report, gesture):
    draft = phase_named(report, "draft")
    preview = draft["move"]
    history = preview["previewHistory"]
    require(preview["gesture"] == gesture, f"missing {gesture} resize")
    require(preview["afterCancel"] == history["pointerUndo"] == [200, 101],
            f"{gesture} resize did not return to the baseline")
    require(history["pointerRedo"] == [210, 101] and history["branched"] == [202, 101],
            f"{gesture} resize redo or branch mismatch")
    for observation in ("externalShortcutIgnored", "noOpRetainedRedo", "stagingBlockedWhileHeld"):
        require(history[observation] is True, f"{gesture} resize: {observation}")
    if gesture == "touch":
        require(history["touchCancelRetainedRedo"] is True, "touch cancel lost redo")
        require(draft["viewport"] == {"width": 390, "height": 844}, "touch viewport mismatch")
    require(preview["postForms"] == ["op=resize&width=210&height=101"],
            f"{gesture} resize submitted an unexpected operation")


def verify(data):
    move, mouse, touch = (data[key] for key in ("move", "resize_mouse", "resize_touch"))
    bound = data["history_bound"]
    binary = move["subject"]["subject"]
    require(mouse["subject"] == touch["subject"], "resize runs used different drivers")
    require(binary == mouse["subject"]["syson-archive-reader-experiment"]
            == bound["binary_sha256"], "runs used different binaries")
    require(move["fixture"] == mouse["fixture"] == touch["fixture"], "fixture mismatch")
    require(move["fixture"]["model_objects"] == 431 and move["fixture"]["rendered_nodes"] == 9,
            "unexpected fixture size")
    verify_move(move)
    verify_resize(mouse, "mouse")
    verify_resize(touch, "touch")
    results = {result["mode"]: (result["undoBound"], result["errors"])
               for result in bound["results"]}
    require(results == {"move": (32, []), "resize": (32, [])}, "history limit mismatch")
    moved = sha256(PACKET.parent / "archive-layout" / "rust-moved.zip")
    restored = sha256(PACKET.parent / "archive-restore" / "rust-restored.zip")
    require(move["final_sha256"] == moved, "move archive differs from native-qualified input")
    require(mouse["final_sha256"] == touch["final_sha256"] == restored,
            "restore archive differs from native-qualified input")


if __name__ == "__main__":
    verify(json.loads((PACKET / "browser.json").read_text()))
    print("Preview history receipts and native-qualified archives match.")

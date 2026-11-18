"""Verify the sealed sanitized C03a observation without a compiler or network."""
import argparse
import copy
from hashlib import sha256
import json
from pathlib import Path
import re

PATH = Path(__file__).with_name("observation.json")
OBSERVATION_SHA256 = "3759ebe8a7aca32c503f1ebf1ca0623090595ed710e8007b337b271e75c94c38"
MAIN = "05a4be35d4842272d0249ab43b9e49295591ef1e"
ORDERS = (
    ("fmt", None),
    ("guarded-tests", None),
    ("clippy", None),
    ("build", None),
    ("exact-candidate", None),
    ("stale-source-ref", "stale head ref"),
    ("omitted-owned-test", "omitted or undeclared source delta file"),
    ("reordered-exports", "candidate generated export/order mismatch"),
    ("wrong-owned-blob", "source blob mismatch"),
    ("guarded-ancestor-alias-positive", None),
    ("guarded-final-link-negative", "regular file required; symlinks and special files refuse"),
    ("guarded-unopened-fifo-negative", "regular file required; symlinks and special files refuse"),
)
SETUP = (
    "guarded-ancestor-alias-positive",
    "guarded-final-link-negative",
    "guarded-unopened-fifo-negative",
)
GUARDED_FILES = {
    "Cargo.lock", "Cargo.toml", "src/lib.rs", "src/main.rs",
    "tests/composition.rs", "tests/rr14_regression.rs",
}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def digest(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None


def verify(record):
    require(set(record) == {"schema", "source", "fixture", "run", "historical", "private_receipt_sha256"}
            and record["schema"] == "c03a-guarded-same-base-fixture-v1", "unexpected observation")
    source = record["source"]
    require(source["c03a_head"] == "9d244f04e76ae0ec3291416b733ae1e35b470905" and
            source["rr14_head"] == "e1c536a2455fb2ffdf4423b93362eff32a2e0b62", "original or guarded source drift")
    require(set(source["guarded_files"]) == {"rust/tools/verify-composition/" + p for p in GUARDED_FILES}
            and all(map(digest, source["guarded_files"].values())) and digest(source["verifier_sha256"]),
            "guarded source or verifier digest missing")

    fixture = record["fixture"]
    require(fixture["current_main"] == MAIN and fixture["checkout_files"] == 341 and fixture["retained_refs"] == 6
            and fixture["candidate_tree"] == "aeca364ad8effb7bf5899d792e9b0a85d2507eb6", "pinned fixture differs")
    require(fixture["export_order"] == ["B25a", "B25b", "B25c", "B25d"] and
            fixture["ancestor_alias_same_report"] is True and fixture["state_unchanged"] is True,
            "fixture order, comparison, or readback differs")
    report = fixture["report"]
    require(report["whole_program_acceptance"] is False and report == {
        "base": "392a8af71df0c344c9c50e8ad922caf4e44e8506",
        "candidate": "5bf8614518da282fb1b1e33c7d7e7e04efe4121e",
        "sources": 4, "selected_files": 281, "generated_files": 1,
        "status": "VERIFIED_EXPLICIT_SOURCE_COMPOSITION", "whole_program_acceptance": False,
    }, "reported fixture acceptance differs")

    run = record["run"]
    require(len(run["commands"]) == len(ORDERS), "planned command missing")
    for result, (case, refusal) in zip(run["commands"], ORDERS, strict=True):
        expected = {"case": case, "exit": int(refusal is not None)}
        if refusal is not None:
            expected["refusal"] = refusal
        require(type(result["exit"]) is int and result == expected, f"wrong exit or refusal for {case}")
    named = run["named_tests"]
    test_names = (json.dumps(named, separators=(",", ":")) + "\n").encode()
    require(len(named) == len(set(named)) == 15 and
            sha256(test_names).hexdigest() == "d0b3446b00768b0ee9e18d9e0a7404b6fccf24b61cdbf3a75af98b3ebfbbc359",
            "successful named tests differ")
    require(tuple(run["successful_setup_cases"]) == SETUP and run["kernel_local_ipc_and_network_denial"] is True,
            "fixture setup or kernel witness missing")
    require(type(run["fifo_milliseconds"]) in (int, float) and
            0 < run["fifo_milliseconds"] < run["fifo_deadline_milliseconds"] == 2000, "FIFO deadline exceeded")
    require(run["owned_scope_empty_or_absent"] is True and run["compiler_lock_released_after_teardown"] is True,
            "scope teardown and lock release unconfirmed")

    historical = record["historical"]
    require(historical["older_same_base_fixture"] == "f19c06915a664eb636f6d0782c6d5191f9a43cd3" and
            historical["older_selected_files"] == 93 and historical["older_generated_files"] == 1,
            "older fixture changed")
    require(historical["ordered_current_main"] == {
        "anchor": "2d411e7a21825c18d43c2476af32365c65401a49", "head": MAIN,
        "crate_files": 25, "ordered_landings": 6, "positive": 1, "negative": 16,
    }, "separate ordered history evidence changed")
    receipts = record["private_receipt_sha256"]
    require(set(receipts) == {"subject", "controller", "scope", "fixture_manifest"}
            and all(map(digest, receipts.values())), "missing local custody digest")


def negative_controls(record):
    def change(*keys, value):
        def apply(target):
            for key in keys[:-1]:
                target = target[key]
            target[keys[-1]] = value
        return apply

    changes = (
        change("source", "rr14_head", value=record["source"]["c03a_head"]),
        change("fixture", "state_unchanged", value=False),
        change("fixture", "export_order", value=["B25b", "B25a", "B25c", "B25d"]),
        change("fixture", "report", "whole_program_acceptance", value=True),
        change("run", "commands", 5, "exit", value=0),
        change("run", "commands", 11, "refusal", value="unrelated failure"),
        change("run", "named_tests", 0, value="another_test"),
        change("run", "fifo_milliseconds", value=2000),
        change("run", "kernel_local_ipc_and_network_denial", value=False),
        change("run", "owned_scope_empty_or_absent", value=False),
        change("run", "compiler_lock_released_after_teardown", value=False),
        change("historical", "ordered_current_main", "crate_files", value=281),
    )
    for index, mutate in enumerate(changes, start=1):
        candidate = copy.deepcopy(record)
        mutate(candidate)
        try:
            verify(candidate)
        except ValueError:
            continue
        raise ValueError(f"altered record accepted in control {index}")
    return len(changes)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    with PATH.open("rb") as source:
        raw = source.read(65537)
    require(len(raw) <= 65536 and sha256(raw).hexdigest() == OBSERVATION_SHA256, "sealed observation differs")
    record = json.loads(raw)
    verify(record)
    if args.self_test:
        count = negative_controls(record)
        print(f"PASS sealed observation and {count} altered-record refusals")
    else:
        print("PASS sealed C03a guarded same-base observation")


if __name__ == "__main__":
    main()

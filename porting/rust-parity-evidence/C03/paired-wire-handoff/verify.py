"""Verify the sealed paired project-wire observation and its control cases."""
import argparse
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path
import re

OBSERVATION = Path(__file__).with_name("observation.json")
SEALED = "f16500c89ad4e80c97fb3fc57921975a49bf9d760e22cebc2b4c02977e1e03d7"
CHECKS = {"derive-private-probe-lock", "locked-metadata", "kernel-witness", "rust-tests", "clippy"}
STAMPS = ["connector", "annotation", "view", "references", "control", "diagram"]
BOUNDARY = {
    "in_memory_caller": True,
    "native_identifiers": False,
    "native_stamp_mapping": False,
    "sql_exercised": False,
    "transaction_accepted": False,
    "production_adopted": False,
}


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def verify(data):
    require(set(data) == {"schema", "source", "run", "boundary", "receipt_sha256"} and
            data["schema"] == "c03-paired-wire-handoff-v1", "unknown observation")
    source = data["source"]
    require(source == {
        "main": "05a4be35d4842272d0249ab43b9e49295591ef1e",
        "donor_head": "d98994ae1b795ff38ad418ce2725f9cc3117b0bc",
        "qualification_head": "36df49b0f9a75c5d069129409d514671dfcdcf39",
        "b06_shape_head": "8ce1ae50bc025214d0ad8e60691921c7fc61f6aa",
        "probe_sha256": "564dc2dddea07c26ad1cd348744aad271df9fe0930206a670168afdc7e8dfb70",
        "donor_evidence_sha256": "c108f3c3b5376f694e70628222af8af8c767dacf4b6b3b1c5da59a80e3640c49",
        "b06_shape_sha256": "6de726b58d39c950830e3d2dbd4b101d5385832347e0446ff3ac80af3b7cc72f",
        "identity_origin": "synthetic-b06-compound-key-shape",
    }, "caller or source evidence differs")
    run = data["run"]
    require(run["checks"].keys() == CHECKS and
            all(type(exit_code) is int and exit_code == 0 for exit_code in run["checks"].values()),
            "planned checks differ")
    require(run["lanes"] == ["semantic", "representation"] and run["stamp_families"] == STAMPS,
            "the paired lanes or six stamp families differ")
    require(run["input_pins"] == 242 and run["registry_tuples"] == 11 and
            run["probe_lock_sha256"] == "2ddd28f19af471799b5d9db28458eeae4e9b371d45119461698dd83d5fbe600d",
            "source or offline lock closure differs")
    names = run["named_tests"]
    compact = (json.dumps(names, separators=(",", ":")) + "\n").encode()
    require(len(names) == len(set(names)) == 8 and
            sha256(compact).hexdigest() == "0777c2eddd1f091325c53b63d78254d7cbb7fe4d806b82b8ea110825d9f7250f",
            "named caller controls differ")
    require(all(run[name] is True for name in
                ("source_inputs_unchanged", "scope_empty_or_absent", "compiler_lock_released_after_teardown")),
            "readback or ordered teardown is missing")
    require(data["boundary"].keys() == BOUNDARY.keys()
            and all(data["boundary"][name] is value for name, value in BOUNDARY.items()),
            "acceptance claim exceeds the in-memory witness")
    receipts = data["receipt_sha256"]
    require(receipts.keys() == {"scope", "subject", "controller"} and
            all(re.fullmatch(r"[0-9a-f]{64}", item) for item in receipts.values()), "receipt pin missing")


def negative_controls(data):
    mutations = [
        (("source", "donor_head"), "0" * 40),
        (("source", "identity_origin"), "native-row-identity"),
        (("run", "checks", "rust-tests"), 1),
        (("run", "lanes"), ["semantic"]),
        (("run", "registry_tuples"), 12),
        (("run", "named_tests", 0), "unrelated_control"),
        (("run", "scope_empty_or_absent"), False),
        (("run", "compiler_lock_released_after_teardown"), False),
        (("boundary", "sql_exercised"), True),
        (("boundary", "production_adopted"), True),
    ]
    mutations.extend((("run", "stamp_families"), STAMPS[:i] + STAMPS[i + 1:])
                     for i in range(len(STAMPS)))
    for route, replacement in mutations:
        amended = deepcopy(data)
        member = amended
        for part in route[:-1]:
            member = member[part]
        member[route[-1]] = replacement
        try:
            verify(amended)
        except ValueError:
            continue
        raise ValueError(f"accepted an altered observation at {route}")
    return len(mutations)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    with OBSERVATION.open("rb") as stream:
        raw = stream.read(65537)
    require(len(raw) <= 65536 and sha256(raw).hexdigest() == SEALED, "sealed observation differs")
    record = json.loads(raw)
    verify(record)
    if args.self_test:
        print(f"PASS sealed handoff and {negative_controls(record)} altered-record refusals")
    else:
        print("PASS sealed paired wire handoff")


if __name__ == "__main__":
    main()

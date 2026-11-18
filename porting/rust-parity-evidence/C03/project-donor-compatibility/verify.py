"""Verify the sealed C03 project-donor compatibility report offline."""
import argparse
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path
import re

PATH = Path(__file__).with_name("observation.json")
SHA256 = "c108f3c3b5376f694e70628222af8af8c767dacf4b6b3b1c5da59a80e3640c49"
DONORS = {
    "historical-81": "7f111c24bb3f79c44c55cdef2d4fa3246e50560b",
    "historical-82": "606b169def89d2b4dabf6a48e03d16f3d024277b",
    "corrected-104": "d98994ae1b795ff38ad418ce2725f9cc3117b0bc",
}
LOCKS = {
    "preparation": "d1d3ca98f73e74def390ef33840bcf4fd9f4812d40856edf7341cb52382a327c",
    "wire": "c6ff61f99e8be67fcedb47dc318c1d5ca1dde9dfdc3f4decb3783f7818ea0956",
}
TESTS = {
    "preparation": (17, "d45dd7fd6a722d92d9b015483bdae54f6a817337e8da65fcf358481cc9374f8d"),
    "wire": (10, "857d2290c03b056165caa8938f002ad6dc305d40cc5b7a76d6e821973a9f5b94"),
}
CHECKS = {"kernel-witness"} | {
    f"{crate}-{kind}" for crate in LOCKS for kind in ("original-locked-metadata", "tests", "clippy")
}


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def verify(data):
    require(set(data) == {"schema", "source", "run", "scope", "receipt_sha256"} and
            data["schema"] == "c03-project-donor-current-compatibility-v1", "unknown evidence shape")
    source = data["source"]
    require(source["main"] == "05a4be35d4842272d0249ab43b9e49295591ef1e" and
            source["fixture_evidence_head"] == "928a2db35c8f108ae64ebf21217d7c769f8cad03" and
            source["fixture_evidence_sha256"] == "3759ebe8a7aca32c503f1ebf1ca0623090595ed710e8007b337b271e75c94c38"
            and source["synthetic_fixture_commit"] == "5bf8614518da282fb1b1e33c7d7e7e04efe4121e"
            and source["current_dependency_files"] == 79, "pinned current closure differs")
    require(source["donors"] == DONORS and source["executed_donor"] == "corrected-104",
            "donor identity differs")
    require(source["included_project_fixture"] == {
        "path": "porting/rust-parity-evidence/C03b/project-fixtures.json",
        "blob": "8da18d65b6d43738db29b7ac493d780fcf368746",
        "sha256": "866841dfd30278df92cd4581ba939e8e181dce04f1015281c4189deb0653d867",
    }, "exact included fixture differs")

    run = data["run"]
    require(run["checks"].keys() == CHECKS and all(type(code) is int and code == 0
            for code in run["checks"].values()), "planned check missing or failed")
    require(run["locks"].keys() == LOCKS.keys() and run["tests"].keys() == TESTS.keys(),
            "crate receipt missing")
    require(type(run["lock_derivations"]) is int and run["lock_derivations"] == 0, "unexpected lock regeneration")
    for label, lock_digest in LOCKS.items():
        lock = run["locks"][label]
        require(type(lock["locked_metadata_exit"]) is int and lock["unchanged"] is True and lock == {
            "sha256": lock_digest, "matched_registry_tuples": 11,
            "locked_metadata_exit": 0, "unchanged": True,
        }, f"{label} lock selection changed")
        suite = run["tests"][label]
        names = suite["named_tests"]
        encoded = (json.dumps(names, separators=(",", ":")) + "\n").encode()
        count, suite_digest = TESTS[label]
        require(len(names) == len(set(names)) == count and sha256(encoded).hexdigest() == suite_digest
                and suite["compile_refusal_doctest"] is (label == "wire"), f"{label} suite differs")
    require(all(run[name] is True
                for name in ("inputs_unchanged", "owned_scope_empty_or_absent", "compiler_lock_released")),
            "input or teardown evidence missing")
    boundary = data["scope"]
    require(boundary["off_tree_compatibility"] is True and boundary["source_adopted"] is False and
            boundary["application_qualified"] is False, "claim exceeds the rehearsal")
    pins = data["receipt_sha256"]
    require(set(pins) == {"scope", "subject", "controller"} and
            all(re.fullmatch(r"[0-9a-f]{64}", value) for value in pins.values()), "local receipt digest invalid")


def negative_controls(data):
    altered = [
        (("source", "fixture_evidence_sha256"), "0" * 64),
        (("source", "executed_donor"), "historical-81"),
        (("source", "included_project_fixture", "blob"), "0" * 40),
        (("run", "checks", "wire-tests"), 1),
        (("run", "locks", "preparation", "unchanged"), False),
        (("run", "locks", "wire", "matched_registry_tuples"), 12),
        (("run", "lock_derivations"), 1),
        (("run", "tests", "preparation", "named_tests", 0), "unexpected_test"),
        (("run", "tests", "wire", "compile_refusal_doctest"), False),
        (("run", "compiler_lock_released"), False),
        (("scope", "source_adopted"), True),
        (("scope", "application_qualified"), True),
    ]
    for path, value in altered:
        mutated = deepcopy(data)
        parent = mutated
        for key in path[:-1]:
            parent = parent[key]
        parent[path[-1]] = value
        try:
            verify(mutated)
        except ValueError:
            continue
        raise ValueError(f"accepted changed observation at {path}")
    return len(altered)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    with PATH.open("rb") as source:
        raw = source.read(65537)
    require(len(raw) <= 65536 and sha256(raw).hexdigest() == SHA256, "sealed report differs")
    data = json.loads(raw)
    verify(data)
    if args.self_test:
        print(f"PASS sealed compatibility report and {negative_controls(data)} altered-record refusals")
    else:
        print("PASS sealed donor compatibility report")


if __name__ == "__main__":
    main()

"""Verify the public B06 row resolution report and its stacked provenance."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
OBSERVATION = "0913106255fd8567fa8d75621a22aa8ba7b9cb1635eba2a331ca317dc4613d62"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def verify(raw):
    if digest(raw) != OBSERVATION:
        raise ValueError("public row observation differs from the independently reviewed projection")
    return json.loads(raw)


def verify_stack(record):
    sources = {
        "paired_sql": "b06-paired-sql",
        "release_binding": "b06-release-repository-binding",
    }
    for key, directory in sources.items():
        raw = (ROOT.parent / directory / "observation.json").read_bytes()
        if digest(raw) != record["provenance"][key]["observation_sha256"]:
            raise ValueError(f"stacked {key} evidence differs")


def self_test(record):
    edits = (
        (("qualification", "case_counts", "refused"), 21),
        (("qualification", "case_counts", "identity_only_positive"), 1),
        (("qualification", "cases", 0, "outcome"), "AcceptPreparedWire"),
        (("qualification", "cases", 6, "outcome"), "Semantic"),
        (("qualification", "compiler_lock_released_after_teardown"), False),
        (("first_attempt", "case_check_exit"), 0),
        (("first_attempt", "resolver_unchanged_in_retry"), False),
        (("inputs", "repeated_fields_projected_from_recorded_join_equalities"), False),
        (("limits", "direct_sql_query_executed_by_this_run"), True),
        (("limits", "native_generation_binding_available"), True),
        (("limits", "native_six_stamp_binding_available"), True),
        (("limits", "prepared_wire_handoff_authorized"), True),
    )
    for path, replacement in edits:
        changed = deepcopy(record)
        current = changed
        for key in path[:-1]:
            current = current[key]
        if current[path[-1]] == replacement:
            raise ValueError(f"mutation did not change {path}")
        current[path[-1]] = replacement
        raw = (json.dumps(changed, sort_keys=True, indent=2) + "\n").encode()
        try:
            verify(raw)
        except ValueError:
            continue
        raise ValueError(f"altered public data accepted at {path}")
    return len(edits)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    record = verify((ROOT / "observation.json").read_bytes())
    verify_stack(record)
    rejected = self_test(record) if args.self_test else 0
    counts = record["qualification"]["case_counts"]
    print(f"PASS {counts['identity_only_positive']} identity-only cases and {counts['refused']} refusals; "
          f"{rejected} altered records rejected")


if __name__ == "__main__":
    main()

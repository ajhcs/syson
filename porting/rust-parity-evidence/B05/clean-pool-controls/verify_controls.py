"""Replay the observed clean SysON pool states against the seeded fixture."""
import hashlib
import json
from pathlib import Path
import runpy
import sys

SEED = Path(__file__).resolve().parent.parent / "release-pool-observation"
OBSERVATIONS_SHA256 = "c0c3e26906f4646f98af23c703a94de58e62fca711984901657a4da4800637fa"
CLEAN_MIGRATIONS = {
    "clean-first-boot": {"applied": 20, "previously_applied": 0},
    "clean-unseeded-clone-restart": {"applied": 0, "previously_applied": 20},
}
CLEAN_QUERY = {
    "applied_changesets": 20,
    "held_changelog_locks": 0,
    "project_rows": 0,
    "transaction_read_only": "on",
}


def compare(clean, seeded, distribution):
    if (clean["schema"] != "syson-b05-clean-pool-controls-v1"
            or clean["artifact"] != seeded["artifact"]
            or clean["fixtures"].keys() != CLEAN_MIGRATIONS.keys()
            or clean["restart_custody"] != {
                "first_boot_cluster_unchanged": True, "restart_clone_inode_disjoint": True}):
        raise ValueError("clean fixture provenance or custody differs")
    expected_capture = {key: seeded["capture"][key] for key in (
        "distinct_physical_backend_count", "connections_closed",
        "jvm_active_processor_count_override", "phase")}
    result = {}
    for name, fixture in clean["fixtures"].items():
        if (fixture["capture"] != expected_capture
                or fixture["migrations"] != CLEAN_MIGRATIONS[name]
                or fixture["unseeded_cluster_query"] != CLEAN_QUERY):
            raise ValueError(f"{name} capture or verified startup state differs")
        if fixture["hikari"] != seeded["hikari"] or fixture["sessions"] != seeded["sessions"]:
            raise ValueError(f"{name} ordered observations differ from the seeded fixture")
        result[name] = {"migrations": fixture["migrations"], "search_path": distribution}
    return result


def main():
    try:
        if len(sys.argv) > 2:
            raise ValueError("usage: verify_controls.py [observations.json]")
        path = (Path(sys.argv[1]) if len(sys.argv) == 2
                else Path(__file__).with_name("observations.json"))
        raw = path.read_bytes()
        if hashlib.sha256(raw).hexdigest() != OBSERVATIONS_SHA256:
            raise ValueError("clean fixture digest differs from the reviewed observations")
        seeded_raw = (SEED / "observation.json").read_bytes()
        seeded_verifier = runpy.run_path(str(SEED / "verify_observation.py"))["verify"]
        result = compare(json.loads(raw), json.loads(seeded_raw), seeded_verifier(seeded_raw))
    except (ValueError, OSError) as failure:
        print(failure, file=sys.stderr)
        return 1
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())

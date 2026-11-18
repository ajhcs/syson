"""Verify the observed session settings in the pinned public fixture."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import sys

OBSERVATION_SHA256 = "73a0d34ecea7fd0c093379ddd2338a52cdfcbd609bed9827a28660dc1ba26e23"
DEFAULT_PATH = '"$user", public'
SESSION_PATH = 'public, "$user", public'
COMMON = {
    "search_path_reset": DEFAULT_PATH,
    "search_path_boot": DEFAULT_PATH,
    "search_path_sourcefile_visible": False,
    "schema": "public",
    "explicit_schemas": "{public}",
    "effective_schemas": "{pg_catalog,public}",
    "auto_commit": True,
    "read_only": False,
    "default_transaction_isolation": "read committed",
    "transaction_isolation": "read committed",
    "transaction_read_only": "off",
    "default_transaction_read_only": "off",
    "statement_timeout": "0",
    "lock_timeout": "0",
}


def verify(raw):
    if hashlib.sha256(raw).hexdigest() != OBSERVATION_SHA256:
        raise ValueError("fixture digest differs from the reviewed observation")
    fixture = json.loads(raw)
    sessions = fixture["sessions"]
    capture = fixture["capture"]
    pool = fixture["hikari"]
    if (fixture["schema"] != "syson-b05-pool-observation-v1"
            or len(sessions) != 10 or capture["distinct_physical_backend_count"] != 10
            or capture["connections_closed"] is not True
            or capture["original_cluster_unchanged"] is not True
            or [row["borrow_order"] for row in sessions] != list(range(1, 11))
            or pool["minimum_idle"] != 10 or pool["maximum_pool_size"] != 10):
        raise ValueError("invalid checkout, pool or teardown observation")
    session_keys = COMMON.keys() | {"borrow_order", "search_path", "search_path_source"}
    if any(row.keys() != session_keys
           or any(type(row[key]) is not type(value) or row[key] != value
                  for key, value in COMMON.items()) for row in sessions):
        raise ValueError("observed common session settings differ")
    distribution = Counter((row["search_path_source"], row["search_path"]) for row in sessions)
    if distribution != {("session", SESSION_PATH): 1, ("default", DEFAULT_PATH): 9}:
        raise ValueError("observed search_path distribution differs")
    return [{"source": source, "search_path": search_path, "count": count}
            for (source, search_path), count in sorted(distribution.items())]


def main():
    try:
        if len(sys.argv) > 2:
            raise ValueError("usage: verify_observation.py [observation.json]")
        fixture_path = (Path(sys.argv[1]) if len(sys.argv) == 2
                        else Path(__file__).with_name("observation.json"))
        result = verify(fixture_path.read_bytes())
    except (ValueError, OSError) as failure:
        print(failure, file=sys.stderr)
        return 1
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())

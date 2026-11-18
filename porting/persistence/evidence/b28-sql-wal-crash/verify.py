from hashlib import sha256
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
PARENT_SHA256 = "6de726b58d39c950830e3d2dbd4b101d5385832347e0446ff3ac80af3b7cc72f"
PHASES = ["acknowledged", "writer_pending", "concurrent_reader", "recovered_reader"]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON key")
        result[key] = value
    return result


def same_json(left, right):
    return json.dumps(left, sort_keys=True) == json.dumps(right, sort_keys=True)


def verify(observation, parent):
    require(set(observation) == {"artifact", "initial_refusal", "trajectory", "transaction", "wal",
                                "recovery", "custody", "scope"}, "unexpected observation fields")
    require(observation["artifact"] == parent["artifact"] and
            observation["artifact"]["server_version"] == "150019", "release context drifted")
    require(same_json(observation["initial_refusal"], {"reason": "quiet host not sustained",
                                               "samples": 65, "server_started": False}),
            "initial refusal changed")

    committed = parent["fresh_reads"][0]["state"]
    rows = observation["trajectory"]
    require([row["phase"] for row in rows] == PHASES, "missing or reordered observation")
    for row in rows:
        require(set(row) == {"phase", "state", "read_only", "unix_socket"} and row["unix_socket"] is True,
                "invalid session observation")
        require(row["read_only"] is (row["phase"] != "writer_pending"), "unexpected reader isolation")
        if row["phase"] != "writer_pending":
            require(same_json(row["state"], committed), "committed state changed")
        else:
            pending = row["state"]["document_sha256"]
            require(re.fullmatch(r"[0-9a-f]{64}", pending) and pending != committed["document_sha256"]
                    and same_json(row["state"], {**committed, "document_bytes": committed["document_bytes"] + 2,
                                                 "document_sha256": pending}),
                    "pending write did not change only the expected document fields")

    require(same_json(observation["transaction"], {"writer_state": "idle in transaction",
                                           "writer_wait": ["Client", "ClientRead"],
                                           "same_xid_observed_before_shutdown": True,
                                           "commit_dispatched_for_pending_write": False,
                                           "pending_timestamp_changed": True,
                                           "pending_client_exit": 0,
                                           "pending_client_reaped": True}), "pending boundary changed")
    wal = observation["wal"]
    require(set(wal) == {"checkpoint_exit", "checkpoint_after_pending_bytes", "flush_after_checkpoint_bytes"}
            and type(wal["checkpoint_exit"]) is int and wal["checkpoint_exit"] == 0
            and type(wal["checkpoint_after_pending_bytes"]) is int and wal["checkpoint_after_pending_bytes"] > 0
            and type(wal["flush_after_checkpoint_bytes"]) is int and wal["flush_after_checkpoint_bytes"] >= 0,
            "insufficient WAL horizon")
    require(same_json(observation["recovery"], {"shutdown": "SIGQUIT", "before_restart": "in production",
        "after_recovery_stop": "shut down", "log_sequence": ["interrupted", "automatic recovery",
        "redo starts", "redo done", "ready for connections"], "fresh_postmaster": True}), "recovery evidence missing")
    require(same_json(observation["custody"], {"original_unchanged": True, "clone_inode_distinct": True,
        "socket_only": True, "postgres_exit_statuses": [0, 0], "scope_absent": True}),
        "resource custody evidence changed")
    require(same_json(observation["scope"], {"application_acceptance": False, "physical_power_loss": False,
        "application_transactions_exercised": False}), "scope overclaimed")


def main():
    historical = (HERE.parent / "b06-paired-sql" / "observation.json").read_bytes()
    require(sha256(historical).hexdigest() == PARENT_SHA256, "exact paired-write parent required")
    observation = (HERE / "observation.json").read_bytes()
    require(len(observation) <= 16384, "observation size limit")
    verify(json.loads(observation, object_pairs_hook=unique_pairs),
           json.loads(historical, object_pairs_hook=unique_pairs))
    print("PASS acknowledged pair survives WAL recovery; pending write does not")


if __name__ == "__main__":
    main()

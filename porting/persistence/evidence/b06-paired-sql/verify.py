from hashlib import sha256
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
F26 = "caf7b4aedeba3a38dca32629620c11df94fd3fd31dda2bb195492d9f7c06548c"
RELEASE = "b380dada415ce05155150f23cb3b74ae21cc45e4"
TARGET = "6ba2ac64e6b16ee2b05752ad55395eebb87d4910"
PHASES = ["before", "before_injected_failure", "after_savepoint_rollback", "after_commit", "after_stale_refusal"]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON key")
        result[key] = value
    return result


def verify(data, changelog):
    require(set(data) == {"artifact", "initial_refusal", "identity_check", "trajectory",
                         "expected_payload_sha256", "fresh_reads", "custody", "scope"}, "unexpected packet shape")
    require(data["artifact"] == {
        "release_sha256":"85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1",
        "release_source": RELEASE, "target_source": TARGET, "server_version": "150019",
    } and changelog["release_source"] == RELEASE and changelog["target_source"] == TARGET
            and len(changelog["observed"]) == 20, "incorrect release context")
    require(data["initial_refusal"] == {
        "mutations_observed": 0, "clean_shutdown": True, "false_predicate": "archive UUID alone equals storage key",
    }, "incorrect initial refusal")
    require(data["identity_check"] == {
        "projects": 1, "documents": 1, "metadata_rows": 1, "content_rows": 1,
        "executed_changes": 20, "migration_locks_held": 0,
        "metadata_uuid_matches_archive": True, "compound_key_verified": True,
        "metadata_content_association_verified": True, "read_only": True, "clean_shutdown": True,
    }, "identity evidence drifted")
    rows = data["trajectory"]
    require([row["phase"] for row in rows] == PHASES, "missing or reordered phase")
    require([set(row) for row in rows] == [{"phase","state"}, {"phase","state"}, {"phase","state","sqlstate"},
            {"phase","state"}, {"phase","state","document_rows","representation_rows"}], "unexpected phase fields")
    for row in rows:
        fields = row["state"]
        require(set(fields) == {"document_bytes","representation_bytes","document_sha256","representation_sha256",
                                "other_fields_sha256","document_marked","representation_marked",
                                "migration_version","last_migration_performed"}, "invalid state shape")
        for subject in ("document", "representation"):
            require(type(fields[subject+"_bytes"]) is int and 0 < fields[subject+"_bytes"] <= 1048576
                    and type(fields[subject+"_marked"]) is bool, "invalid payload observation")
        require(all(re.fullmatch(r"[0-9a-f]{64}", fields[key]) for key in
                    ("document_sha256","representation_sha256","other_fields_sha256")), "invalid digest")
    before, partial, restored, committed, stale = (row["state"] for row in rows)
    require(before["migration_version"] == "2026.7.0-202606240000"
            and before["last_migration_performed"] == "StartDoneAndControlNodeMigrationParticipant"
            and not before["document_marked"] and not before["representation_marked"], "invalid original state")
    expected = data["expected_payload_sha256"]
    require(set(expected) == {"document_sha256","representation_sha256"}
            and all(v != before[k] for k,v in expected.items()), "expected payloads were not changed")
    document_delta = {"document_bytes":before["document_bytes"]+1,
                      "document_sha256":expected["document_sha256"],"document_marked":True}
    representation_delta = {"representation_bytes":before["representation_bytes"]+1,
                            "representation_sha256":expected["representation_sha256"],"representation_marked":True,
                            "migration_version":"2099.0.0-209901020304",
                            "last_migration_performed":"SyntheticPairedWriteProbe"}
    require(partial == {**before, **document_delta}, "late failure did not follow document update")
    require(rows[2]["sqlstate"] == "23502" and restored == before, "rollback did not restore original pair")
    require(committed == {**before, **document_delta, **representation_delta}, "inconsistent committed pair")
    require(all(type(rows[4][field]) is int and rows[4][field] == 0
                for field in ("document_rows","representation_rows")) and stale == committed, "stale update changed data")
    require(data["fresh_reads"] == [{"phase":phase,"state":committed,"read_only":True,"unix_socket":True}
                                   for phase in ("after_commit","after_clean_restart")], "fresh read diverged")
    require(data["custody"] == {"original_unchanged":True,"postmasters_distinct":True,"start_times_distinct":True,
                                "exit_statuses":[0,0],"scope_absent":True}, "custody witness incomplete")
    require(data["scope"] == {"application_acceptance":False,"application_transaction_exercised":False,
                             "crash_or_power_loss_exercised":False}, "overclaimed observation")


def main():
    historical = (HERE.parent/"f26-release-changelog"/"observation.json").read_bytes()
    require(sha256(historical).hexdigest() == F26, "exact parent changelog required")
    def parse(raw):
        return json.loads(raw, object_pairs_hook=unique_pairs)
    verify(parse((HERE/"observation.json").read_bytes()), parse(historical))
    print("PASS paired rollback, commit, stale refusal, and clean restart")


if __name__ == "__main__":
    main()

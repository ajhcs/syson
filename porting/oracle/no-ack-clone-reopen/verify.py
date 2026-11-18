"""Verify the retained projection of the official SysON clone reopen."""
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
OBSERVATION = "245932acc531802d87f15a5ba6e79516513640a9970f65b1fb4b7fc3733f4f0c"
BASE = "5432d999f89e640c3992da4b0583304d796481b360ed758b55220f4a5d82b50e"
SCOPE = "6eded182e6ac986ec016f10444b87cf100f11a7e2a79b1951529e1c8657bd563"
RESULT = "86949f6b93e1fb602e52298aad2cc3d0cd4ea98c20577c9b82539a067164ef0e"
CONTROLLER = "2b6f4c8de3b333e393651e9bec8e1221172044e42f857f3628d4597cf82052c5"
LAYOUT_QUERY = "9de446490f2fa03b07f9325895b873b2dd5f75b84235e142943902ec80369d8e"
RECEIPTS = {"before-java-row.json", "layout-before-java.json", "layout-after-browser.json",
            "after-layout-admission.json", "clone-before-start.json", "source-preservation-after.json",
            "after.json", "after-barrier.json", "after-refused-graphql.json", "after-browser-errors.json", "after.zip"}


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def pinned(path, expected):
    raw = path.read_bytes()
    require(sha256(raw).hexdigest() == expected, f"changed evidence bytes: {path.name}")
    return json.loads(raw)


def check(record, baseline):
    require(record["baseline_pr"] == {
        "number": 386, "head": "aa08dbfc986c8327157b0f4da18fd7498d4f068e", "observation_sha256": BASE,
        "scope_sha256": baseline["controller"]["spec_sha256"],
    }, "original lost-response dependency changed")
    previous = baseline["forwarded_then_aborted"]
    require(baseline["controller"]["status"] == "FAIL_PRESERVED"
            and not any(baseline["restart"]["fresh_browser_artifacts"].values())
            and previous["browser_success_responses"] == 0
            and previous["observation"]["documentation_changed"] is True, "original run boundary changed")
    require(record["release"] == baseline["release"]
            and record["classification"] == "official released SysON stopped-cluster clone reopen only"
            and record["original_lost_response_validated"] is True
            and record["historical_documentation_edit_replayed"] is False, "release or runtime boundary changed")
    require(record["clone"] == {
        "atomic_standby_move_without_recopy": True, "every_file_and_directory_inode_distinct": True,
        "files_and_directories": 1055, "same_bytes_and_modes": True, "source_was_stopped": True,
        "used_sealed_standby": True, "source_still_stopped_and_unchanged": True, "postgres_ready_on_private_socket": True,
    }, "clone custody changed")
    require(record["selected_sql_row"] == {
        "historical_row_matches_except_database_start": True, "new_database_start": True,
        "row_same_through_browser": True, "both_expected": True, "both_independent_read_only_socket": True,
    }, "selected SQL row changed")
    require(record["browser"] == {
        "full_saved_route_matches": True, "documentation_matches_committed_value": True,
        "title_diagram_and_rendered_nodes_same": True, "selected_edit_requests": 0, "selected_edits_forwarded": 0,
        "selected_edit_success_responses": 0, "selected_edit_failed_requests": 0, "redirect_followups": 0,
        "transport_faults": 0, "browser_errors": 0, "unclassified_graphql": 0, "browser_closed_before_seal": True,
    }, "browser recovery or request accounting changed")
    require(record["layout"] == {
        "phase": "after", "cause": "refresh", "auto_layout_state": "UNCHANGED",
        "current_latest_selected_revision": True, "matching_geometry_rows": {"nodes": 1, "labels": 1, "edges": 0},
        "wire_document_sha256": LAYOUT_QUERY, "typed_success": True, "http_status": 200,
        "forwarded": 1, "browser_responses": 1, "browser_failures": 0,
        "selected_content_bytes_same": True, "selected_content_row_and_audit_same": True,
        "selected_metadata_same": True, "peer_representation_rows_same": True, "peer_representation_set_empty": True,
        "same_selected_identity_and_clone": True,
        "all_observations_owned_read_only": True, "content_xmin_changed_before_refresh": True,
        "content_xmin_changed_during_refresh": True, "content_xmin_stable_after_refresh": True,
    }, "layout refresh or persisted rows changed")
    require(record["archive"] == {
        "both_crc_valid": True, "entry_set_same": True, "semantic_member_bytes_same": True,
        "representation_member_bytes_same": True, "manifest_json_same": True,
    }, "archive comparison changed")
    require(record["controller"] == {
        "exit": 0, "status": "PASS_SELECTED_NATIVE_CLONE_REOPEN_ONLY", "verified_inputs_unchanged": True,
        "direct_client_reaped_before_scope_cleanup": True, "scope_empty_or_absent": True, "entry_closed": True,
        "scope_sha256": SCOPE, "result_sha256": RESULT, "controller_sha256": CONTROLLER,
    }, "controller or teardown changed")
    digests = record["private_receipt_sha256"]
    require(set(digests) == RECEIPTS and all(re.fullmatch("[a-f0-9]{64}", value) for value in digests.values()),
            "private receipt set changed")


def reject_mutations(record, baseline):
    edits = (
        ("clone", "every_file_and_directory_inode_distinct", False),
        ("clone", "source_still_stopped_and_unchanged", False),
        ("selected_sql_row", "row_same_through_browser", False),
        ("browser", "selected_edits_forwarded", 1),
        ("layout", "forwarded", 2),
        ("layout", "content_xmin_changed_during_refresh", False),
        ("layout", "peer_representation_set_empty", False),
        ("archive", "representation_member_bytes_same", False),
        ("controller", "scope_empty_or_absent", False),
    )
    for category, field, replacement in edits:
        changed = deepcopy(record)
        changed[category][field] = replacement
        try:
            check(changed, baseline)
        except ValueError:
            continue
        raise ValueError(f"accepted mutation {category}/{field}")
    return len(edits)


def main():
    record = pinned(HERE / "observation.json", OBSERVATION)
    baseline = pinned(HERE.parent / "no-ack-commit/observation.json", BASE)
    check(record, baseline)
    print(f"Verified official clone-reopen projection; rejected {reject_mutations(record, baseline)} mutations.")


if __name__ == "__main__":
    main()

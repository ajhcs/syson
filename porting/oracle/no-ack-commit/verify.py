"""Verify the retained projection of the native commit-before-abort observation."""
from copy import deepcopy
from hashlib import sha256
import json
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent
OBSERVATION = "5432d999f89e640c3992da4b0583304d796481b360ed758b55220f4a5d82b50e"
BASE = "94eadea7d6dc3e4198dc6d5a30a0b5d761ec305ae008f37cd4b12ccfc937675c"
TERMINAL = "b677c850936de807cdcb2035adc51b289d46be0efdd301e8b0c963c0372b0e85"
SCOPE = "6a0dbb841a16285b0a1f20506b2f87cdd1c639f6402a32cef90f0936c00e1436"
PREVIOUS_TERMINAL = "344f68ee9bbcb9de0dac988ce7643b88851364be61b8c81f1936116551878d82"
PREVIOUS_SCOPE = "bd533c09364934868e3ca1d30dc53829df6950536cbc47aaed579cf26ebbbd60"
ARTIFACTS = {"before-barrier.json", "control-barrier.json", "lost-barrier.json", "control-abort-fetch.json",
             "lost-abort-fetch.json", "acknowledged-request.json", "oracle-up.json", "java-after.stdout",
             "java-after.stderr"}
FRESH = {"after-browser.log", "after-barrier.json", "after.json", "after.zip", "result.json"}


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def pinned(path, expected):
    raw = path.read_bytes()
    require(sha256(raw).hexdigest() == expected, "changed evidence bytes")
    return json.loads(raw)


def phase(record, events, counts, *, committed=None):
    forwarded, success, failed, page_errors = counts
    expected = {
        "requests_seen": 1, "forwarded": forwarded, "browser_success_responses": success,
        "browser_failed_requests": failed, "correlated_page_errors": page_errors, "redirect_followups": 0,
        "transport_faults": 0, "browser_closed_before_seal": True, "strict_event_order": True, "events": events,
    }
    if committed is not None:
        expected["observation"] = {
            "same_row_identity": True, "same_database_start": True, "both_match_expected": True,
            "both_read_only_socket": True, "both_repeatable_read": True,
            "row_version_changed": committed, "documentation_changed": committed,
        }
        expected["fetch"] = {"selected_network_error": "net::ERR_FAILED", "other_failed_fetches": 0,
                             "failure_at_or_after_abort": True}
    require(record == expected, "phase chronology, observation or request accounting changed")


def check(record, baseline):
    require(record["baseline_pr"] == {"number": 307, "head": "254e8ebca43fb2d371446e9cb05c06ca7976ed9e",
                                       "restart_evidence_sha256": BASE}, "baseline dependency changed")
    reference = baseline["full_restart"]["result"]
    require(record["release"] == {"version": "v2026.9.3", "source_commit": reference["source_label"],
                                  "jar_sha256": reference["release_sha256"]}, "release identity differs")
    require(record["selected_operation"] == "editTextfield", "selected operation changed")
    phase(record["acknowledged_baseline"], [], (1, 1, 0, 0))
    phase(record["unforwarded_control"],
          ["read_only_before", "browser_edit_intercepted", "unforwarded_browser_abort", "read_only_after_control"],
          (0, 0, 1, 1), committed=False)
    phase(record["forwarded_then_aborted"],
          ["read_only_before", "browser_edit_intercepted", "target_forward", "upstream_ok_response_withheld",
           "read_only_commit_visible", "browser_abort_without_response"], (1, 0, 1, 0), committed=True)
    require(record["control_after_matches_forward_before"] is True, "different start rows")
    stopped = {"status": "FAIL_PRESERVED", "stop_reason": "host_resource_hold",
               "client_reaped_before_scope_cleanup": True, "scope_empty_or_absent": True}
    require(record["controller"] == {**stopped, "controller_sha256": TERMINAL, "spec_sha256": SCOPE,
                                     "entry_closed": True}, "terminal scope status changed")
    require(record["previous_hold"] == {**stopped, "controller_sha256": PREVIOUS_TERMINAL,
                                       "spec_sha256": PREVIOUS_SCOPE, "forwarded_barrier_record_present": False},
            "previous hold misrepresented")
    require(record["restart"] == {"replacement_java_startup_banner": True, "replacement_java_stderr_empty": True,
                                  "fresh_browser_artifacts": dict.fromkeys(FRESH, False)}, "restart boundary changed")
    hashes = record["private_run_artifact_sha256"]
    require(set(hashes) == ARTIFACTS and all(re.fullmatch(r"[a-f0-9]{64}", item) for item in hashes.values()),
            "private capture pins changed")


def reject_mutations(record, baseline):
    edits = (
        (("unforwarded_control", "forwarded"), 1),
        (("unforwarded_control", "observation", "row_version_changed"), True),
        (("forwarded_then_aborted", "browser_success_responses"), 1),
        (("forwarded_then_aborted", "observation", "documentation_changed"), False),
        (("forwarded_then_aborted", "events", 4), "browser_abort_without_response"),
        (("controller", "status"), "PASS"),
        (("restart", "fresh_browser_artifacts", "after.json"), True),
    )
    for route, replacement in edits:
        changed = deepcopy(record)
        row = changed
        for key in route[:-1]:
            row = row[key]
        row[route[-1]] = replacement
        try:
            check(changed, baseline)
        except ValueError:
            continue
        raise ValueError(f"accepted mutation {route}")
    return len(edits)


def main():
    record = pinned(HERE / "observation.json", OBSERVATION)
    baseline = pinned(HERE.parent / "restart/full-backend-restart-v2026.9.3.json", BASE)
    check(record, baseline)
    print(f"Verified native commit-before-abort projection; rejected {reject_mutations(record, baseline)} mutations.")


if __name__ == "__main__":
    main()

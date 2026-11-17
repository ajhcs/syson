# SPDX-License-Identifier: EPL-2.0

import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import differential
import golden
import oracle
import reference


class GoldenTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.directory = Path(tmp.name) / "golden"
        self.source = "package 'Snow 雪';"
        self.result = {"status": "ok", "report": None, "document": {"content": [
            {"eClass": "sysml:Package", "id": "p", "data": {"declaredName": "Snow 雪", "ownedRelationship": []}},
        ]}}

    def capture(self):
        return golden.capture("unicode", self.source, golden.REFERENCE, self.result)

    def test_capture_replays_pinned_reference(self):
        self.assertEqual(
            golden.REFERENCE,
            {"jar_sha256": oracle.PINNED[oracle.JAR], "jar_source_revision": oracle.JAR_SOURCE_REVISION},
        )
        record = self.capture()
        golden.write(self.directory, [record])
        self.assertEqual(golden.load(self.directory, "unicode", self.source), self.result)
        with self.assertRaises(FileExistsError):
            golden.write(self.directory, [record])

    def test_stale_missing_and_corrupt_records_fail(self):
        with self.assertRaisesRegex(golden.GoldenError, "unicode: golden record missing"):
            golden.load(self.directory, "unicode", self.source)
        record = self.capture()
        golden.write(self.directory, [record])
        path = self.directory / "unicode.json"
        with self.assertRaisesRegex(golden.GoldenError, "stale source bytes"):
            golden.load(self.directory, "unicode", self.source + " ")
        record["result"]["report"] = "tampered"
        path.write_text(json.dumps(record), encoding="utf-8")
        with self.assertRaisesRegex(golden.GoldenError, "digest mismatch"):
            golden.load(self.directory, "unicode", self.source)
        record["normalized"]["nodes"] = 20
        record["record_sha256"] = golden.record_digest(record)
        path.write_text(json.dumps(record), encoding="utf-8")
        with self.assertRaisesRegex(golden.GoldenError, "raw and normalized"):
            golden.load(self.directory, "unicode", self.source)
        record["reference"]["jar_source_revision"] = "a" * 40
        record["record_sha256"] = golden.record_digest(record)
        path.write_text(json.dumps(record), encoding="utf-8")
        with self.assertRaisesRegex(golden.GoldenError, "different reference"):
            golden.load(self.directory, "unicode", self.source)

    def test_harness_failure_is_not_a_document_refusal(self):
        payload = {"__typename": "ErrorPayload", "messages": [{"body": "refused"}]}
        with self.assertRaises(reference.ReferenceError):
            reference._payload({"uploadDocument": payload}, "uploadDocument")
        with self.assertRaises(reference.ReferenceProtocolError):
            reference._payload({"createProject": payload}, "createProject")
        with patch.object(reference, "_post", return_value={"errors": [{"message": "JVM failure"}]}):
            with self.assertRaises(reference.ReferenceProtocolError):
                reference.gql("http://127.0.0.1", "query", {})
        with patch.object(reference, "import_text", return_value={"documents": {}, "report": None}):
            result = differential.run_reference("unused", "unicode", self.source)
        self.assertEqual(result["status"], "harness_error")
        with self.assertRaisesRegex(golden.GoldenError, "not recordable"):
            golden.capture("unicode", self.source, golden.REFERENCE, result)

    def test_rust_timeout_malformed_output_and_crash_are_harness_errors(self):
        with patch.object(differential.subprocess, "run",
                          side_effect=subprocess.TimeoutExpired("oracle_import", 60)):
            timed_out = differential.run_rust(self.source)
        self.assertEqual(timed_out["status"], "crash")
        for process in (
            subprocess.CompletedProcess("oracle_import", 1, stdout=b"", stderr=b"process failed"),
            subprocess.CompletedProcess("oracle_import", 0, stdout=b'["unexpected"]', stderr=b""),
            subprocess.CompletedProcess("oracle_import", 0, stdout=b"not JSON", stderr=b""),
        ):
            with patch.object(differential.subprocess, "run", return_value=process):
                self.assertEqual(differential.run_rust(self.source)["status"], "crash")

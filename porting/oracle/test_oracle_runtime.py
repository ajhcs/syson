# SPDX-License-Identifier: EPL-2.0

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import Mock, patch

import oracle
from oracle_runtime import PG_LOADER_LIB, PATH_KEYS, check_runtime, choose_runtime, sha256


class RuntimeTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        self.env = {key: str(self.root / field) for field, key in PATH_KEYS.items()}

    def test_default_and_explicit_selection(self):
        self.assertIs(choose_runtime({}, oracle.DEFAULT_RUNTIME), oracle.DEFAULT_RUNTIME)
        runtime = choose_runtime(self.env, oracle.DEFAULT_RUNTIME)
        self.assertEqual(runtime.jar, self.root / "jar")
        self.assertEqual(runtime.pg_share, self.root / "pg_share")
        self.assertEqual(runtime.postgres_env({}), {})
        self.env[PG_LOADER_LIB] = str(self.root / "separate loader")
        runtime = choose_runtime(self.env, oracle.DEFAULT_RUNTIME)
        base = {"LANG": "C"}
        self.assertEqual(runtime.postgres_env(base), {
            "LANG": "C", "LD_LIBRARY_PATH": str(self.root / "separate loader"),
        })
        self.assertEqual(base, {"LANG": "C"})

    def test_partial_or_relative_configuration_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "set .*SYSON_ORACLE_PG_BIN"):
            choose_runtime({"SYSON_ORACLE_JAR": self.env["SYSON_ORACLE_JAR"]},
                           oracle.DEFAULT_RUNTIME)
        self.env["SYSON_ORACLE_NODE_BIN"] = "node/bin"
        with self.assertRaisesRegex(ValueError, "SYSON_ORACLE_NODE_BIN must be an absolute path"):
            choose_runtime(self.env, oracle.DEFAULT_RUNTIME)

    def test_explicit_start_rejects_missing_or_relative_run_path(self):
        for location, path in ((None, oracle.RUNS), ("relative/runs", Path("relative/runs"))):
            with self.subTest(location=location):
                env = dict(self.env)
                if location is not None:
                    env["SYSON_ORACLE_RUNS"] = location
                with patch.dict(os.environ, env, clear=True), patch("oracle.RUNS", path):
                    with patch("oracle.check_runtime") as validate:
                        with self.assertRaisesRegex(SystemExit, "absolute .* run directory"):
                            oracle.up()
                        validate.assert_not_called()

    def test_failed_publish_leaves_no_partial_current_state(self):
        run = self.root / "run"
        run.mkdir()
        state_path = self.root / "current.json"
        state_path.write_text("old state", encoding="utf-8")
        with patch("oracle.STATE", state_path):
            with patch("oracle.Path.replace", side_effect=OSError("publish failed")):
                with self.assertRaisesRegex(OSError, "publish failed"):
                    oracle.publish_state({"run": str(run)}, run)
            self.assertEqual(state_path.read_text(encoding="utf-8"), "old state")
            self.assertEqual(list(self.root.glob(".oracle-state-*")), [])
            oracle.publish_state({"run": str(run)}, run)
        self.assertEqual(json.loads(state_path.read_text(encoding="utf-8")), {"run": str(run)})

    def test_start_stops_spawned_processes_after_failed_state_publication(self):
        initdb = Mock(pid=101)
        initdb.wait.return_value = 0
        postgres, syson = Mock(pid=102), Mock(pid=103)
        with (
            patch.dict(os.environ, {"HOME": str(self.root)}, clear=True),
            patch("oracle.RUNS", self.root / "runs"),
            patch("oracle.STATE", self.root / "current.json"),
            patch("oracle.check_runtime"),
            patch("oracle.subprocess.Popen", side_effect=[initdb, postgres, syson]),
            patch("oracle.wait"),
            patch("oracle.free_port", side_effect=[10001, 10002]),
            patch("oracle.publish_state", side_effect=OSError("state path is not writable")),
            patch("oracle.stop") as stop,
        ):
            with self.assertRaisesRegex(OSError, "state path is not writable"):
                oracle.up()
        stop.assert_called_once()
        self.assertEqual([(entry["name"], entry["pid"]) for entry in stop.call_args.args[0]],
                         [("postgres", 102), ("syson", 103)])

    def fake_runtime(self):
        runtime = choose_runtime(self.env, oracle.DEFAULT_RUNTIME)
        for directory in (runtime.pg_share, runtime.pg_bin, runtime.pg_extension_lib, runtime.node_bin):
            directory.mkdir()
        for path in (runtime.jar, runtime.pg_share / "postgres.bki",
                     runtime.pg_share / "postgresql.conf.sample",
                     runtime.pg_extension_lib / "plpgsql.so", runtime.java,
                     runtime.pg_bin / "initdb", runtime.pg_bin / "postgres", runtime.node_bin / "node"):
            path.write_bytes(b"test")
            path.chmod(0o700)
        return runtime

    def test_missing_share_and_modified_jar_fail_before_starting_programs(self):
        runtime = self.fake_runtime()
        with patch("oracle_runtime.subprocess.run") as run:
            with self.assertRaisesRegex(ValueError, "pinned asset changed"):
                check_runtime(runtime, {runtime.jar: "wrong"})
            (runtime.pg_share / "postgres.bki").unlink()
            with self.assertRaisesRegex(ValueError, "SYSON_ORACLE_PG_SHARE"):
                check_runtime(runtime, {runtime.jar: sha256(runtime.jar)})
            run.assert_not_called()

    def test_version_checks_reject_wrong_postgres_and_node(self):
        runtime = self.fake_runtime()
        versions = ['openjdk version "21.0.12"', "initdb (PostgreSQL) 15.19",
                    "postgres (PostgreSQL) 15.19", "v24.21.0"]

        def results(values):
            return [subprocess.CompletedProcess([], 0, value, "") for value in values]

        with patch("oracle_runtime.subprocess.run", side_effect=results(versions)):
            check_runtime(runtime, {runtime.jar: sha256(runtime.jar)})
        for index, wrong in ((0, 'openjdk version "17.0.2"'), (1, "initdb (PostgreSQL) 16.8"),
                             (2, "postgres (PostgreSQL) 16.8"), (3, "v22.15.0")):
            with self.subTest(executable=index):
                selected = versions.copy()
                selected[index] = wrong
                with patch("oracle_runtime.subprocess.run", side_effect=results(selected)):
                    with self.assertRaisesRegex(ValueError, "unexpected version"):
                        check_runtime(runtime, {runtime.jar: sha256(runtime.jar)})


if __name__ == "__main__":
    unittest.main()

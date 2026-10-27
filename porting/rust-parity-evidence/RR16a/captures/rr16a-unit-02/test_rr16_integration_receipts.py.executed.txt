#!/usr/bin/env python3
"""Owned stub-process receipt regressions; never execute Cargo or Rust code."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import unittest

RUNNER = Path(__file__).resolve().parents[1] / "run_integration_checks.py"
STUB = r'''#!/usr/bin/python3
import os
from pathlib import Path
import sys
import time
import json
import subprocess
if os.environ.get("RR16_CHILD_PID"):
    child = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(3)"])
    fields = Path("/proc/" + str(child.pid) + "/stat").read_text().rsplit(")", 1)[1].split()
    Path(os.environ["RR16_CHILD_PID"]).write_text(json.dumps({"pid": child.pid, "start": fields[19], "group": os.getpgrp()}))
if os.environ.get("RR16_SLEEP"):
    time.sleep(float(os.environ["RR16_SLEEP"]))
if os.environ.get("RR16_ADD"):
    path = Path(os.environ["RR16_ADD"])
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("injected input\n")
if os.environ.get("RR16_MUTATE"):
    Path(os.environ["RR16_MUTATE"]).write_text("// persistent injected drift\n")
print("owned cargo stub; no compiler executed", flush=True)
print("test result: ok. " + os.environ.get("RR16_TESTS", "1") + " passed; 0 failed; 0 ignored; 0 measured; 0 filtered out", flush=True)
sys.exit(int(os.environ.get("RR16_EXIT", "0")))
'''

class Receipts(unittest.TestCase):
    def setUp(self):
        base = Path(os.environ["SYSON_RR16_TEST_ROOT"]).resolve(strict=True)
        self.assertTrue(base.is_dir() and base.is_absolute() and base.is_relative_to("/workspace"))
        self.directory = Path(tempfile.mkdtemp(prefix="rr16-fixture-", dir=base))
        self.root = self.directory / "source"
        self.root.mkdir()
        self.build = self.directory / "build"
        self.build.mkdir()
        self.bin = self.directory / "bin"
        self.bin.mkdir()
        self.stub = self.bin / "cargo"
        self.stub.write_text(STUB)
        self.stub.chmod(0o755)
        self.env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
        self.env.update(PATH=str(self.bin) + os.pathsep + os.defpath,
                        PYTHONDONTWRITEBYTECODE="1", GIT_CONFIG_GLOBAL=os.devnull,
                        GIT_CONFIG_SYSTEM=os.devnull)
        self.write("porting/run_integration_checks.py", RUNNER.read_bytes())
        for name in ["syson-metamodel", "syson-model", "syson-web", "syson-workflow", "syson-syntax"]:
            self.write(f"rust/crates/{name}/Cargo.toml", f'[package]\nname="{name}"\nversion="0.1.0"\n')
            self.write(f"rust/crates/{name}/src/lib.rs", "// stub fixture only\n")
        self.write("rust/validation/Cargo.toml", '[package]\nname="fixture-validation"\nversion="0.1.0"\n')
        self.write("rust/tools/new-observer/Cargo.toml", '[package]\nname="new-observer"\nversion="0.1.0"\n')
        self.write("rust/tools/new-observer/src/main.rs", "// not executed\n")
        self.write("backend/native-fixture.sql", "-- independently consumed fixture\n")
        for manifest in self.root.rglob("Cargo.toml"):
            manifest.with_name("Cargo.lock").write_text("version = 4\n")
        self.serial = 0
        self.git("init", "-q")
        self.git("add", ".")
        self.git("-c", "user.name=RR16 owned fixture", "-c", "user.email=rr16-fixture@example.invalid", "commit", "-qm", "immutable test candidate")
        self.candidate = self.git("rev-parse", "HEAD").strip()

    def write(self, relative, value):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(value if isinstance(value, bytes) else value.encode())
        return path

    def git(self, *argv):
        result = subprocess.run([shutil.which("git"), "-C", str(self.root), *argv],
                                env=self.env, capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        return result.stdout

    def commit(self):
        self.git("add", ".")
        self.git("-c", "user.name=RR16 owned fixture", "-c", "user.email=rr16-fixture@example.invalid", "commit", "-qm", "new immutable test candidate")
        self.candidate = self.git("rev-parse", "HEAD").strip()

    def command(self, name="syson-metamodel", manifest="rust/crates/syson-metamodel/Cargo.toml", kind="cargo-test"):
        return {"id": name, "kind": kind, "manifest": manifest, "argv": [str(self.stub), "test" if kind == "cargo-test" else "build", "--offline", "--locked", "--jobs", "1", "--manifest-path", manifest], "required_inputs": [manifest], "required_pins": [], "timeout_s": 5}

    def profile(self):
        tools = {self.stub, Path(shutil.which("git")).resolve(), Path(sys.executable).resolve(), Path("/usr/bin/python3").resolve()}
        return {"schema": "RR16a/1", "candidate": self.candidate,
                "commands": [self.command(), self.command("wasm", kind="cargo-build")],
                "held_manifests": {}, "historical_pins": [], "external_inputs": [],
                "tools": [{"path": str(p), "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(tools)],
                "env": {"PATH": str(self.bin) + os.pathsep + os.defpath, "HOME": os.environ["HOME"], "CARGO_HOME": str(self.build / "cargo-home"), "RUSTC": str(Path(sys.executable).resolve())}}

    def run_receipt(self, start="syson-metamodel", profile=None, **environment):
        number = self.serial
        self.serial += 1
        build = self.directory / f"build-{number}"
        build.mkdir()
        evidence = self.directory / f"evidence-{number}"
        admission = self.directory / f"admission-{number}.json"
        profile = self.profile() if profile is None else profile
        profile["env"].update(environment)
        admission.write_text(json.dumps(profile))
        result = subprocess.run([sys.executable, str(self.root / "porting/run_integration_checks.py"),
                                 "--build-dir", str(build), "--evidence-dir", str(evidence),
                                 "--admission", str(admission), "--start-at", start], cwd=self.root,
                                env=self.env, capture_output=True, timeout=20)
        self.evidence = evidence
        return result, json.loads((evidence / "receipt.json").read_text())

    def test_tool_manifest_is_part_of_the_receipt_input_inventory(self):
        _, receipt = self.run_receipt()
        self.assertIn("rust/tools/new-observer/Cargo.toml", receipt["source_sha256"])

    def test_backend_fixture_is_part_of_the_receipt_input_inventory(self):
        _, receipt = self.run_receipt()
        self.assertIn("backend/native-fixture.sql", receipt["source_sha256"])

    def test_source_drift_cannot_return_a_successful_receipt(self):
        result, receipt = self.run_receipt(RR16_MUTATE=str(self.root / "rust/crates/syson-model/src/lib.rs"))
        self.assertNotEqual(receipt["status"], "PASS")
        self.assertNotEqual(result.returncode, 0)

    def test_resume_suffix_is_never_a_broad_pass(self):
        _, receipt = self.run_receipt(start="wasm")
        self.assertNotEqual(receipt["status"], "PASS")

    def test_nonzero_command_remains_a_recorded_failure(self):
        result, receipt = self.run_receipt(RR16_EXIT="7")
        self.assertNotEqual(result.returncode, 0)
        self.assertNotEqual(receipt["status"], "PASS")
        self.assertEqual(receipt["commands"][0]["exit_code"], 7)

    def test_selected_scope_is_bound_but_never_full_acceptance(self):
        result, receipt = self.run_receipt()
        self.assertEqual(result.returncode, 0, receipt)
        self.assertEqual(receipt["status"], "SELECTED_CHECKS_PASSED")
        self.assertFalse(receipt["full_qualification"])
        self.assertEqual(receipt["before"], receipt["after"])
        self.assertEqual(receipt["inventory"]["rust/tools/new-observer/Cargo.toml"]["disposition"], "NOT_SELECTED")
        for command in receipt["commands"]:
            self.assertEqual(command["before"], command["after"])
            for stream in command["streams"].values():
                raw = (self.evidence / stream["path"]).read_bytes()
                self.assertEqual(hashlib.sha256(raw).hexdigest(), stream["sha256"])
        self.assertEqual(hashlib.sha256((self.evidence / "admission.json").read_bytes()).hexdigest(), receipt["admission_sha256"])

    def test_even_every_discovered_manifest_selected_is_not_full_pass(self):
        profile = self.profile()
        profile["commands"] = [self.command("manifest-" + str(i), p.relative_to(self.root).as_posix()) for i, p in enumerate(sorted(self.root.rglob("Cargo.toml")))]
        result, receipt = self.run_receipt(start="manifest-0", profile=profile)
        self.assertEqual(result.returncode, 0, receipt)
        self.assertTrue(all(item["disposition"] == "SELECTED" for item in receipt["inventory"].values()))
        self.assertFalse(receipt["full_qualification"])
        self.assertNotEqual(receipt["status"], "PASS")

    def test_resume_prefix_is_explicitly_unexecuted(self):
        result, receipt = self.run_receipt(start="wasm")
        self.assertEqual(result.returncode, 0, receipt)
        self.assertEqual(receipt["unexecuted_prefix"], ["syson-metamodel"])
        self.assertEqual([c["id"] for c in receipt["commands"]], ["wasm"])
        self.assertFalse(receipt["full_qualification"])

    def test_zero_reported_tests_and_timeout_are_recorded_failures(self):
        result, receipt = self.run_receipt(RR16_TESTS="0")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("no passed cases", receipt["error"])
        profile = self.profile()
        profile["commands"][0]["timeout_s"] = 0.1
        result, receipt = self.run_receipt(profile=profile, RR16_SLEEP="5")
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue(receipt["commands"][0]["timed_out"])
        self.assertEqual(receipt["commands"][0]["signal"], 9)

    def added_input(self, path):
        result, receipt = self.run_receipt(RR16_ADD=str(self.root / path))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("untracked", receipt["error"])
        self.assertEqual(len(receipt["commands"]), 1)

    def test_new_untracked_tool_after_command_blocks_success(self):
        self.added_input("rust/tools/late/Cargo.toml")

    def test_new_untracked_fixture_after_command_blocks_success(self):
        self.added_input("backend/late-fixture.sql")

    def test_dirty_candidate_executable_mode_and_symlink_are_refused(self):
        path = self.root / "rust/crates/syson-metamodel/src/lib.rs"
        path.chmod(0o755)
        result, receipt = self.run_receipt()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("mode drift", receipt["error"])
        path.chmod(0o644)
        (self.root / "linked").symlink_to(path)
        self.commit()
        result, receipt = self.run_receipt()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("unsupported Git mode", receipt["error"])

    def test_required_fixture_lock_pin_and_external_inputs_cannot_be_omitted(self):
        profile = self.profile()
        profile["commands"][0]["required_inputs"].append("backend/missing.sql")
        result, receipt = self.run_receipt(profile=profile)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("required repository input missing", receipt["error"])
        profile = self.profile()
        profile["commands"][0]["required_pins"] = ["missing-historical-input"]
        result, receipt = self.run_receipt(profile=profile)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("historical pin missing", receipt["error"])
        external = self.directory / "external.txt"
        external.write_text("pinned bytes")
        profile = self.profile()
        profile["external_inputs"] = [{"path": str(external), "sha256": hashlib.sha256(external.read_bytes()).hexdigest()}]
        result, receipt = self.run_receipt(profile=profile, RR16_MUTATE=str(external))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("external/tool drift", receipt["error"])
        lock = self.root / "rust/crates/syson-metamodel/Cargo.lock"
        lock.unlink()
        self.commit()
        result, receipt = self.run_receipt()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("lock missing", receipt["error"])

    def test_graph_and_factory_namespaces_never_execute(self):
        for path in ["rust/crates/syson-initializer-graph/Cargo.toml", "rust/validation/factory/child/Cargo.toml"]:
            self.write(path, '[package]\nname="held-fixture"\nversion="0.1.0"\n')
            self.write(str(Path(path).with_name("Cargo.lock")), "version=4\n")
        self.commit()
        for path in ["rust/crates/syson-initializer-graph/Cargo.toml", "rust/validation/factory/child/Cargo.toml"]:
            profile = self.profile()
            profile["commands"] = [self.command(manifest=path)]
            result, receipt = self.run_receipt(profile=profile)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(receipt["commands"], [])
            self.assertEqual(receipt["inventory"][path]["disposition"], "HELD_NOT_RUN")

    def test_short_features_workspace_and_config_argv_do_not_expand_scope(self):
        for tail in [["-F", "raw-factory-candidate"], ["--all-features"], ["--workspace"], ["-p", "held"], ["--config", "build.rustflags=[]"]]:
            profile = self.profile()
            profile["commands"][0]["argv"] += tail
            result, receipt = self.run_receipt(profile=profile)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(receipt["commands"], [])

    def test_dependency_features_registry_build_scripts_and_workspaces_refuse(self):
        meta = "rust/crates/syson-metamodel/Cargo.toml"
        original = (self.root / meta).read_text()
        for suffix in ['[dependencies]\nfoo={path="../syson-model",features=["raw-factory-candidate"]}\n', '[dependencies]\nfoo="=1.0.0"\n', '[features]\ndefault=["raw-factory-candidate"]\n']:
            self.write(meta, original + suffix)
            self.commit()
            result, receipt = self.run_receipt()
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(receipt["commands"], [])
        self.write(meta, original)
        self.write("rust/crates/syson-metamodel/build.rs", "// must not run\n")
        self.commit()
        result, receipt = self.run_receipt()
        self.assertIn("build-script", receipt["error"])
        (self.root / "rust/crates/syson-metamodel/build.rs").unlink()
        self.write("Cargo.toml", '[workspace]\nmembers=["rust/crates/syson-metamodel"]\n')
        self.commit()
        result, receipt = self.run_receipt()
        self.assertIn("workspace", receipt["error"])

    def test_ancestor_and_cargo_home_configuration_created_during_command_refuse(self):
        profile = self.profile()
        home_config = Path(profile["env"]["CARGO_HOME"]) / "config.toml"
        result, receipt = self.run_receipt(profile=profile, RR16_ADD=str(home_config))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("CARGO_HOME configuration", receipt["error"])
        home_config.unlink()
        ancestor = self.directory / ".cargo/config.toml"
        result, receipt = self.run_receipt(RR16_ADD=str(ancestor))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("ancestor Cargo configuration", receipt["error"])

    def test_generated_check_records_pins_and_stale_failure(self):
        script = self.write("porting/generate_check.py", "import sys\nsys.exit(9 if open('generated.txt').read() != 'expected' else 0)\n")
        self.write("generated.txt", "expected")
        self.commit()
        pin_revision = self.candidate
        expected = hashlib.sha256((self.root / "backend/native-fixture.sql").read_bytes()).hexdigest()
        def plan():
            profile = self.profile()
            profile["historical_pins"] = [{"id": "fixture-pin", "revision": pin_revision, "path": "backend/native-fixture.sql", "sha256": expected}]
            profile["commands"] = [{"id": "generated", "kind": "generated-check", "manifest": None, "argv": [str(Path(sys.executable).resolve()), script.relative_to(self.root).as_posix(), "--check"], "required_inputs": [script.relative_to(self.root).as_posix(), "generated.txt"], "required_pins": ["fixture-pin"], "timeout_s": 5}]
            return profile
        result, receipt = self.run_receipt(start="generated", profile=plan())
        self.assertEqual(result.returncode, 0, receipt)
        self.assertEqual(receipt["before"]["historical"]["fixture-pin"]["sha256"], expected)
        self.write("generated.txt", "committed but stale")
        self.commit()
        result, receipt = self.run_receipt(start="generated", profile=plan())
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(receipt["commands"][0]["exit_code"], 9)
        self.assertFalse(receipt["full_qualification"])

    def test_candidate_pin_tools_and_environment_mismatches_refuse_before_execution(self):
        for change in [lambda p: p.update(candidate="0" * 40),
                       lambda p: p["tools"][0].update(sha256="0" * 64),
                       lambda p: p["env"].update(RUSTFLAGS="--cfg unsafe"),
                       lambda p: p["held_manifests"].update({"rust/crates/syson-metamodel/Cargo.toml": "owned hold"})]:
            profile = self.profile()
            change(profile)
            result, receipt = self.run_receipt(profile=profile)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(receipt["commands"], [])

    def test_runtime_build_alias_cannot_mutate_the_candidate(self):
        alias = self.directory / "build-alias"
        alias.symlink_to(self.root, target_is_directory=True)
        admission = self.directory / "alias-admission.json"
        admission.write_text(json.dumps(self.profile()))
        evidence = self.directory / "alias-evidence"
        result = subprocess.run([sys.executable, str(self.root / "porting/run_integration_checks.py"), "--build-dir", str(alias), "--evidence-dir", str(evidence), "--admission", str(admission)], env=self.env, capture_output=True, timeout=20)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(evidence.exists())
        self.assertFalse((self.root / "target").exists())

    def test_git_replace_does_not_substitute_candidate_or_historical_blob(self):
        revision = self.candidate
        path = "backend/native-fixture.sql"
        expected = hashlib.sha256((self.root / path).read_bytes()).hexdigest()
        original_blob = self.git("rev-parse", revision + ":" + path).strip()
        self.write("replacement.txt", "different source authority")
        self.commit()
        replacement_blob = self.git("rev-parse", self.candidate + ":replacement.txt").strip()
        self.git("replace", original_blob, replacement_blob)
        profile = self.profile()
        profile["historical_pins"] = [{"id": "original", "revision": revision, "path": path, "sha256": expected}]
        profile["commands"][0]["required_pins"] = ["original"]
        result, receipt = self.run_receipt(profile=profile)
        self.assertEqual(result.returncode, 0, receipt)
        self.assertEqual(receipt["before"]["historical"]["original"]["sha256"], expected)

    def test_timeout_terminates_the_owned_same_group_descendant(self):
        if not Path("/proc/self/stat").exists():
            self.skipTest("Linux process identity fixture")
        profile = self.profile()
        profile["commands"][0]["timeout_s"] = 0.5
        identity = self.directory / "owned-child.json"
        started = time.monotonic()
        result, receipt = self.run_receipt(profile=profile, RR16_CHILD_PID=str(identity), RR16_SLEEP="5")
        elapsed = time.monotonic() - started
        self.assertNotEqual(result.returncode, 0)
        child = json.loads(identity.read_text())
        self.assertEqual(receipt["commands"][0]["process_group"], child["group"])
        self.assertTrue(receipt["commands"][0]["timed_out"])
        self.assertLess(elapsed, 2, "descendant's inherited pipe survived until natural3s exit")
        path = Path("/proc") / str(child["pid"]) / "stat"
        if path.exists():
            fields = path.read_text().rsplit(")", 1)[1].split()
            self.assertTrue(fields[19] != child["start"] or fields[0] in {"Z", "X"}, "owned original descendant still running")

    def test_historical_tree_path_cannot_be_a_file_pin(self):
        profile = self.profile()
        listing = self.git("show", self.candidate + ":backend").encode()
        profile["historical_pins"] = [{"id": "tree-path", "revision": self.candidate, "path": "backend", "sha256": hashlib.sha256(listing).hexdigest()}]
        profile["commands"][0]["required_pins"] = ["tree-path"]
        result, receipt = self.run_receipt(profile=profile)
        self.assertNotEqual(result.returncode, 0, "Git tree listing was accepted as historical file content")

    def test_historical_tree_oid_cannot_be_a_commit_revision(self):
        profile = self.profile()
        tree = self.git("rev-parse", self.candidate + "^{tree}").strip()
        expected = hashlib.sha256((self.root / "backend/native-fixture.sql").read_bytes()).hexdigest()
        profile["historical_pins"] = [{"id": "tree-revision", "revision": tree, "path": "backend/native-fixture.sql", "sha256": expected}]
        profile["commands"][0]["required_pins"] = ["tree-revision"]
        result, receipt = self.run_receipt(profile=profile)
        self.assertNotEqual(result.returncode, 0, "uncommitted tree OID was accepted as historical commit")

    def test_candidate_tree_head_cannot_be_a_committed_candidate(self):
        tree = self.git("rev-parse", self.candidate + "^{tree}").strip()
        # Only this newly initialized owned fixture Git directory is changed.
        # A raw detached HEAD can name a tree even though branch refs cannot.
        (self.root / ".git/HEAD").write_text(tree + "\n")
        profile = self.profile()
        profile["candidate"] = tree
        result, receipt = self.run_receipt(profile=profile)
        self.assertNotEqual(result.returncode, 0, "tree-valued HEAD was accepted as a committed candidate")

if __name__ == "__main__":
    unittest.main()

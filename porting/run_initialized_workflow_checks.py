#!/usr/bin/env python3
"""Fresh local Rust workflow checks; never replays a historical frozen gate."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


def utc():
    return datetime.now(timezone.utc).isoformat()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="New directory on verified /workspace")
    parser.add_argument("--phase", choices=["focused", "regression"], required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    output = args.output.resolve()
    if not output.is_relative_to(Path("/workspace")) or output.exists():
        parser.error("output must be a new directory on verified /workspace")
    if os.stat("/workspace").st_dev == os.stat("/").st_dev:
        parser.error("workspace is not a separate build volume")
    # The operator must verify Samsung identity with bob-admin before launch.
    output.mkdir(parents=True)
    env = os.environ.copy()
    env.update(CARGO_TARGET_DIR=str(output.parent / "target"), CARGO_BUILD_JOBS="1",
               SYSON_TEST_DIR=str(output / "snapshots"), PYTHONDONTWRITEBYTECODE="1")
    Path(env["SYSON_TEST_DIR"]).mkdir()
    source_files = sorted(p for p in (root / "rust" / "crates").rglob("*")
                          if p.is_file() and (p.suffix in {".rs", ".js", ".html", ".css"}
                          or p.name in {"Cargo.toml", "Cargo.lock"}))
    source_files.append(Path(__file__).resolve())
    hashes = {str(p.relative_to(root)): sha(p.read_bytes()) for p in source_files}
    commands = []

    def cargo(crate, *flags, build=False):
        command = ["cargo", "build" if build else "test", "--offline", "--locked",
                   "--manifest-path", f"rust/crates/{crate}/Cargo.toml", *flags]
        if not build:
            command.extend(["--", "--test-threads=1"])
        commands.append(command)

    if args.phase == "focused":
        for crate in ["syson-workflow", "syson-web"]:
            cargo(crate, "--features", "raw-factory-candidate")
        cargo("syson-web", "--features", "raw-factory-candidate", "--target",
              "wasm32-unknown-unknown", build=True)
    else:
        for crate in ["syson-model", "syson-initializer-graph"]:
            cargo(crate, "--features", "raw-factory-candidate")
        for crate in ["syson-workflow", "syson-web", "syson-initializer", "syson-metamodel",
                      "syson-domain", "syson-editor", "syson-persistence"]:
            cargo(crate)
        cargo("syson-web", "--target", "wasm32-unknown-unknown", build=True)

    receipt = {"schema": "syson.initialized-workflow-local-checks/1", "phase": args.phase,
               "started_utc": utc(), "sources": hashes, "commands": [],
               "environment": {"CARGO_BUILD_JOBS": "1", "test_threads": 1,
                               "network": "cargo --offline", "locked": True,
                               "output": "caller-supplied verified workspace directory"},
               "upstream_runtime": "NOT_RUN", "parity_verified": False}
    result = 0
    for index, command in enumerate(commands):
        entry = {"argv": command, "started_utc": utc()}
        print(f"RUN {index + 1}/{len(commands)}: {' '.join(command)}", flush=True)
        try:
            completed = subprocess.run(command, cwd=root, env=env, capture_output=True,
                                       timeout=180)
            stdout, stderr = completed.stdout, completed.stderr
            entry["exit_code"] = completed.returncode
        except subprocess.TimeoutExpired as error:
            stdout, stderr = error.stdout or b"", error.stderr or b""
            entry.update(exit_code=124, timeout=True)
        for name, data in [("stdout", stdout), ("stderr", stderr)]:
            filename = f"{index + 1:02d}-{name}.txt"
            (output / filename).write_bytes(data)
            entry[name] = {"file": filename, "sha256": sha(data)}
        entry["finished_utc"] = utc()
        receipt["commands"].append(entry)
        print(stdout.decode(errors="replace"), end="", flush=True)
        print(stderr.decode(errors="replace"), end="", flush=True)
        if entry["exit_code"]:
            result = entry["exit_code"]
            break
    receipt["source_drift"] = [str(p.relative_to(root)) for p in source_files
                               if sha(p.read_bytes()) != hashes[str(p.relative_to(root))]]
    receipt.update(finished_utc=utc(), exit_code=result)
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return result or bool(receipt["source_drift"])


if __name__ == "__main__":
    sys.exit(main())

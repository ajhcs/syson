#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Read-only prerequisite blob verification; executes no upstream runtime."""
import hashlib
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
manifest = json.loads((HERE / "source-manifest.json").read_text())
expected = "00c3eee7eb8cdc16c37daeb6589a4767ea37aa29"
if manifest["base_controller_commit"] != expected:
    raise ValueError("Prerequisite pin changed")
for row in manifest["source_inputs"]:
    path = row["path"]
    if Path(path).is_absolute() or ".." in Path(path).parts:
        raise ValueError("Invalid source path")
    raw = subprocess.check_output(["git", "-C", str(ROOT), "show", expected + ":" + path])
    blob = subprocess.check_output(["git", "-C", str(ROOT), "rev-parse", expected + ":" + path], text=True).strip()
    if row["blob"] != blob or row["sha256"] != hashlib.sha256(raw).hexdigest():
        raise ValueError("Source pin mismatch: " + path)
    if (ROOT / path).read_bytes() != raw:
        raise ValueError("Prerequisite source bytes changed: " + path)
print(json.dumps({"status": "PASS", "pinned_sources": len(manifest["source_inputs"]), "upstream_runtime": "NOT_RUN"}))

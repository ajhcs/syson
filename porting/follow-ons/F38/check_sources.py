#!/usr/bin/env python3
"""Verify fixed source object identities, hashes and admitted line bounds."""
import hashlib
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[3]
manifest = json.loads(Path(__file__).with_name("source-manifest.json").read_text())
revision = manifest["upstream_revision"]
if revision != "6ba2ac64e6b16ee2b05752ad55395eebb87d4910":
    raise ValueError("Unexpected upstream pin")
for source in manifest["sources"]:
    spec = revision + ":" + source["path"]
    data = subprocess.check_output(["git", "-C", str(root), "show", spec])
    blob = subprocess.check_output(["git", "-C", str(root), "rev-parse", spec], text=True).strip()
    if blob != source["blob"] or hashlib.sha256(data).hexdigest() != source["sha256"]:
        raise ValueError("Source hash mismatch: " + source["path"])
    if data != (root / source["path"]).read_bytes():
        raise ValueError("Worktree source differs from pinned bytes: " + source["path"])
    for first, last in source["lines"]:
        if not 1 <= first <= last <= len(data.splitlines()):
            raise ValueError("Invalid line bounds: " + source["path"])
print(json.dumps({"status": "PASS", "pinned_sources": len(manifest["sources"]), "upstream_runtime": "NOT_RUN"}))

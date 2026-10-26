#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Pinned local source evidence only; never executes the storage bridge."""
import hashlib
import json
from pathlib import Path
import subprocess
ROOT = Path(__file__).resolve().parents[3]
manifest = json.loads((Path(__file__).parent / 'source-manifest.json').read_text())
if len(manifest['sources']) != 11:
    raise SystemExit('Wrong source closure')
for row in manifest['sources']:
    path = row['path']
    if Path(path).is_absolute() or '..' in Path(path).parts:
        raise SystemExit('Unsafe source path')
    pinned = subprocess.check_output(['git', '-C', str(ROOT), 'show', row['revision'] + ':' + path])
    actual = (ROOT / path).read_bytes()
    if pinned != actual or hashlib.sha256(actual).hexdigest() != row['sha256']:
        raise SystemExit('Source drift: ' + path)
print(json.dumps({'source_pins':'PASS','count':len(manifest['sources']),
                  'sqlite_runtime':'NOT_EXECUTED_BY_SOURCE_CHECK',
                  'native_artifact_correspondence':'UNPROVEN'}))

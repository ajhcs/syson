# Public Rust SnapshotStore process reopen

A Rust writer imported a small SysML model, renamed a part definition, and published a snapshot. After the writer exited and was reaped, a separate Rust reader reopened the file through the public `SnapshotStore` API. The reader recovered the same six object and semantic ID pairs and both endpoints of the feature typing relationship. An independent Python construction of the full canonical model matched the 2,202 published bytes.

The experiment uses the finite importer and file-backed APIs from [Rust source revision `05a4be35d4842272d0249ab43b9e49295591ef1e`](https://github.com/ajhcs/syson/tree/05a4be35d4842272d0249ab43b9e49295591ef1e/rust). Its scope is a synthetic six-object fixture and a local file reopened after normal writer exit. M06, application and UI restart, undo and history, interruption and power-loss recovery, and Eclipse SysON parity remain outside this proof.

## Observed results

| Case | Result |
| --- | --- |
| Import, edit, and publish | Exit 0. `Link` became `Link Rev A <&> 雪🧭`. The initial save left only the published file. |
| Reopen in a fresh process | Exit 0. The entire document matched the independently constructed model. Feature typing `type` and `typedFeature` referenced the expected semantic IDs ending in `0002` and `0004`. |
| Malformed SysML import | Exit 2. Parse error at byte 47. The store directory stayed empty. |
| Save a different document to the existing key | Exit 2. The published file kept its bytes, device, inode, length, and modification time. The save API retained the alternate document in `rig.emf.staged`. A fresh public reader reopened the published model. |
| Open a snapshot shortened by 13 bytes | Exit 2. JSON syntax error at line 65, column 42. The truncated file stayed byte-identical and a fresh reader reopened the original snapshot in its separate directory. |

[The original run receipt](evidence.json) records all seven processes, including PID, wait order, reaping check, exit status, stdout and stderr hashes, and refusal details. Its SHA-256 is `77d75d943ee188aa50bd5494007bd2d6f1f151dcbdca8e595f53bae5f613e035`. The published snapshot SHA-256 is `d8c5fabf88c25faf780720e100413f79f834f4dddf6129a20db6b6bf25c9581a`. The timing samples describe one run.

## Reproduce

Use a Linux x86-64 host with `/proc`, Python 3, Git, tar, GNU coreutils, and a Rust toolchain and Cargo cache that can build the checked-in lock offline. The recorded run used Rust and Cargo 1.99.1 on ext4. Use an owned directory on a local filesystem with hard-link support. The verifier creates a fresh test store and leaves the snapshots and logs there for inspection.

From the repository root, replace `run` with a new absolute path under that directory and run the following commands.

```bash
set -euo pipefail
evidence="$PWD/porting/oracle/results/public-snapshot-reopen"
base=05a4be35d4842272d0249ab43b9e49295591ef1e
run=/absolute/path/to/fresh-snapshot-replay

git cat-file -e "$base^{commit}"
mkdir -m 700 "$run"
mkdir -m 700 "$run/capture" "$run/tmp"
cp -R "$evidence"/{Cargo.toml,Cargo.lock,src,fixtures,verify.py,git-source.sha256,evidence.json} "$run/"
git archive "$base" rust/crates/{syson-ast,syson-ast-adapter,syson-ast-dispatch,syson-ast-values,syson-domain,syson-editor,syson-json-value,syson-linker,syson-metamodel,syson-model,syson-persistence,syson-references,syson-reflection,syson-text-import,syson-workflow} |
  tar -x -C "$run"

cd "$run"
sha256sum --check --quiet git-source.sha256
env CARGO_TARGET_DIR="$run/target" TMPDIR="$run/tmp" \
  cargo build --offline --locked --release -j 1 >capture/build.log 2>&1
(umask 077; python3 -I -B verify.py) >capture/verification.json
sha256sum --check --quiet git-source.sha256

python3 -I -B - <<'PY'
import json
from pathlib import Path

original = json.loads(Path("evidence.json").read_text())
replay = json.loads(Path("capture/verification.json").read_text())
for key in (
    "status", "fixed_ids", "typed_reference_endpoints", "rename",
    "independent_canonical_bytes", "published_sha256",
    "duplicate_diagnostic_stage", "truncated_sha256", "refusals",
    "feature_acceptance",
):
    assert original[key] == replay[key], key
assert list(original["processes"]) == list(replay["processes"])
for label, row in replay["processes"].items():
    for key in ("mode", "exit", "proc_entry_absent_after_wait"):
        assert original["processes"][label][key] == row[key], (label, key)
print(replay["status"])
PY
```

The controller starts each child with a 64 KiB file-size limit, waits up to 10 seconds, and kills and reaps the child on timeout. The recorded run also used a user scope with one CPU of quota, a 256 MiB memory high threshold, a 512 MiB memory maximum, zero swap, and a 48-task limit. Replay PIDs, file metadata, timing, and logs that contain those values vary by run.

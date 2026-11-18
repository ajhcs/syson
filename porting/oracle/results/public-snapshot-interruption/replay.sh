#!/usr/bin/env bash
set -euo pipefail
umask 077

if [[ $# != 1 || $1 != /* || $1 != "$(realpath -m -- "$1")" ]]; then
  printf '%s\n' 'Pass a fresh absolute path with no symlink or dot components.' >&2
  exit 2
fi
run=$1
pack=$(cd -- "$(dirname -- "$0")" && pwd -P)
repo=$(git -C "$pack" rev-parse --show-toplevel)
rust_revision=05a4be35d4842272d0249ab43b9e49295591ef1e
prior_revision=a3eef6a4f3331f77297b636ebb55a1a608154dcc
git -C "$repo" cat-file -e "$rust_revision^{commit}"
git -C "$repo" cat-file -e "$prior_revision^{commit}"
mkdir -m 700 "$run"
mkdir -m 700 "$run/capture" "$run/prior" "$run/tmp"
cp -R "$pack"/{src,verify.py,evidence.json} "$run/"
git -C "$repo" archive "$prior_revision" porting/oracle/results/public-snapshot-reopen |
  tar -x -C "$run/prior" --strip-components=4
cp -R "$run/prior"/{Cargo.lock,Cargo.toml,fixtures} "$run/"
git -C "$repo" archive "$rust_revision" rust/crates/{syson-ast,syson-ast-adapter,syson-ast-dispatch,syson-ast-values,syson-domain,syson-editor,syson-json-value,syson-linker,syson-metamodel,syson-model,syson-persistence,syson-references,syson-reflection,syson-text-import,syson-workflow} |
  tar -x -C "$run"

cd "$run"
sha256sum --check --quiet prior/git-source.sha256
find prior -type f -print0 | sort -z | xargs -0 sha256sum >capture/prior.sha256
env TMPDIR="$run/tmp" CARGO_TARGET_DIR="$run/target" \
  cargo build --offline --locked --release -j 1 >capture/build.log 2>&1
sha256sum Cargo.lock Cargo.toml src/main.rs src/barrier.rs fixtures/rig.sysml verify.py evidence.json \
  target/release/syson-public-snapshot-process-probe >capture/subject.sha256
unit=syson-snapshot-interruption-$BASHPID
systemd-run --user --scope --unit "$unit" \
  -p CPUQuota=100% -p MemoryHigh=256M -p MemoryMax=512M -p MemorySwapMax=0 -p TasksMax=48 \
  env TMPDIR="$run/tmp" setpriv --no-new-privs python3 -I -B verify.py \
  >capture/verification.json 2>capture/scope.stderr
systemctl --user show "$unit.scope" -p LoadState -p ActiveState >capture/scope-after.txt
test "$(cat capture/scope-after.txt)" = $'LoadState=not-found\nActiveState=inactive'
sha256sum --check --quiet prior/git-source.sha256
sha256sum --check --quiet capture/prior.sha256
sha256sum --check --quiet capture/subject.sha256

python3 -I -B - <<'PY'
import json
from pathlib import Path

expected = json.loads(Path("evidence.json").read_text())
observed = json.loads(Path("capture/verification.json").read_text())

def files(state):
    return {
        name: None if info is None else (info["sha256"], info["size"], info["hard_links"])
        for name, info in state.items()
    }

for key in ("status", "unique_and_reaped_children", "model_sha256"):
    assert expected[key] == observed[key], key
assert expected["normal_writer_steps"] == observed["normal"]["writer"]["report"]["completed"]
assert list(expected["interruption_cases"]) == list(observed["interruption_cases"])
for point, original in expected["interruption_cases"].items():
    actual = observed["interruption_cases"][point]
    writer = actual["writer"]
    assert (original["waitid"]["code"], original["waitid"]["signal"]) == (writer["waitid"]["code"], writer["waitid"]["signal"])
    assert original["writer_completed"] == writer["barrier"]["completed"]
    assert files(original["disk_after_wait"]) == files(writer["after_wait"])
    assert files(original["disk_after_retry"]) == files(actual["after_retry"])
    assert original["candidate_load_exit"] == actual["fresh"]["candidate"]["exit"]
    assert original["candidate_load_after_retry_exit"] == actual["fresh_after_retry"]["candidate"]["exit"]
    assert original["first_blocked_operation"] == actual["first_blocked_operation"]
    assert {k:v for k,v in original["retry"].items() if k != "pid"} == {
        k:v for k,v in actual["retry"]["report"].items() if k != "pid"
    }
print(observed["status"])
PY

#!/bin/bash
# SPDX-License-Identifier: EPL-2.0
set -euo pipefail
harness=$(cd -- "$(dirname -- "$0")" && pwd -P)

if [[ $# -gt 0 && $1 == --inside ]]; then
    shift
    scope=$1
    receipt=$2
    output=$3
    node=$4
    case $(< /proc/self/cgroup) in
        0::*/"$scope.scope") ;;
        *) exit 1 ;;
    esac
    invocation=$(/usr/bin/systemctl --user show "$scope.scope" -p InvocationID --value)
    [[ $invocation =~ ^[[:xdigit:]]{32}$ ]]
    (set -o noclobber; printf '%s\n' "$invocation" > "$receipt")
    [[ ! -e $receipt.cancel ]]
    exec /usr/bin/python3 -I -B "$harness/restart.py" \
        "$harness/../oracle.py" "$output" --node "$node"
fi

if [[ $# != 2 ]]; then
    printf 'Usage: bash %s NEW_WORKSPACE_OUTPUT NODE_EXECUTABLE\n' "$0" >&2
    exit 2
fi
output=$(realpath -m -- "$1")
node=$(realpath -e -- "$2")
[[ "$output" == /workspace/build/* && ! -e "$output" && -x "$node" ]]
scope=syson-reference-restart-$(< /proc/sys/kernel/random/uuid)
receipt="$output.$scope.invocation"
[[ $(/usr/bin/systemctl --user show "$scope.scope" -p LoadState --value) == not-found ]]

cleanup() {
    if [[ ${complete:-0} == 1 ]]; then return; fi
    local saved current
    touch -- "$receipt.cancel"
    if [[ -s $receipt ]]; then
        saved=$(< "$receipt")
        current=$(/usr/bin/systemctl --user show "$scope.scope" -p InvocationID --value)
        if [[ -n $current && $saved == "$current" ]]; then
            /usr/bin/systemctl --user stop "$scope.scope"
        fi
    fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

printf 'Reference scope %s.scope\n' "$scope"
/usr/bin/systemd-run --user --scope --unit="$scope" \
    -p CPUQuota=200% -p MemoryHigh=3G -p MemoryMax=4G -p MemorySwapMax=0 -p TasksMax=256 \
    /usr/bin/timeout --kill-after=15s 210s /bin/bash "$harness/run.sh" --inside \
    "$scope" "$receipt" "$output" "$node" &
wait "$!"
[[ -s "$output/result.json" && ! -e "$output/current.json" ]]
[[ $(/usr/bin/systemctl --user show "$scope.scope" -p LoadState --value) == not-found ]]
complete=1

#!/usr/bin/env python3
"""Run one newly implemented std-only crate serially; preserve original receipts."""
import argparse
import hashlib
import json
import os
import pathlib
import subprocess
from datetime import datetime, timezone

def stamp():
    return datetime.now(timezone.utc).isoformat()

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('worktree',type=pathlib.Path)
    parser.add_argument('crate')
    parser.add_argument('run')
    args=parser.parse_args()
    root=args.worktree.resolve()
    assert str(root).startswith('/workspace/worktrees/builderbob/syson/')
    crate=root/'rust/crates'/args.crate
    assert crate.is_dir() and '/' not in args.crate and '/' not in args.run
    output=root/'target/follow-on-checks'/args.run
    output.mkdir(parents=True,exist_ok=False)
    env=os.environ.copy()
    env.update(CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='1',
               CARGO_TARGET_DIR=str(root/'target/follow-on-build'),
               CARGO_HOME=str(root/'target/follow-on-cargo-home'),
               TMPDIR=str(root/'target/follow-on-tmp'))
    for k in ['CARGO_TARGET_DIR','CARGO_HOME','TMPDIR']:
        pathlib.Path(env[k]).mkdir(parents=True,exist_ok=True)
    manifest=str(crate/'Cargo.toml')
    # These new standalone crates have no registry dependencies. Refuse a hidden
    # build script or an added dependency before executing the compiler.
    import tomllib
    data=tomllib.loads((crate/'Cargo.toml').read_text())
    assert not (crate/'build.rs').exists()
    for key in ['dependencies','dev-dependencies','build-dependencies','target']:
        assert not data.get(key),f'Unexpected dependency section: {key}'
    assert not data.get('package',{}).get('build')
    def freeze():
        return {str(p.relative_to(root)):sha(p) for p in sorted(crate.rglob('*')) if p.is_file() and 'target' not in p.relative_to(crate).parts}
    before=freeze()
    result=dict(schema_version=1,source_base=subprocess.check_output(['git','-C',str(root),'rev-parse','HEAD'],text=True).strip(),
                worktree=str(root),crate=args.crate,started_utc=stamp(),commands=[],
                source_before=before,runner_sha256=sha(pathlib.Path(__file__)),
                execution='COORDINATOR_SERIAL_ONE_JOB',upstream_runtime='NOT_RUN',upstream_parity_verified=False)
    commands=[['cargo','fmt','--manifest-path',manifest,'--','--check'],
              ['cargo','test','--offline','--locked','--manifest-path',manifest,'--','--test-threads=1'],
              ['cargo','clippy','--offline','--locked','--manifest-path',manifest,'--all-targets','--','-D','warnings']]
    try:
        for i,command in enumerate(commands):
            entry=dict(argv=command,started_utc=stamp())
            try:
                child=subprocess.run(command,cwd=root,env=env,capture_output=True,timeout=120)
                code=child.returncode
                stdout,stderr=child.stdout,child.stderr
            except subprocess.TimeoutExpired as error:
                code=124
                stdout,stderr=error.stdout or b'',error.stderr or b''
            out=output/f'{i:02d}-stdout.txt';err=output/f'{i:02d}-stderr.txt'
            out.write_bytes(stdout);err.write_bytes(stderr)
            entry.update(completed_utc=stamp(),exit_code=code,stdout_sha256=sha(out),stderr_sha256=sha(err),stdout=str(out),stderr=str(err))
            result['commands'].append(entry)
            if code:
                result['status']='FAIL'
                break
        else:
            result['status']='PASS'
    finally:
        result.update(completed_utc=stamp(),source_after=freeze())
        result['source_unchanged']=result['source_before']==result['source_after']
        if not result['source_unchanged']:
            result['status']='SOURCE_DRIFT'
        (output/'receipt.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(dict(status=result['status'],receipt=str(output/'receipt.json'),commands=[c['exit_code'] for c in result['commands']],source_unchanged=result['source_unchanged'])))
    raise SystemExit(0 if result['status']=='PASS' else 1)

if __name__=='__main__':
    main()

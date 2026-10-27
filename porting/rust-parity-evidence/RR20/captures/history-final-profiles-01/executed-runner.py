from pathlib import Path
import sys,json,subprocess,os,fcntl,hashlib,time,signal,tomllib
b=Path('/workspace/build/builderbob/syson-rereview-batch9-20261027');uid=sys.argv[1];run=sys.argv[2];w=Path('/workspace/worktrees/builderbob/syson/rr-batch9-base' if uid=='BASE' else '/workspace/worktrees/builderbob/syson/rr20-history-budgets');o=b/run;o.mkdir();c=o/'consumer';(c/'src').mkdir(parents=True);src=Path('/workspace/worktrees/builderbob/syson/rr20-history-budgets/porting/rust-parity-evidence/RR20/history_probe.rs');data=src.read_bytes();assert hashlib.sha256(data).hexdigest()=='a918e88e41a2567cf7d9ff86c6258711c4b031046aaf7ee68bfcdcc979092dc7';(c/'src/main.rs').write_bytes(data);(c/'Cargo.toml').write_text(f'[package]\nname="rr20-history-probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\nsyson-model={{path="{w}/rust/crates/syson-model"}}\nserde_json="=1.0.151"\n')
r={'unit':uid,'status':'FAILED_ATTEMPT','probe_sha256':hashlib.sha256(data).hexdigest(),'commands':[],'measurements':[]};(o/'executed-runner.py').write_bytes(Path(__file__).read_bytes());lock=open('/workspace/build/builderbob/syson-audit-20261026/build/compiler.lock','a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);env=dict(os.environ,CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',CARGO_TARGET_DIR=str(b/(uid.lower()+'-probe-target')),TMPDIR=str(o));source={str(p.relative_to(w)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (w/'rust').rglob('*') if p.is_file()};r['source_before']=source
try:
 commands=[['cargo','generate-lockfile','--offline','--manifest-path',str(c/'Cargo.toml')],['cargo','build','--release','--offline','--locked','--jobs','1','--manifest-path',str(c/'Cargo.toml')]]
 for n,argv in enumerate(commands):
  p=subprocess.run(argv,cwd=c,env=env,capture_output=True,timeout=180);(o/f'{n:02}-stdout.txt').write_bytes(p.stdout);(o/f'{n:02}-stderr.txt').write_bytes(p.stderr);r['commands'].append(dict(argv=argv,exit_code=p.returncode));assert p.returncode==0,p.stderr
 binary=b/(uid.lower()+'-probe-target/release/rr20-history-probe');r['binary_sha256']=hashlib.sha256(binary.read_bytes()).hexdigest()
 for nodes in [1000,4096]:
  for commits in [64,256]:
   label=f'{nodes}-{commits}';argv=['/usr/bin/time','-v',str(binary),str(nodes),str(commits)];t=time.monotonic();p=subprocess.Popen(argv,cwd=c,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
   try:out,err=p.communicate(timeout=120)
   except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);out,err=p.communicate();raise RuntimeError('Owned probe group timedout')
   (o/(label+'-stdout.txt')).write_bytes(out);(o/(label+'-stderr.txt')).write_bytes(err);r['commands'].append(dict(argv=argv,exit_code=p.returncode,wall_seconds=time.monotonic()-t));assert p.returncode==0,err;r['measurements'].append(json.loads(out))
 r['source_after']={str(p.relative_to(w)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (w/'rust').rglob('*') if p.is_file()};assert source==r['source_after'];r['status']='PASS_FOUR_HOST_PROFILES;NOT_UI_OR_PREEMPTIVE_CANCELLATION_QUALIFICATION'
finally:(o/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({k:v for k,v in r.items() if k not in ['source_before','source_after']},indent=2))

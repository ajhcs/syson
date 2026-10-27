from datetime import datetime,timezone
from functools import partial
import hashlib,http.server,json,os,signal,subprocess,sys,threading,ctypes,time
from pathlib import Path
b=Path('/workspace/build/builderbob/syson-rereview-batch12-20261027')
root=Path(sys.argv[1]); builddir=b/sys.argv[2]; run=sys.argv[3]; scripts=[Path(x) for x in sys.argv[4:]]
out=b/run;out.mkdir(); web=out/'web';web.mkdir();assets=root/'rust/crates/syson-web/web'
release=Path('/workspace/cache/builderbob/global-tools/maintenance/staged/playwright/1.64.1')
node=Path('/workspace/build/builderbob/runtimes/node-v24.21.0-linux-x64/bin/node')
browser=release/'browsers/chromium_headless_shell-1248/chrome-headless-shell-linux64/chrome-headless-shell'
module=release/'node_modules/playwright'
assert all(x.exists() for x in [node,browser,module,*scripts])
wasm=builddir/'syson_web.wasm'
for n in ['index.html','loader.js','style.css']:(web/n).write_bytes((assets/n).read_bytes())
(web/'syson_web.wasm').write_bytes(wasm.read_bytes())
def stamp():
    return datetime.now(timezone.utc).isoformat()

def sha(data):
    return hashlib.sha256(data).hexdigest()

def require(condition, message):
    if not condition:
        raise RuntimeError(message)

def freeze():
    entries = [*scripts, wasm, *[assets / n for n in ['index.html', 'loader.js', 'style.css']],
               b/'RR18a/fixtures/escaped-import.json', root/'porting/rust-parity-evidence/RR18a/fixtures/escaped-import.json']
    for script in scripts:
        companion = script.parent.parent / 'RR18a/fixtures/escaped-import.json'
        if companion.is_file():
            entries.append(companion)
        fixtures = script.parent / 'fixtures'
        if fixtures.is_dir():
            entries.extend(p for p in fixtures.rglob('*') if p.is_file())
    return {(str(p.relative_to(root)) if p.is_relative_to(root) else str(p)): sha(p.read_bytes()) for p in entries}



source_before=freeze()
served={name:sha((web/name).read_bytes()) for name in ['index.html','loader.js','style.css','syson_web.wasm']}
build=json.loads((builddir/'receipt.json').read_text())
require(build['status'].startswith('PASS') and build['source_unchanged'],'Build not passed')
require(served['syson_web.wasm']==build['wasm_sha256'],'WASM differs from build')
require(all(sha((root/p).read_bytes())==d for p,d in build['source_after'].items()),'Build source changed')
(out/'executed-runner.py').write_bytes(Path(__file__).read_bytes())
# Adopt detached browser grandchildren if their Node parent exits. This affects
# only this runner process and permits exact owned-PID cleanup on failure.
if ctypes.CDLL(None, use_errno=True).prctl(36, 1, 0, 0, 0) != 0:
    raise OSError(ctypes.get_errno(), 'Cannot establish owned descendant cleanup')

def descendants():
    rows = {}
    for path in Path('/proc').glob('[0-9]*/stat'):
        try:
            fields = path.read_text().rsplit(')', 1)[1].split()
            rows[int(path.parent.name)] = dict(ppid=int(fields[1]), group=int(fields[2]), start=fields[19], state=fields[0])
        except (OSError, ValueError, IndexError):
            pass
    owned = {os.getpid()}
    while True:
        new = {pid for pid, row in rows.items() if row['ppid'] in owned}
        if new <= owned:
            break
        owned |= new
    return {pid: rows[pid] for pid in owned if pid != os.getpid()}

def cleanup():
    captured = descendants()
    for sig in [signal.SIGTERM, signal.SIGKILL]:
        for pid, row in descendants().items():
            captured[pid] = row
            try:
                # Recheck start identity before signalling an exact owned PID.
                current = descendants().get(pid)
                if current and current['start'] == row['start']:
                    os.kill(pid, sig)
            except ProcessLookupError:
                pass
        end = time.monotonic() + 2
        while time.monotonic() < end:
            try:
                while os.waitpid(-1, os.WNOHANG)[0]:
                    pass
            except ChildProcessError:
                pass
            if not descendants():
                return dict(captured=captured, remaining={})
            time.sleep(.05)
    return dict(captured=captured, remaining=descendants())

class Handler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, fmt, *args):
        with (out / 'http.log').open('a') as stream:
            stream.write(fmt % args + '\n')

server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), partial(Handler, directory=str(web)))
thread = threading.Thread(target=server.serve_forever, daemon=True)
thread.start()
url = f'http://127.0.0.1:{server.server_port}/'
env = os.environ.copy()
env.pop('NODE_OPTIONS', None)
env.update(PLAYWRIGHT_BROWSERS_PATH=str(release / 'browsers'),
           SYSON_PLAYWRIGHT_MODULE=str(module), SYSON_PREVIEW_URL=url,
           LD_LIBRARY_PATH='/workspace/cache/builderbob/global-tools/browser-deps/sysroot/usr/lib/x86_64-linux-gnu')
env['PATH'] = str(node.parent) + os.pathsep + env.get('PATH', '')
env['TMPDIR'] = str(out / 'tmp')
Path(env['TMPDIR']).mkdir()
for key, directory in [('XDG_CACHE_HOME', 'cache'), ('XDG_CONFIG_HOME', 'config'), ('XDG_STATE_HOME', 'state')]:
    env[key] = str(out / directory)
    Path(env[key]).mkdir()
r = dict(run=run, status='FAIL_SETUP', started_utc=stamp(), preview_url=url, browser_path='Browser plugin not available; regular installed Playwright',
         source_before=source_before, served_sha256=served, build_capture=sys.argv[2], commands=[], runner_sha256=sha(Path(__file__).read_bytes()),
         node_sha256=sha(node.read_bytes()), browser_sha256=sha(browser.read_bytes()),
         upstream_runtime='NOT_RUN', server_scope='owned loopback static WASM preview only')
try:
    for script in scripts:
        target = out / script.stem
        target.mkdir()
        env['SYSON_BROWSER_ARTIFACT_DIR'] = str(target)
        argv = [str(node), str(script)]
        entry = dict(argv=argv, started_utc=stamp(), artifacts=str(target))
        child = subprocess.Popen(argv, cwd=root, env=env, stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, start_new_session=True)
        entry['owned_process_group'] = child.pid
        try:
            stdout, stderr = child.communicate(timeout=180)
            code = child.returncode
        except subprocess.TimeoutExpired:
            entry['timeout_cleanup'] = cleanup()
            try:
                stdout, stderr = child.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                entry['second_timeout_cleanup'] = cleanup()
                try:
                    stdout, stderr = child.communicate(timeout=3)
                except subprocess.TimeoutExpired as error:
                    stdout, stderr = error.stdout or b'', error.stderr or b''
                    entry['pipes_remained_open_after_cleanup'] = True
                    child.stdout.close()
                    child.stderr.close()
            code = 124
            entry['timeout_terminated_owned_group'] = True
        (target / 'stdout.txt').write_bytes(stdout)
        (target / 'stderr.txt').write_bytes(stderr)
        entry.update(completed_utc=stamp(), exit_code=code,
                     stdout_sha256=sha(stdout), stderr_sha256=sha(stderr))
        entry['descendant_cleanup'] = cleanup()
        if entry['descendant_cleanup']['remaining']:
            code = entry['exit_code'] = 125
        r['commands'].append(entry)
        if code:
            r['status'] = 'FAIL'
            break
    else:
        r['status'] = 'PASS'
finally:
    r['final_descendant_cleanup'] = cleanup()
    if r['final_descendant_cleanup']['remaining']:
        r['status'] = 'FAIL_OWNED_PROCESS_CLEANUP'
    server.shutdown()
    server.server_close()
    r.update(completed_utc=stamp(), source_after=freeze())
    r['source_unchanged'] = r['source_before'] == r['source_after']
    if not r['source_unchanged']:
        r['status'] = 'SOURCE_DRIFT'
    r['artifacts'] = {str(p.relative_to(out)): sha(p.read_bytes()) for p in out.rglob('*')
                      if p.is_file() and not set(p.relative_to(out).parts).intersection({'tmp', 'cache', 'config', 'state'})}
    (out / 'receipt.json').write_text(json.dumps(r, indent=2) + '\n')
print(json.dumps({'status': r['status'], 'url': url, 'receipt': str(out / 'receipt.json'),
                  'commands': [c['exit_code'] for c in r['commands']], 'source_unchanged': r['source_unchanged']}))
sys.exit(0 if r['status'] == 'PASS' else 1)

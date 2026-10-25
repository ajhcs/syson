#!/usr/bin/env python3
"""Serve only the captured local WASM app and run installed Playwright workflows."""
from datetime import datetime, timezone
from functools import partial
import hashlib
import http.server
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import threading
import ctypes
import time

run = sys.argv[1]
if not run.startswith('f36-browser-') or '/' in run:
    raise SystemExit('Unexpected run ID')
root = Path('/workspace/worktrees/builderbob/syson/f36-inspector-accessibility')
out = Path('/workspace/build/builderbob/syson-batch7-20261025/browser') / run
out.mkdir(parents=True, exist_ok=False)
web = out / 'web'
web.mkdir()
release = Path('/workspace/cache/builderbob/global-tools/maintenance/staged/playwright/1.64.1')
node = Path('/workspace/build/builderbob/runtimes/node-v24.21.0-linux-x64/bin/node')
browser = release / 'browsers/chromium_headless_shell-1248/chrome-headless-shell-linux64/chrome-headless-shell'
module = release / 'node_modules/playwright'
if not all(p.exists() for p in [node, browser, module]):
    raise SystemExit('Installed browser/tool input missing; no installation allowed')
assets = root / 'rust/crates/syson-web/web'
for name in ['index.html', 'loader.js', 'style.css']:
    (web / name).write_bytes((assets / name).read_bytes())
wasm = root / 'target/batch7-wasm/syson_web.wasm'
(web / 'syson_web.wasm').write_bytes(wasm.read_bytes())
scripts = [root / 'rust/browser-tests' / name for name in ['inspector_accessibility.cjs', 'layout.cjs', 'authoring.cjs', 'scalars.cjs']]
if not all(p.is_file() for p in scripts):
    raise SystemExit('Browser scripts missing; no server started')

def stamp():
    return datetime.now(timezone.utc).isoformat()

def sha(data):
    return hashlib.sha256(data).hexdigest()

def require(condition, message):
    if not condition:
        raise RuntimeError(message)

def freeze():
    return {str(p.relative_to(root)): sha(p.read_bytes())
            for p in [*scripts, wasm, *[assets / n for n in ['index.html', 'loader.js', 'style.css']]]}

source_before = freeze()
served = {name: sha((web / name).read_bytes()) for name in ['index.html', 'loader.js', 'style.css', 'syson_web.wasm']}
require(all(served[name] == source_before[str((assets / name).relative_to(root))] for name in ['index.html', 'loader.js', 'style.css']), 'Served assets differ from source')
require(served['syson_web.wasm'] == source_before[str(wasm.relative_to(root))], 'Served WASM differs from source artifact')
build = json.loads((root / 'target/batch7-checks' / os.environ['SYSON_BUILD_CAPTURE'] / 'receipt.json').read_text())
require(build['unit'] == 'F36' and build['status'] == 'PASS' and build['source_unchanged'], 'Build did not pass with stable sources')
require(build['commands'][-1]['exit_code'] == 0 and 'wasm32-unknown-unknown' in build['commands'][-1]['argv'], 'WASM build missing')
require(build['wasm_sha256'] == served['syson_web.wasm'], 'Served WASM differs from successful build')
require(all(sha((root / p).read_bytes()) == digest for p, digest in build['source_after'].items()), 'Sources changed after successful build')
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
         source_before=source_before, served_sha256=served, build_capture=build['run'], commands=[], runner_sha256=sha(Path(__file__).read_bytes()),
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

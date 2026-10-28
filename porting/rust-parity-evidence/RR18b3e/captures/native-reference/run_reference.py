# SPDX-License-Identifier: EPL-2.0
"""Run only after coordinator EXECUTE grant; no service install, external DB or reused state."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, ctypes, hashlib, http.client, json, os, pwd, re, signal, socket, subprocess, sys, time

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--execute', action='store_true')
parser.add_argument('--epoch', required=True)
args = parser.parse_args()
if not args.execute:
    parser.error('Preparation is complete; explicit coordinator EXECUTE grant and --execute are required.')
assert ROOT.is_relative_to('/workspace') and re.fullmatch(r'[a-z0-9_-]{1,8}', args.epoch)
assert os.getuid() != 0 and pwd.getpwuid(os.getuid()).pw_name == 'builderbob'
os.umask(0o077)
admission = json.loads((ROOT / 'admission.json').read_text())
sha = lambda data: hashlib.sha256(data).hexdigest()
stamp = lambda: datetime.now(timezone.utc).isoformat()
for filename, digest in admission['tool_sha256'].items():
    assert sha(Path(filename).read_bytes()) == digest, filename
for filename, digest in admission['prepared_source_sha256'].items():
    assert sha((ROOT / filename).read_bytes()) == digest, filename
runs = ROOT / 'runs'
runs.mkdir(exist_ok=True, mode=0o700)
run = runs / args.epoch
run.mkdir(exist_ok=False, mode=0o700)
for name in ['data', 's', 'tmp', 'prefs', 'browser', 'cache', 'config', 'state']:
    (run / name).mkdir(mode=0o700)
receipt = {'status': 'STARTING', 'started_at': stamp(), 'admission_sha256': sha((ROOT/'admission.json').read_bytes()),
           'runner_sha256': sha(Path(__file__).read_bytes()), 'review': admission, 'commands': [], 'observed_source': {},
           'service_registration': False, 'production_database': False, 'working_root': str(run)}
children = []
reserved = []
code = 1

def checkpoint():
    (run / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')

def descendants():
    rows = {}
    for path in Path('/proc').glob('[0-9]*/stat'):
        try:
            f = path.read_text().rsplit(')', 1)[1].split()
            rows[int(path.parent.name)] = {'ppid': int(f[1]), 'start': f[19], 'state': f[0]}
        except (OSError, ValueError, IndexError):
            pass
    owned = {os.getpid()}
    while True:
        new = {pid for pid, row in rows.items() if row['ppid'] in owned}
        if new <= owned:
            break
        owned |= new
    return {pid: rows[pid] for pid in owned if pid != os.getpid()}

def reap():
    while True:
        try:
            pid, status = os.waitpid(-1, os.WNOHANG)
        except ChildProcessError:
            return
        if not pid:
            return
        for child in children:
            if child['process'].pid == pid:
                child['process'].returncode = os.waitstatus_to_exitcode(status)
        receipt.setdefault('reaped', []).append({'pid': pid, 'exit_code': os.waitstatus_to_exitcode(status)})

def cleanup():
    captured, sent = {}, []
    for sig, seconds in [(signal.SIGTERM, 8), (signal.SIGKILL, 3)]:
        initial = descendants()
        captured.update(initial)
        for pid, row in initial.items():
            current = descendants().get(pid)
            if current and current['start'] == row['start']:
                chosen = signal.SIGINT if sig == signal.SIGTERM and any(c['name']=='postgres' and c['process'].pid==pid for c in children) else sig
                try:
                    os.kill(pid, chosen)
                    sent.append({'pid': pid, 'start': row['start'], 'signal': int(chosen)})
                except ProcessLookupError:
                    pass
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            reap()
            if not descendants():
                return {'captured': captured, 'signals': sent, 'remaining': {}}
            time.sleep(.05)
    reap()
    return {'captured': captured, 'signals': sent, 'remaining': descendants()}

def start(name, argv, environment):
    out = (run / (name + '.stdout.txt')).open('wb')
    err = (run / (name + '.stderr.txt')).open('wb')
    process = subprocess.Popen(argv, cwd=run, env=environment, stdout=out, stderr=err, start_new_session=True)
    record = {'name': name, 'argv': argv, 'pid': process.pid, 'started_at': stamp(), 'identity': descendants().get(process.pid)}
    children.append({'name': name, 'process': process, 'stdout': out, 'stderr': err, 'record': record})
    receipt['commands'].append(record)
    checkpoint()
    return process

def completed(name, argv, environment, timeout):
    process = start(name, argv, environment)
    result = process.wait(timeout=timeout)
    receipt['commands'][-1].update(exit_code=result, finished_at=stamp())
    checkpoint()
    if result:
        raise RuntimeError(f'{name} exited {result}')

try:
    if ctypes.CDLL(None, use_errno=True).prctl(36, 1, 0, 0, 0) != 0:
        raise OSError(ctypes.get_errno(), 'Cannot own detached browser descendants')
    for _ in range(2):
        s = socket.socket()
        s.bind(('127.0.0.1', 0))
        reserved.append(s)
    pg_port, web_port = [s.getsockname()[1] for s in reserved]
    assert len((str(run/'s')+'/.s.PGSQL.'+str(pg_port)).encode()) < 108
    url = f'http://127.0.0.1:{web_port}/'
    receipt['binds'] = [f'127.0.0.1:{pg_port}', f'127.0.0.1:{web_port}']
    receipt['port_policy'] = 'Kernel-selected ports reserved until immediately before each exact owned loopback server starts; no existing listener reused.'
    tools = admission['tools']
    pg = Path(tools['postgres_directory'])
    env = {'PATH': str(Path(tools['node']).parent)+':/usr/bin:/bin', 'HOME': os.environ['HOME'], 'LANG': 'C', 'LC_ALL': 'C',
           'TMPDIR': str(run/'tmp'), 'XDG_CACHE_HOME': str(run/'cache'), 'XDG_CONFIG_HOME': str(run/'config'), 'XDG_STATE_HOME': str(run/'state'),
           'LD_LIBRARY_PATH': tools['postgres_library_directory']}
    completed('postgres-version', [str(pg/'postgres'), '--version'], env, 20)
    completed('initdb', [str(pg/'initdb'), '-D', str(run/'data'), '-L', tools['postgres_share_directory'], '--locale=C', '--encoding=UTF8', '--auth=trust', '--no-instructions'], env, 60)
    reserved[0].close()
    database = start('postgres', [str(pg/'postgres'), '-D', str(run/'data'), '-p', str(pg_port), '-h', '127.0.0.1', '-k', str(run/'s'),
                                '-c', 'unix_socket_permissions=0700', '-c', 'shared_buffers=64MB', '-c', 'max_connections=30',
                                '-c', 'dynamic_library_path='+tools['postgres_extension_directory']], env)
    deadline = time.monotonic()+30
    while time.monotonic()<deadline:
        assert database.poll() is None, 'Owned Postgres exited'
        try:
            with socket.create_connection(('127.0.0.1',pg_port),timeout=.3):
                break
        except OSError:
            time.sleep(.2)
    else:
        raise TimeoutError('Owned Postgres readiness timeout')
    config = run/'audit.properties'
    config.write_text(f'server.address=127.0.0.1\nserver.port={web_port}\nspring.datasource.url=jdbc:postgresql://127.0.0.1:{pg_port}/postgres\nspring.datasource.username=builderbob\nspring.datasource.password=\nliquibase.analytics.enabled=false\nspring.liquibase.analytics-enabled=false\nmanagement.server.port=-1\nsirius.components.cors.allowedOriginPatterns=http://127.0.0.1:{web_port}\n')
    javaenv = {k:v for k,v in env.items() if k!='LD_LIBRARY_PATH'}
    javaenv['JAVA_TOOL_OPTIONS'] = '-Dliquibase.analytics.enabled=false'
    reserved[1].close()
    app = start('syson', [tools['java'],'-Xms256m','-Xmx1536m','-XX:ActiveProcessorCount=2','-Djava.io.tmpdir='+str(run/'tmp'),
                         '-Djava.util.prefs.userRoot='+str(run/'prefs'),'-jar',tools['jar'],
                         '--spring.config.location=classpath:/application.properties','--spring.config.additional-location=file:'+str(config)], javaenv)
    deadline = time.monotonic()+180
    while time.monotonic()<deadline:
        assert database.poll() is None and app.poll() is None, 'Owned native process exited'
        connection = http.client.HTTPConnection('127.0.0.1',web_port,timeout=2)
        try:
            connection.request('GET','/')
            response = connection.getresponse()
            if response.status==200:
                response.read()
                break
        except OSError:
            pass
        finally:
            connection.close()
        time.sleep(.3)
    else:
        raise TimeoutError('Owned SysON readiness timeout')
    receipt.update(status='READY',preview_url=url,ready_at=stamp())
    checkpoint()
    print(json.dumps({'status':'READY','url':url,'receipt':str(run/'receipt.json')}),flush=True)
    browserenv = dict(javaenv, SYSON_NATIVE_URL=url, SYSON_NATIVE_OUTPUT=str(run/'browser'),
                      SYSON_PLAYWRIGHT_MODULE=tools['playwright_module'], SYSON_CHROMIUM_PATH=tools['chromium'],
                      LD_LIBRARY_PATH=tools['browser_library_directory'])
    completed('browser', [tools['node'], str(ROOT/'native_package.cjs')], browserenv, 300)
    result = json.loads((run/'browser/native-results.json').read_text())
    assert result['status']=='PASS_BOUNDED_NATIVE_ARTIFACT_OBSERVATION'
    for filename, digest in admission['prepared_source_sha256'].items():
        value = sha((ROOT/filename).read_bytes())
        receipt['observed_source'][filename] = value
        assert value == digest, filename
    receipt['status'] = 'PASS_BOUNDED_NATIVE_ARTIFACT_OBSERVATION'
    code = 0
except BaseException as error:
    receipt.update(status='FAILED_ATTEMPT',error=repr(error))
    print(json.dumps({'status':receipt['status'],'error':repr(error),'receipt':str(run/'receipt.json')}),flush=True)
finally:
    for reserved_socket in reserved:
        reserved_socket.close()
    receipt['cleanup'] = cleanup()
    for child in children:
        child['stdout'].close()
        child['stderr'].close()
        child['record'].setdefault('exit_code',child['process'].poll())
        for suffix in ['stdout','stderr']:
            p=run/(child['name']+'.'+suffix+'.txt')
            child['record'][suffix+'_sha256']=sha(p.read_bytes())
    if receipt['cleanup']['remaining']:
        receipt['status']='FAIL_OWNED_DESCENDANTS_REMAIN'
        code=1
    receipt['finished_at']=stamp()
    checkpoint()
    print(json.dumps({'status':receipt['status'],'receipt':str(run/'receipt.json'),'owned_remaining':receipt['cleanup']['remaining']}),flush=True)
sys.exit(code)

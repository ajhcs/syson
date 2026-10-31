"""Passive, source-pinned Central bootstrap; never executes acquired code."""
from pathlib import Path, PurePosixPath
import fcntl, hashlib, importlib.util, json, os, pwd, re, ssl, sys, time
import urllib.error, urllib.request
import contextlib, signal

CONTROLS = {'cpu.max': '100000 100000', 'memory.high': '536870912',
            'memory.max': '1073741824', 'memory.swap.max': '0', 'pids.max': '32'}
PREFIX = 'syson-b33-closure-'
OUTPUT_ROOT = Path('/workspace/build/builderbob')
BASE = 'https://repo.maven.apache.org/maven2/'
REQUEST_SET_SHA = '9c710bb285eff7ffb52c1608a2e8497451dd86bbd4057ddb444c04d4fcc415a0'
PAYLOAD_LIMIT, METADATA_LIMIT = 128 * 1024**2, 96 * 1024**2
MIB = 1024**2
SUFFIXES = ['', '.sha512', '.sha256', '.sha1', '.asc']
OWNED_SHA = 'ff18c5d2e6af6a8b21a4d59129401d9c9274f4d4727e2e448eae331c72f6b92d'


def require(ok, message):
    if not ok:
        raise ValueError(message)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate manifest key')
        result[key] = value
    return result


def requests(raw):
    require(len(raw) <= 256 * 1024 and hashlib.sha256(raw).hexdigest() == REQUEST_SET_SHA,
            'fixed manifest hash/size changed')
    proposal = json.loads(raw, object_pairs_hook=unique)
    limits = proposal['limits']
    expected_limits = {'exact_payloads': 79, 'exact_requests': 395,
        'payload_and_sidecar_total_bytes': PAYLOAD_LIMIT, 'per_response_header_pairs': 64,
        'per_response_header_UTF8_bytes': 16384, 'all_metadata_and_log_bytes_proposed': METADATA_LIMIT,
        'whole_phase_subject_seconds': 900, 'scope_RuntimeMaxSec': 1020,
        'task_disk_poll_bytes': 512 * MIB}
    require(all(type(limits[k]) is int and limits[k] == v for k, v in expected_limits.items()),
            'fixed manifest limits changed')
    payloads, rows, expected = proposal['new_payloads'], proposal['requests'], []
    require(len(payloads) == 79 and len(rows) == 395, 'fixed manifest count changed')
    for payload in payloads:
        relative, kind = payload['relative_artifact'], payload['kind']
        parts = relative.split('/')
        require(len(parts) >= 4 and all(re.fullmatch(r'[A-Za-z0-9_.-]+', p)
                and p not in ('.', '..') for p in parts), 'invalid artifact coordinate path')
        artifact, version, filename = parts[-3:]
        require(kind in ('pom', 'jar') and filename == f'{artifact}-{version}.{kind}',
                'wrong artifact filename')
        coordinate = payload.get('coordinate')
        require((coordinate is not None and coordinate == {'groupId': '.'.join(parts[:-3]),
                'artifactId': artifact, 'type': 'jar', 'version': version}) or
                (coordinate is None and relative ==
                 'org/apache/maven/plugins/maven-plugins/47/maven-plugins-47.pom'),
                'wrong artifact coordinate')
        maximum = (2 if kind == 'pom' else 16) * MIB
        require(payload['url'] == BASE + relative and payload['maximum_bytes'] == maximum,
                'wrong payload URL/limit')
        for suffix in SUFFIXES:
            expected.append({'url': BASE + relative + suffix,
                'relative_path': 'downloads/' + relative + suffix + '.partial',
                'maximum_bytes': 65536 if suffix else maximum,
                'role': 'checksum_or_unverified_signature' if suffix else kind,
                'allowed_status': [200, 404] if suffix else [200]})
    require(rows == expected and len({r['url'] for r in rows}) == 395 and
            len({r['relative_path'] for r in rows}) == 395, 'ordered requests/duplicates changed')
    return rows


def destination(out, row):
    relative = PurePosixPath(row['relative_path'])
    require(str(relative) == row['relative_path'] and not relative.is_absolute() and
            relative.parts[0] == 'downloads' and all(p not in ('.', '..') for p in relative.parts),
            'noncanonical nested destination')
    parent = out
    for part in relative.parts[:-1]:
        parent = parent / part
        require(not parent.is_symlink(), 'nested symlink refused')
        if not parent.exists():
            parent.mkdir(mode=0o700)
        require(parent.is_dir() and parent.resolve() == parent, 'noncanonical nested parent')
    target = parent / relative.name
    require(not target.exists() and not target.is_symlink(), 'existing destination refused')
    return target


def remaining(total):
    require(type(total) is int and 0 <= total <= PAYLOAD_LIMIT, 'aggregate payload limit')
    return PAYLOAD_LIMIT - total


class Metadata:
    def __init__(self, used=0, limit=95 * MIB, reserve=MIB):
        require(0 < reserve < limit <= 95 * MIB and 0 <= used <= limit - reserve,
                'metadata profile limit')
        self.used, self.limit, self.reserve, self.last = used, limit, reserve, None

    def write(self, path, value, final=False):
        raw = (json.dumps(value, indent=2) + '\n').encode('utf-8')
        require(len(raw) <= self.limit - self.used - (0 if final else self.reserve),
                'metadata byte limit')
        with path.open('xb') as output:
            output.write(raw)
        self.used += len(raw)

    def response(self, path, value):
        self.last = value
        self.write(path, value)
        self.last = None

    def finish(self, path, record):
        try:
            self.write(path, record, final=True)
        except ValueError:
            previous, count = record.get('status'), len(record.get('requests', []))
            retained = {key: record[key] for key in ('inputs_before', 'inputs_after', 'controls',
                        'error', 'error_prefix_only') if key in record}
            record.clear()
            record.update(status='FAIL_METADATA_LIMIT', previous_status=previous,
                          metadata_error='full receipt exceeds profile; prior files retained',
                          completed_requests=count, failed_response=self.last, **retained)
            self.write(path, record, final=True)


def fresh_output(out):
    require(out.parent.resolve() == out.parent and out.resolve() == out and not out.is_symlink()
            and out.is_relative_to(OUTPUT_ROOT) and out.is_dir()
            and out.stat().st_uid == os.getuid() and out.stat().st_mode & 0o077 == 0
            and not any(out.iterdir()), 'fresh owned output required')


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, newurl):
        return None


@contextlib.contextmanager
def request_deadline(seconds):
    require(seconds > 0 and signal.getitimer(signal.ITIMER_REAL) == (0.0, 0.0), 'existing/invalid request timer')
    def expired(signum, frame):
        raise TimeoutError('request wall deadline')
    previous = signal.signal(signal.SIGALRM, expired)
    signal.setitimer(signal.ITIMER_REAL, seconds)
    try:
        yield
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, previous)


def transfer(opener, row, destination, available, metadata=None):
    metadata = metadata or Metadata()
    started = time.monotonic()
    result = {'url': row['url'], 'relative_path': row['relative_path'], 'role': row['role'],
              'status': None, 'final_url': None, 'headers': [], 'bytes': 0}
    try:
        with request_deadline(60):
            request = urllib.request.Request(row['url'], headers={'User-Agent': 'Syson-Passive-Bootstrap/1.0'})
            try:
                response = opener.open(request, timeout=15)
            except urllib.error.HTTPError as error:
                response = error
            result.update(status=response.code, final_url=response.geturl())
            with response, destination.open('xb') as output:
                header_bytes = 0
                for key, value in response.headers.items():
                    header_bytes += len(key.encode('utf-8')) + len(value.encode('utf-8')) + 4
                    require(len(result['headers']) < 64 and header_bytes <= 16384, 'header metadata limit')
                    result['headers'].append((key, value))
                require(result['final_url'] == row['url'], 'redirect refused')
                maximum = min(row['maximum_bytes'], available)
                while True:
                    raw = response.read(min(65536, maximum - result['bytes'] + 1))
                    if not raw:
                        break
                    keep = raw[:maximum - result['bytes']]
                    output.write(keep)
                    result['bytes'] += len(keep)
                    require(len(keep) == len(raw), 'stream byte limit')
        return result
    finally:
        result['seconds'] = time.monotonic() - started
        result['sha256'] = digest(destination) if destination.exists() else None
        metadata.response(destination.with_suffix(destination.suffix + '.response.json'), result)


def checksums(out, rows):
    result = []
    for payload in (row for row in rows if row['role'] in ('pom', 'jar')):
        artifact = out / payload['relative_path']
        for suffix, algorithm in [('.sha512', 'sha512'), ('.sha256', 'sha256'), ('.sha1', 'sha1')]:
            row = next(row for row in rows if row['url'] == payload['url'] + suffix)
            if row['status'] != 200:
                continue
            expected = (out / row['relative_path']).read_text().strip()
            require(re.fullmatch('[0-9a-fA-F]{' + str(hashlib.new(algorithm).digest_size * 2) + '}', expected), 'invalid sidecar')
            actual = hashlib.new(algorithm, artifact.read_bytes()).hexdigest()
            require(expected.lower() == actual, 'checksum mismatch')
            result.append({'artifact': payload['url'][len(BASE):], 'algorithm': algorithm, 'match': True})
    return result


def main():
    spec_path, expected, descriptor, unit, cookie = sys.argv[1:]
    require(digest(__file__) == os.environ['SYSON_BOOTSTRAP_SOURCE_SHA256'], 'source pin changed')
    require(Path(spec_path).stat().st_size <= 256 * 1024, 'specification framing limit')
    require(digest(spec_path) == expected, 'specification pin changed')
    spec = json.loads(Path(spec_path).read_bytes()); out = Path(spec['out'])
    fresh_output(out)
    home = pwd.getpwuid(os.getuid())
    require(home.pw_name == 'builderbob' and os.environ['HOME'] == home.pw_dir, 'operator changed')
    require(digest(spec['ownership_helper']) == OWNED_SHA, 'ownership provider changed')
    loader = importlib.util.spec_from_file_location('owned', spec['ownership_helper'])
    owned = importlib.util.module_from_spec(loader); loader.loader.exec_module(owned)
    proof = owned.attest(unit, cookie, PREFIX, out / 'scope-proof.json', os.environ)
    group = Path('/sys/fs/cgroup') / proof['control_group'].lstrip('/')
    metadata = Metadata(used=(out / 'scope-proof.json').stat().st_size)
    record = {'status': 'FAIL_PRESERVED', 'requests': []}
    try:
        record['controls'] = {key: (group / key).read_text().strip() for key in CONTROLS}
        require(record['controls'] == CONTROLS, 'actual controls changed')
        expected_lock, observed_lock = Path(spec['compiler_lock']).stat(), os.fstat(int(descriptor))
        require((expected_lock.st_dev, expected_lock.st_ino) == (observed_lock.st_dev, observed_lock.st_ino), 'lock descriptor mismatch')
        fcntl.flock(int(descriptor), fcntl.LOCK_EX | fcntl.LOCK_NB)
        record['inputs_before'] = {p: digest(p) for p in spec['inputs']}
        require(record['inputs_before'] == spec['input_sha256'], 'input differs from frozen admission')
        require(Path(spec['proposal']).stat().st_size <= 256 * 1024, 'manifest framing limit')
        require(digest(spec['proposal']) == spec['proposal_sha256'], 'proposal changed')
        rows = requests(Path(spec['proposal']).read_bytes())
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect(),
                    urllib.request.HTTPSHandler(context=ssl.create_default_context()))
        started, total = time.monotonic(), 0
        for row in rows:
            require(time.monotonic() - started < 900, 'phase deadline')
            value = transfer(opener, row, destination(out, row), remaining(total), metadata)
            record['requests'].append(value); total += value['bytes']
            require(value['status'] in row['allowed_status'], 'HTTP failure retained; no fallback')
        record['checksums'] = checksums(out, record['requests'])
        record.update(status='PASS_PASSIVE_ACQUISITION_ONLY', signature_verified=False,
                      trust='Sidecars are same-origin consistency only; no acquired code may execute from this result')
    except BaseException as error:
        record['error'] = repr(error)[:4096]
        record['error_prefix_only'] = len(repr(error)) > 4096
    finally:
        record['inputs_after'] = {p: digest(p) for p in spec['inputs']}
        if record.get('inputs_before') != record['inputs_after'] or digest(spec_path) != expected:
            record['status'] = 'FAIL_INPUT_DRIFT'
        metadata.finish(out / 'receipt.json', record)
    require(record['status'] == 'PASS_PASSIVE_ACQUISITION_ONLY', 'passive acquisition failed')


if __name__ == '__main__':
    main()

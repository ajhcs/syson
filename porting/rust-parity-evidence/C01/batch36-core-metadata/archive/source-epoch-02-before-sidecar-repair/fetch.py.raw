"""Fixed 56-primary metadata client; data acquisition grants no code trust."""
from pathlib import Path
import hashlib, json, os, pwd, sys, time, types

PROVIDER = Path(__file__).resolve().parents[1] / 'batch33-passive-closure' / 'fetch.py'
PROVIDER_SHA = 'faaa2a57387bf4f6880ea3189d95ec75ffcc02a1df53703fae335d57f09aef25'
MANIFEST_SHA = 'd4c51a84aa264eaa057d42f4d208744573520085dca116103b874790c8fb445d'
PRIMARY_SHA = 'fd3874c62b6d7068fe9bb42446c65098ca55f425657fe8b4f4997fdbab0647af'
MIB = 1024**2
BODY_LIMIT, GETTER_METADATA = 16 * MIB, 63 * MIB
BASE = 'https://repo.maven.apache.org/maven2/'


def load_verified(path, expected, name):
    allowed = {'syson_verified_passive_transport': PROVIDER_SHA,
               'syson_verified_scope_owner': 'ff18c5d2e6af6a8b21a4d59129401d9c9274f4d4727e2e448eae331c72f6b92d'}
    if expected != allowed.get(name):
        raise ValueError('pinned provider hash changed')
    path = Path(path)
    if path.resolve() != path or path.is_symlink():
        raise ValueError('noncanonical pinned provider')
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != expected:
        raise ValueError('pinned provider hash changed')
    module = types.ModuleType(name)
    module.__file__ = str(path)
    exec(compile(raw, str(path), 'exec'), module.__dict__)
    return module


base = load_verified(PROVIDER, PROVIDER_SHA, 'syson_verified_passive_transport')
require = base.require


def encoded_size(value):
    return len((json.dumps(value, indent=2) + '\n').encode('utf-8'))


def requests(raw):
    require(len(raw) <= 256 * 1024 and hashlib.sha256(raw).hexdigest() == MANIFEST_SHA,
            'fixed metadata manifest hash/size changed')
    manifest = json.loads(raw, object_pairs_hook=base.unique)
    fixed = {'exact_primaries': 56, 'exact_requests': 280,
        'payload_and_sidecar_total_bytes': BODY_LIMIT, 'per_POM_bytes': 2 * MIB,
        'per_version_metadata_bytes': 256 * 1024, 'per_sidecar_bytes': 65536,
        'per_response_header_pairs': 64, 'per_response_header_UTF8_bytes': 16384,
        'getter_metadata_proof_receipt_bytes': GETTER_METADATA,
        'getter_final_failure_receipt_reserve_bytes': MIB,
        'root_child_stdout_stderr_bytes': 960 * 1024, 'root_controller_JSON_bytes': 64 * 1024,
        'all_metadata_and_log_bytes': 64 * MIB, 'total_logical_owned_write_bound_bytes': 80 * MIB,
        'manifest_and_spec_each_maximum_bytes': 256 * 1024,
        'whole_phase_subject_seconds': 900, 'scope_RuntimeMaxSec': 1020}
    limits = manifest['limits']
    require(set(limits) == set(fixed) and all(type(limits[k]) is int and limits[k] == v
            for k, v in fixed.items()), 'fixed metadata profile changed')
    primaries, rows = manifest['new_primaries'], manifest['requests']
    require(len(primaries) == 56 and len(rows) == 280, 'fixed metadata count changed')
    projection = [(p['relative_artifact'], p['kind']) for p in primaries]
    seal = hashlib.sha256(json.dumps(projection, separators=(',', ':')).encode()).hexdigest()
    require(seal == PRIMARY_SHA, 'fixed primary set/order changed')
    expected = []
    for index, item in enumerate(primaries):
        relative, kind = item['relative_artifact'], item['kind']
        parts = relative.split('/')
        require(all(base.re.fullmatch(r'[A-Za-z0-9_.-]+', p) and p not in ('.', '..')
                    for p in parts), 'noncanonical primary path')
        require(kind in ('pom', 'version_metadata'), 'wrong primary role')
        if kind == 'pom':
            group, artifact, version = '.'.join(parts[:-3]), parts[-3], parts[-2]
            require(parts[-1] == artifact + '-' + version + '.pom', 'wrong POM filename')
            coordinate = {'groupId': group, 'artifactId': artifact, 'version': version}
            maximum = 2 * MIB
        else:
            require(parts[-1] == 'maven-metadata.xml', 'wrong metadata filename')
            coordinate = {'groupId': '.'.join(parts[:-2]), 'artifactId': parts[-2]}
            maximum = 256 * 1024
        require(item['coordinate'] == coordinate and item['url'] == BASE + relative
                and type(item['primary_index']) is int and item['primary_index'] == index
                and type(item['maximum_bytes']) is int and item['maximum_bytes'] == maximum
                and type(item['allowed_status']) is list
                and all(type(status) is int for status in item['allowed_status'])
                and item['allowed_status'] == [200], 'fixed primary coordinate/index/limit/status changed')
        for suffix in base.SUFFIXES:
            row = {'url': BASE + relative + suffix,
                'relative_path': 'downloads/' + relative + suffix + '.partial',
                'maximum_bytes': 65536 if suffix else maximum,
                'role': kind if not suffix else 'signature' if suffix == '.asc' else 'checksum',
                'allowed_status': [200, 404] if suffix else [200], 'primary_index': index}
            if suffix in ('.sha512', '.sha256', '.sha1'):
                row['algorithm'] = suffix[1:]
            expected.append(row)
    require(all(type(r['primary_index']) is int and type(r['maximum_bytes']) is int
                and type(r['allowed_status']) is list
                and all(type(status) is int for status in r['allowed_status'])
                for r in rows) and rows == expected and len({r['url'] for r in rows}) == 280
                and len({r['relative_path'] for r in rows}) == 280, 'fixed ordered rows changed')
    return rows


def remaining(total):
    require(type(total) is int and 0 <= total <= BODY_LIMIT, 'metadata aggregate body limit')
    return BODY_LIMIT - total


class Metadata(base.Metadata):
    def __init__(self, used=0):
        require(type(used) is int and 0 <= used <= MIB, 'static proof framing limit')
        super().__init__(used=used, limit=GETTER_METADATA, reserve=MIB)

    def response(self, path, value):
        self.last = value
        require(encoded_size({k: v for k, v in value.items() if k != 'headers'}) <= 4096,
                'response framing limit')
        super().response(path, value)


def checksums(out, rows):
    # Copy roles only for the immutable provider's pom/jar filter; keep receipts unchanged.
    adapted = [dict(row, role='pom') if row['role'] == 'version_metadata' else dict(row)
               for row in rows]
    return base.checksums(out, adapted)


def controls(proof):
    group = Path('/sys/fs/cgroup') / proof['control_group'].lstrip('/')
    return {key: (group / key).read_text().strip() for key in base.CONTROLS}


def opener():
    return base.urllib.request.build_opener(base.urllib.request.ProxyHandler({}), base.NoRedirect(),
            base.urllib.request.HTTPSHandler(context=base.ssl.create_default_context()))


def static_frame(proof_bytes, record):
    return proof_bytes + encoded_size({k: v for k, v in record.items() if k != 'requests'})


def main():
    spec_path, expected, descriptor, unit, cookie = sys.argv[1:]
    require(base.digest(__file__) == os.environ['SYSON_BOOTSTRAP_SOURCE_SHA256'], 'source pin changed')
    require(Path(spec_path).stat().st_size <= 256 * 1024, 'specification framing limit')
    require(base.digest(spec_path) == expected, 'specification pin changed')
    spec = json.loads(Path(spec_path).read_bytes(), object_pairs_hook=base.unique)
    out = Path(spec['out'])
    base.fresh_output(out)
    home = pwd.getpwuid(os.getuid())
    require(home.pw_name == 'builderbob' and os.environ['HOME'] == home.pw_dir, 'operator changed')
    owned = load_verified(spec['ownership_helper'], base.OWNED_SHA, 'syson_verified_scope_owner')
    proof = owned.attest(unit, cookie, base.PREFIX, out / 'scope-proof.json', os.environ)
    proof_bytes = (out / 'scope-proof.json').stat().st_size
    metadata = Metadata(used=proof_bytes)
    record = {'status': 'FAIL_PRESERVED', 'requests': [],
              'profile': {'body_bytes': BODY_LIMIT, 'getter_metadata_bytes': GETTER_METADATA,
                          'root_log_bytes': MIB, 'logical_total_bytes': 80 * MIB}}
    try:
        record['controls'] = controls(proof)
        require(record['controls'] == base.CONTROLS, 'actual controls changed')
        wanted, observed = Path(spec['compiler_lock']).stat(), os.fstat(int(descriptor))
        require((wanted.st_dev, wanted.st_ino) == (observed.st_dev, observed.st_ino), 'lock descriptor mismatch')
        base.fcntl.flock(int(descriptor), base.fcntl.LOCK_EX | base.fcntl.LOCK_NB)
        require(spec['input_sha256'].get(str(PROVIDER)) == PROVIDER_SHA, 'provider absent from frozen inputs')
        require(len(spec['inputs']) == len(set(spec['inputs'])), 'duplicate declared input')
        before = {p: base.digest(p) for p in spec['inputs']}
        require(before == spec['input_sha256'], 'input differs from frozen admission')
        require(proof_bytes + encoded_size({'before': before, 'after': before})
                <= MIB - 128 * 1024, 'static input framing limit')
        record['inputs_before'] = before
        require(Path(spec['proposal']).stat().st_size <= 256 * 1024, 'manifest framing limit')
        require(base.digest(spec['proposal']) == spec['proposal_sha256'] == MANIFEST_SHA, 'proposal changed')
        rows = requests(Path(spec['proposal']).read_bytes())
        client, started, total = opener(), time.monotonic(), 0
        for row in rows:
            require(time.monotonic() - started < 900, 'phase deadline')
            value = base.transfer(client, row, base.destination(out, row), remaining(total), metadata)
            record['requests'].append(value)
            total += value['bytes']
            require(value['status'] in row['allowed_status'], 'HTTP failure retained; no fallback')
        record['body_bytes'] = total
        record['checksums'] = checksums(out, record['requests'])
        record.update(status='PASS_PASSIVE_ACQUISITION_ONLY', signature_verified=False,
                      trust='Same-origin consistency only; no acquired code may execute')
    except BaseException as error:
        record['error'] = repr(error)[:4096]
        record['error_prefix_only'] = len(repr(error)) > 4096
    finally:
        try:
            after = {p: base.digest(p) for p in spec['inputs']}
            require(proof_bytes + encoded_size({'before': record.get('inputs_before'), 'after': after})
                    <= MIB - 128 * 1024, 'static input framing limit')
            record['inputs_after'] = after
            if ('inputs_before' in record and record['inputs_before'] != after) or base.digest(spec_path) != expected:
                record['status'] = 'FAIL_INPUT_DRIFT'
        except (OSError, ValueError) as error:
            record.update(status='FAIL_INPUT_DRIFT', error=repr(error)[:4096])
        if static_frame(proof_bytes, record) > MIB:
            record = {'status': 'FAIL_STATIC_FRAME', 'completed_requests': len(record['requests']),
                      'full_record_sha256': hashlib.sha256(json.dumps(record).encode()).hexdigest()}
        metadata.finish(out / 'receipt.json', record)
    require(record['status'] == 'PASS_PASSIVE_ACQUISITION_ONLY', 'metadata passive acquisition failed')


if __name__ == '__main__':
    main()

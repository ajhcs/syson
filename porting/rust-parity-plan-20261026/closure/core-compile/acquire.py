"""Finite root-admitted model epochs; acquired bytes never grant code trust."""
from pathlib import Path
import hashlib
import json
import os
import pwd
import sys
import time
import types

ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT.parent / 'rust-parity-evidence' / 'C01'
HELPER = EVIDENCE / 'batch36-core-metadata' / 'fetch.py'
HELPER_SHA = 'eabb3fe426d2894b1106b86c527cb495033720005b61671c7b89331a1b0153d6'
TRANSPORT = EVIDENCE / 'batch33-passive-closure' / 'fetch.py'
TRANSPORT_SHA = 'faaa2a57387bf4f6880ea3189d95ec75ffcc02a1df53703fae335d57f09aef25'
OWNER = EVIDENCE / 'scope-ownership-repair' / 'scope_ownership.py'
OWNER_SHA = 'ff18c5d2e6af6a8b21a4d59129401d9c9274f4d4727e2e448eae331c72f6b92d'
MIB = 1024**2
EPOCHS = {'models-01': {'manifest': 'requests/models-01.json',
    'sha256': '682c4b8569beeb95defb30b315496b5f474d133d99170b4eb79827466c117f49',
    'primary_sha256': 'c622f3e843134dfa48966435087085b4af5c328e1f0289ec25cf13732a66c698',
    'primaries': 35, 'requests': 175, 'roles': ('pom', 'version_metadata')},
    'models-02': {'manifest': 'requests/models-02.json',
        'sha256': '7434575a1030365ba75b4f0650d36183a1e930efd714c367cf4e810db4445ef4',
        'primary_sha256': '61db2cd50e4a9cfcac49801208c2060093bfe68fab141fcca4fc0caa690aaa42',
        'primaries': 18, 'requests': 90, 'roles': ('pom',)},
    'models-03': {'manifest': 'requests/models-03.json',
        'sha256': '7eafb6e5436dd04692e43a1f3d87a204dc206ed3fe55a80f9cae2d128f1be428',
        'primary_sha256': 'c4f24591cad25f9aac6f8e2c6a6aa78e1f6eef396497af42c057546a7b602458',
        'primaries': 5, 'requests': 25, 'roles': ('pom',)},
    'models-04': {'manifest': 'requests/models-04.json',
        'sha256': 'e0d4b420cdcdcf23023731af35511fb7fb78402e9d492a87840800c661814289',
        'primary_sha256': 'a08fa9a23d192d508568b3ac9f863e40c06b0bef55665d21782adf06b30e2e71',
        'primaries': 1, 'requests': 5, 'roles': ('pom',)},
    'jacoco-binaries-01': {'manifest': 'requests/jacoco-binaries-01.json',
        'sha256': '9ec6ebc4b9805b4117ad1b13534f620316bdab5758b5a8a562d23da1561910e6',
        'primary_sha256': '75994f43fe21c11ccda503d8610d90eb392249217217d4138fdcebb7d4fe7e76',
        'primaries': 10, 'requests': 50, 'roles': ('jar',)},
    'commons-text-junit-bom-01': {'manifest': 'requests/commons-text-junit-bom-01.json',
        'sha256': '65626d1e844c0b780b13b359a058eca4a03e88d371c06bb236ecd9b0620555fd',
        'primary_sha256': '96472db392963c04d9294e62b1cd7522b0d78a04976eda7eda30b92ca06c8d8c',
        'primaries': 1, 'requests': 5, 'roles': ('pom',)},
    'eclipse-filesystem-metadata-01': {'manifest': 'requests/eclipse-filesystem-metadata-01.json',
        'sha256': '72fbfd8426066a6a6434df5a72d49400232f4598ff4a73adf37007c173b997fc',
        'primary_sha256': '3810ba9daa40ec9d232881d3ae58727130f2f6b38f221c76a127c9216b7e9cba',
        'primaries': 1, 'requests': 5, 'roles': ('version_metadata',)},
    'eclipse-filesystem-descriptors-01': {'manifest': 'requests/eclipse-filesystem-descriptors-01.json',
        'sha256': '559988c53686025a527d483e8e884793e8d41e6cdbb73c33315741cbe381a079',
        'primary_sha256': 'd18c4a89b1d813ff26ffb3f9ecfad54575711e7369f980fd1ee75bf76f051555',
        'primaries': 25, 'requests': 125, 'roles': ('pom',)}}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def load_verified(path, expected, name):
    allowed = {'core_metadata_primitives': (HELPER, HELPER_SHA),
               'core_scope_owner': (OWNER, OWNER_SHA)}
    require(name in allowed and (path, expected) == allowed[name], 'fixed provider binding changed')
    require(path.resolve() == path and not path.is_symlink() and path.stat().st_size <= 65536,
            'canonical bounded provider required')
    with path.open('rb') as stream:
        raw = stream.read(65537)
    require(len(raw) <= 65536 and hashlib.sha256(raw).hexdigest() == expected, 'whole provider pin changed')
    module = types.ModuleType(name)
    module.__file__ = str(path)
    exec(compile(raw, str(path), 'exec'), module.__dict__)
    return module


helper = load_verified(HELPER, HELPER_SHA, 'core_metadata_primitives')
base = helper.base
fresh_output, controls, opener = base.fresh_output, helper.controls, helper.opener


def epoch(identifier):
    require(type(identifier) is str and identifier in EPOCHS, 'unadmitted finite epoch')
    return EPOCHS[identifier]


def read_small(path, maximum):
    require(path.resolve() == path and not path.is_symlink() and path.stat().st_size <= maximum,
            'canonical bounded input required')
    with path.open('rb') as stream:
        raw = stream.read(maximum + 1)
    require(len(raw) <= maximum, 'input byte limit')
    return raw


def requests(identifier, raw):
    fixed = epoch(identifier)
    require(type(raw) is bytes and len(raw) <= 256 * 1024 and
            hashlib.sha256(raw).hexdigest() == fixed['sha256'], 'fixed manifest hash/size changed')
    manifest = json.loads(raw, object_pairs_hook=base.unique)
    require(manifest['schema'] == 'C01-NORMAL-CORE-CLOSURE-REQUESTS/1' and
            manifest['epoch'] == identifier, 'fixed manifest identity changed')
    profile = {'exact_primaries': fixed['primaries'], 'exact_requests': fixed['requests'],
        'payload_and_sidecar_total_bytes': 16 * MIB, 'per_POM_bytes': 2 * MIB,
        'per_version_metadata_bytes': 256 * 1024, 'per_sidecar_bytes': 65536,
        'per_response_header_pairs': 64, 'per_response_header_UTF8_bytes': 16384,
        'getter_metadata_proof_receipt_bytes': 63 * MIB,
        'getter_final_failure_receipt_reserve_bytes': MIB,
        'root_child_stdout_stderr_bytes': 960 * 1024, 'root_controller_JSON_bytes': 64 * 1024,
        'all_metadata_and_log_bytes': 64 * MIB, 'total_logical_owned_write_bound_bytes': 80 * MIB,
        'manifest_and_spec_each_maximum_bytes': 256 * 1024,
        'whole_phase_subject_seconds': 900, 'scope_RuntimeMaxSec': 1020}
    if fixed['roles'] == ('jar',):
        profile['per_JAR_bytes'] = MIB
    limits = manifest['limits']
    require(type(limits) is dict and set(limits) == set(profile) and all(
        type(limits[k]) is int and limits[k] == v for k, v in profile.items()), 'fixed profile changed')
    primaries, rows = manifest['new_primaries'], manifest['requests']
    require(type(primaries) is list and type(rows) is list and len(primaries) == fixed['primaries']
            and len(rows) == fixed['requests'], 'fixed epoch counts changed')
    projection = [(p['relative_artifact'], p['kind']) for p in primaries]
    seal = hashlib.sha256(json.dumps(projection, separators=(',', ':')).encode()).hexdigest()
    require(seal == fixed['primary_sha256'], 'independent ordered primary set changed')
    expected = []
    for index, primary in enumerate(primaries):
        relative, kind = primary['relative_artifact'], primary['kind']
        parts = relative.split('/')
        require(kind in fixed['roles'] and len(parts) >= 3 and all(
            base.re.fullmatch(r'[A-Za-z0-9_.-]+', p) and p not in ('.', '..') for p in parts),
            'noncanonical primary role/path')
        if kind == 'pom':
            require(len(parts) >= 4 and parts[-1] == parts[-3] + '-' + parts[-2] + '.pom',
                    'fixed POM filename')
            coordinate = {'groupId': '.'.join(parts[:-3]), 'artifactId': parts[-3], 'version': parts[-2]}
            maximum = 2 * MIB
        elif kind == 'jar':
            require(len(parts) >= 4, 'fixed JAR path depth')
            stem = parts[-3] + '-' + parts[-2]
            name = parts[-1]
            require(name in (stem + '.jar', stem + '-runtime.jar'), 'fixed JAR classifier/filename')
            classifier = 'runtime' if name == stem + '-runtime.jar' else ''
            coordinate = {'groupId': '.'.join(parts[:-3]), 'artifactId': parts[-3],
                          'version': parts[-2], 'classifier': classifier}
            require((classifier == 'runtime') == (coordinate['artifactId'] == 'org.jacoco.agent'),
                    'fixed JaCoCo runtime classifier')
            maximum = MIB
        else:
            require(parts[-1] == 'maven-metadata.xml', 'fixed metadata filename')
            coordinate = {'groupId': '.'.join(parts[:-2]), 'artifactId': parts[-2]}
            maximum = 256 * 1024
        require(primary['coordinate'] == coordinate and primary['url'] == base.BASE + relative and
            type(primary['primary_index']) is int and primary['primary_index'] == index and
            type(primary['maximum_bytes']) is int and primary['maximum_bytes'] == maximum and
            type(primary['allowed_status']) is list and
            all(type(s) is int for s in primary['allowed_status']) and primary['allowed_status'] == [200],
            'fixed primary coordinate/index/cap/status changed')
        for suffix in base.SUFFIXES:
            row = {'url': base.BASE + relative + suffix,
                'relative_path': 'downloads/' + relative + suffix + '.partial',
                'maximum_bytes': 65536 if suffix else maximum,
                'role': kind if not suffix else 'signature' if suffix == '.asc' else 'checksum',
                'allowed_status': [200, 404] if suffix else [200], 'primary_index': index}
            if suffix in ('.sha512', '.sha256', '.sha1'):
                row['algorithm'] = suffix[1:]
            expected.append(row)
    require(all(type(r['primary_index']) is int and type(r['maximum_bytes']) is int and
        type(r['allowed_status']) is list and all(type(s) is int for s in r['allowed_status']) for r in rows)
        and rows == expected and len({r['url'] for r in rows}) == fixed['requests'] and
        len({r['relative_path'] for r in rows}) == fixed['requests'], 'fixed ordered requests changed')
    return rows


def jar_checksums(out, rows):
    results = []
    by_url = {row['url']: row for row in rows}
    for payload in (row for row in rows if row['role'] == 'jar'):
        body = read_small(out / payload['relative_path'], MIB)
        require(body, 'empty captured JAR')
        matched = 0
        for algorithm in ('sha512', 'sha256', 'sha1'):
            row = by_url[payload['url'] + '.' + algorithm]
            if row['status'] == 404:
                continue
            require(row['status'] == 200, 'unapproved checksum status')
            sidecar = read_small(out / row['relative_path'], 65536)
            expected = helper.sidecar_digest(sidecar, algorithm, payload['url'].rsplit('/', 1)[-1])
            require(expected.lower() == hashlib.new(algorithm, body).hexdigest(), 'checksum mismatch')
            results.append({'artifact': payload['url'][len(base.BASE):], 'algorithm': algorithm, 'match': True})
            matched += 1
        require(matched > 0, 'no available matching checksum for captured JAR')
    return results


def main():
    spec_path, expected, descriptor, unit, cookie = sys.argv[1:]
    require(base.digest(__file__) == os.environ['SYSON_BOOTSTRAP_SOURCE_SHA256'], 'source pin changed')
    raw_spec = read_small(Path(spec_path), 256 * 1024)
    require(hashlib.sha256(raw_spec).hexdigest() == expected, 'specification pin changed')
    spec = json.loads(raw_spec, object_pairs_hook=base.unique)
    identifier = spec['epoch']
    fixed = epoch(identifier)
    manifest_path = Path(__file__).resolve().parent / fixed['manifest']
    require(Path(spec['proposal']) == manifest_path and spec['proposal_sha256'] == fixed['sha256'],
            'fixed epoch manifest binding changed')
    raw_manifest = read_small(manifest_path, 256 * 1024)
    rows = requests(identifier, raw_manifest)
    out = Path(spec['out'])
    fresh_output(out)
    home = pwd.getpwuid(os.getuid())
    require(home.pw_name == 'builderbob' and os.environ['HOME'] == home.pw_dir, 'operator changed')
    require(Path(spec['ownership_helper']) == OWNER, 'fixed ownership helper changed')
    owned = load_verified(OWNER, OWNER_SHA, 'core_scope_owner')
    proof = owned.attest(unit, cookie, base.PREFIX, out / 'scope-proof.json', os.environ)
    proof_bytes = (out / 'scope-proof.json').stat().st_size
    metadata = helper.Metadata(used=proof_bytes)
    record = {'status': 'FAIL_PRESERVED', 'epoch': identifier, 'requests': [],
        'profile': {'body_bytes': 16 * MIB, 'getter_metadata_bytes': 63 * MIB,
                    'root_log_bytes': MIB, 'logical_total_bytes': 80 * MIB}}
    try:
        record['controls'] = controls(proof)
        require(record['controls'] == base.CONTROLS, 'actual controls changed')
        wanted, observed = Path(spec['compiler_lock']).stat(), os.fstat(int(descriptor))
        require((wanted.st_dev, wanted.st_ino) == (observed.st_dev, observed.st_ino), 'lock descriptor mismatch')
        base.fcntl.flock(int(descriptor), base.fcntl.LOCK_EX | base.fcntl.LOCK_NB)
        required = {str(HELPER): HELPER_SHA, str(TRANSPORT): TRANSPORT_SHA, str(OWNER): OWNER_SHA,
                    str(manifest_path): fixed['sha256'], str(Path(__file__).resolve()): base.digest(__file__)}
        require(type(spec['inputs']) is list and len(spec['inputs']) == len(set(spec['inputs'])) and
            all(spec['input_sha256'].get(p) == h for p, h in required.items()), 'frozen provider/input closure')
        before = {p: base.digest(p) for p in spec['inputs']}
        require(before == spec['input_sha256'], 'input differs from frozen admission')
        require(proof_bytes + helper.encoded_size({'before': before, 'after': before}) <= MIB - 128 * 1024,
                'static input framing limit')
        record['inputs_before'] = before
        response_bound = fixed['requests'] * 2 * (6 * 16384 + 5120 + 4096)
        require(response_bound + 2 * MIB <= 63 * MIB, 'serialized fixed response profile does not fit')
        client, started, total = opener(), time.monotonic(), 0
        for row in rows:
            require(time.monotonic() - started < 900, 'phase deadline')
            value = base.transfer(client, row, base.destination(out, row), helper.remaining(total), metadata)
            record['requests'].append(value)
            total += value['bytes']
            require(value['status'] in row['allowed_status'], 'HTTP failure retained; no fallback')
        record['body_bytes'] = total
        checksum = jar_checksums if fixed['roles'] == ('jar',) else helper.checksums
        record['checksums'] = checksum(out, record['requests'])
        record.update(status='PASS_PASSIVE_ACQUISITION_ONLY', signature_verified=False,
            trust='Same-origin consistency only; no acquired code may execute', closure_complete=False,
            offline_compile_verified=False)
    except BaseException as error:
        record['error'] = repr(error)[:4096]
        record['error_prefix_only'] = len(repr(error)) > 4096
    finally:
        try:
            after = {p: base.digest(p) for p in spec['inputs']}
            require(proof_bytes + helper.encoded_size({'before': record.get('inputs_before'), 'after': after})
                    <= MIB - 128 * 1024, 'static input framing limit')
            record['inputs_after'] = after
            if ('inputs_before' in record and record['inputs_before'] != after) or base.digest(spec_path) != expected:
                record['status'] = 'FAIL_INPUT_DRIFT'
        except (OSError, ValueError) as error:
            record.update(status='FAIL_INPUT_DRIFT', error=repr(error)[:4096])
        if helper.static_frame(proof_bytes, record) > MIB:
            record = {'status': 'FAIL_STATIC_FRAME', 'completed_requests': len(record['requests']),
                'full_record_sha256': hashlib.sha256(json.dumps(record).encode()).hexdigest()}
        metadata.finish(out / 'receipt.json', record)
    require(record['status'] == 'PASS_PASSIVE_ACQUISITION_ONLY', 'finite passive acquisition failed')


if __name__ == '__main__':
    main()

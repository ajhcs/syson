"""Fixed direct-core data client; immutable transport provider, no code admission."""
from pathlib import Path
import hashlib, json, os, pwd, sys, time, types

PROVIDER = Path(__file__).resolve().parents[1] / 'batch33-passive-closure' / 'fetch.py'
PROVIDER_SHA = 'faaa2a57387bf4f6880ea3189d95ec75ffcc02a1df53703fae335d57f09aef25'
MANIFEST_SHA = 'f9e8ae517ca7323326ebe63f0a2974febd3c966a51bb0f0c23d80434214516ff'
MIB = 1024**2
BODY_LIMIT, GETTER_METADATA, PROFILE_METADATA = 64 * MIB, 31 * MIB, 32 * MIB
BASE = 'https://repo.maven.apache.org/maven2/'
COMPILE = [
    ('org.eclipse.emf', 'org.eclipse.emf.common', '2.45.0'),
    ('org.eclipse.emf', 'org.eclipse.emf.ecore', '2.42.0'),
    ('org.eclipse.emf', 'org.eclipse.emf.ecore.xmi', '2.40.0'),
    ('org.eclipse.emf', 'org.eclipse.emf.codegen.ecore', '2.46.0'),
    ('com.fasterxml.uuid', 'java-uuid-generator', '5.1.0'),
    ('org.apache.commons', 'commons-text', '1.10.0'),
]
PLUGINS = [
    ('org.jacoco', 'jacoco-maven-plugin', '0.8.14'),
    ('org.apache.maven.plugins', 'maven-resources-plugin', '3.5.0'),
    ('org.apache.maven.plugins', 'maven-compiler-plugin', '3.15.0'),
    ('org.apache.maven.plugins', 'maven-source-plugin', '3.3.1'),
    ('org.apache.maven.plugins', 'maven-jar-plugin', '3.5.1'),
]
TEST_ROOTS = [('org.springframework.boot', 'spring-boot-starter-test', '4.1.1'),
              ('org.testcontainers', 'testcontainers-junit-jupiter', '2.0.5')]


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


def expected_payloads():
    pairs = [(gav, extension, None) for gav in COMPILE + PLUGINS for extension in ['pom', 'jar']]
    pairs += [(gav, 'jar', 'sources') for gav in PLUGINS[:3]]
    pairs += [(gav, 'pom', None) for gav in TEST_ROOTS]
    return pairs + [(('org.apache.commons', 'commons-parent', '92'), 'pom', None)]


def requests(raw):
    require(len(raw) <= 256 * 1024 and hashlib.sha256(raw).hexdigest() == MANIFEST_SHA,
            'fixed direct manifest hash/size changed')
    manifest = json.loads(raw, object_pairs_hook=base.unique)
    limits = manifest['limits']
    fixed = {'exact_payloads': 28, 'exact_requests': 140,
        'payload_and_sidecar_total_bytes': BODY_LIMIT, 'per_POM_bytes': 2 * MIB,
        'per_JAR_or_sources_bytes': 16 * MIB, 'per_sidecar_bytes': 65536,
        'per_response_header_pairs': 64, 'per_response_header_UTF8_bytes': 16384,
        'all_metadata_and_log_bytes': PROFILE_METADATA,
        'getter_metadata_proof_receipt_bytes': GETTER_METADATA,
        'getter_final_failure_receipt_reserve_bytes': MIB,
        'root_child_stdout_stderr_bytes': 960 * 1024, 'root_controller_JSON_bytes': 64 * 1024,
        'manifest_and_spec_each_maximum_bytes': 256 * 1024,
        'whole_phase_subject_seconds': 900, 'scope_RuntimeMaxSec': 1020,
        'disk_polling_profile_bytes': 512 * MIB}
    require(all(type(limits[k]) is int and limits[k] == v for k, v in fixed.items()),
            'fixed direct profile changed')
    payloads, rows = manifest['new_payloads'], manifest['requests']
    require(len(payloads) == 28 and len(rows) == 140, 'fixed direct count changed')
    expected = []
    for item, (gav, extension, classifier) in zip(payloads, expected_payloads()):
        coordinate = dict(zip(['groupId', 'artifactId', 'version'], gav))
        filename = gav[1] + '-' + gav[2] + ('-' + classifier if classifier else '') + '.' + extension
        relative = '/'.join([gav[0].replace('.', '/'), gav[1], gav[2], filename])
        maximum = (2 if extension == 'pom' else 16) * MIB
        require(item['coordinate'] == coordinate and item['extension'] == item['kind'] == extension
                and 'classifier' in item and item['classifier'] == classifier
                and item['relative_artifact'] == relative and item['url'] == BASE + relative
                and type(item['maximum_bytes']) is int and item['maximum_bytes'] == maximum,
                'fixed direct coordinate/classifier changed')
        for suffix in base.SUFFIXES:
            expected.append({'url': BASE + relative + suffix,
                'relative_path': 'downloads/' + relative + suffix + '.partial',
                'maximum_bytes': 65536 if suffix else maximum,
                'role': 'checksum_or_unverified_signature' if suffix else extension,
                'allowed_status': [200, 404] if suffix else [200]})
    require(rows == expected and len({r['url'] for r in rows}) == 140 and
            len({r['relative_path'] for r in rows}) == 140, 'fixed direct ordered requests changed')
    return rows


def remaining(total):
    require(type(total) is int and 0 <= total <= BODY_LIMIT, 'direct aggregate body limit')
    return BODY_LIMIT - total


def controls(proof):
    group = Path('/sys/fs/cgroup') / proof['control_group'].lstrip('/')
    return {key: (group / key).read_text().strip() for key in base.CONTROLS}


def opener():
    return base.urllib.request.build_opener(base.urllib.request.ProxyHandler({}), base.NoRedirect(),
            base.urllib.request.HTTPSHandler(context=base.ssl.create_default_context()))


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
    metadata = base.Metadata(used=(out / 'scope-proof.json').stat().st_size,
                             limit=GETTER_METADATA, reserve=MIB)
    record = {'status': 'FAIL_PRESERVED', 'requests': [],
              'profile': {'body_bytes': BODY_LIMIT, 'getter_metadata_bytes': metadata.limit,
                          'root_log_bytes': MIB, 'logical_total_bytes': BODY_LIMIT + metadata.limit + MIB}}
    try:
        record['controls'] = controls(proof)
        require(record['controls'] == base.CONTROLS, 'actual controls changed')
        wanted, observed = Path(spec['compiler_lock']).stat(), os.fstat(int(descriptor))
        require((wanted.st_dev, wanted.st_ino) == (observed.st_dev, observed.st_ino), 'lock descriptor mismatch')
        base.fcntl.flock(int(descriptor), base.fcntl.LOCK_EX | base.fcntl.LOCK_NB)
        require(spec['input_sha256'].get(str(PROVIDER)) == PROVIDER_SHA, 'provider absent from frozen inputs')
        require(len(spec['inputs']) == len(set(spec['inputs'])), 'duplicate declared input')
        record['inputs_before'] = {p: base.digest(p) for p in spec['inputs']}
        require(record['inputs_before'] == spec['input_sha256'], 'input differs from frozen admission')
        require(encoded_size({'before': record['inputs_before'], 'after': record['inputs_before']})
                <= MIB - 65536, 'static input framing limit')
        require(Path(spec['proposal']).stat().st_size <= 256 * 1024, 'manifest framing limit')
        require(base.digest(spec['proposal']) == spec['proposal_sha256'] == MANIFEST_SHA, 'proposal changed')
        rows = requests(Path(spec['proposal']).read_bytes())
        client, started, total = opener(), time.monotonic(), 0
        for row in rows:
            require(time.monotonic() - started < 900, 'phase deadline')
            value = base.transfer(client, row, base.destination(out, row), remaining(total), metadata)
            record['requests'].append(value)
            total += value['bytes']
            require(encoded_size({k: v for k, v in value.items() if k != 'headers'}) <= 4096,
                    'response framing limit')
            require(value['status'] in row['allowed_status'], 'HTTP failure retained; no fallback')
        record['body_bytes'] = total
        record['checksums'] = base.checksums(out, record['requests'])
        record.update(status='PASS_PASSIVE_ACQUISITION_ONLY', signature_verified=False,
                      trust='Same-origin consistency only; no acquired code may execute')
    except BaseException as error:
        record['error'] = repr(error)[:4096]
        record['error_prefix_only'] = len(repr(error)) > 4096
    finally:
        try:
            record['inputs_after'] = {p: base.digest(p) for p in spec['inputs']}
            if record.get('inputs_before') != record['inputs_after'] or base.digest(spec_path) != expected:
                record['status'] = 'FAIL_INPUT_DRIFT'
        except (OSError, ValueError) as error:
            record.update(status='FAIL_INPUT_DRIFT', error=repr(error)[:4096])
        if encoded_size({k: v for k, v in record.items() if k != 'requests'}) > MIB:
            record['status'] = 'FAIL_FRAME_LIMIT'
        metadata.finish(out / 'receipt.json', record)
    require(record['status'] == 'PASS_PASSIVE_ACQUISITION_ONLY', 'direct passive acquisition failed')


if __name__ == '__main__':
    main()

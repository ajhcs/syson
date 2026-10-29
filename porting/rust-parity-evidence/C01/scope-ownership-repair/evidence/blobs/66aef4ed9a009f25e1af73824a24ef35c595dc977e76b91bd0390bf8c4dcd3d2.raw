"""Fetch six fixed primary Maven artifacts; inspect ZIP/XML as data, never code."""
import argparse
import fcntl
import hashlib
import json
import os
import re
from pathlib import Path, PurePosixPath
import signal
import ssl
import stat
import sys
import time
from urllib.parse import urlsplit
from urllib.request import build_opener, HTTPSHandler, HTTPRedirectHandler, ProxyHandler, Request
import xml.etree.ElementTree as ET
import zipfile

BASE = Path(__file__).resolve().parent
PROPOSAL_SHA256 = 'f26e4564b8bcf4ab0c16f7f0cf61570fe6b0f0e109e74e0d58f1a8db2c7dfa4c'
OUTPUT = Path('/workspace/build/builderbob/syson-batch22-20261029/c01-maven-bootstrap-run-01')


def require(ok, message):
    if not ok:
        raise RuntimeError(message)


def hashed(path, algorithm='sha256'):
    with Path(path).open('rb') as source:
        return hashlib.file_digest(source, algorithm).hexdigest()


def inputs(proposal):
    require(hashed(BASE / 'proposal.json') == PROPOSAL_SHA256, 'bootstrap proposal drift')
    for name, digest in proposal['bindings'].items():
        require(hashed(name) == digest, 'bound public input changed: ' + name)
    return {**proposal['bindings'], str(BASE / 'proposal.json'): hashed(BASE / 'proposal.json'),
            str(Path(__file__)): hashed(__file__)}


def url_allowed(url, proposal):
    allowed = {item[key] for item in proposal['artifacts'] for key in ['url', 'checksum_url']}
    value = urlsplit(url)
    require(url in allowed and value.scheme == 'https' and value.hostname == proposal['host'] and
            value.port in (None, 443) and not value.username and not value.password and
            not value.query and not value.fragment, 'not a literal approved HTTPS URL')


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, *args):
        raise RuntimeError('all bootstrap redirects refused')


def interrupted(signum, frame):
    raise RuntimeError('bootstrap interrupted by signal ' + str(signum))


def fetch(opener, url, name, expected_bytes, proposal, record, deadline):
    url_allowed(url, proposal)
    require(Path(name).name == name and name not in ['', '.', '..'], 'unsafe output basename')
    path = OUTPUT / 'downloads' / name
    event = {'url': url, 'path': str(path), 'bytes': 0, 'complete': False}
    record['requests'].append(event)
    remaining = min(proposal['bounds']['request_seconds'], deadline - time.monotonic())
    require(remaining > 0, 'bootstrap deadline reached')
    signal.setitimer(signal.ITIMER_REAL, remaining)
    try:
        with path.open('xb') as target:
            with opener.open(Request(url, headers={'User-Agent': 'SysON-C01-bootstrap',
                    'Accept-Encoding': 'identity'}), timeout=proposal['bounds']['socket_seconds']) as response:
                require(response.status == 200 and response.url == url, 'bootstrap HTTP status/URL differs')
                declared = response.headers.get('Content-Length')
                require(declared is None or declared == str(expected_bytes), 'unexpected Content-Length')
                event['headers'] = {key: response.headers.get(key) for key in
                                    ['Content-Length', 'Content-Type', 'ETag', 'Last-Modified']}
                while True:
                    chunk = response.read(min(65536, expected_bytes - event['bytes'] + 1))
                    if not chunk:
                        break
                    event['bytes'] += len(chunk)
                    record['entity_bytes'] += len(chunk)
                    require(event['bytes'] <= expected_bytes and
                            record['entity_bytes'] <= proposal['bounds']['entity_bytes'], 'HTTP body byte bound')
                    target.write(chunk)
                require(event['bytes'] == expected_bytes, 'truncated HTTP body')
                event['complete'] = True
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        if path.is_file():
            event.update(retained_bytes=path.stat().st_size, sha256=hashed(path))
    return path


def verify_checksum(path, item):
    require(hashed(path) == item['checksum_sha256'] and path.read_bytes().decode('ascii').strip() == item['sha1'],
            'official checksum metadata differs')


def verify_primary(path, item):
    require(hashed(path, 'sha1') == item['sha1'], 'primary artifact digest differs')


def controls(proposal, unit):
    cgroup = Path('/proc/self/cgroup').read_text().strip()
    require(re.fullmatch(re.escape(proposal['cgroup']['scope_prefix']) + r'[0-9a-f]{32}\.scope', unit)
            and cgroup.startswith('0::/') and cgroup.endswith('/' + unit),
            'not the owned bootstrap resource scope')
    directory = Path('/sys/fs/cgroup') / cgroup[3:].lstrip('/')
    actual = {key: (directory / key).read_text().strip() for key in proposal['cgroup'] if key != 'scope_prefix'}
    require(actual == {key: value for key, value in proposal['cgroup'].items() if key != 'scope_prefix'},
            'actual bootstrap resource controls differ')
    return {'cgroup': cgroup, 'controls': actual}


def write_data(name, raw, proposal):
    require(Path(name).name == name and name not in ['', '.', '..'], 'unsafe selected output basename')
    used = sum(path.stat().st_size for path in OUTPUT.rglob('*') if path.is_file())
    require(used + len(raw) <= proposal['bounds']['output_bytes'], 'logical output byte bound')
    with (OUTPUT / 'inspection' / name).open('xb') as target:
        target.write(raw)


def xml_data(raw):
    require(len(raw) <= 2097152, 'XML byte bound')
    decoded = raw.decode('utf-8-sig')
    require('\x00' not in decoded and '<!DOCTYPE' not in decoded.upper() and '<!ENTITY' not in decoded.upper(),
            'XML encoding/DTD/entity restriction')
    return ET.fromstring(decoded)


def children(node, name):
    return [child for child in node if child.tag.rsplit('}', 1)[-1] == name]


def text(node, name):
    values = children(node, name)
    require(len(values) <= 1, 'duplicate descriptor scalar: ' + name)
    return values[0].text if values else None


def descriptor(raw, item, proposal):
    node = xml_data(raw)
    require(node.tag.rsplit('}', 1)[-1] == 'plugin' and text(node, 'groupId') == 'org.apache.maven.plugins' and
            text(node, 'artifactId') == item['artifact'] and text(node, 'version') == item['version'],
            'plugin descriptor coordinates differ')
    groups = children(node, 'mojos')
    require(len(groups) == 1, 'missing/duplicate mojo inventory')
    mojos = [{child.tag.rsplit('}', 1)[-1]: child.text for child in mojo
              if child.tag.rsplit('}', 1)[-1] not in ['parameters', 'configuration', 'requirements']}
             for mojo in children(groups[0], 'mojo')]
    goals = [text(mojo, 'goal') for mojo in children(groups[0], 'mojo')]
    require(all(isinstance(goal, str) and goal for goal in goals) and len(goals) == len(set(goals)) and
            proposal['expected_goals'][item['artifact']] in goals, 'target/unique plugin goal inventory differs')
    return {'artifact': item['artifact'], 'version': item['version'], 'mojos': mojos,
            'raw_sha256': hashlib.sha256(raw).hexdigest(),
            'limit': 'Selected summary only; complete raw descriptor retained, no effective/plugin/dependency execution.'}


def jar_inventory(path, item, proposal):
    bounds = proposal['bounds']
    with zipfile.ZipFile(path) as archive:
        members = archive.infolist()
        require(0 < len(members) <= bounds['zip_entries'], 'ZIP entry count bound')
        names, rows, expanded = {}, [], 0
        for member in members:
            name = member.filename
            canonical = name.rstrip('/')
            relative = PurePosixPath(canonical)
            kind = stat.S_IFMT(member.external_attr >> 16)
            require(canonical and name == canonical + ('/' if member.is_dir() else '') and
                    ':' not in relative.parts[0] and str(relative) == canonical and not relative.is_absolute() and
                    '..' not in relative.parts and '\\' not in name and canonical not in names and
                    len(name.encode('utf-8')) <= bounds['zip_name_bytes'], 'ZIP path/duplicate restriction')
            require(not member.flag_bits & 1 and member.compress_type in [zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED] and
                    kind in [0, stat.S_IFREG, stat.S_IFDIR] and not (member.external_attr >> 16) & 0o7000,
                    'ZIP encryption/compression/file type restriction')
            require(member.file_size <= bounds['zip_entry_bytes'], 'ZIP individual expansion bound')
            expanded += member.file_size
            require(expanded <= bounds['zip_uncompressed_bytes'], 'ZIP aggregate expansion bound')
            names[canonical] = member
            rows.append({'name': name, 'bytes': member.file_size, 'compressed': member.compress_size,
                         'method': member.compress_type, 'crc32': member.CRC})
        for name in names:
            require(all(str(parent) not in names or names[str(parent)].is_dir()
                        for parent in PurePosixPath(name).parents if str(parent) != '.'), 'ZIP file/directory conflict')
        selected, summary = [], None
        if not item['name'].endswith('-sources.jar'):
            wanted = [proposal['selected_descriptor'],
                      'META-INF/maven/org.apache.maven.plugins/' + item['artifact'] + '/pom.xml']
            require(wanted[0] in names, 'required plugin descriptor absent')
            for index, name in enumerate(wanted):
                if name not in names:
                    continue
                member = names[name]
                require(not member.is_dir() and member.file_size <= bounds['selected_entry_bytes'], 'selected ZIP bound')
                with archive.open(member) as stream:
                    raw = stream.read(bounds['selected_entry_bytes'] + 1)
                require(len(raw) == member.file_size, 'selected ZIP length differs')
                xml_data(raw)
                output = item['artifact'] + ('-plugin.xml' if index == 0 else '-embedded-pom.xml')
                write_data(output, raw, proposal)
                selected.append({'entry': name, 'output': output, 'sha256': hashlib.sha256(raw).hexdigest()})
                if index == 0:
                    summary = descriptor(raw, item, proposal)
        return {'archive': item['name'], 'entries': rows, 'selected': selected, 'descriptor': summary,
                'sources_limit': 'Source archive inventory only; no class/source code executed.'}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--scope-unit', required=True)
    parser.add_argument('--lock-fd', required=True, type=int)
    args = parser.parse_args()
    os.umask(0o077)
    require(not os.path.lexists(OUTPUT) and OUTPUT.parent.resolve() == OUTPUT.parent and
            OUTPUT.is_relative_to('/workspace'), 'fresh canonical workspace output required')
    OUTPUT.mkdir(mode=0o700)
    record, proposal = {'status': 'RUNNING', 'requests': [], 'entity_bytes': 0, 'acquired_code_executed': False}, None
    try:
        raw = (BASE / 'proposal.json').read_bytes()
        require(hashlib.sha256(raw).hexdigest() == PROPOSAL_SHA256, 'bootstrap proposal changed')
        proposal = json.loads(raw)
        require(str(OUTPUT) == proposal['run_root'], 'bootstrap output binding changed')
        expected, actual = Path(proposal['lock']).stat(), os.fstat(args.lock_fd)
        require((expected.st_dev, expected.st_ino) == (actual.st_dev, actual.st_ino), 'wrong inherited compiler lock')
        fcntl.flock(args.lock_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        record['controls'], record['inputs_before'] = controls(proposal, args.scope_unit), inputs(proposal)
        for name in ['downloads', 'inspection']:
            (OUTPUT / name).mkdir(mode=0o700)
        for signum in [signal.SIGALRM, signal.SIGTERM, signal.SIGINT]:
            signal.signal(signum, interrupted)
        deadline = time.monotonic() + proposal['bounds']['phase_seconds']
        opener = build_opener(ProxyHandler({}), NoRedirect(),
                             HTTPSHandler(context=ssl.create_default_context(cafile=proposal['ca_file'])))
        for item in proposal['artifacts']:
            checksum = fetch(opener, item['checksum_url'], item['name'] + '.sha1', item['checksum_bytes'],
                             proposal, record, deadline)
            verify_checksum(checksum, item)
            path = fetch(opener, item['url'], item['name'], item['bytes'], proposal, record, deadline)
            verify_primary(path, item)
            if item['name'].endswith('.jar'):
                data = jar_inventory(path, item, proposal)
            else:
                node = xml_data(path.read_bytes())
                require(node.tag.rsplit('}', 1)[-1] == 'project', 'POM root differs')
                data = {'pom': item['name'], 'raw_sha256': hashed(path),
                        'direct': {key: text(node, key) for key in ['groupId', 'artifactId', 'version']},
                        'limit': 'Raw POM only; no parent/property/BOM/effective resolution.'}
            write_data(item['name'] + '-inspection.json', json.dumps(data, indent=2).encode(), proposal)
        require(time.monotonic() <= deadline, 'bootstrap phase deadline reached')
        record['status'] = 'PASS_PRIMARY_DATA_ACQUISITION_ONLY;PLUGIN_EXECUTION_AND_BUILD_OPEN'
    except BaseException as error:
        record.update(status='FAIL_PARTIAL_ATTEMPT_PRESERVED', error=repr(error))
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        try:
            if proposal is not None and 'inputs_before' in record:
                record['inputs_after'] = inputs(proposal)
                require(record['inputs_after'] == record['inputs_before'], 'bootstrap input drift')
        except BaseException as error:
            record.update(status='FAIL_PARTIAL_ATTEMPT_PRESERVED', input_after_error=repr(error))
        record['retained'] = {}
        try:
            for path in OUTPUT.rglob('*'):
                if path.is_file():
                    record['retained'][str(path.relative_to(OUTPUT))] = {
                        'bytes': path.stat().st_size, 'sha256': hashed(path)}
        except BaseException as error:
            record.update(status='FAIL_PARTIAL_ATTEMPT_PRESERVED', retention_error=repr(error))
        (OUTPUT / 'receipt.json').write_text(json.dumps(record, indent=2) + '\n')
    print(json.dumps({'status': record['status'], 'receipt': str(OUTPUT / 'receipt.json')}))
    return 0 if record['status'].startswith('PASS_') else 1


if __name__ == '__main__':
    raise SystemExit(main())

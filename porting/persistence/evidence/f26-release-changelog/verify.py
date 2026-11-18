#!/usr/bin/env python3
"""Verify the F26 source, pinned release ZIP and projected history offline."""
import argparse
from hashlib import sha256
from io import BytesIO
import json
from pathlib import Path, PurePosixPath
import subprocess
import sys
from xml.etree import ElementTree as ET
from zipfile import ZipFile

PACKET = Path(__file__).resolve().parent
REPO = PACKET.parents[3]
DONOR = 'fc15b4bdd4363af8e29ea5a842981bb21378a497'
SOURCE = '3191e2b8990373afee3415f6b26f8c5039a28321'
SOURCE_BASE = 'ad814c232ee24109639c1d52239cbeec6f4a3e92'
PLAN_ROOT = 'porting/persistence/schema-contract/'
XML_INPUT = 'porting/source-map/inherited/sirius-liquibase-documents.jsonl'
SOURCE_PREFIX = 'packages/sirius-web/backend/sirius-web-infrastructure/src/main/resources/'
NS = '{http://www.liquibase.org/xml/ns/dbchangelog}'
RELEASE_SHA = '85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1'
NESTED_SHA = 'b7ae31909aab4bac29109d1e740e84d7730ad056d506d10b43f721e4453e9413'
NESTED_JAR = 'BOOT-INF/lib/sirius-web-infrastructure-2026.9.3.jar'
PLAN_SHA = '3f2e5dff66e3ac4bf50e7f1020dbd87b72db6e289482067d406dd3c4919c37e8'
OBSERVATION_SHA = 'caf7b4aedeba3a38dca32629620c11df94fd3fd31dda2bb195492d9f7c06548c'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return sha256(data).hexdigest()


def git(*argv):
    return subprocess.check_output(['git', '-C', str(REPO), *argv], timeout=20)


def path_of(path):
    require(path.startswith(SOURCE_PREFIX), f'Unexpected source path: {path}')
    return path.removeprefix(SOURCE_PREFIX)


def donor_inputs():
    manifest = json.loads(git('show', DONOR + ':' + PLAN_ROOT + 'source-manifest.json'))
    require(manifest['source_base'] == SOURCE_BASE and manifest['candidate_revision'] == SOURCE,
            'F26 source provenance changed')
    inputs = {}
    for entry in manifest['inputs']:
        path = entry['path']
        oid = git('rev-parse', SOURCE_BASE + ':' + path).decode().strip()
        require(oid == entry['blob'], f'Git blob differs: {path}')
        data = git('cat-file', 'blob', oid)
        require(digest(data) == entry['sha256'], f'Source bytes differ: {path}')
        inputs[path] = data
    require(len(inputs) == 4, 'F26 source manifest must have four distinct inputs')
    plan_bytes = git('show', DONOR + ':' + PLAN_ROOT + 'candidate-plan.json')
    require(digest(plan_bytes) == PLAN_SHA, 'F26 plan bytes changed')
    plan = json.loads(plan_bytes)
    documents = {}
    for line in inputs[XML_INPUT].splitlines():
        row = json.loads(line)
        name = path_of(row['path'])
        data = row['xml'].encode('utf-8')
        require(row['source_commit'] == SOURCE and digest(data) == row['sha256'],
                f'Source XML provenance or bytes differ: {name}')
        require(name not in documents, f'Duplicate source XML: {name}')
        documents[name] = data
    expected = {path_of(row['path']): row['sha256'] for row in plan['document_order']}
    require(len(documents) == 29 and {k: digest(v) for k, v in documents.items()} == expected,
            'F26 plan and source XML inventory differ')
    return plan, documents


def read_member(archive, info, maximum):
    require(info.file_size <= maximum, f'Archive member exceeds cap: {info.filename}')
    with archive.open(info) as stream:
        data = stream.read(maximum + 1)
    require(len(data) == info.file_size, f'Wrong member length: {info.filename}')
    return data


def release_inputs(path):
    with path.open('rb') as stream:
        require(path.stat().st_size == 139_691_999, 'Wrong release JAR length')
        checksum = sha256()
        while chunk := stream.read(1 << 20):
            checksum.update(chunk)
        require(checksum.hexdigest() == RELEASE_SHA, 'Wrong release JAR hash')
        with ZipFile(stream) as outer:
            entries = outer.infolist()
            require(len(entries) <= 20_000, 'Oversized application archive inventory')
            members = [info for info in entries if info.filename == NESTED_JAR]
            require(len(members) == 1, 'Expected one embedded Sirius infrastructure JAR')
            require(not any(i.filename.startswith('BOOT-INF/classes/db/') and not i.is_dir()
                            for i in entries), 'Unexpected top-level database resources')
            child = read_member(outer, members[0], 100_000)
    require(digest(child) == NESTED_SHA, 'Wrong embedded Sirius infrastructure JAR hash')
    with ZipFile(BytesIO(child)) as archive:
        entries = archive.infolist()
        require(len(entries) <= 400, 'Oversized embedded archive inventory')
        docs = [i for i in entries if i.filename.startswith('db/') and not i.is_dir()]
        require(len(docs) == len({i.filename for i in docs}) == 29, 'Expected 29 distinct resources')
        require(sum(i.file_size for i in docs) == 50_655, 'Wrong total XML length')
        return {i.filename: read_member(archive, i, 64 * 1024) for i in docs}


def release_declarations(documents, master):
    order, includes, identities = [], [], []

    def visit(name):
        require(name in documents and name not in order, f'Invalid include target: {name}')
        order.append(name)
        root = ET.fromstring(documents[name])
        require(root.tag == NS + 'databaseChangeLog', f'Wrong XML root: {name}')
        for ordinal, element in enumerate(root):
            if element.tag == NS + 'include':
                relative = element.attrib.get('relativeToChangelogFile', 'false')
                require(relative in {'true', 'false'}, f'Invalid relative path control: {name}')
                target = element.attrib['file']
                if relative == 'true':
                    target = (PurePosixPath(name).parent / target).as_posix()
                includes.append((name, ordinal, target, element.attrib))
                visit(target)
            else:
                require(element.tag == NS + 'changeSet', f'Unexpected XML declaration: {name}')
                identities.append((name, element.attrib['id'], element.attrib['author']))

    visit(master)
    return order, includes, identities


def verify(plan, source, packaged, observed):
    require(set(source) == set(packaged), 'Source and embedded resource membership differs')
    for name, data in source.items():
        require(packaged[name] == data, f'Source and release XML bytes differ: {name}')
    order, includes, identities = release_declarations(packaged, path_of(plan['master']))
    require(order == [path_of(row['path']) for row in plan['document_order']],
            'Document traversal order differs')
    require(len(includes) == 28 and includes == [
        (path_of(r['from']), r['ordinal'], path_of(r['target']), r['attributes']) for r in plan['include_order']
    ], 'Include traversal or attributes differ')
    require(len(identities) == 20 and identities == [
        (path_of(name), identifier, author) for name, identifier, author in
        (r['identity'] for r in plan['changesets'])
    ], 'F26 changeset order or identities differ')
    require([(r['filename'], r['id'], r['author']) for r in observed] == identities,
            'Observed changeset order or identities differ')
    for i, row in enumerate(observed):
        require(row['position'] == i and row['orderexecuted'] == i + 1
                and row['exectype'] == 'EXECUTED' and row['liquibase'] == '4.33.0',
                f'Unexpected observed history fields at position {i}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--jar', required=True, type=Path, help='Official SysON 2026.9.3 application JAR')
    args = parser.parse_args()
    raw = (PACKET / 'observation.json').read_bytes()
    require(digest(raw) == OBSERVATION_SHA, 'Published history bytes changed')
    plan, documents = donor_inputs()
    verify(plan, documents, release_inputs(args.jar), json.loads(raw)['observed'])
    print('PASS: 29 exact XML resources, 28 ordered includes, 20 ordered EXECUTED changesets')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        sys.exit(f'F26 verification failed: {error}')

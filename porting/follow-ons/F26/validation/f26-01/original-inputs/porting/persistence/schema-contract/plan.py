#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Freeze source declaration order; never resolve Liquibase controls or execute SQL."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import subprocess
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
PIN = '3191e2b8990373afee3415f6b26f8c5039a28321'
BASE = 'ad814c232ee24109639c1d52239cbeec6f4a3e92'
RESOURCE = 'packages/sirius-web/backend/sirius-web-infrastructure/src/main/resources/'
MASTER = RESOURCE + 'db/db.changelog-master.xml'
NS = 'http://www.liquibase.org/xml/ns/dbchangelog'
DOCUMENTS = 'porting/source-map/inherited/sirius-liquibase-documents.jsonl'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def strict_json(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, 'duplicate JSON key')
            result[key] = value
        return result
    return json.loads(raw, object_pairs_hook=pairs)


def safe_path(path):
    p = PurePosixPath(path)
    require(isinstance(path, str) and not p.is_absolute() and '\\' not in path,
            'invalid resource path')
    require('..' not in p.parts and str(p) == path and path.startswith(RESOURCE),
            'resource path escapes or is not canonical')
    return path


def tag(element):
    require(element.tag.startswith('{' + NS + '}'), 'foreign XML namespace')
    return element.tag[len(NS) + 2:]


def tree(element):
    require(tag(element) not in {'sqlFile', 'loadData', 'loadUpdateData', 'customChange',
                               'customPrecondition', 'executeCommand'},
            'external payload or custom execution unsupported')
    require(not set(element.attrib).intersection({'file', 'path', 'class', 'className', 'resource'}),
            'external payload reference unsupported')
    return {'tag': tag(element), 'attributes': dict(sorted(element.attrib.items())),
            'text': element.text, 'children': [tree(child) for child in element]}


def documents(records):
    result = {}
    for record in records:
        path = safe_path(record['path'])
        require(path not in result, 'duplicate source document')
        require(record['source_commit'] == PIN, 'wrong candidate revision')
        raw = record['xml'].encode('utf-8')
        require(sha(raw) == record['sha256'], 'XML source hash mismatch')
        require(b'<!DOCTYPE' not in raw.upper() and b'<!ENTITY' not in raw.upper(),
                'DTD/entities not admitted')
        result[path] = raw
    return result


def build_plan(blobs, master=MASTER):
    """Depth-first declared includes only; all controls remain unevaluated data."""
    order, changes, includes, active, visited, identities = [], [], [], set(), set(), set()
    global_controls = []

    def visit(path):
        safe_path(path)
        require(path not in active, 'include cycle')
        require(path not in visited, 'repeated include unsupported')
        require(path in blobs, 'missing included document')
        active.add(path)
        visited.add(path)
        raw = blobs[path]
        require(b'<!DOCTYPE' not in raw.upper() and b'<!ENTITY' not in raw.upper(),
                'DTD/entities not admitted')
        require(b'${' not in raw, 'property substitution unsupported')
        root = ET.fromstring(raw)
        require(tag(root) == 'databaseChangeLog', 'unexpected XML root')
        require('logicalFilePath' not in root.attrib, 'logicalFilePath unsupported')
        order.append({'path': path, 'sha256': sha(raw), 'attributes': dict(root.attrib)})
        for ordinal, child in enumerate(root):
            kind = tag(child)
            if kind == 'include':
                require(set(child.attrib) <= {'file', 'relativeToChangelogFile'},
                        'conditional or unknown include attributes unsupported')
                relative = child.attrib.get('relativeToChangelogFile', 'false')
                require(relative in {'true', 'false'}, 'invalid include relative flag')
                name = child.attrib['file']
                target = str(PurePosixPath(path).parent / name) if relative == 'true' else RESOURCE + name
                safe_path(target)
                includes.append({'from': path, 'ordinal': ordinal,
                                 'attributes': dict(child.attrib), 'target': target})
                visit(target)
            elif kind == 'preConditions':
                global_controls.append({'path': path, 'ordinal': ordinal, 'declaration': tree(child)})
            elif kind == 'changeSet':
                attrs = dict(child.attrib)
                require('logicalFilePath' not in attrs, 'logicalFilePath unsupported')
                require(attrs.get('id') and attrs.get('author'), 'missing changeset identity')
                identity = (path, attrs['id'], attrs['author'])
                require(identity not in identities, 'duplicate changeset identity')
                identities.add(identity)
                declarations = [tree(e) for e in child]
                controls = [e for e in declarations if e['tag'] in {'preConditions', 'rollback', 'validCheckSum'}]
                changes.append({'sequence': len(changes), 'identity': list(identity),
                                'attributes': attrs, 'declarations': declarations,
                                'controls': controls, 'liquibase_checksum': 'NOT_COMPUTED',
                                'runtime_decision': 'UNEVALUATED'})
            else:
                raise ValueError('unsupported changelog child: ' + kind)
        active.remove(path)

    visit(master)
    require(set(blobs) == visited, 'unreachable supplied document')
    return {'schema': 1, 'candidate_revision': PIN, 'master': master,
            'document_order': order, 'include_order': includes, 'changesets': changes,
            'global_controls': global_controls, 'declaration_count': len(changes),
            'artifact_correspondence': 'UNPROVEN', 'applied_schema': 'UNKNOWN',
            'liquibase_history': 'UNAVAILABLE', 'database_execution': 'NOT_RUN',
            'execution_blockers': ['candidate-to-resolved-artifact correspondence',
                'exact Liquibase/JDBC/DBMS versions and extensions',
                'synthetic database state, history and checksum baseline',
                'runtime context/label/DBMS/precondition evaluation and rollback review']}


def verify_sources():
    manifest = strict_json((HERE / 'source-manifest.json').read_text())
    require(manifest['source_base'] == BASE and manifest['candidate_revision'] == PIN,
            'manifest pin mismatch')
    for source in manifest['inputs']:
        path = source['path']
        require(not PurePosixPath(path).is_absolute() and '..' not in PurePosixPath(path).parts,
                'manifest path traversal')
        raw = subprocess.check_output(['git', '-C', str(ROOT), 'show', BASE + ':' + path])
        blob = subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', BASE + ':' + path], text=True).strip()
        require(source['blob'] == blob and source['sha256'] == sha(raw), 'frozen Git input drift')
        require((ROOT / path).read_bytes() == raw, 'worktree input drift')
    return manifest


def candidate_plan():
    verify_sources()
    return build_plan(documents(strict_json(line) for line in (ROOT / DOCUMENTS).read_text().splitlines()))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write', action='store_true', help='regenerate declaration plan only; never execute a migration')
    args = parser.parse_args()
    result = candidate_plan()
    raw = (json.dumps(result, indent=2, ensure_ascii=False) + '\n').encode()
    output = HERE / 'candidate-plan.json'
    if args.write:
        output.write_bytes(raw)
    else:
        require(output.read_bytes() == raw, 'candidate plan drift')
    print(json.dumps({'status': 'PASS', 'documents': len(result['document_order']),
        'includes': len(result['include_order']), 'changesets': len(result['changesets']),
        'artifact_correspondence': result['artifact_correspondence'], 'database_execution': 'NOT_RUN'}))


if __name__ == '__main__':
    main()

"""Replay the edited-archive fixture checks without starting a service."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parent
SOURCE_SHA256 = 'fa6a3e6bf3b569dd8e285d5b18a1181cfea2ef177d594d36ba150130374a6f5a'
EDITED_SHA256 = '251d1ca323d375f980d9c996065af4faeb0fd62a07ba9c0a3c78d37689e6e65e'


def document(path, expected_hash):
    if hashlib.sha256(path.read_bytes()).hexdigest() != expected_hash:
        raise ValueError(f'Fixture bytes changed: {path.name}')
    with ZipFile(path) as payload:
        if payload.testzip() is not None or len(payload.namelist()) != 3:
            raise ValueError(f'Invalid member set: {path.name}')
        model, = [name for name in payload.namelist() if '/documents/' in name]
        sidecars = {name: payload.read(name) for name in payload.namelist() if name != model}
        return model, json.loads(payload.read(model)), sidecars


def definitions(value):
    if isinstance(value, dict):
        if value.get('eClass') == 'sysml:PartDefinition':
            yield value
        for child in value.values():
            yield from definitions(child)
    if isinstance(value, list):
        for child in value:
            yield from definitions(child)


def main():
    provenance = json.loads((ROOT / 'provenance.json').read_bytes())
    for name, expected in provenance['retained_sha256'].items():
        if Path(name).name != name or hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != expected:
            raise ValueError(f'Retained evidence differs from its provenance: {name}')
    edited = ROOT / 'browser-edited-rim.zip'
    name, before, sidecars = document(ROOT / 'source-tire.zip', SOURCE_SHA256)
    edited_name, after, edited_sidecars = document(edited, EDITED_SHA256)
    definition, = definitions(before)
    if name != edited_name or sidecars != edited_sidecars or definition['data']['declaredName'] != 'Tire':
        raise ValueError('Unexpected source or sidecar change')
    definition['data']['declaredName'] = 'Rim'
    if before != after:
        raise ValueError('The model changed beyond the selected rename')
    observed = subprocess.check_output([
        sys.executable, '-I', '-B', str(ROOT.parents[1] / 'archive-upload-compare.py'),
        str(edited), str(ROOT / 'native-reexport-rim.zip'), str(ROOT / 'capture.json'),
        '--fixture-sha256', EDITED_SHA256, '--definition-name', 'Rim',
    ])
    comparison = json.loads(observed)
    if comparison != json.loads((ROOT / 'comparison.json').read_bytes()):
        raise ValueError('Native re-export comparison changed')
    capture = json.loads((ROOT / 'capture.json').read_bytes())
    browser = json.loads((ROOT / 'native-browser.json').read_bytes())
    native = browser['reloaded']
    if (native['projectId'] != capture['output']['project']['id']
            or native['representationId'] != comparison['identity_mapping']['representation'][1]
            or browser['first'] != native or browser['pageErrors']
            or native['definition'] != 'Rim' or native['usage'] != 'wheel'
            or native['diagramPackageNodes'] != 1):
        raise ValueError('Native browser observation disagrees')
    draft = json.loads((ROOT / 'rust-draft-restart.json').read_bytes())
    if (draft['archive_sha256'] != EDITED_SHA256
            or draft['stop_signals'] != ['SIGKILL', 'SIGTERM', 'SIGTERM']
            or not draft['final_was_absent_when_first_process_stopped']
            or not draft['checkpoint_identity_unchanged'] or not draft['owned_scope_removed']):
        raise ValueError('Draft restart observation disagrees')
    browsers = draft['browsers']
    if (len(browsers) != 4 or len({row['revision_path'] for row in browsers}) != 1
            or [row['post_statuses'] for row in browsers] != [[303], [303], [], []]
            or any(row['definition'] != 'Rim' or row['events'] for row in browsers)):
        raise ValueError('Draft browser observation disagrees')
    print('PASS edited archive, draft restart and native re-export')


if __name__ == '__main__':
    main()

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
ALLOY_SHA256 = 'b67ea7dba420d285f919d356406f80fd6c4985afac4be896b5b72194d7845178'
ELITE_SHA256 = 'ec3307615e2ac7ba03b48ee8d50467ae480a4c7d1c800f61d9ff18c97472fe80'


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


def check_rename(edited, fixture_hash, definition_name):
    name, before, sidecars = document(ROOT / 'source-tire.zip', SOURCE_SHA256)
    edited_name, after, edited_sidecars = document(edited, fixture_hash)
    definition, = definitions(before)
    if name != edited_name or sidecars != edited_sidecars or definition['data']['declaredName'] != 'Tire':
        raise ValueError('Unexpected source or sidecar change')
    definition['data']['declaredName'] = definition_name
    if before != after:
        raise ValueError('The model changed beyond the selected rename')


def check_native(edited, fixture_hash, definition_name, reexport_name, suffix=''):
    capture_path = ROOT / f'capture{suffix}.json'
    observed = subprocess.check_output([
        sys.executable, '-I', '-B', str(ROOT.parents[1] / 'archive-upload-compare.py'),
        str(edited), str(ROOT / reexport_name), str(capture_path),
        '--fixture-sha256', fixture_hash, '--definition-name', definition_name,
    ])
    comparison = json.loads(observed)
    if comparison != json.loads((ROOT / f'comparison{suffix}.json').read_bytes()):
        raise ValueError('Native re-export comparison changed')
    capture = json.loads(capture_path.read_bytes())
    browser = json.loads((ROOT / f'native-browser{suffix}.json').read_bytes())
    native = browser['reloaded']
    if (native['projectId'] != capture['output']['project']['id']
            or native['representationId'] != comparison['identity_mapping']['representation'][1]
            or browser['first'] != native or browser['pageErrors']
            or native['definition'] != definition_name or native['usage'] != 'wheel'
            or native['diagramPackageNodes'] != 1):
        raise ValueError('Native browser observation disagrees')


def check_draft(receipt_name, fixture_hash, definition_name):
    draft = json.loads((ROOT / receipt_name).read_bytes())
    if (draft['archive_sha256'] != fixture_hash
            or draft['stop_signals'] != ['SIGKILL', 'SIGTERM', 'SIGTERM']
            or not draft['final_was_absent_when_first_process_stopped']
            or not draft['checkpoint_identity_unchanged'] or not draft['owned_scope_removed']):
        raise ValueError('Draft restart observation disagrees')
    browsers = draft['browsers']
    if (len(browsers) != 4 or len({row['revision_path'] for row in browsers}) != 1
            or [row['post_statuses'] for row in browsers] != [[303], [303], [], []]
            or any(row['definition'] != definition_name or row['events'] for row in browsers)):
        raise ValueError('Draft browser observation disagrees')


def main():
    provenance = json.loads((ROOT / 'provenance.json').read_bytes())
    for name, expected in provenance['retained_sha256'].items():
        if Path(name).name != name or hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != expected:
            raise ValueError(f'Retained evidence differs from its provenance: {name}')

    rim = ROOT / 'browser-edited-rim.zip'
    alloy = ROOT / 'browser-edited-alloy.zip'
    elite = ROOT / 'browser-edited-elite.zip'
    check_rename(rim, EDITED_SHA256, 'Rim')
    check_draft('rust-draft-restart.json', EDITED_SHA256, 'Rim')
    check_native(rim, EDITED_SHA256, 'Rim', 'native-reexport-rim.zip')
    check_rename(alloy, ALLOY_SHA256, 'Alloy Rim')
    check_draft('rust-draft-alloy.json', ALLOY_SHA256, 'Alloy Rim')
    check_rename(elite, ELITE_SHA256, 'Jante Élite')
    check_draft('rust-draft-elite.json', ELITE_SHA256, 'Jante Élite')
    check_native(elite, ELITE_SHA256, 'Jante Élite', 'native-reexport-elite.zip', '-elite')
    print('PASS edited archives, draft restart and native re-exports')


if __name__ == '__main__':
    main()

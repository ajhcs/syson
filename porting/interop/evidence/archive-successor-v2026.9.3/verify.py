"""Replay the browser lineage and native SysON evidence without starting a service."""
import hashlib
import json
from pathlib import Path
import runpy
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
PARENT = ROOT.parent / 'archive-edited-reopen-v2026.9.3'
BASE = runpy.run_path(str(PARENT / 'verify.py'))
FIRST = 'Jante Élite'
SECOND = 'Jante Sport'
SUCCESSOR_SHA256 = '1b5b5421ff6a7006e38a7c1ea585b4526505e428daa82ca148bc8af98797c5ec'


def read(name):
    return json.loads((ROOT / name).read_bytes())


def require(condition, message):
    if not condition:
        raise ValueError(message)


def verify_browser(browser, first_hash, second_hash):
    expected_names = {'original': 'Tire', 'first': FIRST, 'second': SECOND}
    expected_hashes = {'original': BASE['SOURCE_SHA256'], 'first': first_hash, 'second': second_hash}
    require(browser['stop_signals'] == ['SIGTERM', 'SIGKILL', 'SIGTERM', 'SIGTERM']
            and browser['owned_scope_removed'] and browser['parent_files_retained']
            and browser['successor_authority_retained'] and browser['draft_retained']
            and browser['unpublished_at_stop'], 'Restart and file custody disagree')
    binding = browser['parent_binding']
    require(binding['parent_zip_sha256'] == first_hash
            and binding['parent_checkpoint_sha256'] == binding['observed_parent_checkpoint_sha256']
            and binding['ticket_sha256'] == binding['observed_ticket_sha256'],
            'Successor authority does not bind the captured parent')
    phases = browser['phases']
    require([row['phase'] for row in phases] ==
            ['first', 'stage-second', 'resume-second', 'desktop', 'mobile'], 'Browser phases changed')
    require([row['posts'] for row in phases] == [[303, 303], [303], [303], [], []],
            'Unexpected browser write')
    require(all(row['routes'] == phases[0]['routes'] and not row['events'] for row in phases),
            'Browser route or console observation changed')
    require(all(phases[index]['parent_link'] == phases[0]['routes']['first'] for index in (1, 3, 4)),
            'Parent navigation differs')
    require(phases[0]['names'] == {'original': 'Tire', 'firstDraft': FIRST}
            and phases[0]['downloads'] == {'first': first_hash}, 'First revision differs')
    require(phases[1]['names'] == {'first': FIRST, 'secondDraft': SECOND}
            and phases[1]['downloads'] == {}, 'Successor draft differs')
    require(phases[2]['names'] == {'second': SECOND}
            and phases[2]['downloads'] == {'second': second_hash}, 'Recovered successor differs')
    for reopened in phases[3:]:
        require(reopened['names'] == expected_names and reopened['downloads'] == expected_hashes,
                'Routes and downloads disagree after restart')


def main():
    retained = read('provenance.json')['retained_sha256']
    require(set(retained) == {
        'browser.json', 'browser-successor.zip',
        'capture.json', 'comparison.json', 'native-browser.json', 'native-reexport.zip',
    }, 'Unexpected evidence set')
    for name, expected in retained.items():
        observed = hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
        require(observed == expected, f'Evidence changed for {name}')
    first_hash = BASE['ELITE_SHA256']
    second_hash = SUCCESSOR_SHA256
    first = PARENT / 'browser-edited-elite.zip'
    second = ROOT / 'browser-successor.zip'
    BASE['check_rename'](first, first_hash, FIRST)
    BASE['check_rename'](second, second_hash, SECOND)
    verify_browser(read('browser.json'), first_hash, second_hash)

    observed = subprocess.check_output([
        sys.executable, '-I', '-B', str(ROOT.parents[1] / 'archive-upload-compare.py'),
        str(second), str(ROOT / 'native-reexport.zip'), str(ROOT / 'capture.json'),
        '--fixture-sha256', second_hash, '--definition-name', SECOND,
    ])
    comparison = json.loads(observed)
    require(comparison == read('comparison.json')
            and comparison['result'] == 'equal_after_identity_mapping', 'Native archive differs')
    capture = read('capture.json')
    refused = capture['missing_manifest_control']
    require(refused['projects_after'] == 0 and refused['graphql_response']['data'] is None
            and refused['graphql_response']['errors'][0]['path'] == ['uploadProject'],
            'Incomplete archive refusal differs')
    browser = read('native-browser.json')
    native = browser['reloaded']
    require(native == browser['first'] and not browser['pageErrors']
            and native['projectId'] == capture['output']['project']['id']
            and native['representationId'] == comparison['identity_mapping']['representation'][1]
            and native['definition'] == SECOND and native['usage'] == 'wheel'
            and native['diagramPackageNodes'] == 1, 'Native browser observation disagrees')
    print('PASS archive lineage, draft recovery, and native re-export')


if __name__ == '__main__':
    main()

"""Verify the retained browser and archive custody observations."""
from hashlib import sha256
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PREVIOUS = ROOT.parent
EXPECTED = {
    'stale': ([(303, 303), (303,), (303, 409), ()], 4, [], [2, 3]),
    'before-failure': ([(303, 303), (303,), (500, 500), (303,)], 4, [2], [3]),
    'uncertain': ([(303, 303), (303,), (503,), (503,), (303,), ()], 6, [2], [4, 5]),
}


def require(value, reason):
    if not value:
        raise ValueError(reason)


def verify(evidence, current, parent):
    require(set(evidence['cases']) == set(EXPECTED), 'Recovery case inventory differs')
    for case, (posts, count, forced, downloads) in EXPECTED.items():
        row = evidence['cases'][case]
        stages = row['phases']
        require([tuple(stage['posts']) for stage in stages] == posts,
                f'Browser response sequence differs in {case}')
        signals = ['SIGKILL' if index in forced else 'SIGTERM' for index in range(count)]
        require(row['services'] == signals and row['scope_removed'],
                f'Runtime teardown differs in {case}')
        require(row['all_initial_files_retained']
                and row['new_artifacts'] == ['.archive-successor.confirmed.json', 'archive-successor.zip'],
                f'File custody differs in {case}')
        require(row['confirmed_archive_sha256'] == row['archive_sha256'] == current
                and row['parent_sha256'] == parent
                and row['observed_checkpoint_sha256'] == row['confirmed_checkpoint_sha256'],
                f'Confirmation differs from the exact checkpoint or archive in {case}')
        denied = row['denied_resources']
        require(len(denied) == (3 if case == 'uncertain' else 1)
                and all(len(result) == 4 and set(result.values()) == {404} for result in denied),
                f'Unpublished archive endpoints became accessible in {case}')
        require(row['candidate_retained'] is (True if case == 'uncertain' else None),
                f'Unexpected candidate file identity in {case}')
        require(stages[0]['downloads'] == {'first': parent}, f'Wrong parent archive in {case}')
        for index, stage in enumerate(stages[1:], 1):
            expected = {'second': current} if index in downloads else {}
            require(stage['downloads'] == expected, f'Unexpected download in {case}')
    require(evidence['cases']['uncertain']['phases'][2]['viewport'] == {'width': 390, 'height': 844},
            'Mobile viewport observation differs')


def main():
    raw = (ROOT / 'browser.json').read_bytes()
    provenance = json.loads((ROOT / 'provenance.json').read_bytes())
    require(sha256(raw).hexdigest() == provenance['browser_sha256'], 'Evidence receipt changed')
    current = PREVIOUS / 'archive-successor-v2026.9.3' / 'browser-successor.zip'
    parent = PREVIOUS / 'archive-edited-reopen-v2026.9.3' / 'browser-edited-elite.zip'
    verify(json.loads(raw), sha256(current.read_bytes()).hexdigest(), sha256(parent.read_bytes()).hexdigest())
    print('PASS stale tab, retryable save, and explicit archive confirmation')


if __name__ == '__main__':
    main()

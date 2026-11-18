"""Verify the retained archive recovery touch and keyboard observations."""
from hashlib import sha256
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CASES = {
    '320-uncertain': ('uncertain', 320, 12),
    '390-before-failure': ('before-failure', 390, 11),
    '390-stale': ('stale', 390, 12),
    '0-uncertain': ('uncertain', 1440, 12),
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def check_action(action, width):
    require(action['width'] >= 44 and action['height'] >= 44, 'Control target is undersized')
    if width == 1440:
        focus = action['focus']
        require(action['mode'] == 'keyboard' and focus['matched'] and focus['width'] >= 2
                and focus['style'] != 'none' and focus['offset'] > 0, 'Visible keyboard focus is missing')
    else:
        device = action['device']
        require(action['mode'] == 'touch' and action['event'] == {'type':'touch', 'trusted':True}
                and device['coarse'] and device['width'] == width and device['dpr'] == 3
                and device['touchPoints'] > 0, 'Trusted touch at the expected viewport is missing')


def verify(runs, recovery, archive):
    require(set(runs) == set(CASES), 'Browser run inventory differs')
    for name, (case, width, count) in CASES.items():
        run = runs[name]
        require(run['all_inputs_retained'] and run['scope_removed'] and run['archive_sha256'] == archive,
                f'Archive custody differs in {name}')
        phases = run['phases']
        identity = lambda row: (row['phase'],row['posts'])
        require(list(map(identity,phases)) == list(map(identity,recovery['cases'][case]['phases'])),
                f'Browser response sequence differs in {name}')
        actions = [action for row in phases for action in row['actions']]
        require(len(actions) == count, f'Action count differs in {name}')
        for action in actions:
            check_action(action, width)
        errors = [status['role'] for row in phases if any(code >= 400 for code in row['posts'])
                  for status in row['statuses']]
        require('alert' in errors, f'Error alert semantics missing in {name}')

    resize = runs['320-uncertain']['phases'][2]['resize']
    require([row['viewport'] for row in resize] ==
            [{'width':568,'height':320}, {'width':320,'height':568}], 'Reflow observation differs')
    for row in resize:
        require(row['width'] == row['viewport']['width'] == row['scrollWidth'] and row['dpr'] == 3
                and row['target']['width'] >= 44 and row['target']['height'] >= 44,
                'Pending confirmation target does not fit after resize')


def main():
    raw = (ROOT / 'browser.json').read_bytes()
    provenance = json.loads((ROOT / 'provenance.json').read_bytes())
    require(sha256(raw).hexdigest() == provenance['browser_sha256'], 'Browser evidence changed')
    recovery = json.loads((ROOT.parent / 'archive-save-recovery/browser.json').read_bytes())
    archive = ROOT.parent / 'archive-successor-v2026.9.3/browser-successor.zip'
    verify(json.loads(raw)['runs'], recovery, sha256(archive.read_bytes()).hexdigest())
    print('PASS archive touch targets, trusted input, keyboard focus, and viewport reflow')


if __name__ == '__main__':
    main()

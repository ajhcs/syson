"""Replay the width-only layout resize and native archive transfer."""
from hashlib import sha256
import json
from pathlib import Path
import runpy

ROOT = Path(__file__).resolve().parent
LAYOUT = runpy.run_path(str(ROOT.parent / 'archive-layout' / 'verify.py'))
HIERARCHY = LAYOUT['HIERARCHY']
load = LAYOUT['load']
START = {'width':200,'height':101}
RESULT = {'width':210,'height':101}
POSITION = {'x':9,'y':46}


def require(value, message):
    if not value:
        raise ValueError(message)


def check_archive():
    model, manifest, before = load(ROOT / 'rust-parent.zip')
    edited_model, edited_manifest, after = load(ROOT / 'rust-resized.zip')
    require(model == edited_model and manifest == edited_manifest, 'Protected archive members changed')
    tree = HIERARCHY['diagram_nodes'](json.loads(model), before)
    HIERARCHY['diagram_nodes'](json.loads(model), after)
    node, = [row['id'] for row in tree if row['text'] == 'Drivetrain']
    record = before['representation']['layoutData']['nodeLayoutData'][node]
    require(record['size'] == START and record['position'] == POSITION and not record['resizedByUser'],
            'Unexpected initial geometry')
    record['size'] = RESULT
    record['resizedByUser'] = True
    require(before == after, 'Resize changed other layout data')
    return node


def reference():
    model, manifest, before = load(ROOT / 'native-before-resize.zip')
    new_model, new_manifest, after = load(ROOT / 'native-after-reload.zip')
    require(model == new_model and manifest == new_manifest, 'Native resize changed model or manifest')
    browser = json.loads((ROOT / 'native-resize.json').read_bytes())
    node = browser['node']
    original = before['representation']['layoutData']['nodeLayoutData']
    resized = after['representation']['layoutData']['nodeLayoutData']
    require(original[node]['size'] == START and original[node]['position'] == POSITION
            and not original[node]['resizedByUser'], 'Native reference baseline differs')
    original[node]['size'] = {'width':213,'height':101}
    original[node]['resizedByUser'] = True
    handles = {f'handleNode_{node}_{side}' for side in ('left','right','top','bottom')}
    require(handles == set(resized) - set(original), 'Native selection handle inventory differs')
    for handle in handles:
        require(resized[handle]['size'] == {'width':1,'height':1}, 'Native handle dimensions differ')
        original[handle] = resized[handle]
    require(before == after and browser['after'] == browser['reloaded'] and not browser['pageErrors'],
            'Unexpected native reference resize or reload difference')


def check_browser(browser, node, digest):
    require(browser['scope_removed'] and browser['retained_earlier_files']
            and browser['signals'] == ['SIGTERM','SIGKILL','SIGTERM','SIGTERM']
            and browser['final_sha256'] == digest, 'Unexpected restart or archive custody')
    phases = browser['phases']
    require([row['phase'] for row in phases] == ['first','draft','recover','reopen','reopen-mobile'],
            'Unexpected browser lifecycle')
    move = phases[1]['move']
    gesture = 'touch' if browser['kind']=='touch' else 'mouse'
    require(move['nudge'] == [201,101] and move['arrow'] == [201,102]
            and move['afterCancel'] == [200,101] and move['gesture'] == gesture
            and move['dragged'] == [210,101] and move['postForms'] == ['op=resize&width=210&height=101'],
            'Resize preview, gesture, or staged request differs')
    for phase in phases[2:]:
        for observation in phase['browserWork']:
            row, = [item for item in observation['hierarchy'] if item['id']==node]
            require([row['x'],row['y'],row['width'],row['height']] == [9,46,210,101],
                    'Reopened diagram differs from saved geometry')
    require(all(row['archive'] == digest for row in phases[-2:]), 'Browser downloads differ')


def reimport(pins):
    left, source, source_uri = HIERARCHY['snapshot'](ROOT / 'rust-resized.zip')
    right, target, target_uri = HIERARCHY['snapshot'](ROOT / 'native-reexport.zip')
    require(source.keys() == target.keys() and not set(source.values()) & set(target.values()),
            'Native generated identity domains differ')
    ids = {source[key]:target[key] for key in source}
    ids[source_uri] = target_uri
    require(tuple(HIERARCHY['normalize'](item,ids) for item in left)==right,
            'Native normalized manifest, model or diagram differs')
    capture = json.loads((ROOT / 'native-capture.json').read_bytes())
    require(capture['release']=='v2026.9.3' and capture['input']['sha256']==pins['rust-resized.zip']
            and capture['output']['sha256']==pins['native-reexport.zip']
            and capture['missing_manifest_control']['projects_after']==0, 'Native transfer pins differ')
    browser = json.loads((ROOT / 'native-browser.json').read_bytes())
    view = browser['reloaded']
    geometry = view['geometry']
    require(view==browser['first'] and not browser['pageErrors']
            and view['projectId']==target['project'] and view['representationId']==target['representation']
            and view['definition']=='DriveShaft V4' and len(view['rendered'])==9
            and all(abs(geometry['size'][key]-value)<0.001 for key,value in RESULT.items())
            and all(abs(geometry['offset'][key]-value)<0.001 for key,value in POSITION.items()),
            'Resized node did not reopen with the same native dimensions and position')


def main():
    pins = json.loads((ROOT / 'provenance.json').read_bytes())
    for name, digest in pins.items():
        require(Path(name).name == name and sha256((ROOT / name).read_bytes()).hexdigest()==digest,
                f'Evidence changed for {name}')
    node = check_archive()
    browsers = json.loads((ROOT / 'browser.json').read_bytes())
    require([row['kind'] for row in browsers]==['desktop','touch']
            and browsers[0]['subject']==browsers[1]['subject'], 'Browser input campaigns differ')
    for browser in browsers:
        check_browser(browser,node,pins['rust-resized.zip'])
    reference()
    reimport(pins)
    print('PASS desktop and touch resize, forced-stop recovery, and native SysON archive exchange')


if __name__ == '__main__':
    main()

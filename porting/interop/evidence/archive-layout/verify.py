"""Replay the exact persisted layout change and native transfer."""
from hashlib import sha256
import json
from pathlib import Path
import runpy
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parent
PRIOR = ROOT.parent / 'archive-hierarchy'
HIERARCHY = runpy.run_path(str(PRIOR / 'verify.py'))
ORIGINAL = {'x':9, 'y':46}
MOVED = {'x':19, 'y':50}


def require(value, message):
    if not value:
        raise ValueError(message)


def load(path):
    with ZipFile(path) as archive:
        entries = archive.namelist()
        require(archive.testzip() is None and len(set(entries)) == len(entries) == 3, 'Invalid archive')
        document, = [entry for entry in entries if '/documents/' in entry]
        representation, = [entry for entry in entries if '/representations/' in entry]
        manifest, = [entry for entry in entries if entry.endswith('/manifest.json')]
        return archive.read(document), archive.read(manifest), json.loads(archive.read(representation))


def layout_only(parent, child):
    original, manifest, before = load(parent)
    edited, edited_manifest, after = load(child)
    require(original == edited and manifest == edited_manifest, 'Semantic model or manifest bytes changed')
    tree = HIERARCHY['diagram_nodes'](json.loads(original), before)
    HIERARCHY['diagram_nodes'](json.loads(edited), after)
    selected, = [row for row in tree if row['text'] == 'Drivetrain']
    node = selected['id']
    record = before['representation']['layoutData']['nodeLayoutData'][node]
    require(record['position'] == ORIGINAL and not record['movedByUser'], 'Unexpected base position')
    record['position'] = MOVED
    record['movedByUser'] = True
    require(after == before, 'Layout change exceeds the selected node move')
    return node


def reference_move():
    model, manifest, before = load(ROOT / 'native-before-move.zip')
    edited_model, edited_manifest, after = load(ROOT / 'native-after-reload.zip')
    require(model == edited_model and manifest == edited_manifest, 'Native move changed model or manifest')
    browser = json.loads((ROOT / 'native-move.json').read_bytes())
    node = browser['node']
    original_layout = before['representation']['layoutData']['nodeLayoutData']
    edited_layout = after['representation']['layoutData']['nodeLayoutData']
    record = original_layout[node]
    position = edited_layout[node]['position']
    require(record['position'] == ORIGINAL and not record['movedByUser']
            and position['x'] > ORIGINAL['x'] and position['y'] == ORIGINAL['y'],
            'Native move did not use the expected local frame')
    record['position'] = position
    record['movedByUser'] = True
    handles = set(edited_layout) - set(original_layout)
    require(handles == {f'handleNode_{node}_{side}' for side in ('left','right','top','bottom')},
            'Native selection handle inventory differs')
    for key in handles:
        require(edited_layout[key]['size'] == {'width':1,'height':1}, 'Native handle size differs')
        original_layout[key] = edited_layout[key]
    require(before == after, 'Unexpected native movement difference')
    require(not browser['pageErrors'] and browser['after'] == browser['reloaded']
            and abs(browser['reloaded']['x'] - position['x']) < 0.001
            and abs(browser['reloaded']['y'] - position['y']) < 0.001, 'Native movement did not survive reload')


def browser_move(receipt, digest, node):
    require(receipt['scope_removed'] and receipt['signals'] == ['SIGTERM','SIGKILL','SIGTERM','SIGTERM']
            and receipt['retained_earlier_files'] and receipt['final_sha256'] == digest,
            'Browser scope or forced-stop lifecycle differs')
    phases = receipt['phases']
    require([row['phase'] for row in phases] == ['first','draft','recover','reopen','reopen-mobile'],
            'Browser lifecycle differs')
    moved = [MOVED['x'],MOVED['y']]
    move = phases[1]['move']
    require(move['nudge'][0] > ORIGINAL['x'] and move['nudge'][1] == ORIGINAL['y']
            and move['afterCancel'] == [ORIGINAL['x'],ORIGINAL['y']]
            and move['dragged'] == moved and move['postForms'] == ['op=move&x=19&y=50'],
            'Keyboard cancel or exact drag differs')
    for phase in phases[2:]:
        for observation in phase['browserWork']:
            selected, = [row for row in observation['hierarchy'] if row['id'] == node]
            require([selected['x'],selected['y']] == moved, 'Browser did not reopen the saved location')
    require(all(row['archive'] == digest for row in phases[-2:]), 'Browser downloads diverged')


def reimport(pins):
    before, source_ids, source_uri = HIERARCHY['snapshot'](ROOT / 'rust-moved.zip')
    after, target_ids, target_uri = HIERARCHY['snapshot'](ROOT / 'native-rust-reexport.zip')
    require(source_ids.keys() == target_ids.keys() and not set(source_ids.values()) & set(target_ids.values()),
            'Native identity correspondence differs')
    mapping = {source_ids[key]:target_ids[key] for key in source_ids}
    mapping[source_uri] = target_uri
    require(tuple(HIERARCHY['normalize'](item,mapping) for item in before) == after,
            'Native normalized manifest, model or diagram changed')
    capture = json.loads((ROOT / 'native-rust-capture.json').read_bytes())
    require(capture['release'] == 'v2026.9.3' and capture['input']['sha256'] == pins['rust-moved.zip']
            and capture['output']['sha256'] == pins['native-rust-reexport.zip']
            and capture['missing_manifest_control']['projects_after'] == 0, 'Native upload evidence differs')
    browser = json.loads((ROOT / 'native-rust-browser.json').read_bytes())
    view = browser['reloaded']
    require(view == browser['first'] and not browser['pageErrors'] and view['definition'] == 'DriveShaft V4'
            and view['projectId'] == target_ids['project'] and view['representationId'] == target_ids['representation']
            and len(view['rendered']) == 9 and view['usage'] == 'driveShaftInstance'
            and all(abs(view['geometry']['offset'][axis] - MOVED[axis]) < 0.001 for axis in ('x','y')),
            'Native browser did not reopen the selected move')


def main():
    pins = json.loads((ROOT / 'provenance.json').read_bytes())
    for name, digest in pins.items():
        require(Path(name).name == name and sha256((ROOT / name).read_bytes()).hexdigest() == digest,
                f'Evidence changed for {name}')
    node = layout_only(ROOT / 'rust-parent.zip', ROOT / 'rust-moved.zip')
    browser = json.loads((ROOT / 'rust-browser.json').read_bytes())
    browser_move(browser, pins['rust-moved.zip'], node)
    reference_move()
    reimport(pins)
    print('PASS selected layout move, forced-stop recovery and full native SysON round trip')


if __name__ == '__main__':
    main()

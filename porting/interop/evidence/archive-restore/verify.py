"""Replay immutable archive restore and compare live SysON history evidence."""
from hashlib import sha256
import json
from pathlib import Path
import runpy
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parent
RESIZE = runpy.run_path(str(ROOT.parent / 'archive-resize' / 'verify.py'))
LAYOUT = RESIZE['LAYOUT']
HIERARCHY = RESIZE['HIERARCHY']
load = LAYOUT['load']


def require(value, message):
    if not value:
        raise ValueError(message)


def members(name):
    with ZipFile(ROOT / name) as archive:
        keys = archive.namelist()
        require(archive.testzip() is None and len(keys) == len(set(keys)) == 3, 'Invalid archive')
        return {key:archive.read(key) for key in keys}


def check_native_history():
    history = json.loads((ROOT / 'native-history.json').read_bytes())
    require(history['undoRequests'] == ['undo','layoutDiagram']
            and history['redoRequests'] == ['redo','layoutDiagram']
            and history['postReloadRequests'] == [] and history['rawRedoLayoutRetainedOnReload'],
            'Native shortcut observation differs')
    node = history['selected']
    original = None
    for stage, width, flag in [('before',200,False),('resized',213,True),
                               ('undone',200,False),('reloaded',213,True)]:
        model, manifest, view = load(ROOT / f'native-history-{stage}.zip')
        positions = view['representation']['layoutData']['nodeLayoutData']
        target = positions[node]
        require(target['size'] == {'width':width,'height':101} and target['position'] == {'x':9,'y':46}
                and target['resizedByUser'] == flag, 'Native history stored geometry differs')
        target['size']['width'] = 200
        target['resizedByUser'] = False
        for side in ('left','right','top','bottom'):
            positions.pop(f'handleNode_{node}_{side}',None)
        if original is None:
            original = model,manifest,view
        require((model,manifest,view) == original, 'Unexpected non-history change in native archive')


def check_browser(browser, pins):
    require(browser['scope_removed'] and browser['retained_earlier_files'] and browser['fresh_inode'],
            'Browser archive custody differs')
    require(browser['signals'] == ['SIGTERM','SIGKILL','SIGTERM','SIGKILL','SIGTERM','SIGTERM'],
            'Separate resize and restore process stops differ')
    phases = browser['phases']
    expected = ['first','draft','recover','restore','recover-restore','reopen','reopen-mobile']
    require([row['phase'] for row in phases] == expected, 'Unexpected browser lifecycle')
    require(phases[3]['restoreCalls'] == ['op=restore'], 'Unexpected restore request')
    node = browser['layout_edit']['node']
    for phase in phases[4:]:
        selected, = [row for row in phase['browserWork'][-1]['hierarchy'] if row['id']==node]
        require([selected[key] for key in ('x','y','width','height')]==[9,46,200,101],
                'Browser did not reopen the restored dimensions')
    original = sha256((ROOT.parent / 'archive-hierarchy/native-source.zip').read_bytes()).hexdigest()
    history = {'original':original,'previous':pins['rust-previous.zip'],'resized':pins['rust-resized.zip']}
    for phase in phases[3:]:
        require(phase['history']['parent'].endswith('/archive-successor')
                and phase['history']['previous'].endswith('/archive-rim'), 'Browser history links differ')
    for phase in phases[-2:]:
        require(phase['priorDownloads'] == history and phase['archive']==pins['rust-restored.zip'],
                'One or more historical archive downloads differ')


def check_native_reimport(pins):
    left, source, uri = HIERARCHY['snapshot'](ROOT / 'rust-restored.zip')
    right, target, target_uri = HIERARCHY['snapshot'](ROOT / 'native-reexport.zip')
    require(source.keys() == target.keys() and not set(source.values()) & set(target.values()),
            'Native identity domains differ')
    ids = {source[key]:target[key] for key in source}
    ids[uri] = target_uri
    require(tuple(HIERARCHY['normalize'](item,ids) for item in left)==right,
            'Native normalized restored archive differs')
    capture = json.loads((ROOT / 'native-capture.json').read_bytes())
    require(capture['release']=='v2026.9.3' and capture['input']['sha256']==pins['rust-restored.zip']
            and capture['output']['sha256']==pins['native-reexport.zip']
            and capture['missing_manifest_control']['projects_after']==0, 'Native transfer pins differ')
    browser = json.loads((ROOT / 'native-browser.json').read_bytes())
    view = browser['reloaded']
    geometry = view['geometry']
    require(view==browser['first'] and not browser['pageErrors']
            and view['projectId']==target['project'] and view['representationId']==target['representation']
            and view['definition']=='DriveShaft V4' and len(view['rendered'])==9
            and abs(geometry['size']['width']-200)<0.001 and abs(geometry['size']['height']-101)<0.001
            and abs(geometry['offset']['x']-9)<0.001 and abs(geometry['offset']['y']-46)<0.001,
            'Native browser did not reopen the restored geometry')


def main():
    pins = json.loads((ROOT / 'provenance.json').read_bytes())
    for name, digest in pins.items():
        require(Path(name).name == name and sha256((ROOT / name).read_bytes()).hexdigest()==digest,
                f'Evidence changed for {name}')
    require(pins['rust-previous.zip']==sha256((ROOT.parent/'archive-resize/rust-parent.zip').read_bytes()).hexdigest()
            and pins['rust-resized.zip']==sha256((ROOT.parent/'archive-resize/rust-resized.zip').read_bytes()).hexdigest(),
            'Earlier revisions do not match the qualified resize archives')
    require(pins['rust-restored.zip']!=pins['rust-previous.zip']
            and members('rust-restored.zip')==members('rust-previous.zip'),
            'Restore did not preserve all prior member bytes in a distinct ZIP')
    check_browser(json.loads((ROOT / 'browser.json').read_bytes()),pins)
    check_native_history()
    check_native_reimport(pins)
    print('PASS immutable restore, native Undo/Redo observations, and full SysON round trip')


if __name__ == '__main__':
    main()

"""Replay the native hierarchy and the Rust archive preservation checks."""
from hashlib import sha256
import json
from pathlib import Path
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parent


def require(value, message):
    if not value:
        raise ValueError(message)


def descendants(value):
    if isinstance(value, dict):
        if 'eClass' in value:
            yield value
        for child in value.values():
            yield from descendants(child)
    elif isinstance(value, list):
        for child in value:
            yield from descendants(child)


def open_archive(name):
    with ZipFile(ROOT / name) as archive:
        names = archive.namelist()
        require(archive.testzip() is None and len(names) == len(set(names)) == 3, 'Archive members differ')
        document, = [entry for entry in names if '/documents/' in entry]
        diagram, = [entry for entry in names if '/representations/' in entry]
        model = json.loads(archive.read(document))
        view = json.loads(archive.read(diagram))
        return model, view, {entry:archive.read(entry) for entry in names if entry != document}


def diagram_nodes(model, view):
    objects = {row['id']:row for row in descendants(model)}
    require(len(objects) == 431, 'Expected 431 model objects')
    view_usage = objects[view['targetObjectId']]
    require(view_usage['eClass'] == 'sysml:ViewUsage', 'Representation target is not a ViewUsage')
    exposes = [row['data']['importedMembership'] for row in view_usage['data']['ownedRelationship']
               if row['eClass'] == 'sysml:MembershipExpose']
    layout = view['representation']['layoutData']
    rows = []

    def walk(node, parent):
        node_id = node['id']
        label_id = node['insideLabel']['id']
        box = layout['nodeLayoutData'][node_id]
        require(node['type'] == 'customnode:sysmlpackage'
                and list(node).index('type') < list(node).index('style'), 'Native node wire order differs')
        rows.append(dict(id=node_id,parent=parent,emf=node['targetObjectId'],label=label_id,
                         text=node['insideLabel']['text'],x=box['position']['x'],y=box['position']['y'],
                         width=box['size']['width'],height=box['size']['height']))
        for child in node['childNodes']:
            walk(child, node_id)

    root, = view['representation']['nodes']
    require(len(root['childNodes']) == 8 and not view['representation']['edges'], 'Native tree differs')
    walk(root, '')
    require(len(rows) == len({row['id'] for row in rows}) == 9
            and {row['id'] for row in rows} == set(layout['nodeLayoutData'])
            and {row['label'] for row in rows} == set(layout['labelLayoutData']),
            'Node or label layout has missing or orphan entries')
    targets = set()
    for membership in exposes:
        owner = objects[membership]
        require(owner['eClass'] == 'sysml:OwningMembership', 'Exposure points outside ownership')
        item, = owner['data']['ownedRelatedElement']
        require(item['eClass'] == 'sysml:Package', 'Exposure target is not a package')
        targets.add(item['id'])
    require(len(exposes) == 9 and targets == {row['emf'] for row in rows},
            'Diagram nodes do not match the exact semantic exposures')
    return rows


def renamed(source_name, change_name, old, new):
    before, view, source_sidecars = open_archive(source_name)
    after, new_view, edited_sidecars = open_archive(change_name)
    require(view == new_view and source_sidecars == edited_sidecars, 'Diagram or manifest bytes changed')
    selected, = [row for row in descendants(before) if row['eClass'] == 'sysml:PartDefinition'
                 and row['data'].get('declaredName') == old]
    selected['data']['declaredName'] = new
    require(before == after, 'Edit changed semantic model beyond selected name')
    return selected['data']['elementId']


def check_browser(expected, receipt):
    require(receipt['owned_scope_removed'] and receipt['archived_nodes'] == expected, 'Browser hierarchy differs')
    run, = receipt['runs']
    require(run['retained_earlier_files'] and run['service_signals'] == ['SIGTERM','SIGKILL','SIGTERM','SIGTERM'],
            'Forced-stop recovery differs')
    phases = run['phases']
    require([row['phase'] for row in phases] == ['first','draft','recover','reopen','reopen-mobile'],
            'Browser lifecycle differs')
    for phase in phases:
        require(phase['browserWork'] and all(view['hierarchy'] == expected for view in phase['browserWork']),
                'Rendered hierarchy differs')
    for row in phases[-2:]:
        require(row['archive'] == receipt['final_sha256'] and row['outline']['open']
                and row['outline']['minimumLabelHeight'] >= 16
                and row['outline']['labels'] == [node['text'] for node in expected],
                'Archive download or package outline differs')


def snapshot(name):
    model, view, sidecars = open_archive(name)
    nodes = diagram_nodes(model, view)
    manifest, = (json.loads(raw) for path, raw in sidecars.items() if path.endswith('/manifest.json'))
    document, = manifest['documentIdsToName']
    representation, = manifest['representations']
    metadata = manifest['representations'][representation]
    members = {row['id']:row['data']['elementId'] for row in descendants(model)}
    ids = {'project':view['projectId'], 'document':document, 'representation':representation}
    ids.update({f'object:{stable}':physical for physical, stable in members.items()})
    for row in nodes:
        stable = members[row['emf']]
        ids[f'node:{stable}'] = row['id']
        ids[f'label:{stable}'] = row['label']
    require(len(ids) == len(set(ids.values())) == 452, 'Archive identity roles differ')
    require(sum(row['eClass']=='sysml:FeatureTyping' for row in descendants(model)) == 81,
            'Typing relationship count changed')
    return (manifest, model, view), ids, metadata['targetObjectURI']


def normalize(value, ids):
    if isinstance(value, dict):
        output = {ids.get(key,key):normalize(item,ids) for key,item in value.items()}
        require(len(output) == len(value), 'Mapped identities collide')
        return output
    if isinstance(value, list):
        return [normalize(item,ids) for item in value]
    return ids.get(value,value) if isinstance(value,str) else value


def check_native(pins, hierarchy):
    left, a, from_uri = snapshot('rust-successor.zip')
    right, b, to_uri = snapshot('native-reexport.zip')
    require(a.keys() == b.keys() and not set(a.values()) & set(b.values()), 'Native identity domains differ')
    ids = {a[key]:b[key] for key in a}
    ids[from_uri] = to_uri
    require(tuple(normalize(document,ids) for document in left) == right,
            'Native manifest, model, or diagram changed')
    capture = json.loads((ROOT / 'native-final-capture.json').read_bytes())
    require(capture['release'] == 'v2026.9.3' and capture['input']['sha256'] == pins['rust-successor.zip']
            and capture['output']['sha256'] == pins['native-reexport.zip']
            and capture['missing_manifest_control']['projects_after'] == 0, 'Native transfer pins differ')
    browser = json.loads((ROOT / 'native-final-browser.json').read_bytes())
    native = browser['reloaded']
    require(native == browser['first'] and not browser['pageErrors']
            and native['projectId'] == b['project'] and native['representationId'] == b['representation']
            and native['rendered'] == sorted(node['text'] for node in hierarchy)
            and native['definition'] == 'DriveShaft V5' and native['usage'] == 'driveShaftInstance',
            'Native browser did not reopen the exact edited hierarchy')


def main():
    pins = json.loads((ROOT / 'provenance.json').read_bytes())
    for name, digest in pins.items():
        require(Path(name).name == name and sha256((ROOT / name).read_bytes()).hexdigest() == digest,
                f'Evidence differs in {name}')
    original, view, _ = open_archive('native-source.zip')
    hierarchy = diagram_nodes(original, view)
    first = renamed('native-source.zip', 'rust-first.zip', 'DriveShaft V3', 'DriveShaft V4')
    second = renamed('rust-first.zip', 'rust-successor.zip', 'DriveShaft V4', 'DriveShaft V5')
    require(first == second, 'Edited definition identity changed')
    browser = json.loads((ROOT / 'rust-browser.json').read_bytes())
    require(browser['fixture']['sha256'] == pins['native-source.zip']
            and browser['final_sha256'] == pins['rust-successor.zip'], 'Browser archive pins differ')
    check_browser(hierarchy, browser)
    check_native(pins, hierarchy)
    print('PASS nine-node archive recovery and full normalized SysON re-export')


if __name__ == '__main__':
    main()

"""Check observed native outcomes for the diagram JSON member-order control."""
from hashlib import sha256
import json
import runpy
from pathlib import Path
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parent


def read(name):
    with ZipFile(ROOT / name) as archive:
        if archive.testzip() is not None:
            raise ValueError('Invalid native archive fixture')
        members = {member: archive.read(member) for member in archive.namelist()}
    view, = [member for member in members if '/representations/' in member]
    return json.loads(members.pop(view)), members


def names(diagram):
    return sorted(node['targetObjectLabel'] for node in diagram['representation']['nodes'])


def check_browser(file, expected):
    observation = json.loads((ROOT / file).read_bytes())
    reloaded = observation['reloaded']
    if (reloaded != observation['first'] or observation['pageErrors']
            or sorted(reloaded['rendered']) != expected
            or reloaded['definition'] != 'DriveShaft V3'
            or reloaded['usage'] != 'driveShaftInstance'
            or reloaded['lastPackageDefinition'] != 'Body/TrunkLid'):
        raise ValueError('Native browser reload observation differs')


def saved_layout(diagram):
    layout = diagram['representation']['layoutData']
    return len(layout['nodeLayoutData']), len(layout['labelLayoutData'])


def check_exposure():
    before, old_members = read('native-before-exposure.zip')
    after, new_members = read('native-exposed.zip')
    root, = after['representation']['nodes']
    child, = root['childNodes']
    layout = after['representation']['layoutData']
    if (names(before) != ['VehicleAssembly'] or root['targetObjectLabel'] != 'VehicleAssembly'
            or child['targetObjectLabel'] != 'Drivetrain' or child['id'] not in layout['nodeLayoutData']
            or child['insideLabel']['id'] not in layout['labelLayoutData']):
        raise ValueError('Native child or its live layout differs')
    old_model, = (json.loads(value) for name, value in old_members.items() if '/documents/' in name)
    new_model, = (json.loads(value) for name, value in new_members.items() if '/documents/' in name)
    objects = runpy.run_path(str(ROOT / 'verify.py'))['objects']
    old_exposures = {row['id'] for row in objects(old_model) if row['eClass'] == 'sysml:MembershipExpose'}
    view, = [row for row in objects(new_model) if row['eClass'] == 'sysml:ViewUsage']
    changes = view['data']['ownedRelationship']
    added, = [row for row in changes if row['eClass'] == 'sysml:MembershipExpose' and row['id'] not in old_exposures]
    owner, = [row for row in objects(new_model) if row['id'] == added['data']['importedMembership']]
    if (owner['eClass'] != 'sysml:OwningMembership'
            or child['targetObjectId'] not in {row['id'] for row in objects(owner)}):
        raise ValueError('Native exposure does not own the displayed package')
    changes.remove(added)
    if old_model != new_model:
        raise ValueError('Model changed beyond the added exposure')
    browser = json.loads((ROOT / 'native-exposure-browser.json').read_bytes())
    if (browser['first']['rendered'] != ['VehicleAssembly'] or browser['pageErrors']
            or browser['after'] != ['Drivetrain','VehicleAssembly'] or browser['after'] != browser['reloaded']):
        raise ValueError('Native browser exposure differs')


def main():
    original, original_members = read('vehicle-successor.zip')
    corrected, corrected_members = read('vehicle-type-before-style.zip')
    if original != corrected or original_members != corrected_members or len(names(original)) != 9:
        raise ValueError('The comparison must change only representation field ordering')
    for old, new in zip(original['representation']['nodes'], corrected['representation']['nodes']):
        before, after = list(old), list(new)
        if not (before.index('style') < before.index('type') and after.index('type') < after.index('style')
                and [key for key in before if key != 'type'] == [key for key in after if key != 'type']):
            raise ValueError('Unexpected node key order')
    failed, _ = read('native-original-reexport.zip')
    if names(failed) != ['VehicleAssembly']:
        raise ValueError('Original native diagram loss observation differs')
    check_browser('native-original-browser.json', names(failed))
    reordered, _ = read('native-reordered-reexport.zip')
    if (names(reordered) != ['VehicleAssembly'] or saved_layout(failed) != (0, 0)
            or saved_layout(reordered) != saved_layout(corrected)
            or saved_layout(corrected) != (9, 9)):
        raise ValueError('The order control did not reproduce the saved layout difference')
    check_browser('native-reordered-browser.json', names(reordered))
    for scenario, fixture in [('original','vehicle-successor.zip'),
                              ('reordered','vehicle-type-before-style.zip')]:
        observation = json.loads((ROOT / f'native-{scenario}-observation.json').read_bytes())
        capture = json.loads((ROOT / f'native-{scenario}-capture.json').read_bytes())
        if (not observation['normalized_model_equal'] or observation['stable_objects'] != 423
                or observation['typing_links'] != 81 or observation['raw_project_counts'] != [0, 0, 1]
                or observation['input'] != capture['input'] or observation['output'] != capture['output']
                or capture['release'] != 'v2026.9.3'):
            raise ValueError('Normalized native model observation differs')
        for key, name in [('input', fixture), ('output', f'native-{scenario}-reexport.zip')]:
            if sha256((ROOT / name).read_bytes()).hexdigest() != capture[key]['sha256']:
                raise ValueError('Native capture archive digest differs')
    check_exposure()
    print('PASS reproduced native diagram loss and isolated field-order control')


if __name__ == '__main__':
    main()

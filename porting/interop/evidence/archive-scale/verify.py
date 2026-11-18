"""Verify the retained archive scale evidence."""
from hashlib import sha256
import json
from pathlib import Path
from statistics import median
from zipfile import ZipFile

ROOT = Path(__file__).resolve().parent


def require(value, message):
    if not value:
        raise ValueError(message)


def objects(value):
    if isinstance(value, dict):
        if 'eClass' in value:
            yield value
        for child in value.values():
            yield from objects(child)
    elif isinstance(value, list):
        for child in value:
            yield from objects(child)


def read(name):
    with ZipFile(ROOT / name) as archive:
        names = archive.namelist()
        require(archive.testzip() is None and len(names) == len(set(names)) == 3, 'Invalid archive')
        model, = [entry for entry in names if '/documents/' in entry]
        data = json.loads(archive.read(model))
        require(sum(1 for _ in objects(data)) == 423, 'Model size differs')
        return data, {entry: archive.read(entry) for entry in names if entry != model}


def compare(before_name, after_name, old, new):
    before, old_sidecars = read(before_name)
    after, new_sidecars = read(after_name)
    require(old_sidecars == new_sidecars, 'Manifest or diagram changed')
    target, = [row for row in objects(before) if row['eClass'] == 'sysml:PartDefinition'
               and row['data'].get('declaredName') == old]
    target['data']['declaredName'] = new
    require(before == after, 'Unexpected semantic difference')


def measurements(campaign, nodes):
    require(campaign['owned_scope_removed'], 'Scope remained active')
    runs = campaign['runs']
    require([row['warmup'] for row in runs] == [True, True] + [False] * 7, 'Wrong sample schedule')
    timings = {}
    for run in runs:
        signals = ['SIGTERM', 'SIGKILL', 'SIGTERM', 'SIGTERM']
        require(run['service_signals'] == signals, 'No forced restart')
        require(run['retained_earlier_files'] and run['final_sha256'] == campaign['final_sha256'],
                'Custody changed')
        phases = run['phases']
        require(phases[-1]['archive'] == run['final_sha256'], 'Downloaded ZIP differs')
        for phase in phases:
            require(all(view['supported'] and view['nodeCount'] == nodes for view in phase['browserWork']),
                    'Rendered diagram count differs')
            if not run['warmup']:
                for key, elapsed in phase['measurements'].items():
                    require(0 < elapsed < float('inf'), 'Invalid elapsed measurement')
                    timings.setdefault(key, []).append(elapsed)
    require(len(timings) == 9 and all(len(values) == 7 for values in timings.values()), 'Insufficient samples')
    return timings


def main():
    pins = json.loads((ROOT / 'provenance.json').read_bytes())
    for name, digest in pins.items():
        require(Path(name).name == name and sha256((ROOT / name).read_bytes()).hexdigest() == digest,
                f'Evidence changed for {name}')
    compare('vehicle-source.zip', 'vehicle-first.zip', 'DriveShaft', 'DriveShaft V2')
    compare('vehicle-first.zip', 'vehicle-successor.zip', 'DriveShaft V2', 'DriveShaft V3')
    data = json.loads((ROOT / 'browser.json').read_bytes())
    require(data['small']['subject'] == data['vehicle']['subject'], 'Benchmark binaries differ')
    require(data['vehicle']['fixture']['sha256'] == pins['vehicle-source.zip']
            and data['vehicle']['final_sha256'] == pins['vehicle-successor.zip'], 'Browser ZIP differs')
    for name, nodes in [('small', 1), ('vehicle', 9)]:
        for metric, values in measurements(data[name], nodes).items():
            print(name, metric, *(f'{value:.2f}' for value in (min(values), median(values), max(values))))
    print('PASS archive scale evidence')


if __name__ == '__main__':
    main()

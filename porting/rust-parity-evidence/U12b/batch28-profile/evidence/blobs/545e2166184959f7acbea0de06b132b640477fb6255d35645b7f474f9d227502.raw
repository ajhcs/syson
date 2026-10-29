"""Fixed seven-measured-observation ranges; never pool groups or claim p95."""
from pathlib import Path
import json
base = Path('/workspace/build/builderbob/syson-batch28-20261029/u12b-profile-01')
groups = [(n, shape, False) for n in [32, 64, 128] for shape in ['disconnected', 'chain']]
groups += [(n, shape, True) for n in [64, 128] for shape in ['disconnected', 'chain']]
report = {'scope': 'Finite placement API ranges only; default successes and lowered-quota controls separate; no native/UI/p95/capacity acceptance', 'groups': []}
for index, (count, shape, lowered) in enumerate(groups):
    paths = [base / f'sample-{index}-{repetition}-stdout.txt' for repetition in range(2, 9)]
    rows = [json.loads(path.read_bytes()) for path in paths]
    expected = 'LOWERED_COMBINED_WORK_REFUSAL' if lowered else 'PLACED_CANONICAL_FOREST'
    if any(row['expected_outcome'] is not True or row['status'] != expected for row in rows):
        raise RuntimeError('unexpected fixed placement sample')
    metrics = {'api_ns': [row['api_ns'] for row in rows],
               'requested_bytes': [row['allocations']['requested_bytes'] for row in rows],
               'phase_peak_above_initial_live': [row['allocations']['phase_peak_above_initial_live'] for row in rows],
               'signed_live_delta_bytes': [row['signed_live_delta_bytes'] for row in rows],
               'process_peak_rss_kib': [row['process_peak_rss_kib'] for row in rows]}
    report['groups'].append({'nodes': count, 'topology': shape, 'lowered_combined_quota': lowered,
        'status': expected, 'warmups': 2, 'measured': 7,
        'ranges': {name: [min(values), max(values)] for name, values in metrics.items()},
        'raw_samples': [path.name for path in paths]})
(base / 'ranges.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report))

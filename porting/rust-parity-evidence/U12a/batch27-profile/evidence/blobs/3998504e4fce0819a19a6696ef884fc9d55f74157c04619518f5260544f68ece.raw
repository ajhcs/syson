from pathlib import Path
import json

base = Path(__file__).parent / 'u12a-profile-01'
report = {'scope': 'API ranges only; no UI/native/placement/capacity/p95 acceptance', 'groups': []}
for count in [32, 64, 128, 256, 1024]:
    for topology in ['flat', 'star']:
        paths = [base / f'sample-{count}-{topology}-{index}-stdout.txt' for index in range(2, 9)]
        rows = [json.loads(path.read_bytes()) for path in paths]
        if any(row['expected_outcome'] is not True for row in rows):
            raise RuntimeError('unexpected observation in range source')
        metrics = {'api_ns': [row['api_ns'] for row in rows],
                   'process_peak_rss_kib': [row['process_peak_rss_kib'] for row in rows],
                   'requested_bytes': [row['allocations']['requested_bytes'] for row in rows],
                   'phase_peak_above_initial_live': [row['allocations']['phase_peak_above_initial_live'] for row in rows],
                   'live_delta_bytes': [row['live_delta_bytes'] for row in rows]}
        report['groups'].append({'nodes': count, 'topology': topology, 'warmups': 2,
            'measured': 7, 'statuses': sorted({row['status'] for row in rows}),
            'ranges': {name: [min(values), max(values)] for name, values in metrics.items()},
            'raw_samples': [path.name for path in paths]})
(base / 'ranges.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps([{key: row[key] for key in ['nodes', 'topology', 'statuses', 'ranges']} for row in report['groups']]))

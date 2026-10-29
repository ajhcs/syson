"""Separate min/max API ranges; no pooled p95, UI or capacity acceptance."""
from pathlib import Path
import json
import hashlib

base = Path('/workspace/build/builderbob/syson-batch27-20261029/u11b-search-profile-01')
if (base / 'ranges.json').exists():
    raise RuntimeError('preserve existing range epoch')
report = {'scope': 'Search API ranges only; process VmHWM also includes admission/setup', 'groups': []}
for count in [32, 64, 128, 256, 1024]:
    for density in [1, 2]:
        paths = [base / f'sample-{count}-{density}-{index}-stdout.txt' for index in range(2, 9)]
        raw = [path.read_bytes() for path in paths]
        rows = [json.loads(value) for value in raw]
        if any(row['expected_outcome'] is not True for row in rows):
            raise RuntimeError('unexpected observation in range source')
        metrics = {'api_ns': [row['api_ns'] for row in rows],
                   'process_peak_rss_kib': [row['process_peak_rss_kib'] for row in rows],
                   'requested_bytes': [(row['allocations'] or {}).get('requested_bytes') for row in rows],
                   'new_peak_bytes': [(row['allocations'] or {}).get('phase_peak_above_initial_live') for row in rows],
                   'net_live_delta_bytes': [row['live_delta_bytes'] for row in rows]}
        ranges = {key: [min(v for v in values if v is not None), max(v for v in values if v is not None)]
                  if any(v is not None for v in values) else None for key, values in metrics.items()}
        report['groups'].append({'nodes': count, 'edges_per_node': density, 'warmups': 2, 'measured': 7,
            'statuses': sorted({row['status'] for row in rows}), 'errors': sorted({row['error'] for row in rows if row['error']}),
            'search_calls': sum(row['search_called'] for row in rows), 'ranges': ranges,
            'raw_inputs': {path.name: hashlib.sha256(value).hexdigest() for path, value in zip(paths, raw)}})
(base / 'ranges.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({'groups': len(report['groups']), 'scope': report['scope']}))

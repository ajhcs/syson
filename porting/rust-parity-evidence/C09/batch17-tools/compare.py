"""Compare selected original-runtime/public Rust fields; preserve all raw input."""
import argparse, copy, json
from pathlib import Path

def equal(left, right):
    if type(left) is not type(right): return False
    if isinstance(left, dict):
        return left.keys() == right.keys() and all(equal(left[k], right[k]) for k in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(equal(a, b) for a, b in zip(left, right))
    return left == right

def reflection(native, rust):
    native = native['observation']
    errors = []
    for field in ['all_generated_types', 'ast_types_filtered', 'generated_references', 'positions']:
        if not equal(native[field], rust[field]): errors.append(field)
    if len(native['rows']) != 189 or len(rust['rows']) != 189: errors.append('row count')
    for i, (left, right) in enumerate(zip(native['rows'], rust['rows'])):
        for field in ['name', 'ancestors', 'descendants', 'mandatory', 'subtype_row']:
            if not equal(left[field], right[field]): errors.append(f'row {i}: {field}')
    return errors

def references(native, rust):
    errors = []
    for case in ['local', 'external']:
        for kind in ['source', 'target']:
            value = dict(rust['cases'][case][kind])
            # This caller key is supplied explicitly by the two probe recipes.
            # External URIs and every node/semantic identity remain literal.
            if value['resource'] == 'vehicle': value['resource'] = 'memory:/Batmobile.json'
            if not equal(value, native['cases'][case][kind]): errors.append(f'{case}: {kind}')
        if not equal(rust['cases'][case]['type_is_Type'], native['cases'][case]['type_is_Type']):
            errors.append(case + ': assignability')
    if not equal(native['missing_without_load'], rust['missing_without_load']):
        errors.append('missing resource')
    return errors

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('kind', choices=['reflection', 'references'])
    parser.add_argument('native', type=Path); parser.add_argument('rust', type=Path)
    parser.add_argument('--self-test', action='store_true'); args = parser.parse_args()
    native = json.loads(args.native.read_text()); rust = json.loads(args.rust.read_text())
    compare = reflection if args.kind == 'reflection' else references
    errors = compare(native, rust)
    if args.self_test:
        assert not errors, errors
        mutations = []
        if args.kind == 'reflection':
            for field in ['subtype_row', 'ancestors', 'mandatory']:
                changed = copy.deepcopy(rust)
                row = next(r for r in changed['rows'] if len(r[field]) >= 2)
                if field == 'subtype_row': row[field][0] = not row[field][0]
                else: row[field][0], row[field][1] = row[field][1], row[field][0]
                mutations.append(changed)
            changed = copy.deepcopy(rust); changed['rows'].pop(); mutations.append(changed)
            changed = copy.deepcopy(rust); changed['rows'][0]['subtype_row'][0] = int(changed['rows'][0]['subtype_row'][0]); mutations.append(changed)
        else:
            for case, field in [('local', 'node_id'), ('external', 'element_id'), ('external', 'resource')]:
                changed = copy.deepcopy(rust); changed['cases'][case]['target'][field] += 'DIVERGENT'
                mutations.append(changed)
            changed = copy.deepcopy(rust); changed['cases']['local']['type_is_Type'] = 1; mutations.append(changed)
        for changed in mutations: assert compare(native, changed), 'divergence accepted'
    print(json.dumps({'scope': 'Selected fields only; no feature acceptance',
                      'kind': args.kind, 'differences': errors,
                      'negative_controls': args.self_test}))
    return 2 if errors else 0

if __name__ == '__main__': raise SystemExit(main())

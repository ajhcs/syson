"""Strict selected-method comparison; function bindings never execute callbacks."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import re

BUNDLE = '47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd'
CORPUS = '2a0416405914ed4c26273d0d17796fbf04f2d093001ecc05e9282a4e68664f54'
MODULES = {
    '../syside-languageserver/lib/model/types.js': 'afd39d9ebedb6ff1a6cca2d4f42341d5883c9509e81f3b89981f1cc0bea79440',
    '../syside-languageserver/lib/services/parser/parser.js': 'ced8f9cab928606d13a323bcaca8693025c2ae381fd9ed2b0bcf27b45707991e',
    '../../node_modules/.pnpm/langium@1.2.1/node_modules/langium/lib/parser/cst-node-builder.js': '5515bf68b0f3d79a26feebbef259c513879c6043aff06ae0e866885564dd3b4a',
}
# Source registration identity only. Rust does not expose a function or arity.
BINDINGS = [
    ('FixOperatorExpression', 'OperatorExpression', 'fixOperatorExpression'),
    ('AddLoopMember', 'WhileLoopActionUsage', 'addLoopMember'),
    ('FinalizeImport', 'Import', 'finalizeImport'),
    ('CreateEmptyTransitionParameters', 'TransitionUsage', 'createEmptyParametersInTransitionUsage'),
    ('CreateMissingSuccessionEnds', 'SuccessionAsUsage', 'createMissingEndsInSuccessionAsUsage'),
]
NATIVE_FUNCTIONS = {symbol: rust_id for rust_id, _, symbol in BINDINGS}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def strict_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key: ' + key)
        result[key] = value
    return result


def loads(text):
    def constant(value):
        raise ValueError('non-JSON number: ' + value)
    return json.loads(text, object_pairs_hook=strict_object, parse_constant=constant)


def keys(value, expected, path):
    require(type(value) is dict and set(value) == set(expected),
            path + ': missing or extra descriptor fields')


def descriptor(value, side, path):
    require(type(value) is dict and type(value.get('kind')) is str,
            path + ': descriptor object required')
    kind = value['kind']
    fields = {'undefined': ['kind'], 'null': ['kind'],
              'boolean': ['kind', 'value'], 'number': ['kind', 'bits'],
              'string': ['kind', 'value'], 'array': ['kind', 'items'],
              'map': ['kind', 'entries']}
    if kind == 'function' and side == 'native':
        keys(value, ['kind', 'name', 'arity'], path)
        require(value['name'] in NATIVE_FUNCTIONS and type(value['arity']) is int
                and value['arity'] == 2, path + ': pinned native function descriptor drift')
        return
    if kind == 'postprocessor' and side == 'rust':
        keys(value, ['kind', 'name'], path)
        require(value['name'] in NATIVE_FUNCTIONS.values(),
                path + ': unbound Rust processor ID')
        return
    require(kind in fields, path + ': unsupported descriptor kind ' + kind)
    keys(value, fields[kind], path)
    if kind in ('boolean', 'string'):
        require(type(value['value']) is (bool if kind == 'boolean' else str),
                path + ': wrong primitive value type')
    elif kind == 'number':
        require(type(value['bits']) is str and
                re.fullmatch('[0-9a-f]{16}', value['bits']) is not None,
                path + ': exact binary64 hex bits required')
    elif kind == 'array':
        require(type(value['items']) is list, path + ': items array required')
        for i, child in enumerate(value['items']):
            descriptor(child, side, path + '/items/' + str(i))
    elif kind == 'map':
        require(type(value['entries']) is list, path + ': ordered map entries required')
        seen = set()
        for i, pair in enumerate(value['entries']):
            require(type(pair) is list and len(pair) == 2 and type(pair[0]) is str,
                    path + ': malformed map pair')
            require(pair[0] not in seen, path + ': duplicate map key')
            seen.add(pair[0])
            descriptor(pair[1], side, path + '/entries/' + str(i) + '/1')


def differences(native, rust, path, found, bindings):
    if type(native) is dict and type(rust) is dict:
        if native.get('kind') == 'function' and rust.get('kind') == 'postprocessor':
            bindings.append({'path': path, 'native_descriptor': native,
                             'rust_descriptor': rust,
                             'meaning': 'Registered selection identity only; callback behavior unrun'})
            if NATIVE_FUNCTIONS[native['name']] != rust['name']:
                found.append({'path': path, 'kind': 'registration_identity',
                              'native': native, 'rust': rust})
            return
        for key in sorted(set(native) | set(rust)):
            child = path + '/' + key
            if key not in native or key not in rust:
                found.append({'path': child, 'kind': 'field_presence',
                              'native_present': key in native, 'rust_present': key in rust})
            else:
                differences(native[key], rust[key], child, found, bindings)
    elif type(native) is list and type(rust) is list:
        if len(native) != len(rust):
            found.append({'path': path, 'kind': 'array_length',
                          'native': len(native), 'rust': len(rust)})
        for i, (left, right) in enumerate(zip(native, rust)):
            differences(left, right, path + '/' + str(i), found, bindings)
    elif type(native) is not type(rust) or native != rust:
        found.append({'path': path, 'kind': 'exact_value', 'native': native, 'rust': rust})


def compare(corpus, native, rust, corpus_hash):
    require(corpus_hash == CORPUS, 'reviewed corpus bytes changed')
    require(type(corpus.get('schema')) is int and corpus['schema'] == 1,
            'corpus schema')
    cases = corpus['cases']
    require(type(cases) is list and len(cases) == 42, '42 complete corpus cases required')
    ids = [case['id'] for case in cases]
    require(all(type(i) is str and i for i in ids) and len(set(ids)) == len(ids),
            'duplicate or invalid corpus case IDs')
    p = native['provenance']
    require(p['bundle_sha256'] == BUNDLE and p['corpus_sha256'] == corpus_hash
            and p['corpus_cases'] == len(cases), 'native bundle/corpus binding')
    require(p['embedded_module_sha256'] == MODULES, 'native original module binding')
    require(native['native_constructors'] == ['TypesIndex', 'SysMLCstNodeBuilder'],
            'original constructor identity')
    expected = [dict(rust_id=r, source_registration=t, source_symbol=s)
                for r, t, s in BINDINGS]
    require(rust['postprocessor_bindings'] == expected, 'five source identity bindings')
    for label, document in [('native', native), ('rust', rust)]:
        require(type(document.get('schema')) is int and document['schema'] == 1,
                label + ': schema')
        rows = document['observations']
        require(type(rows) is list and [row.get('id') for row in rows] == ids,
                label + ': complete ordered corpus observation IDs')
        for row in rows:
            keys(row, ['id', 'value'], label + '/' + row['id'])
            descriptor(row['value'], label, label + '/' + row['id'])
    found, paired, rows = [], [], []
    for case, left, right in zip(cases, native['observations'], rust['observations']):
        start = len(found)
        differences(left['value'], right['value'], case['id'], found, paired)
        rows.append({'id': case['id'], 'constructor': case['constructor'],
                     'method': case['method'], 'mode': case['mode'],
                     'difference_count': len(found) - start})
    return {'status': 'DIFFERENCES_REMAIN' if found else 'PASS_SELECTED_DISPATCH_CASES',
            'case_results': rows, 'differences': found, 'selection_bindings': paired,
            'open_items': ['Configured native Node20 and exact deployed artifact correspondence',
                           'Callback application, side effects and shared payload identity',
                           'Real F12/parser/CST/AST/cache/linker/importer callers',
                           'UI, performance, application lifecycle and full feature acceptance'],
            'meaning': 'Exact selected descriptor/order comparison plus explicit source selection bindings; no callback equivalence or full parity claim'}


def self_test(corpus, native, rust, corpus_hash):
    baseline = compare(corpus, native, rust, corpus_hash)
    def row(document, id):
        return next(r for r in document['observations'] if r['id'] == id)['value']
    controls = []
    def reject(label, mutate):
        left, right = copy.deepcopy(native), copy.deepcopy(rust)
        mutate(left, right)
        try:
            result = compare(corpus, left, right, corpus_hash)
        except (ValueError, KeyError, TypeError):
            controls.append({'id': label, 'result': 'REJECTED_INPUT_OR_BINDING'})
            return
        added = [d for d in result['differences'] if d not in baseline['differences']]
        require(bool(added), 'pipeline accepted divergence: ' + label)
        controls.append({'id': label, 'result': 'DETECTED_DIVERGENCE', 'paths': [d['path'] for d in added]})
    reject('omit_case', lambda n, r: r['observations'].pop())
    reject('reorder_cases', lambda n, r: r['observations'].reverse())
    reject('bool_is_not_integer', lambda n, r: row(r, 'truthy-boolean-default')['entries'][0][1].update(value=1))
    reject('null_is_not_absent', lambda n, r: row(r, 'construct-unknown').update(kind='null'))
    reject('exact_binary64', lambda n, r: row(r, 'truthy-number-default').update(bits='3ff0000000000001'))
    reject('default_selection', lambda n, r: row(r, 'falsey-ancestor-default').update(value='wrong-default'))
    reject('array_order', lambda n, r: row(r, 'merge-blocks-reversed')['items'].reverse())
    reject('repeated_payloads', lambda n, r: row(r, 'merge-repeated-items')['items'].pop())
    reject('chain_array_not_flattened', lambda n, r: row(r, 'chain-array-payloads')['items'].__setitem__(0, {'kind': 'string', 'value': 'f1'}))
    reject('retain_empty_map_entries', lambda n, r: row(r, 'chain-falsey-all-map')['entries'].pop())
    reject('map_order', lambda n, r: row(r, 'truthy-boolean-default')['entries'].reverse())
    reject('native_arity_retained', lambda n, r: row(n, 'pinned-operator').update(arity=3))
    reject('processor_identity', lambda n, r: row(r, 'pinned-operator').update(name='AddLoopMember'))
    reject('source_registration', lambda n, r: r['postprocessor_bindings'][0].update(source_registration='Feature'))
    reject('corpus_binding', lambda n, r: n['provenance'].update(corpus_sha256='0' * 64))
    reject('extra_descriptor_field', lambda n, r: row(r, 'construct-undefined').update(value=None))
    # Parse the entire retained observation with a duplicate schema key, not a toy descriptor.
    try:
        loads('{"schema":1,' + json.dumps(native)[1:])
    except ValueError:
        controls.append({'id': 'duplicate_json_key', 'result': 'REJECTED_INPUT_OR_BINDING'})
    else:
        raise AssertionError('duplicate native capture key accepted')
    return {'status': 'PASS_COMPLETE_PIPELINE_DIVERGENCE_CONTROLS', 'controls': controls}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('corpus', 'native', 'rust', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    supplied = {name: getattr(args, name) for name in ('corpus', 'native', 'rust', 'output')}
    require(all(p.is_absolute() for p in supplied.values()), 'absolute paths required')
    workspace = Path('/workspace').resolve(strict=True)
    require(workspace == Path('/workspace'), 'verified workspace root must be canonical')
    paths = {name: path.resolve(strict=True) for name, path in supplied.items()
             if name != 'output'}
    require(all(path.is_file() and path.is_relative_to(workspace) for path in paths.values()),
            'canonical input files must remain under /workspace')
    parent = args.output.parent.resolve(strict=True)
    require(parent.is_dir() and parent.is_relative_to(workspace)
            and args.output.name not in ('', '.', '..'), 'canonical output parent must remain under /workspace')
    output = parent / args.output.name
    require(not output.exists() and not output.is_symlink(), 'preserve existing output or symlink')
    paths['output'] = output
    inputs = {name: path.read_bytes() for name, path in paths.items() if name != 'output'}
    documents = {name: loads(raw.decode('utf8')) for name, raw in inputs.items()}
    hashes = {name: hashlib.sha256(raw).hexdigest() for name, raw in inputs.items()}
    result = compare(documents['corpus'], documents['native'], documents['rust'], hashes['corpus'])
    if args.self_test:
        result['controls'] = self_test(documents['corpus'], documents['native'], documents['rust'], hashes['corpus'])
    result['inputs'] = {name: {'path': str(paths[name]), 'sha256': digest} for name, digest in hashes.items()}
    result['comparator_sha256'] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    with output.open('x') as stream:
        stream.write(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({'status': result['status'], 'cases': len(result['case_results']),
                      'differences': len(result['differences']), 'output': str(output)}))
    return 1 if result['differences'] else 0


if __name__ == '__main__':
    raise SystemExit(main())

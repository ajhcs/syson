"""Compare named synthetic operator captures; retain differences and OPEN gates."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct

CORPUS = '24cda4f62b2368cdc48cd9c2c976ba5c67ba295160e4bdc9483f9dc1d886632a'
BUNDLE = '47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd'
META = {'$type', '$container', '$containerProperty', '$containerIndex', '$childIndex', '$cstNode'}
ABSENT = {'state': 'ABSENT'}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def loads(raw):
    def pairs(items):
        out = {}
        for key, value in items:
            require(key not in out, 'duplicate JSON key: ' + key)
            out[key] = value
        return out
    def constant(value):
        raise ValueError('nonfinite JSON: ' + value)
    return json.loads(raw, object_pairs_hook=pairs, parse_constant=constant)


def exact(a, b):
    if type(a) is not type(b):
        return False
    if type(a) is dict:
        return set(a) == set(b) and all(exact(a[k], b[k]) for k in a)
    if type(a) is list:
        return len(a) == len(b) and all(exact(x, y) for x, y in zip(a, b))
    return a == b


def keys(v, wanted, label):
    require(type(v) is dict and set(v) == set(wanted), label + ': exact fields required')


def unsigned(v):
    return type(v) is int and v >= 0


def hex_bits(v):
    return type(v) is str and len(v) == 16 and all(c in '0123456789abcdef' for c in v)


def pointer(key):
    return key.replace('~', '~0').replace('/', '~1')


def utf16(text):
    raw = text.encode('utf-16-le', errors='surrogatepass')
    return [int.from_bytes(raw[i:i + 2], 'little') for i in range(0, len(raw), 2)]


def validate_rust(snapshot, arena=False):
    require(type(snapshot['nodes']) is list and snapshot['nodes'], 'nonempty public nodes')
    ids = []
    for node in snapshot['nodes']:
        keys(node, {'id', 'node_type', 'fields', 'origin', 'container'}, 'public node')
        require(unsigned(node['id']) and type(node['node_type']) is str and type(node['fields']) is dict, 'node identity/type')
        ids.append(node['id'])
    require(len(ids) == len(set(ids)) and unsigned(snapshot['root']) and snapshot['root'] in ids, 'public root/IDs')
    def lexeme(value):
        keys(value, {'bytes', 'utf16', 'text'}, 'lexeme')
        require(type(value['text']) is str, 'lexeme text')
        for axis in ('bytes', 'utf16'):
            a = value[axis]
            require(type(a) is list and len(a) == 2 and all(unsigned(v) for v in a) and a[0] <= a[1], 'lexeme offsets')
    def datum(d):
        keys(d, {'value', 'lexeme'}, 'Datum')
        if d['lexeme'] is not None: lexeme(d['lexeme'])
        v = d['value']; kind = v['kind']
        shapes = {'null': {'kind'}, 'boolean': {'kind', 'value'}, 'number': {'kind', 'bits'},
                  'string': {'kind', 'units'}, 'node': {'kind', 'id'}, 'array': {'kind', 'items'},
                  'unresolved_reference': {'kind', 'text', 'target_type'}}
        require(kind in shapes, 'public value tag'); keys(v, shapes[kind], 'public value')
        if kind == 'boolean': require(type(v['value']) is bool, 'public Boolean')
        if kind == 'number': require(hex_bits(v['bits']), 'public binary64')
        if kind == 'string': require(type(v['units']) is list and all(unsigned(x) and x <= 65535 for x in v['units']), 'UTF-16 units')
        if kind == 'node': require(unsigned(v['id']) and v['id'] in ids, 'public child handle')
        if kind == 'array':
            require(type(v['items']) is list, 'public Array')
            for child in v['items']: datum(child)
        if kind == 'unresolved_reference': require(type(v['text']) is str and type(v['target_type']) is str, 'public reference')
    for node in snapshot['nodes']:
        if node['origin'] is not None: lexeme(node['origin'])
        if node['container'] is not None:
            c = node['container']; keys(c, {'node', 'property', 'index'}, 'container')
            require(unsigned(c['node']) and c['node'] in ids and type(c['property']) is str, 'container parent')
            require(c['index'] is None or unsigned(c['index']), 'container index')
        for d in node['fields'].values(): datum(d)
    return {n['id']: n for n in snapshot['nodes']}


def public_target(snapshot, path):
    require(path == 'root' or path.startswith('root/'), 'public target path')
    nodes = {n['id']: n for n in snapshot['nodes']}
    target = nodes[snapshot['root']]
    parts = iter(path.split('/')[1:])
    for key in parts:
        value = target['fields'][key]['value']
        if value['kind'] == 'array':
            value = value['items'][int(next(parts))]['value']
        require(value['kind'] == 'node', 'public target must identify a node')
        target = nodes[value['id']]
    return target


class Comparison:
    def __init__(self, spec):
        self.spec, self.differences, self.open_items = spec, [], []

    def diff(self, path, kind, native=ABSENT, rust=ABSENT):
        self.differences.append(dict(path=path, kind=kind, native=native, rust=rust,
                                     boundary='cache' if '/$children' in path else 'structural'))

    def opened(self, path, kind, detail):
        self.open_items.append(dict(path=path, kind=kind, detail=detail))

    def report_contract(self, step, before_nodes, after_nodes):
        report, path = step['report'], step['target_path']
        if report['kind'] in {'NotSelected', 'rust_error'}:
            return
        before = public_target(step['before'], path)
        after = public_target(step['after'], path)
        operand = before['fields'].get('operands')
        value = operand['value'] if operand is not None else None
        normalized = value is None or value['kind'] == 'null'
        if value is not None:
            if value['kind'] == 'boolean': normalized = not value['value']
            if value['kind'] == 'string': normalized = not value['units']
            if value['kind'] == 'number':
                number = struct.unpack('>d', bytes.fromhex(value['bits']))[0]
                normalized = number == 0.0 or number != number
        empty = normalized or value['kind'] == 'array' and not value['items']
        operator = before['fields'].get('operator', {}).get('value', {})
        insert = empty and operator.get('kind') == 'string' and operator['units'] in [
            utf16(name) for name in ('istype', 'hastype', '@', 'as')]
        expected = {'kind': 'Inserted' if insert else 'Unchanged',
                    'normalized_operands': normalized, 'authority': 'Rust public Report contract'}
        if report['kind'] != expected['kind'] or report['normalized_operands'] != normalized:
            self.diff(path, 'Rust_Report_kind_or_normalization', expected, report)
        if report['kind'] == 'Unchanged':
            wanted = copy.deepcopy(step['before'])
            if normalized:
                public_target(wanted, path)['fields']['operands'] = {
                    'value': {'kind': 'array', 'items': []}, 'lexeme': None}
            if not exact(wanted, step['after']):
                self.diff(path, 'Rust_Unchanged_report_mutation', wanted, step['after'])
            return
        expression, membership, feature = [report[k] for k in ('expression', 'membership', 'feature')]
        if set(after_nodes) - set(before_nodes) != {expression, membership, feature}:
            self.diff(path, 'Rust_Inserted_report_new_IDs', sorted(set(after_nodes) - set(before_nodes)), report)
        links = [after['fields']['operands']['value'],
                 after_nodes[expression]['fields']['expression']['value'],
                 after_nodes[membership]['fields']['target']['value']]
        wanted_links = [{'kind': 'array', 'items': [
            {'value': {'kind': 'node', 'id': expression}, 'lexeme': None}]},
            {'kind': 'node', 'id': membership}, {'kind': 'node', 'id': feature}]
        if not exact(links, wanted_links):
            self.diff(path, 'Rust_Inserted_report_chain', wanted_links, links)

    def graph(self, graph):
        objects = graph['objects']; require(type(objects) is list, 'native object graph')
        by = {o['id']: o for o in objects}
        require(len(by) == len(objects) and all(type(i) is str for i in by), 'native object IDs')
        fields = {}
        for o in objects:
            require(o['kind'] in {'ast', 'array', 'object'}, 'native object kind')
            keys(o, {'id', 'kind', 'property_keys', 'properties'} | ({'node_type'} if o['kind'] == 'ast' else set()), 'native object record')
            if o['kind'] == 'ast': require(type(o['node_type']) is str, 'native AST type')
            entries = o['properties']; require(type(entries) is list, 'native own fields')
            require(exact([e['key'] for e in entries], o['property_keys']), 'native own-key inventory')
            require(all(type(e['key']) is str for e in entries), 'symbol-key disposition required')
            require(len({e['key'] for e in entries}) == len(entries), 'duplicate native field')
            fields[o['id']] = {e['key']: e.get('value', {'kind': 'unevaluated_accessor'}) for e in entries}
            for e in entries:
                d = e['descriptor']; require(type(d) is dict, 'native descriptor retained')
                accessor = 'accessor' in d
                keys(d, {'enumerable', 'configurable', 'accessor', 'getter', 'setter'} if accessor else {'enumerable', 'configurable', 'writable'}, 'native property descriptor')
                require(all(type(v) is bool for v in d.values()), 'native descriptor flags')
                keys(e, {'key', 'descriptor'} if accessor else {'key', 'descriptor', 'value'}, 'native own-property record')
        return by, fields

    def compare_final(self, ng, rs):
        objects, props = self.graph(ng); rnodes = validate_rust(rs, arena=True)
        np, rp, nby, rby, na, ra = {}, {}, {}, {}, [], []
        def object_for(v):
            return objects[v['$object']] if type(v) is dict and set(v) == {'$object'} else None
        def integer(v):
            keys(v, {'kind', 'bits'}, 'native numeric index'); require(v['kind'] == 'number' and hex_bits(v['bits']), 'native index bits')
            n = struct.unpack('>d', bytes.fromhex(v['bits']))[0]
            require(0 <= n <= 2 ** 53 - 1 and n == int(n) and struct.pack('>d', float(int(n))).hex() == v['bits'], 'canonical native array index')
            return int(n)
        def items(o):
            f = props[o['id']]; length = integer(f['length'])
            extra = set(f) - {'length'} - {str(i) for i in range(length)}
            if extra: self.opened(o['id'], 'native_array_extra_properties', sorted(extra))
            return [f.get(str(i), {'kind': 'array_hole'}) for i in range(length)]
        def nwalk(v, path):
            o = object_for(v)
            if not o: return
            if o['kind'] == 'array':
                for i, child in enumerate(items(o)): nwalk(child, path + '/' + str(i))
            elif o['kind'] == 'ast':
                if o['id'] in np: na.append(dict(path=path, canonical=np[o['id']])); return
                np[o['id']], nby[path] = path, o
                for key, child in props[o['id']].items():
                    if not key.startswith('$'): nwalk(child, path + '/' + pointer(key))
        def rwalk(v, path):
            if v['kind'] == 'array':
                for i, child in enumerate(v['items']): rwalk(child['value'], path + '/' + str(i))
            elif v['kind'] == 'node':
                ident = v['id']
                if ident in rp: ra.append(dict(path=path, canonical=rp[ident])); return
                rp[ident], rby[path] = path, rnodes[ident]
                for key, child in rnodes[ident]['fields'].items():
                    if not key.startswith('$'): rwalk(child['value'], path + '/' + pointer(key))
        nwalk(ng['root'], 'root'); rwalk(dict(kind='node', id=rs['root']), 'root')
        def nv(v):
            o = object_for(v)
            if o:
                if o['kind'] == 'ast':
                    require(o['id'] in np, 'cache-only native node needs structural disposition')
                    return dict(kind='node', path=np[o['id']])
                if o['kind'] == 'array': return dict(kind='array', items=[nv(x) for x in items(o)])
                self.opened(o['id'], 'native_object_without_public_value_equivalent', o)
                return dict(kind='native_object', id=o['id'])
            require(type(v) is dict and type(v.get('kind')) is str, 'native scalar descriptor')
            if v['kind'] == 'string':
                keys(v, {'kind', 'value'}, 'native string'); require(type(v['value']) is str, 'native string value')
                return dict(kind='string', units=utf16(v['value']))
            if v['kind'] == 'number': require(hex_bits(v['bits']), 'native binary64')
            if v['kind'] == 'boolean': require(type(v['value']) is bool, 'native Boolean')
            return v
        def rv(v):
            if v['kind'] == 'node': return dict(kind='node', path=rp.get(v['id'], 'UNMAPPED:' + str(v['id'])))
            if v['kind'] == 'array': return dict(kind='array', items=[rv(d['value']) for d in v['items']])
            return v
        if not exact(na, ra): self.diff('root', 'structural_aliases', na, ra)
        if set(rnodes) != set(rp): self.opened('root', 'public_orphan_nodes', sorted(set(rnodes) - set(rp)))
        for path in sorted(set(nby) | set(rby)):
            n, r = nby.get(path), rby.get(path)
            if n is None or r is None: self.diff(path, 'node_presence', n or ABSENT, r or ABSENT); continue
            f = props[n['id']]
            if n['node_type'] != r['node_type']: self.diff(path, 'node_type', n['node_type'], r['node_type'])
            if r['origin'] is not None: self.diff(path, 'unexpected_synthetic_node_CST', None, r['origin'])
            if not exact(f.get('$type'), {'kind': 'string', 'value': n['node_type']}): self.diff(path, 'native_type_inventory', n['node_type'], f.get('$type', ABSENT))
            for key in sorted((set(f) - META) | set(r['fields'])):
                at = path + '/' + pointer(key)
                if key not in f or key not in r['fields']: self.diff(at, 'field_presence', f.get(key, ABSENT), r['fields'].get(key, ABSENT)); continue
                a, b = nv(f[key]), rv(r['fields'][key]['value'])
                if not exact(a, b): self.diff(at, 'field_value', a, b)
                if r['fields'][key]['lexeme'] is not None: self.diff(at, 'unexpected_synthetic_Datum_CST', None, r['fields'][key]['lexeme'])
            parent = None
            if '$container' in f:
                owner = object_for(f['$container']); require(owner and owner['id'] in np, 'native parent identity')
                p = f.get('$containerProperty'); keys(p, {'kind', 'value'}, 'native parent property'); require(p['kind'] == 'string', 'native parent property string')
                parent = dict(parent=np[owner['id']], property=p['value'], index=integer(f['$containerIndex']) if '$containerIndex' in f else None)
            actual = r['container']
            if actual is not None: actual = dict(parent=rp[actual['node']], property=actual['property'], index=actual['index'])
            if not exact(parent, actual): self.diff(path, 'container', parent, actual)
            if '$childIndex' in f: self.opened(path, 'native_child_index_unexposed', f['$childIndex'])
            if '$cstNode' in f: self.opened(path, 'native_CST_presence_requires_disposition', f['$cstNode'])
        self.opened('root', 'descriptors_array_identity_and_lifecycle', 'Raw own descriptors, insertion order, stable native array/object identities and before/after graphs retained. Public Rust values/finish parenting do not expose equivalent JS identity/descriptors or immediate lifecycle.')

    def run(self, native, rust):
        require(native['domain'] == rust['domain'] == self.spec['domain'], 'case domain')
        if 'capture_error' in native or 'capture_error' in rust:
            self.diff('root', 'capture_failure', native.get('capture_error', ABSENT), rust.get('capture_error', ABSENT)); return
        self.graph(native['before']); self.graph(native['after_handler']); self.graph(native['final'])
        require([s['target_path'] for s in native['steps']] == self.spec['targets'], 'native steps/targets')
        for index, step in enumerate(native['steps']):
            self.graph(step['before']); self.graph(step['after'])
            if 'native_error' in step:
                actual = rust['steps'][index]['report'] if index < len(rust['steps']) else ABSENT
                self.diff(step['target_path'], 'native_handler_exception', step['native_error'], actual); return
            require(type(step['invoked']) is bool, 'native invocation flag')
            if step['invoked']:
                require(exact(step['selection'], {'kind': 'function', 'name': 'fixOperatorExpression', 'arity': 2}) and exact(step['native_return'], {'kind': 'undefined'}) and 'native_error' not in step, 'actual original invocation/void return')
            else:
                require(exact(step['native_return'], {'kind': 'not_invoked'}), 'other handler native return is not fabricated')
        domain, outcome = self.spec['domain'], rust['outcome']
        if domain == 'rust-unrepresentable':
            require(outcome['kind'] == 'input_unsupported' and rust['steps'] == [], 'explicit undefined cannot become absent')
            self.opened('root', 'undefined_value_not_representable', dict(native_steps=native['steps'], rust=outcome)); return
        require(exact(native['setup_calls'], rust['setup_calls']), 'matched input setup calls')
        validate_rust(rust['before'])
        def phase(graph, snapshot, label):
            before_diff, before_open = len(self.differences), len(self.open_items)
            self.compare_final(graph, snapshot)
            for d in self.differences[before_diff:]: d['phase'] = label
            for o in self.open_items[before_open:]: o['phase'] = label
        phase(native['before'], rust['before'], 'input_before_handler')
        require([s['target_path'] for s in rust['steps']] == self.spec['targets'], 'Rust steps/targets')
        for index, (n, r) in enumerate(zip(native['steps'], rust['steps'])):
            before_nodes = validate_rust(r['before']); after_nodes = validate_rust(r['after'])
            report = r['report']; kind = report['kind']
            shapes = {'NotSelected': {'kind'}, 'Unchanged': {'kind', 'normalized_operands'},
                      'Inserted': {'kind', 'expression', 'membership', 'feature', 'normalized_operands'},
                      'rust_error': {'kind', 'error'}}
            require(kind in shapes, 'Rust Report kind'); keys(report, shapes[kind], 'Rust Report')
            if 'normalized_operands' in report: require(type(report['normalized_operands']) is bool, 'Rust Report Boolean')
            if kind == 'Inserted':
                values = [report[k] for k in ('expression', 'membership', 'feature')]
                require(all(unsigned(v) and v in after_nodes for v in values) and len(set(values)) == 3, 'Rust inserted IDs')
                require([after_nodes[v]['node_type'] for v in values] == ['FeatureReferenceExpression', 'ReturnParameterMembership', 'Feature'], 'Rust Report IDs must identify actual inserted types')
            if kind == 'rust_error': require(type(report['error']) is str, 'Rust error text')
            if n['invoked']:
                require(r['report']['kind'] in {'Inserted', 'Unchanged', 'rust_error'}, 'selected Rust report')
            else:
                require(r['report']['kind'] == 'NotSelected', 'other handler must remain uninvoked')
            self.report_contract(r, before_nodes, after_nodes)
            phase(n['before'], r['before'], 'step_' + str(index) + '_before')
            phase(n['after'], r['after'], 'step_' + str(index) + '_after')
        if domain == 'rust-restricted':
            require(outcome['kind'] == 'rust_error' and outcome['published_arena'] is False, 'restriction must publish no arena')
            require(all(s['report'] == {'kind': 'rust_error', 'error': 'UnsupportedOperands'} and exact(s['before'], s['after']) for s in rust['steps']), 'typed operand restriction must preserve public state')
            self.opened('root', 'truthy_nonarray_Rust_admission_policy', dict(native=native['final'], rust=outcome)); return
        require(outcome['kind'] == 'arena', 'compatible case did not finish')
        require(native['finalization']['kind'] == 'original_assignMandatoryProperties'
                and rust['finalization']['kind'] == 'Builder::finish', 'explicit completion methods')
        phase(native['after_handler'], rust['after_apply'], 'raw_after_handler')
        phase(native['final'], outcome, 'synthetic_mandatory_completion_and_kernel_finish')
        self.opened('root', 'immediate_parent_and_cache_lifecycle', 'Input creation/default stages are matched explicitly; raw after-handler and separate original mandatory completion/Rust finish stages remain retained. Immediate parent/cache descriptor lifecycle has no equivalent public mutable Rust fields.')


def compare(corpus, native, rust):
    cases = corpus['cases']; require(type(cases) is list and len(cases) == 42, '42 assigned synthetic cases')
    ids = [c['id'] for c in cases]; require(len(set(ids)) == len(ids), 'unique case IDs')
    for capture in (native, rust):
        require(type(capture.get('schema')) is int and capture['schema'] == 1, 'capture schema')
        require([c['id'] for c in capture['cases']] == ids, 'complete ordered captures')
    require(native['provenance']['bundle_sha256'] == BUNDLE and native['provenance']['corpus_sha256'] == CORPUS, 'native source/corpus provenance')
    require(exact(native['handler'], {'name': 'fixOperatorExpression', 'arity': 2}), 'registered original handler')
    rows = []
    for spec, n, r in zip(cases, native['cases'], rust['cases']):
        case = Comparison(spec)
        try: case.run(n, r)
        except Exception as e: case.diff('root', 'capture_or_schema_failure', dict(type=type(e).__name__, message=str(e)), {'comparison': 'INCOMPLETE'})
        rows.append(dict(id=spec['id'], domain=spec['domain'], differences=case.differences, open_items=case.open_items))
    return rows


def controls(corpus, native, rust):
    baseline = compare(corpus, native, rust)
    def row(doc, id): return next(r for r in doc['cases'] if r['id'] == id)
    def root(doc, id):
        r = row(doc, id)['outcome']; return next(n for n in r['nodes'] if n['id'] == r['root'])
    def false_insert(native, rust):
        steps = row(rust, 'repeated-application')['steps']
        steps[-1]['report'] = dict(steps[0]['report'], normalized_operands=False)
    mutations = [
        ('omit_case', lambda n, r: r['cases'].pop()),
        ('case_order', lambda n, r: r['cases'].reverse()),
        ('bool_handle', lambda n, r: root(r, 'classification-as').update(id=True)),
        ('missing_container', lambda n, r: root(r, 'classification-as').pop('container')),
        ('unexpected_origin', lambda n, r: root(r, 'classification-as').update(origin={'bytes': [0, 1], 'utf16': [0, 1], 'text': 'x'})),
        ('absent_vs_null', lambda n, r: root(r, 'operator-null')['fields'].pop('operator')),
        ('primitive_type', lambda n, r: root(r, 'classification-as')['fields']['operator'].update(value={'kind': 'boolean', 'value': True})),
        ('number_sign', lambda n, r: root(r, 'nonempty-primitive-array')['fields']['operands']['value']['items'][0]['value'].update(bits='0000000000000000')),
        ('array_order', lambda n, r: root(r, 'binary-order-preserved')['fields']['operands']['value']['items'].reverse()),
        ('alias_cycle', lambda n, r: root(r, 'classification-as')['fields']['operands']['value']['items'][0]['value'].update(id=root(r, 'classification-as')['id'])),
        ('void_is_not_Report', lambda n, r: row(n, 'classification-as')['steps'][0].update(native_return={'kind': 'Inserted'})),
        ('other_handler_invoked', lambda n, r: row(n, 'other-loop-not-invoked')['steps'][0].update(invoked=True)),
        ('corpus_binding', lambda n, r: n['provenance'].update(corpus_sha256='0' * 64)),
        ('inserted_normalization_flag', lambda n, r: row(r, 'classification-as')['steps'][0]['report'].update(normalized_operands=False)),
        ('unchanged_normalization_flag', lambda n, r: row(r, 'operator-absent')['steps'][0]['report'].update(normalized_operands=False)),
        ('repeated_step_fabricated_Insert', false_insert),
    ]
    passed = []
    for name, mutate in mutations:
        n, r = copy.deepcopy((native, rust)); mutate(n, r)
        try:
            result = compare(corpus, n, r)
        except (ValueError, KeyError, TypeError):
            passed.append(name)
            continue
        require(not exact(result, baseline), 'divergence control unchanged: ' + name)
        passed.append(name)
    return passed


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('corpus', 'native', 'rust', 'out'): p.add_argument('--' + name, type=Path, required=True)
    p.add_argument('--self-test', action='store_true'); a = p.parse_args(); workspace = Path('/workspace').resolve(strict=True)
    inputs = {}
    for name in ('corpus', 'native', 'rust'):
        path = getattr(a, name); require(path.is_absolute(), 'absolute input required'); path = path.resolve(strict=True)
        require(path.is_file() and path.is_relative_to(workspace), 'workspace input required'); inputs[name] = path.read_bytes()
    require(a.out.is_absolute(), 'absolute output required'); parent = a.out.parent.resolve(strict=True)
    require(parent.is_relative_to(workspace), 'workspace output parent required'); out = parent / a.out.name
    require(not out.exists() and not out.is_symlink(), 'new output leaf required')
    hashes = {k: hashlib.sha256(v).hexdigest() for k, v in inputs.items()}; require(hashes['corpus'] == CORPUS, 'assigned corpus bytes')
    corpus, native, rust = [loads(inputs[k]) for k in ('corpus', 'native', 'rust')]
    require(type(corpus['schema']) is int and corpus['schema'] == 1 and type(rust['corpus_bytes']) is int and rust['corpus_bytes'] == len(inputs['corpus']), 'corpus schema/byte count')
    rows = compare(corpus, native, rust)
    report = dict(status='OPEN_REQUIRED_OPERATOR_QUALIFICATION', case_results=rows,
        differences=[dict(case=r['id'], **d) for r in rows for d in r['differences']],
        open_items=[dict(case=r['id'], **o) for r in rows for o in r['open_items']], inputs=hashes,
        scope='Actual registered original callback and public Rust synthetic F12 scope only; no text parser, semantic/import/UI/capacity or full handler acceptance')
    report['open_items'].append(dict(kind='runtime_and_real_caller_closure', detail='Configured Node20/deployed artifact, source/binary execution receipt, full cache/P14 lifecycle and real expression/parser/importer callers remain required. Rust quota/cancel/foreign/spent-ID guarantees are separate.'))
    report['controls'] = controls(corpus, native, rust) if a.self_test else 'NOT_RUN'
    invalid = any(d['kind'] in {'capture_failure', 'capture_or_schema_failure'} for d in report['differences'])
    if invalid: report['status'] = 'FAIL_CAPTURE_OR_COMPARISON_CONTRACT'
    report['comparator_sha256'] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    with out.open('x') as stream: stream.write(json.dumps(report, indent=2, ensure_ascii=True) + '\n')
    print(json.dumps(dict(status=report['status'], cases=len(rows), differences=len(report['differences']), open_items=len(report['open_items']))))
    return 2 if invalid else 1


if __name__ == '__main__':
    try:
        code = main()
    except Exception as error:
        print(json.dumps({'status': 'FAIL_CAPTURE_OR_COMPARISON_CONTRACT',
                          'type': type(error).__name__, 'error': str(error)}))
        code = 2
    raise SystemExit(code)

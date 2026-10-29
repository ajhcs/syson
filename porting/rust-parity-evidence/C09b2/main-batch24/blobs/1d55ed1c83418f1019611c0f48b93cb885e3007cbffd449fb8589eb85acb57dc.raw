"""Compare finite P01 projections through fresh source-pinned C09a admission.

Compare the complete AST census, ordered selected names/source origins, finite
ownership and raw reference text/spans. Missing native fields remain OPEN.
Full fields/caches, reference targets/getters, lifecycle, native artifacts,
importer, UI and complete P01/C09/application parity remain unqualified.
Assigned from rejected candidate 16daa02867fcbb06cafc12f67d14c9d21f7898646392dcf15ecb12584c1d10a8.
"""
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
from types import ModuleType

CAPTURE_SHA256 = '57c141ae620d103b1fe6104286cb887a62ec8dcccaf45ed34fc6baf64985a88c'


def _require(ok, message):
    if not ok:
        raise ValueError(message)


def _load_capture(path):
    raw = path.read_bytes()
    _require(hashlib.sha256(raw).hexdigest() == CAPTURE_SHA256,
             'capture admission source hash mismatch')
    module = ModuleType('p01_pinned_capture')
    module.__file__ = str(path)
    exec(compile(raw, str(path), 'exec'), module.__dict__)
    return module


def _native_projection(case, tables, capture):
    source = case['source']
    objects, cst, fields, root = tables
    open_items, selected, origins, references = [], [], [], []

    def missing(path):
        label = case['id'] + '/objects/' + path
        if label not in open_items:
            open_items.append(label)
        return {'unavailable': label}

    def span(ref, path):
        if ref is None or ref == capture.UNEVALUATED or ref == {'kind': 'undefined'}:
            return missing(path)
        _require(type(ref) is dict and set(ref) == {'$cst'}, 'native origin reference shape')
        return capture.native_span(source, cst[ref['$cst']]['span'])

    def origin(obj):
        return span(fields[obj['id']].get('$cstNode'), obj['id'] + '/$cstNode')

    def array(values, key, path):
        if key not in values or values[key] == capture.UNEVALUATED:
            return missing(path)
        value = values[key]
        if value is None or value == {'kind': 'undefined'}:
            return None
        _require(type(value) is dict and set(value) == {'$object'}, 'native array reference shape')
        obj = objects[value['$object']]
        _require(obj['kind'] == 'array', 'native array object kind')
        data = fields[obj['id']]
        length = data.get('length')
        _require(type(length) is int and length >= 0 and
                 set(data) == {'length'} | {str(i) for i in range(length)}, 'native array inventory')
        return [data[str(i)] for i in range(length)]

    for obj in objects.values():
        if obj['kind'] != 'ast':
            continue
        oid, values = obj['id'], fields[obj['id']]
        if obj['node_type'] in capture.SELECTED:
            name = values.get('declaredName')
            node_origin = origin(obj)
            name_origin = None
            if name == capture.UNEVALUATED:
                name_units = missing(oid + '/declaredName')
                name_origin = missing(oid + '/declaredName/source')
            elif name is None or name == {'kind': 'undefined'}:
                name_units = None
            else:
                _require(type(name) is str, 'native selected name schema')
                name_units = capture.units(name)
                candidates = []
                if 'unavailable' not in node_origin:
                    for node in cst.values():
                        bounds = node['span']
                        if ('_tokenType' in capture.props(node) and not bounds['hidden'] and
                                bounds['text'] == name and node_origin['utf16'][0] <=
                                bounds['offset'] <= bounds['end'] <= node_origin['utf16'][1]):
                            candidates.append(capture.native_span(source, bounds))
                name_origin = (min(candidates, key=lambda value: value['utf16']) if candidates
                               else missing(oid + '/declaredName/source'))
            selected.append({'type': obj['node_type'], 'name_units': name_units,
                             'name_origin': name_origin})
            if oid != root['id']:
                origins.append({'type': obj['node_type'], 'origin': node_origin})
        if obj['node_type'] == 'TypeReference':
            parts = array(values, 'parts', oid + '/parts')
            row = {'origin': origin(obj), 'parts': [] if type(parts) is list else parts}
            for index, part in enumerate(parts if type(parts) is list else []):
                path = oid + '/parts/' + str(index)
                _require(type(part) is dict and set(part) == {'$object'}, 'native reference part shape')
                ref = objects[part['$object']]
                _require(ref['kind'] == 'reference', 'native reference part kind')
                data = fields[ref['id']]
                if '$refText' not in data or data['$refText'] == capture.UNEVALUATED:
                    text = missing(path + '/$refText')
                elif data['$refText'] is None or data['$refText'] == {'kind': 'undefined'}:
                    text = None
                else:
                    _require(type(data['$refText']) is str, 'native raw reference text schema')
                    text = capture.units(data['$refText'])
                row['parts'].append({'text_units': text,
                                     'origin': span(data.get('$refNode'), path + '/$refNode')})
            references.append(row)
    projection = {'selected': selected, 'selected_node_origins': origins,
                  'root_origin': origin(root), 'references': references,
                  'ast_type_census': dict(sorted(Counter(obj['node_type'] for obj in objects.values()
                                                       if obj['kind'] == 'ast').items()))}
    return projection, open_items, [obj for obj in objects.values() if obj['kind'] == 'reference']


def _rust_projection(case, capture):
    arena = case['outcome']
    nodes = {node['id']: node for node in arena['nodes']}
    selected, origins, references, incoming = [], [], [], {ident: [] for ident in nodes}

    def datum(value, owner, field, index=None):
        item = value['value']
        if item['kind'] == 'node':
            incoming[item['id']].append({'node': owner, 'property': field, 'index': index})
        elif item['kind'] == 'array':
            for position, child in enumerate(item['items']):
                datum(child, owner, field, position)

    for node in nodes.values():
        for name, field in node['fields'].items():
            datum(field, node['id'], name)
    for ident, node in nodes.items():
        _require((node['container'] is None and not incoming[ident]) if ident == arena['root']
                 else incoming[ident] == [node['container']], 'Rust finite ownership/containment mismatch')
    pending, reached = [arena['root']], set()
    children = {ident: [] for ident in nodes}
    for ident, owners in incoming.items():
        for owner in owners:
            children[owner['node']].append(ident)
    while pending:
        ident = pending.pop()
        _require(ident not in reached, 'Rust finite ownership cycle/alias')
        reached.add(ident)
        pending.extend(children[ident])
    _require(reached == set(nodes), 'Rust finite unreachable node')
    for node in nodes.values():
        if node['node_type'] in capture.SELECTED:
            field = node['fields'].get('declaredName')
            _require(field is None or field['value']['kind'] == 'string', 'Rust selected name schema')
            selected.append({'type': node['node_type'], 'name_units': field['value']['units'] if field else None,
                             'name_origin': field['lexeme'] if field else None})
            if node['id'] != arena['root']:
                origins.append({'type': node['node_type'], 'origin': node['origin']})
        if node['node_type'] == 'TypeReference':
            field = node['fields'].get('parts')
            _require(field is not None and field['value']['kind'] == 'array', 'Rust TypeReference parts schema')
            row = {'origin': node['origin'], 'parts': []}
            for part in field['value']['items']:
                _require(part['value']['kind'] == 'unresolved_reference' and part['lexeme'] is not None,
                         'Rust raw reference/source lexeme unavailable')
                row['parts'].append({'text_units': capture.units(part['value']['text']), 'origin': part['lexeme']})
            references.append(row)
    return {'selected': selected, 'selected_node_origins': origins,
            'root_origin': nodes[arena['root']]['origin'], 'references': references,
            'ast_type_census': dict(sorted(Counter(node['node_type'] for node in nodes.values()).items()))}, [
                node for node in nodes.values() if node['node_type'] == 'TypeReference']


def _parse_error(source, value):
    unsupported = re.fullmatch(r'Unsupported \{ offset: ([0-9]+) \}', value)
    malformed = re.fullmatch(r'Malformed \{ offset: ([0-9]+), expected: ("(?:[^"\\]|\\.)*") \}', value)
    if unsupported or malformed:
        match = unsupported or malformed
        offset = int(match[1])
        _require(offset <= len(source.encode()), 'Rust error offset outside source')
        source.encode()[:offset].decode()
        return ('Unsupported' if unsupported else 'Malformed', offset,
                json.loads(malformed[2]) if malformed else None)
    _require(value in {'Limit', 'Cancelled', 'InvalidLimits', 'Metadata', 'Conversion'},
             'malformed or unrecognized Rust error payload')
    return (value, None, None)


def _available_differences(expected, actual, path):
    if (type(expected) is dict and set(expected) == {'unavailable'} and
            type(expected['unavailable']) is str):
        return []
    if type(expected) is not type(actual):
        return [path]
    if type(expected) is dict:
        differences = [path + '/keys'] if set(expected) != set(actual) else []
        for key in expected:
            if key in actual:
                differences.extend(_available_differences(expected[key], actual[key], path + '/' + str(key)))
        return differences
    if type(expected) is list:
        differences = [path + '/length'] if len(expected) != len(actual) else []
        for index, (left, right) in enumerate(zip(expected, actual)):
            differences.extend(_available_differences(left, right, path + '/' + str(index)))
        return differences
    return [] if expected == actual else [path]


def _compare_context(context, capture):
    report = []
    for native, rust in zip(context['native']['cases'], context['rust']['cases']):
        cid = native['id']
        np, unavailable, refs = _native_projection(native, context['native_tables'][cid], capture)
        if cid == 'quoted-error-offset':
            unavailable.append('Pinned native recovery columns lag by one at UTF-16 offsets 17, 18, 32; coordinate parity remains OPEN')
        row = {'id': cid, 'native_lexer_errors': native['lexer_errors'],
               'native_parser_errors': native['parser_errors'], 'native_references_raw': refs,
               'native_projection': np, 'differences': [], 'difference_paths': [], 'open_items': unavailable}
        recovery = bool(native['lexer_errors'] or native['parser_errors'])
        outcome = rust['outcome']
        if outcome['kind'] == 'arena':
            rp, raw_refs = _rust_projection(rust, capture)
            row.update(rust_projection=rp, rust_references_raw=raw_refs)
            for key in np:
                paths = _available_differences(np[key], rp[key], key)
                if paths:
                    row['differences'].append(key)
                    row['difference_paths'].extend(paths)
            if refs or raw_refs:
                row['open_items'].append('Reference target types, getters and lifecycle are not qualified by raw text/CST projection')
            if recovery:
                row['differences'].append('Rust_published_native_recovered_input')
        else:
            parsed = _parse_error(native['source'], outcome['error'])
            row['rust_error'] = outcome['error']
            if not recovery and capture.RESTRICTIONS.get(cid) == parsed:
                row['open_reason'] = 'P01 explicit semantic-comment/multiline-note restriction'
            elif recovery and capture.RECOVERIES.get(cid) == parsed:
                row['open_reason'] = 'Native recovered diagnostics; exact known Rust finite refusal'
            else:
                row['differences'].append('unexpected_Rust_refusal')
        row['status'] = ('DIVERGED' if row['differences'] else 'OPEN' if row.get('open_reason') or
                         row['open_items'] else 'COMPATIBLE_SELECTED_PROJECTION')
        report.append(row)
    return {'schema': 1, 'scope': __doc__, 'cases': report,
            'difference_count': sum(len(row['differences']) for row in report),
            'divergent_cases': sum(row['status'] == 'DIVERGED' for row in report),
            'full_parity_claimed': False}


def compare(corpus_raw, native_raw, rust_raw, receipt_raw, constructor_path, capture_path):
    """Readmit four byte inputs on every call; return a finite comparison report.

    Sources execute only after their exact byte hashes match. Receipt declarations
    lack an output digest; independently review actual artifact/stdout causality.
    """
    path = Path(capture_path)
    capture = _load_capture(path)
    context = capture.admit(corpus_raw, native_raw, rust_raw, receipt_raw, constructor_path)
    try:
        result = _compare_context(context, capture)
    except (KeyError, TypeError, AttributeError, IndexError, RecursionError) as error:
        raise ValueError('malformed projection shape: ' + str(error)) from error
    result.update(native_provenance=context['native']['provenance'],
                  qualification_binding=context['qualification_binding'],
                  input_sha256=context['input_sha256'],
                  capture_module={'path': str(path), 'sha256': CAPTURE_SHA256},
                  constructor_comparator=context['constructor_comparator'])
    return result

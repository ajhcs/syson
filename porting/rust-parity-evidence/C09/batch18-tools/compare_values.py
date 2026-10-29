"""Compare exact typed public/native values; restrictions remain separate."""
import argparse, copy, json
from pathlib import Path

def same(left, right):
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return left.keys() == right.keys() and all(same(left[k], right[k]) for k in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(same(a, b) for a, b in zip(left, right))
    return left == right

def compare(corpus, native, rust):
    cases = corpus['cases']
    ids = [c['id'] for c in cases]
    assert len(set(ids)) == len(ids)
    assert [r['id'] for r in rust['observations']] == ids
    comparable = [c['id'] for c in cases if not c.get('expected_restriction')]
    assert [r['id'] for r in native['observations']] == comparable
    rows = {r['id']: r['value'] for r in rust['observations']}
    differences = [dict(id=r['id'], native=r['value'], rust=rows[r['id']])
                   for r in native['observations'] if not same(r['value'], rows[r['id']])]
    restrictions = []
    expected = [c for c in cases if c.get('expected_restriction')]
    assert [r['id'] for r in native['restrictions']] == [c['id'] for c in expected]
    for c, n in zip(expected, native['restrictions']):
        assert rows[c['id']] == {'kind': 'rust_error', 'name': c['expected_restriction']}
        restrictions.append(dict(id=c['id'], native=n['native_value'], rust=rows[c['id']]))
    return dict(comparable_cases=len(comparable), differences=differences, restrictions=restrictions)

def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ['corpus', 'native', 'rust', 'out']:
        p.add_argument('--'+name, type=Path, required=True)
    a = p.parse_args()
    corpus, native, rust = [json.loads(x.read_text()) for x in [a.corpus, a.native, a.rust]]
    result = compare(corpus, native, rust)
    # Refuse boolean/number coercion, changed bits, UTF-16 contents and ordering.
    controls = [({'kind':'boolean','value':True}, {'kind':'boolean','value':1}),
                ({'kind':'number','bits':'8000000000000000'}, {'kind':'number','bits':'0000000000000000'}),
                ({'kind':'string','units':[0xd83d]}, {'kind':'string','units':[0xfffd]}),
                ([1,2], [2,1])]
    assert all(not same(x,y) for x,y in controls)
    result['negative_controls'] = 'PASS_4_TYPED_DIVERGENCE_CONTROLS'
    result['scope'] = 'Selected original converter/name methods on recorded Node; no parser, importer, UI or native deployment closure.'
    with a.out.open('x') as f:
        json.dump(result, f, indent=2); f.write('\n')
    print(json.dumps({'comparable_cases':result['comparable_cases'], 'differences':len(result['differences']), 'restrictions':len(result['restrictions'])}))
    raise SystemExit(1 if result['differences'] else 0)

if __name__ == '__main__':
    main()

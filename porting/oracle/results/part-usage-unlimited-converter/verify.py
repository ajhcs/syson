#!/usr/bin/env python3
"""Verify the exact [*] converter complement to the retained Rust evidence."""
from hashlib import sha256
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCE = 'package P { part def System; part systems : System [*]; }'
BASE = 'dc87302a448ee3423094a78bd840c900031cde5f'
FILES = ('cases.jsonl', 'receipt.json', 'verify.py')
WIRE = ["cross-references point at: reference=['id'] rust=['elementId']"]


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def digest(data):
    return sha256(data).hexdigest()


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def walk_ast(value):
    if isinstance(value,dict):
        if '$type' in value:
            yield value
        for child in value.values():
            yield from walk_ast(child)
    elif isinstance(value,list):
        for child in value:
            yield from walk_ast(child)


def verify(receipt, raw, base_files, parent, compare):
    require(receipt['schema'] == 1 and receipt['scope'] == 'single exact [*] release converter and retained Rust snapshots',
            'evidence scope')
    require(receipt['base_pull_request'] == 332 and receipt['base_commit'] == BASE, 'exact stacked parent')
    require(set(base_files) == set(FILES) and
            {name: digest(data) for name, data in base_files.items()} == receipt['base_files_sha256'],
            'exact retained parent files')
    require(receipt['release_sha256'] ==
            '85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1' and
            receipt['source_revision'] == 'b380dada415ce05155150f23cb3b74ae21cc45e4', 'release identity')
    require(receipt['java_sha256'] == '999c141a6e620c71aa4230ccb62341a89162f37818753ff006d766bf935dcbe1' and
            receipt['ast_cli_sha256'] == '47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd' and
            receipt['class_sha256'] == 'd7fa46a62f36b11e4ca96fba5678180fb434111b85487da90ad43a3e7b29bc97' and
            receipt['classpath_entries'] == 312 and receipt['classpath_manifest_sha256'] ==
            'cd366f59b021d835a0a293a8daa3daaab5445d365933690886e7d376d6255da4', 'exact retained JVM inputs')
    require(receipt['converter_terminal'] == 0 and receipt['scope_priority_nice'] == 10 and
            receipt['rust_reexecuted'] is False and receipt['converter_only'] is True and receipt['app_http'] == 'open',
            'converter-only execution and prior Rust qualification')
    require(receipt['cleanup'] == dict(scope='inactive',scope_processes=0,db_state_present=False), 'runtime cleanup')
    previous = json.loads(base_files['receipt.json'])
    history = receipt['history']
    require(previous['native_fresh_case'] == 'open' and previous['native_attempts'] == history['original_http_attempts'] and
            [entry['terminal'] for entry in history['original_http_attempts']] == [1,88] and
            history['latest_http_terminal'] == history['stopped_converter_terminal'] == 88 and
            history['latest_http_result'] == 'no model', 'unsuccessful prior attempts remain recorded')
    require(history['latest_http_sha256'] == '49bdc4da681decf35b8528dc7358c7d2224b424be9d87e14396b2219ef1279ce' and
            history['stopped_converter_sha256'] ==
            'cc2813ba8b78ee54ccdc5e1eb35b302c8369ca572a4ab42ac87e4c641558637c', 'prior receipt pins')
    require(receipt['case_sha256'] == digest(raw), 'single case manifest digest')
    case = json.loads(raw)
    for text, checksum in [('source','source_sha256'),('ast_java_input','ast_sha256'),('model_emf','model_sha256'),
                           ('facts_tsv','facts_sha256'),('messages_tsv','messages_sha256')]:
        require(digest(case[text].encode()) == case[checksum], 'case artifact bytes')
    require(case['source'] == SOURCE, 'exact source input')
    require(case['messages_tsv'] == '', 'zero converter messages')

    ast = json.loads(case['ast_java_input'])
    require(ast['$cstNode']['text'] == SOURCE, 'bundled AST source')
    usage, = (node for node in walk_ast(ast)
              if node['$type'] == 'PartUsage' and node.get('declaredName') == 'systems')
    outer = usage['multiplicity']
    rng = outer['target']
    member = rng['range']
    infinity = member['target']
    require([node['$type'] for node in (outer,rng,member,infinity)] ==
            ['OwningMembership','MultiplicityRange','OwningMembership','LiteralInfinity'], 'exact AST range path')
    require(rng['$cstNode']['text'] == '[*]' and infinity['$cstNode']['text'] == '*' and
            not any(node['$type'] == 'OperatorExpression' for node in walk_ast(ast)), 'exact AST star token')

    row, = (x for x in map(json.loads,base_files['cases.jsonl'].splitlines()) if x['case'] == 'unlimited')
    rust, rust_path = parent.saved(row,parent.POSITIVE['unlimited'])
    retained = receipt['retained_rust']
    require(retained['no_new_rust_execution'] is True and
            retained['binary_sha256'] == previous['binary_sha256']['probe'] and
            retained['reopen_receipt_sha256'] == history['latest_http_sha256'] and
            retained['stdout_sha256'] == row['stage_sha256'] and
            retained['snapshot_sha256'] == row['snapshot_sha256'] and
            retained['reopen_sha256'] == row['reopen_sha256'], 'exact already executed Rust snapshot')

    native = json.loads(case['model_emf'])
    use = parent.named(native,'PartUsage','systems')
    typ = parent.named(native,'PartDefinition','System')
    require(parent.project(use,typ['id'],'id') == rust_path, 'same stored Infinity path')
    outer = use['data']['ownedRelationship'][0]
    multiplicity = parent.child(outer,'OwningMembership','ownedRelatedElement')
    member = parent.child(multiplicity,'MultiplicityRange','ownedRelationship')
    bound = parent.child(member,'OwningMembership','ownedRelatedElement')
    require(all(node['eClass'] not in ('sysml:LiteralInteger','sysml:OperatorExpression')
                for node in parent.walk(native)), 'derived zero and operator absent from native serialization')
    facts = {parts[0]:parts[1:] for parts in (line.split('\t') for line in case['facts_tsv'].splitlines())}
    require(facts['library_resources'] == ['94'] and facts['imported_nodes'] == ['12'] and facts['messages'] == ['0'],
            'converter counts')
    require(facts['derived_lower'] == ['0','detached'] and
            facts['upper'] == ['*',bound['id'],bound['data']['elementId']], 'derived lower and stored upper identities')
    require(facts['part_usage'] == ['systems',use['id'],use['data']['elementId']] and
            facts['type'] == ['System',typ['id'],typ['data']['elementId']] and
            facts['range'] == [multiplicity['id'],multiplicity['data']['elementId']], 'actual converter facts')

    namespace, = native['content']
    require(namespace['eClass'] == 'sysml:Namespace' and set(namespace['data']) == {'elementId','ownedRelationship'},
            'raw unannotated root')
    outer, = namespace['data']['ownedRelationship']
    require(set(outer['data']) == {'elementId','ownedRelatedElement'}, 'exact native wrapper membership')
    package = parent.child(outer,'OwningMembership','ownedRelatedElement')
    require(package['eClass'] == 'sysml:Package' and package['data']['declaredName'] == 'P', 'exact Package projection')
    require(compare(native,rust) == receipt['comparison']['raw'] ==
            dict(structural_match=False,structural_diffs=[dict(path='$[0]',reference='sysml:Namespace',rust='sysml:Package')],
                 wire_diffs=WIRE,reference_nodes=12,rust_nodes=10), 'raw model retains Namespace difference')
    require(compare(dict(native,content=[package]),rust) == receipt['comparison']['package'] ==
            dict(structural_match=True,structural_diffs=[],wire_diffs=WIRE,reference_nodes=10,rust_nodes=10),
            'checked ten-node native Package projection')


def main():
    prior = HERE.parent/'part-usage-unlimited'
    parent = load_module('unlimited_parent_verify',prior/'verify.py')
    comparison = load_module('compare',HERE.parents[1]/'compare.py')
    verify(json.loads((HERE/'receipt.json').read_bytes()),(HERE/'case.json').read_bytes(),
           {name:(prior/name).read_bytes() for name in FILES},parent,comparison.compare)
    print('exact [*] converter matches the retained ten-node Rust Package; HTTP capture open')


if __name__ == '__main__':
    main()

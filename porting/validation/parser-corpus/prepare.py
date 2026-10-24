#!/usr/bin/env python3
"""Prepare synthetic fixture bytes and source-only assertions; never execute upstream code."""
import argparse
import hashlib
import json
import pathlib
import subprocess

PIN = '1326f234cae23b0e4777d5a48a069b17e9e332f1'
BUNDLE_PIN = '6ba2ac64e6b16ee2b05752ad55395eebb87d4910'
ROOT = pathlib.Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()

# Literal bytes are intentional: raw backslashes, real LF, mixed CRLF and U+2028.
CASES = [
 ('pc01', 'raw-names-strings.sysml', r'''package 'Quoted\n😀' { attribute 'label\'x' = "raw\ntext"; }'''+'\n', 'SysML', ['Package','AttributeUsage','Name','LiteralString','UNRESTRICTED_NAME','STRING_VALUE'], 'complete_syntax_candidate',
  'Observe raw quoted terminal spelling before later name sanitation; do not JSON-decode lexemes.',
  'Keep declared name, sanitized semantic name and CLI projection as separate observations.'),
 ('pc02', 'multiline-string.sysml', 'package Text { attribute s = "first\nsecond\\n"; }\n', 'SysML', ['Package','AttributeUsage','LiteralString','STRING_VALUE'], 'complete_syntax_candidate',
  'The terminal regex permits a literal LF inside double quotes.',
  'Observe quote retention and literal LF versus backslash-n independently; terminal conversion is not JSON decoding.'),
 ('pc03', 'number-spelling.sysml', 'package Numbers { attribute a = 1; attribute b = 1.0; attribute c = 1e0; attribute d = .5; }\n', 'SysML', ['Package','AttributeUsage','LiteralNumber','NUMBER'], 'complete_syntax_candidate',
  'Retain four distinct raw numeric spellings and CST text even if converted numbers coincide.',
  'Do not infer SysON integer/rational import classifiers from the diagnostic CLI output.'),
 ('pc04', 'number-boundaries.sysml', 'package Bounds { attribute large = 9007199254740993; attribute tiny = 1e-400; attribute huge = 1e400; attribute neg = -0; }\n', 'SysML', ['Package','AttributeUsage','LiteralNumber','NUMBER','UnaryExpression'], 'complete_syntax_candidate',
  'Observe binary64 rounding, underflow, overflow and serializer treatment; do not assume arbitrary precision or finite values.',
  'Minus is a unary expression here; -0 is not a signed NUMBER terminal. Do not infer primitive signed zero from spelling alone.'),
 ('pc05', 'malformed-number-value.sysml', 'package Odd { attribute strange = 1e++-2; }\n', 'SysML', ['Package','AttributeUsage','LiteralNumber','NUMBER','EXPONENT'], 'lexically_accepted_number_conversion_unassessed',
  'EXPONENT permits repeated signs, so lexical acceptance is separate from successful number conversion.',
  'Record conversion diagnostics/exit and any serialized value independently; no expected wire value is supplied.'),
 ('pc06', 'boolean-defaults.sysml', 'package Flags { attribute yes = true; attribute no = false; }\n', 'SysML', ['Package','AttributeUsage','LiteralBoolean'], 'complete_syntax_candidate',
  'Inspect matched ?= presence versus mandatory-boolean default construction.',
  'Do not equate LiteralBoolean assignment/default behavior with a boolean terminal converter.'),
 ('pc07', 'comments.sysml', '//* hidden block */\r\n// hidden line\u2028still same note\r\npackage Notes { /* visible comment */ doc /* body */ }\r\n', 'SysML', ['Package','Comment','Documentation','REGULAR_COMMENT','ML_NOTE','SL_NOTE','WS'], 'complete_syntax_candidate',
  'Hidden ML_NOTE/SL_NOTE/WS differ from visible REGULAR_COMMENT.',
  'Observe comment/documentation body projection separately; retain CRLF/U+2028 bytes and coordinate conventions.'),
 ('pc08', 'datatype-reference.kerml', 'namespace Types { class Vehicle; feature car typed by Vehicle; }\n', 'KerML', ['Namespace','Class','Feature','TypedByToken','OwnedFeatureTyping','TypeReference'], 'complete_syntax_candidate',
  'Exercise the reachable TypedByToken datatype fragment and source no-hidden-space accumulation contract.',
  'Record reference text, parts, resolution and projection separately; matching names alone are not proof of resolution.'),
 ('pc09', 'unresolved-reference.sysml', 'package Ref { part car : Missing; }\n', 'SysML', ['Package','PartUsage','FeatureType','TypeReference'], 'complete_syntax_candidate_semantic_diagnostics_unassessed',
  'Syntax completeness and missing-target semantic diagnostics are separate outcomes.',
  'Reference construction retains text and unresolved state; no resolved model or wire AST is assumed.'),
 ('pc10', 'actions-precedence.sysml', 'package Expr { attribute calc = 1 + 2 * 3; attribute empty = (); }\n', 'SysML', ['Package','AttributeUsage','OwnedExpression','AdditiveExpression','MultiplicativeExpression','NullExpression'], 'complete_syntax_candidate',
  'Observe action-created operand nesting, += source order and = operator assignment independently.',
  'An empty/null-expression construct is separate from absent AST or semantic null policy. No expected tree is supplied.'),
 ('pc11', 'invalid-suffix.sysml', 'package Kept { part def A; }\n}\n', 'SysML', ['Package','PartDefinition'], 'strict_full_input_syntax_rejection_candidate',
  'Record full-input syntax error separately from any recovered prefix object.',
  'Diagnostic CLI can exit before stringify; missing stdout AST does not prove that no recovered AST exists.'),
 ('pc12', 'unterminated-string.sysml', 'package Broken { attribute x = "unterminated\n', 'SysML', ['Package','AttributeUsage','STRING_VALUE'], 'lexical_error_and_recovery_unassessed',
  'Record lexer diagnostics, parser diagnostics, exit and any emitted AST separately.',
  'Do not substitute strict Rust fail-fast rejection for upstream recovery or assume a diagnostic CLI AST is available.'),
]


def build(repo):
    def git(*args):
        return subprocess.check_output(['git', '-C', str(repo), *args])
    sources = {
        'bundle': {'revision': BUNDLE_PIN, 'path':'backend/application/syson-sysml-import/src/main/resources/syside-cli.js', 'sha256':'47d8fe11fda1b58b6257463087f663be953c2ab6c2b5e1e2ccc44161ed5cd0bd', 'regions':{'conversion':'41808-41920','construction':'41020-41185','secondary_diagnostic':'114568-114629','file_provider':'113748-113778','stdlib_none_guards':'107724,107890,108109,108124,108651'}},
        'ast_plan': {'revision':git('rev-parse','0e7a1815e^{commit}').decode().strip(),'path':'porting/source-map/syside-source/AST-VALUE-CONVERSION.md'},
        'generated_grammar': {'revision':PIN,'path':'packages/syside-languageserver/src/generated/grammar.ts','sha256':'fca1a64377f64c1aefb5476e89227cbc24f064ca40b7a58d6608646cae66149d'},
        'runner': {'path':'porting/parser-oracle/run_reference.py','sha256':'074425971218521895f8dc2d8468c9791d99c30de903b4fd1d52d62914e320c7','status':'read-only reviewed; execution not performed'},
    }
    plan=sources['ast_plan'];plan['sha256']=sha(git('show',plan['revision']+':'+plan['path']))
    if sha((repo/sources['runner']['path']).read_bytes())!=sources['runner']['sha256']:raise ValueError('reviewed runner changed')
    if sha(git('show',BUNDLE_PIN+':'+sources['bundle']['path']))!=sources['bundle']['sha256']:raise ValueError('pinned bundle hash mismatch')
    document=repo/'porting/source-map/syside-source/semantic-source-documents.jsonl'
    for line in document.open():
        d=json.loads(line)
        for key,suffix in [('naming','src/model/naming.ts'),('serialization','src/utils/common.ts'),('construction_hooks','src/services/parser/parser.ts'),('document_builder','workspace/document-builder.ts'),('literal_number_model','src/model/KerML/expressions/literal-number.ts')]:
            if d['path'].endswith(suffix):
                if sha(d['source'].encode())!=d['sha256']:raise ValueError('source hash mismatch')
                sources[key]={'revision':PIN,'path':d['path'],'sha256':d['sha256'],'retained_in':'porting/source-map/syside-source/semantic-source-documents.jsonl'}
    rows=[]
    files={}
    for cid,name,text,language,rules,hypothesis,lexical,construction in CASES:
        data=text.encode('utf-8');files['fixtures/'+name]=data
        grammar=json.loads((repo/f'rust/crates/syson-syntax/data/{language.lower()}.json').read_bytes())
        names={r['name']:i for i,r in enumerate(grammar['rules'])}
        refs=[{'source':'generated_grammar','language':language,'rule':rule,'fragment':f'#/rules@{names[rule]}', 'activation':'automatic_hidden_terminal' if grammar['rules'][names[rule]].get('hidden') else 'entry_reachable'} for rule in rules]
        rows.append({'id':cid,'path':'fixtures/'+name,'language':language,'bytes':len(data),'sha256':sha(data),
          'provenance':{'kind':'new_public_synthetic','copied_upstream_fixture':False,'source_refs':refs,'source_only_hypothesis':hypothesis},
          'layer_assertions':[
            {'id':cid+'.capture','layer':'process','source_refs':['bundle'],'assertion':'Capture exact argv/input hashes/UTC/exit/timeout and raw streams. Outer runner exit is not the reference exit.'},
            {'id':cid+'.lexical-syntax','layer':'lexical_and_syntax','source_refs':['generated_grammar'],'assertion':lexical},
            {'id':cid+'.construction','layer':'construction_and_projection','source_refs':['ast_plan','bundle'],'assertion':construction},
            {'id':cid+'.semantic','layer':'semantic_diagnostics','source_refs':['document_builder','bundle'],'assertion':'Record diagnostics independently from syntax and AST. Standard library is none; semantic validity is not asserted.'}],
          'observation':{'status':'PENDING_NOT_RUN','reference_exit_code':None,'lexer_diagnostics':None,'parser_diagnostics':None,'semantic_diagnostics':None,'serialized_ast':None,'recovered_ast_availability':'UNASSESSED; diagnostic route exits before stringify on severity-one diagnostics','rust_comparison':'NOT_RUN','run_artifacts':[]},
          'expected_wire_golden':None})
    manifest={'schema_version':1,'status':'PREPARATION_ONLY_REQUIRES_REVIEW','upstream_execution':'NOT_RUN','source_rule_dispositions':'UNPORTED','strict_verified_rows_added':0,'sources':sources,
      'bounds':{'fixtures':12,'max_fixture_bytes':1024,'total_fixture_bytes':sum(r['bytes'] for r in rows),'encoding':'UTF-8','imports_included':False,'invocations':'one reviewed fixture per serial process; no batch auto-discovery','timeout_seconds_per_fixture':60,'max_serial_timeout_seconds':720,'raw_stream_byte_cap':'NOT_ENFORCED_BY_REVIEWED_RUNNER'},
      'proposed_command':{'program':'${VERIFIED_NODE24}','expected_node_version':'v24.21.0','node_binary_sha256':'7fde7b8afa198da66257f42ee2001d874c7355631e6d1579a5fb5ef1f246df4c','argv':['${PINNED_BUNDLE}','dump','--validate','--stdlib','none','${CORPUS_ROOT}/${case.path}'],'mode':'secondary diagnostic path; distinct from primary Java import dump defaults','boundary':'default managed network-restricted execution; not unsandboxed escalation','authorization':'this preparation does not authorize execution'},'fixtures':rows}
    files['manifest.json']=(json.dumps(manifest,ensure_ascii=False,sort_keys=True,indent=2)+'\n').encode()
    return files


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--repository',type=pathlib.Path,required=True)
    p.add_argument('--check',action='store_true')
    args=p.parse_args();files=build(args.repository)
    for name,data in files.items():
        path=ROOT/name
        if args.check:
            if path.read_bytes()!=data:raise ValueError('prepared bytes differ: '+name)
        else:
            path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)
    print(json.dumps({'status':'CHECKED' if args.check else 'PREPARED','fixtures':len(CASES),'upstream_execution':'NOT_RUN'}))


if __name__=='__main__':main()

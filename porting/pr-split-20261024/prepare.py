#!/usr/bin/env python3
"""Prepare a lossless, source-pinned PR decomposition without changing Git refs/index."""
import hashlib
import json
import pathlib
import posixpath
import subprocess
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = pathlib.Path(__file__).resolve().parent
PIN = 'ad814c232ee24109639c1d52239cbeec6f4a3e92'
BASE = '6ba2ac64e6b16ee2b05752ad55395eebb87d4910'

def git(*args):
    return subprocess.check_output(['git', '-C', str(ROOT), *args])

def tree(ref):
    result = {}
    for row in git('ls-tree', '-rz', ref).split(b'\0'):
        if not row:
            continue
        meta, name = row.split(b'\t', 1)
        mode, kind, sha = meta.decode().split()
        result[name.decode()] = {'path': name.decode(), 'mode': mode, 'type': kind, 'sha': sha}
    return result

def content(entry):
    return git('cat-file', 'blob', entry['sha'])

SPECS = [
    ('S01', 'Pinned source and dependency evidence', [], 'evidence', 'Source inventories and reproducible extractors; generated captures are evidence, not implemented behavior.'),
    ('S02', 'Rust dependency notices and byte-preservation settings', [], 'foundation', 'Retain dependency license texts, ignore rules and exact transcript line-ending attributes.'),
    ('S03', 'Domain naming, identities and shared query algorithms', ['S02'], 'domain', 'Source-derived naming, UUID and ordered lazy traversal contracts; no native object runtime.'),
    ('S04', 'Diagram rotation and bounded geometry algorithms', ['S02'], 'editor', 'Pure rotation/layout-result merge, leaf geometry and note appearance; no layout engine or full renderer.'),
    ('S05', 'Standard-library ID migration algorithms', ['S02'], 'persistence', 'Map parsing, reference/diagram migration decisions and export filter; no database compatibility.'),
    ('S06', 'Static metamodel registry and generator', ['S02'], 'model', 'Pinned Ecore declarations, inheritance and feature queries; generated metadata is separate from semantic operations.'),
    ('S07', 'Transactional model editing and typed scalar capabilities', ['S03', 'S06'], 'model', 'Historical coherent model snapshot ae1fc55be: stable JSON identities, local transactions, scalar editing and selected constructors.'),
    ('S08', 'Default-off raw factory and selected graph admission', ['S07'], 'model', 'Final model delta: factory candidate, containment, selected scalar admission and PortDefinition reference API. Independent factory acceptance remains pending.'),
    ('S09', 'Grammar-driven syntax recognizer and independent regressions', ['S01', 'S02'], 'parser', 'Lexer/CST recognition with budgets and cancellation; AST and semantic import are not performed.'),
    ('S10', 'Transactional AST operation kernel and independent regressions', ['S01', 'S02'], 'parser', 'Owner-qualified handles, defaults, operation ordering and rollback; not a grammar-to-AST adapter.'),
    ('S11', 'Parser reference corpus and native observation tooling', ['S09'], 'validation', 'Preserved reference/native observations and observer CLI; no new reference execution or parity promotion.'),
    ('S12', 'Ordered declarative initializer planner', ['S06'], 'model', 'Source-derived schedules and explicit context gaps; graph effects are not applied by this crate.'),
    ('S13', 'Atomic initialized graph construction', ['S08', 'S12'], 'model', 'Coherent graph snapshot 65404593b: local naming, staging, Connection/Interface/AcceptAction/Port constructors and their tests.'),
    ('S14', 'Resource-scoped native query adapter', ['S13'], 'domain', 'Final graph delta: ordered immutable resources, scoped identity and source-derived effective-name traversal; imports remain supplied observations.'),
    ('S15', 'Connected workflow and immutable local snapshots', ['S04', 'S05', 'S14'], 'workflow', 'Caller-owned initialized transactions, local history and selected JSON save/reload; not PostgreSQL or durable collaborative history.'),
    ('S16', 'Rust WASM authoring UI and browser checks', ['S15'], 'frontend', 'Properties, supported construction, staging/history and import/export UI; raw-factory workflow stays opt-in.'),
    ('S17', 'Behavior and execution-backlog integrity gates', ['S01'], 'validation', 'Python consistency checkers, schema/fixtures and historical serialized backlog; existing held leases are preserved.'),
    ('S18', 'Integrated independent Rust regression harness', ['S10', 'S11', 'S16', 'S17'], 'validation', 'Cross-crate regression tests and integration runners; preserve original executed source/harness attribution.'),
    ('S19', 'Backend, history, recovery and interchange source inventories', ['S01'], 'evidence', 'Candidate contracts for unported areas; no backend implementation, runtime execution or inventory closure.'),
    ('S20', 'Historical integration checkpoints and receipt archive', ['S18', 'S19'], 'evidence', 'Remaining immutable checkpoints, run logs and coordination artifacts; these historical receipts do not qualify the new split commits.'),
]

def owner(path):
    if path.startswith('porting/source-map/'):
        return 'S01'
    if path in ['.gitattributes', 'rust/.gitignore', 'porting/.gitignore', 'rust/THIRD-PARTY-NOTICES.md'] or path.startswith('rust/third-party/'):
        return 'S02'
    if path.startswith('rust/crates/syson-domain/') or path in ['porting/domain/README.md', 'porting/domain/source-sha256.txt']:
        return 'S03'
    if path.startswith('rust/crates/syson-editor/') or path in ['porting/frontend/README.md', 'porting/frontend/source-sha256.txt']:
        return 'S04'
    if path.startswith(('rust/crates/syson-persistence/', 'porting/persistence/')):
        return 'S05'
    if path.startswith('rust/crates/syson-metamodel/') or path in ['porting/domain/METAMODEL.md', 'porting/validation/M3-REGISTRY-INDEPENDENT-REVIEW.md', 'porting/validation/M3-REGISTRY-SUMMARY.json']:
        return 'S06'
    if path.startswith('rust/crates/syson-model/'):
        return 'S08'
    if path in ['porting/domain/CREATION-CAPABILITIES.md', 'porting/domain/MODEL-ADAPTER.md', 'porting/domain/MODEL-DEPENDENCIES.md', 'porting/domain/MODEL-SCALARS.md', 'porting/validation/M4-SCALAR-INDEPENDENT-REVIEW.md', 'porting/validation/M6-CONSTRUCTION-TEXT-INDEPENDENT-REVIEW.md', 'porting/validation/fixtures/source-membership-matrix.json']:
        return 'S07'
    if path.startswith(('porting/domain/factory-preparation/', 'porting/domain/containment-adapter/', 'porting/area-inventories/MODEL/')):
        return 'S08'
    if path.startswith(('rust/crates/syson-syntax/', 'rust/validation/syntax/', 'porting/syntax/')) or path in ['porting/validation/M7-INDEPENDENT-REVIEW.md', 'porting/validation/fixtures/parser-oracle.sysml', 'porting/validation/fixtures/syntax-terminal-base.json']:
        return 'S09'
    if path.startswith(('rust/crates/syson-ast/', 'rust/validation/ast/', 'porting/ast/', 'porting/validation/ast-review-01/')) or path == 'porting/validation/M8-INDEPENDENT-REVIEW.md':
        return 'S10'
    if path.startswith(('rust/tools/syntax-observe/', 'porting/parser-oracle/', 'porting/validation/parser-corpus/', 'porting/runs/parser-', 'porting/runs/syntax-review-01/')) or path == 'porting/validation/run_syntax_review.py':
        return 'S11'
    if path.startswith(('rust/crates/syson-initializer/', 'porting/domain/initializer-planner/', 'porting/runs/initializer-planner-01/')):
        return 'S12'
    if path.startswith('rust/crates/syson-initializer-graph/'):
        return 'S14'
    if path.startswith(('porting/domain/initializer-graph/', 'porting/domain/accept-action-native/', 'porting/domain/interface-definition-native/', 'porting/domain/port-definition-native/', 'porting/runs/initializer-containment-01/', 'porting/runs/initializer-graph-01/', 'porting/runs/initializer-naming-01/', 'porting/runs/connection-definition-factory-helper-01/', 'porting/pending-publication/initializer-containment-01/')):
        return 'S13'
    if path.startswith(('porting/domain/native-query/', 'porting/runs/native-query-20261024/')):
        return 'S14'
    if path.startswith('rust/crates/syson-workflow/') or path == 'porting/CONNECTED-WORKFLOW.md':
        return 'S15'
    if path.startswith(('rust/crates/syson-web/', 'rust/browser-tests/', 'porting/restarts/2026-10-24/')) or path in ['porting/frontend/web.md', 'porting/validation/M5-UI-CAPABILITY-REVIEW.md', 'porting/run_initialized_workflow_checks.py']:
        return 'S16'
    if path.startswith(('porting/ledger-schema/', 'porting/validation/execution-plan-review/', 'porting/runs/execution-plan-01/')) or path in ['porting/behavior-ledger.json', 'porting/execution-backlog.json', 'porting/PARITY-LEDGER.json', 'porting/EXECUTION-PLAN.md', 'porting/MILESTONES.json', 'rust/validation/scripts/check_behavior_ledger.py', 'rust/validation/scripts/check_execution_backlog.py', 'rust/validation/scripts/check_metamodel_inventory.py'] or (path.startswith('rust/validation/tests/') and path.endswith('.py')):
        return 'S17'
    if path.startswith('rust/validation/') or path in ['porting/run_integration_checks.py', 'porting/domain/VALIDATION.md', 'porting/validation/M1-INDEPENDENT-REVIEW.md', 'porting/validation/M2-INDEPENDENT-REVIEW.md']:
        return 'S18'
    if path.startswith('porting/area-inventories/'):
        return 'S19'
    return 'S20'

def prepare():
    assert git('rev-parse', PIN).decode().strip() == PIN, 'Pinned source object is unavailable.'
    final, baseline = tree(PIN), tree(BASE)
    changed = {p: e for p, e in final.items() if e != baseline.get(p)}
    assert not set(baseline).difference(final), 'Deletion handling requires explicit review.'
    records = {}
    for sid, title, deps, lane, scope in SPECS:
        records[sid] = dict(id=sid, title=title, dependencies=deps, lane=lane, scope=scope,
                            branch=f'codex/syson-split-{sid.lower()}-20261024', entries=[])
    # Keep two intermediate coherent snapshots to isolate the large raw factory
    # and the later ResourceSet adapter without editing implementation bytes.
    for sid, ref, prefix in [('S07', 'ae1fc55be', 'rust/crates/syson-model/'), ('S13', '65404593b', 'rust/crates/syson-initializer-graph/')]:
        revision = git('rev-parse', ref).decode().strip()
        records[sid]['intermediate_source_revision'] = revision
        records[sid]['entries'] += [e for p, e in tree(revision).items() if p.startswith(prefix)]
    for path, entry in changed.items():
        records[owner(path)]['entries'].append(entry)

    overlays = {}
    def closure(sid):
        result = {}
        for dep in records[sid]['dependencies']:
            for path, entry in overlays[dep].items():
                if path in result and result[path] != entry:
                    raise AssertionError(f'{sid}: conflicting dependency versions: {path}')
                result[path] = entry
        return result

    all_applied = dict(baseline)
    dependency_checks, include_checks = 0, 0
    import re
    for sid, r in records.items():
        before = closure(sid)
        own = {e['path']: e for e in r['entries']}
        r['entries'] = [e for p, e in sorted(own.items()) if before.get(p, baseline.get(p)) != e]
        after = {**before, **{e['path']: e for e in r['entries']}}
        overlays[sid] = after
        all_applied.update({e['path']: e for e in r['entries']})
        available = {**baseline, **after}
        for path, entry in after.items():
            if path.endswith('Cargo.toml'):
                data = tomllib.loads(content(entry).decode())
                sections = [data.get(k, {}) for k in ['dependencies', 'dev-dependencies', 'build-dependencies']]
                for target in data.get('target', {}).values():
                    sections += [target.get(k, {}) for k in ['dependencies', 'dev-dependencies', 'build-dependencies']]
                for section in sections:
                    for dep in section.values():
                        if isinstance(dep, dict) and 'path' in dep:
                            target = posixpath.normpath(posixpath.join(posixpath.dirname(path), dep['path'], 'Cargo.toml'))
                            assert target in available, f'{sid}: missing dependency {path} -> {target}'
                            dependency_checks += 1
            if path.endswith('.rs'):
                source = content(entry).decode()
                for match in re.finditer(r'include_(?:str|bytes)!\(\s*"([^"\n]+)"\s*\)', source):
                    target = posixpath.normpath(posixpath.join(posixpath.dirname(path), match[1]))
                    assert target in available, f'{sid}: missing included fixture {path} -> {target}'
                    include_checks += 1
        r['changed_files'] = len(r['entries'])
        r['source_lines'] = sum(content(e).count(b'\n') for e in r['entries'] if e['path'].endswith(('.rs', '.py', '.cjs', '.js')))
        r['total_bytes'] = sum(len(content(e)) for e in r['entries'])
    assert all_applied == final, 'Split does not reconstruct the original frozen tree exactly.'
    manifest = dict(schema_version=1, source_pr='https://github.com/ajhcs/syson/pull/1',
                    source_revision=PIN, upstream_baseline=BASE, baseline_tree=git('rev-parse', BASE+'^{tree}').decode().strip(),
                    source_changed_files=len(changed), units=list(records.values()),
                    validation=dict(exact_final_tree_reconstruction=True, path_dependency_checks=dependency_checks,
                                    literal_include_checks=include_checks, implementation_bytes_edited=False,
                                    new_native_or_browser_tests_run=False, historical_receipts_requalified=False))
    (OUT/'existing-split.json').write_text(json.dumps(manifest, indent=2)+'\n')
    for r in records.values():
        print(r['id'], r['changed_files'], r['source_lines'], r['title'])
    print(json.dumps(manifest['validation']))

if __name__ == '__main__':
    prepare()

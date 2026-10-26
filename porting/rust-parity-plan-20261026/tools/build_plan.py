#!/usr/bin/env python3
"""Normalize detailed lane briefs and generate a dependency-checked PR handbook."""
import argparse
import collections
import hashlib
import json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--plan-dir',type=Path,default=Path(__file__).resolve().parent.parent)
PLAN = parser.parse_args().plan_dir.resolve()
ROOT = PLAN.parents[1]
UNITS = []
for file in sorted((PLAN/'lanes').glob('*.json')):
    data = json.loads(file.read_text())
    for raw in data.get('units', data.get('prs', data.get('records', []))):
        unit = dict(raw)
        unit['dependencies'] = unit.get('dependencies', unit.get('deps', []))
        unit['paths'] = [p.split(' (')[0] for p in unit.get('paths', unit.get('write_paths', []))]
        unit['external_pr_details'] = unit.get('external_prs', [])
        unit['external_prs'] = [int((x.get('url') if isinstance(x,dict) else str(x)).rstrip('/').split('/')[-1].replace('#','')) for x in unit.get('external_prs', [])]
        unit['lane'] = unit.get('lane', data.get('lane', file.stem))
        unit['source_fragment'] = 'lanes/'+file.name
        unit['source_fragment_sha256'] = hashlib.sha256(file.read_bytes()).hexdigest()
        UNITS.append(unit)
UNITS.append(dict(id='C99', title='Accept complete matched SysON parity and Rust release closure', lane='acceptance',
 dependencies=[u['id'] for u in UNITS], external_prs=[66],
 paths=['porting/rust-parity-evidence/final-acceptance/'],
 source_refs=['porting/PARITY-LEDGER.json','porting/source-map/behavior-boundaries.md'],
 tasks=['Require a terminal reviewed disposition for every primary and discovered obligation; required unsupported, unknown, unrun, candidate-only or preparation-only entries block acceptance.',
        'Verify matched native and Rust artifacts, full production-language/process closure, all view/user-workflow evidence and capacity/usability/recovery results on the exact composed tree.',
        'Rebuild the release candidate cleanly, verify dependency/notices and archive migration/reload, and independently inspect all intentional differences.',
        'Publish explicit feature/data/API/UI/runtime compatibility evidence and remaining nonparity limits; deployment remains a separate action.'],
 verification=['The completion validator must fail when any required obligation or child is unresolved and when an evidence hash/revision is stale.',
               'Run full composed acceptance and release checks through C04-C09 and the lane qualification suites; no summary PASS substitutes for the individual evidence.',
               'Independent reviewer confirms the actual input/artifact closure, user journeys and no application Java/Node/Python/handwritten-JS fallback.'],
 commit_plan=['Freeze immutable final candidate and acceptance manifest.','Record full evidence, independent disposition and exact qualification boundaries.'],
 budget='Acceptance/evidence PR only; repairs found here get separate bounded implementation PRs and reopen this gate.',
 done_when='Every required parity obligation is accepted on the same exact target/candidate closure and all release gates pass; otherwise the plan remains incomplete.',
 state='PLANNED_BLOCKED_ON_ALL_REMAINING_UNITS',source_fragment='tools/build_plan.py',source_fragment_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()))
assert len(UNITS) == len({u['id'] for u in UNITS}), 'Duplicate unit ID'
byid = {u['id']: u for u in UNITS}
for u in UNITS:
    assert u['title'] and u['tasks'] and u['verification'] and u['paths'], u['id']
    for dep in u['dependencies']:
        assert dep in byid, (u['id'], 'Undefined dependency', dep)
        assert dep != u['id'], u['id']

# Root-reviewed cross-lane integration edges are explicit and separate from lane snapshots.
edges = {
 'B25a':['B25S'], 'B25b':['B25S'], 'B25c':['B25S'], 'B25d':['B25S'],
 'M15':['C01g'], 'B23':['C01h'],
 'U03':['B08','B23'], 'U04':['B19'], 'U05':['B21'],
 'U07':['B11','B12'], 'U08':['B13','B17','B18'],
 'U09':['B20'], 'U16':['B14'], 'U21':['B22'],
 'U22':['B22'], 'U24':['B24'], 'U26':['B15','B16'],
}
for uid, deps in edges.items():
    for dep in deps:
        assert uid in byid and dep in byid, (uid,dep)
        if dep not in byid[uid]['dependencies']:
            byid[uid]['dependencies'].append(dep)
            byid[uid].setdefault('coordinator_added_dependencies', []).append(dep)
for parent, children in {'C01':['C01a','C01b','C01c','C01d','C01e','C01f','C01g','C01h'], 'C02':['C02a'], 'C03':['C03a','C03b','C03c']}.items():
    byid[parent]['acceptance_children'] = children
    byid[parent]['bounded_child_disposition'] = 'Selected evidence tools only; all original parent tasks and downstream parent gates remain required.'
providers = [u['id'] for u in UNITS if u['id'].startswith('V')]
for uid in ['U18','U19','U20']:
    byid[uid]['dependencies'] += [d for d in providers if d not in byid[uid]['dependencies']]
    byid[uid]['coordinator_added_dependencies'] = providers

# Shared lexical test directories were proposals. Assign concrete separate files
# before dispatch so their independent production modules can remain parallel.
for uid in ['P02','P03','P04','P05']:
    original=byid[uid]['paths']
    byid[uid]['paths']=[f'rust/crates/syson-ast-adapter/tests/{uid.lower()}_lexical.rs' if p=='rust/crates/syson-ast-adapter/tests/lexical/' else p for p in original]
    byid[uid]['coordinator_write_refinement']='Replace broad common lexical test-directory lease with this specific integration-test file; coordinator still owns module registration.'

overlay=json.loads((PLAN/'cross-lane-review.json').read_text())
for item in overlay['unit_overlays']:
    u=byid[item['unit']]
    u['implementation_ready_dependencies']=sorted(set(u['dependencies']+item['implementation_ready_dependencies']))
    u['acceptance_gates']=item['acceptance_gates']
    u['cross_lane_readiness_rationale']=item['rationale']
for uid, gate in overlay['joint_acceptance_gates'].items():
    byid[uid]['acceptance_implementation_inputs']=gate['inputs']
    byid[uid]['joint_acceptance_meaning']=gate['meaning']

def ancestors(uid, trail=()):
    assert uid not in trail, 'Dependency cycle: '+' -> '.join(trail+(uid,))
    result=set()
    for dep in byid[uid]['dependencies']:
        result.add(dep)
        result.update(ancestors(dep,trail+(uid,)))
    return result

memo={}
def cached_ancestors(uid,trail=()):
    assert uid not in trail, 'Dependency cycle: '+' -> '.join(trail+(uid,))
    if uid not in memo:
        result=set()
        for dep in byid[uid]['dependencies']:
            result.add(dep);result.update(cached_ancestors(dep,trail+(uid,)))
        memo[uid]=result
    return memo[uid]
for uid in byid:cached_ancestors(uid)

# Compile two distinct readiness states. Reciprocal actual-caller verification
# consumes implementation-ready peers; it must not require their acceptance first.
stage_edges={}
for uid,u in byid.items():
    ideps=u.get('implementation_ready_dependencies',u['dependencies'])
    stage_edges['implementation:'+uid]=['implementation:'+dep for dep in ideps]
    children=u.get('acceptance_children',[])
    assert isinstance(children,list),(uid,'acceptance_children must name IDs')
    adeps=['implementation:'+uid]+['accepted:'+dep for dep in children+u.get('acceptance_gates',[])]+['implementation:'+dep for dep in u.get('acceptance_implementation_inputs',[])]
    if uid=='C99':adeps+=['accepted:'+dep for dep in byid if dep!='C99']
    stage_edges['accepted:'+uid]=sorted(set(adeps))
for uid in overlay['joint_acceptance_gates']:
    pending=list(byid[uid]['acceptance_implementation_inputs']);needed=set()
    while pending:
        item=pending.pop()
        if item in needed:continue
        require_item=byid[item]
        needed.add(item)
        pending.extend(require_item.get('implementation_ready_dependencies',require_item['dependencies']))
        pending.extend(require_item.get('acceptance_children',[]))
    byid[uid]['expanded_implementation_inputs']=sorted(needed)
    stage_edges['accepted:'+uid]=sorted(set(stage_edges['accepted:'+uid]+['implementation:'+item for item in needed]))
stage_seen=set();stage_active=set()
def stage_check(node):
    assert node in stage_edges,('Undefined readiness node',node)
    assert node not in stage_active,('Readiness cycle',node)
    if node in stage_seen:return
    stage_active.add(node)
    for dep in stage_edges[node]:stage_check(dep)
    stage_active.remove(node);stage_seen.add(node)
for node in stage_edges:stage_check(node)

def overlap(a,b):
    a=a.rstrip('/');b=b.rstrip('/')
    return a==b or a.startswith(b+'/') or b.startswith(a+'/')
conflicts=[]
for i,a in enumerate(UNITS):
    for b in UNITS[i+1:]:
        if b['id'] in memo[a['id']] or a['id'] in memo[b['id']]:continue
        pairs=[(x,y) for x in a['paths'] for y in b['paths'] if overlap(x,y)]
        if pairs:conflicts.append({'a':a['id'],'b':b['id'],'paths':pairs,'policy':'Never dispatch together; acquire a fresh owner and order their shared writes.'})

# A bounded dispatch simulation; a wave means potential source authors, never compiler/runtime concurrency.
done=set();waves=[]
while len(done)<len(byid):
    ready=sorted((u['id'] for u in UNITS if u['id'] not in done and set(u['dependencies'])<=done), key=lambda uid:(uid not in ['P01','I01','B01'],uid))
    assert ready, 'No acyclic dispatch remains'
    wave=[]
    for uid in ready:
        if len(wave)==3:break
        if any(any(overlap(a,b) for a in byid[uid]['paths'] for b in byid[v]['paths']) for v in wave):continue
        wave.append(uid)
    assert wave
    waves.append(wave);done.update(wave)

old=json.loads((ROOT/'porting/pr-split-20261024/future-prs.json').read_text())
audited=json.loads((ROOT/'porting/audits/20261026/audit-matrix.json').read_text())
remaining=sorted({u['id'] for u in old['units']}-{u['id'] for u in audited['units']})
mapping={}
for oldid in remaining:
    entry=overlay['legacy14mapping'][oldid]
    mapping[oldid]=entry
    assert entry['direct'] and all(uid in byid for uid in entry['direct']+entry['expanded']),oldid

result={'schema':1,'target_source':'6ba2ac64e6b16ee2b05752ad55395eebb87d4910',
 'state':'ADOPTED_REVIEWABLE_PLAN;_IMPLEMENTATION_AND_NATIVE_CLOSURE_REMAIN_OPEN',
 'unit_count':len(UNITS),'full_parity_claimed':False,
 'count_meaning':'Named review units, including verification and explicitly generated source batches; discovery may add bounded children. Not a feature count, percentage or guaranteed final PR count.',
 'lane_counts':dict(collections.Counter(u['source_fragment'] for u in UNITS)),
 'first_independent_batch':['P01','I01','B01'],
 'execution_policy':'At most3source authors; single coordinator serializes builds, native runtimes, DB and browser. Source readiness does not mean landable or parity accepted.',
 'old_remaining_brief_mapping':mapping,'unordered_write_conflicts':conflicts,
 'readiness_graph':stage_edges,'readiness_graph_acyclic':True,
 'readiness_meaning':overlay['milestones'],
 'source_author_dispatch_waves':waves,'units':sorted(UNITS,key=lambda u:u['id'])}
(PLAN/'plan.json').write_text(json.dumps(result,indent=2)+'\n')
briefs=PLAN/'prs';briefs.mkdir(exist_ok=True)
def render(value):
    if isinstance(value,str):return value
    if isinstance(value,list):return '\n'.join('- '+render(x) for x in value)
    if not isinstance(value,dict):return str(value)
    return '\n'.join('- **'+str(k).replace('_',' ')+':** '+(render(v) or 'None declared.') for k,v in value.items())
for u in result['units']:
    sections=[f"# {u['id']}: {u['title']}",f"Lane: {u['lane']}. State: {u.get('state',u.get('status','PLANNED'))}.",
     '## Source preparation prerequisites',render(u['dependencies']) or 'No new-plan prerequisite.',
     'Existing PR prerequisites: '+(', '.join(f'[#'+str(x)+'](https://github.com/ajhcs/syson/pull/'+str(x)+')' for x in u['external_prs']) or 'None declared; source/runtime gates still apply.'),
     '## Implementation readiness',render(u.get('implementation_ready_dependencies',u['dependencies'])) or 'The unit must compile and pass its local behavior tests against its frozen existing prerequisites.',
     '## Feature acceptance dependencies',render({'joint gates':u.get('acceptance_gates',[]),'children':u.get('acceptance_children',[]),'joint implementation inputs':u.get('acceptance_implementation_inputs',[]),'expanded implementation inputs':u.get('expanded_implementation_inputs',[])}),
     '## Write scope',render(u['paths']), '## Tasks',render(u['tasks']),
     '## Verification',render(u['verification']), '## Commit plan',render(u.get('commit_plan',u.get('commits',[]))),
     '## Review budget',render(u.get('budget',u.get('review_budget','<=400 handwritten production lines and <=900 authored lines; split before execution if larger.'))),
     '## Completion',render(u.get('done_when',u.get('completion_criteria','Every task, source obligation and applicable execution gate must have independently reviewed evidence; otherwise remain open.'))),
     '## Source evidence',render(u.get('source_refs',[])),
     f"Original lane record: [{u['source_fragment']}]({ '../'+u['source_fragment'] }). Source snapshots stay immutable; plan.json adds the explicit coordinator dependency overlay."]
    (briefs/(u['id']+'.md')).write_text('\n\n'.join(sections)+'\n')
index=['# Remaining Rust parity PR index','The counts below are review units, not completed features. Prerequisites here describe source preparation; each linked brief and plan.json separately defines implementation readiness, child acceptance and joint gates. Read [execution and acceptance](README.md) before dispatch.','| ID | PR scope | Source prerequisites | Existing PRs |','|---|---|---|---|']
for u in result['units']:
    index.append('| ['+u['id']+'](prs/'+u['id']+'.md) | '+u['title'].replace('|','/')+' | '+', '.join(u['dependencies'])+' | '+', '.join('#'+str(n) for n in u['external_prs'])+' |')
(PLAN/'PR-INDEX.md').write_text('\n\n'.join(index[:2])+'\n\n'+'\n'.join(index[2:])+'\n')
(PLAN/'dispatch.json').write_text(json.dumps({'meaning':'Potential topological source-author waves only. At dispatch recheck actual state, leases, all prerequisites and resource allocation; no runtime parallelism inferred.','max_concurrent_authors':3,'first_batch':['P01','I01','B01'],'waves':waves,'write_conflicts':conflicts},indent=2)+'\n')
print(json.dumps({'units':len(UNITS),'waves':len(waves),'write_conflicts':len(conflicts),'old_remaining_mapping':mapping}))

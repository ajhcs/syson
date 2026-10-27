#!/usr/bin/env python3
"""Read-only plan and completion-ledger checks; no production runtime actions."""
import argparse
import copy
import hashlib
import json
from pathlib import Path

def require(ok,message):
    if not ok:raise ValueError(message)

def acyclic(graph):
    seen=set();active=set()
    def visit(node):
        require(node in graph,'Undefined dependency: '+node)
        require(node not in active,'Dependency cycle at '+node)
        if node in seen:return
        active.add(node)
        for dep in graph[node]:visit(dep)
        active.remove(node);seen.add(node)
    for node in graph:visit(node)

def validate(plan,root):
    units=plan['units'];ids={u['id'] for u in units};byid={u['id']:u for u in units}
    require(len(ids)==len(units)==plan['unit_count'],'Duplicate/missing unit count')
    require(not plan['full_parity_claimed'],'A plan must not claim feature parity')
    graph={u['id']:u['dependencies'] for u in units};acyclic(graph)
    stages=plan['readiness_graph'];acyclic(stages)
    require(set(stages)=={prefix+uid for prefix in ['implementation:','accepted:'] for uid in ids},'Readiness node mismatch')
    fragments={}
    for u in units:
        for child in u.get('acceptance_children',[]):
            require('accepted:'+child in stages['accepted:'+u['id']],'Missing acceptance child '+child)
        fragment=root/u['source_fragment']
        require(fragment.is_file(),'Missing source fragment '+str(fragment))
        if fragment not in fragments:fragments[fragment]=hashlib.sha256(fragment.read_bytes()).hexdigest()
        require(fragments[fragment]==u['source_fragment_sha256'],'Changed source fragment '+str(fragment))
        overlay=u.get('rereview_overlay')
        if overlay:
            path=root/overlay['path']
            require(path.is_file() and hashlib.sha256(path.read_bytes()).hexdigest()==overlay['sha256'],'Changed re-review overlay '+u['id'])
        require((root/'prs'/(u['id']+'.md')).is_file(),'Missing PR brief '+u['id'])
        if u.get('acceptance_implementation_inputs'):
            pending=list(u['acceptance_implementation_inputs']);needed=set()
            while pending:
                child=pending.pop()
                if child in needed:continue
                needed.add(child);record=byid[child]
                pending.extend(record.get('implementation_ready_dependencies',record['dependencies']))
                pending.extend(record.get('acceptance_children',[]))
            require({'implementation:'+child for child in needed}<=set(stages['accepted:'+u['id']]),'Joint gate skips semantic child implementations: '+u['id'])
    # Reconcile normalized obligations against the authoritative overlay and lane.
    review_path=root/'rereview-overlay.json'
    review=json.loads(review_path.read_text())
    review_sha=hashlib.sha256(review_path.read_bytes()).hexdigest()
    executed=json.loads((root/'execution.json').read_text()).get('units',{})
    for uid,item in review['units'].items():
        require(uid in byid,'Missing re-review owner '+uid)
        unit=byid[uid]
        require(unit.get('rereview_overlay')=={'path':'rereview-overlay.json','sha256':review_sha},'Missing/changed re-review annotation '+uid)
        repairs=set(item['repair_units'])
        require(repairs<=set(unit.get('acceptance_children',[])),'Missing re-review acceptance child '+uid)
        require({'accepted:'+child for child in repairs}<=set(stages['accepted:'+uid]),'Missing re-review accepted edge '+uid)
        if uid not in executed:
            require(repairs<=set(unit['dependencies']),'Missing re-review source prerequisite '+uid)
            require({'implementation:'+child for child in repairs}<=set(stages['implementation:'+uid]),'Missing re-review implementation edge '+uid)
        require(set(item['finding_ids'])<=set(unit.get('rereview_findings',[])),'Missing re-review finding '+uid)
        require(all(task in unit['tasks'] for task in item['tasks']),'Missing re-review task '+uid)
        require(all(check in unit['verification'] for check in item['verification']),'Missing re-review verification '+uid)
    repairs=json.loads((root/'lanes/rereview.json').read_text())['units']
    for raw in repairs:
        require(raw['id'] in byid,'Missing repair unit '+raw['id'])
        unit=byid[raw['id']]
        for key in ['dependencies','paths','tasks','verification','state','owner_plan_units','budget']+(['coordinator_wiring'] if raw.get('coordinator_wiring') else []):
            require(raw[key]==unit.get(key),'Changed repair lane obligation '+raw['id']+':'+key)
    for uid,unit in byid.items():
        expected={'implementation:'+dep for dep in unit.get('implementation_ready_dependencies',unit['dependencies'])}
        require(expected==set(stages['implementation:'+uid]),'Implementation readiness edges disagree '+uid)
    require(set('accepted:'+uid for uid in ids-{'C99'})<=set(stages['accepted:C99']),'Final gate skips units')
    require(not plan['unordered_write_conflicts'],'Unordered shared writes remain')
    require(len(plan['old_remaining_brief_mapping'])==14,'Missing original remaining brief')
    require(all(v['direct'] for v in plan['old_remaining_brief_mapping'].values()),'Unmapped remaining brief')
    parser=json.loads((root/'lanes/parser.json').read_text());coverage=parser['coverage']
    for key,count in [('grammar_rule_owners',735),('ast_tuple_owners',848),('export_dispatch_owners',82)]:
        owners=coverage[key];require(len(owners)==count,'Parser source assignment count mismatch '+key)
        for row in owners.values():require(all(uid in ids for uid in (row if isinstance(row,list) else [row])),'Undefined parser owner')
    model=json.loads((root/'lanes/model.json').read_text())
    for key,count in [('classifier_assignment',186),('declared_feature_assignment',419),('declared_operation_assignment',70)]:
        rows=model[key];require(len(rows)==count,'Model assignment count mismatch '+key)
        for row in rows:
            owner=row.get('assigned_record',row.get('schema_record'))
            require(owner in ids,'Undefined model owner '+str(owner))
    return {'units':len(units),'source_graph_acyclic':True,'readiness_graph_nodes':len(stages),'readiness_graph_acyclic':True,'unordered_write_conflicts':0,'all_14_original_remaining_briefs_mapped':True,'source_assignment_counts_verified':True}

def completion(plan,state,repo):
    ids={u['id'] for u in plan['units']};errors=[];entries=state.get('units',{})
    candidate=state.get('candidate_revision') or '';target=plan['target_source']
    if not isinstance(candidate,str):candidate=''
    closure=state.get('closure',{})
    if not closure.get('inventory_complete') or closure.get('artifact_binding')!='MATCHED' or closure.get('unresolved_required'):
        errors.append('Pinned runtime/source obligation closure remains open')
    for child in closure.get('discovered_children',[]):
        if child not in ids:errors.append('Discovered child absent from plan: '+child)
    for uid in sorted(ids):
        entry=entries.get(uid,{})
        if entry.get('feature_acceptance')!='FEATURE_ACCEPTED':errors.append('Not feature-accepted: '+uid);continue
        proof=entry.get('acceptance_evidence',{})
        if len(candidate)!=40 or proof.get('candidate_revision')!=candidate or proof.get('target_source')!=target:
            errors.append('Missing exact candidate/target acceptance: '+uid)
        receipts=proof.get('receipts',[])
        if not receipts:errors.append('Missing acceptance receipts: '+uid)
        for receipt in receipts:
            p=(repo/receipt['path']).resolve()
            if not p.is_relative_to(repo.resolve()) or not p.is_file() or hashlib.sha256(p.read_bytes()).hexdigest()!=receipt['sha256']:
                errors.append('Missing/changed acceptance receipt: '+uid)
    return errors

def self_test(plan,root):
    graph={'implementation:adapter':['implementation:model'],'implementation:model':[],
           'accepted:adapter':['implementation:adapter','accepted:joint'],
           'accepted:joint':['implementation:adapter','implementation:model']}
    acyclic(graph)
    bad=copy.deepcopy(graph);bad['accepted:joint']=['accepted:adapter']
    try:acyclic(bad)
    except ValueError:pass
    else:raise AssertionError('Reciprocal acceptance cycle was accepted')
    try:acyclic({'one':['missing']})
    except ValueError:pass
    else:raise AssertionError('Undefined dependency was accepted')
    fake={'units':{u['id']:{'feature_acceptance':'FEATURE_ACCEPTED'} for u in plan['units']},'closure':{'inventory_complete':True,'artifact_binding':'MATCHED','unresolved_required':[]}}
    require(bool(completion(plan,fake,Path('/nonexistent'))),'Bare accepted labels bypassed evidence gate')
    fake['candidate_revision']=None
    require(bool(completion(plan,fake,Path('/nonexistent'))),'Null candidate bypassed evidence gate')
    bad=copy.deepcopy(plan)
    bad['readiness_graph']['accepted:C06'].remove('implementation:M348')
    try:validate(bad,root)
    except ValueError as e:require('semantic child' in str(e),'Wrong rejection for missing semantic child')
    else:raise AssertionError('Verifier accepted an adapter without its concrete semantic child')

    bad=copy.deepcopy(plan)
    annotated=next(u for u in bad['units'] if u.get('rereview_overlay'))
    annotated['rereview_overlay']['sha256']='0'*64
    try:validate(bad,root)
    except ValueError as e:require('re-review overlay' in str(e),'Wrong rejection for changed re-review overlay')
    else:raise AssertionError('Stale re-review overlay accepted')

    for change,expected in [('annotation','annotation'),('child','acceptance child'),('source','source prerequisite'),('edge','implementation edge')]:
        bad=copy.deepcopy(plan);unit=next(u for u in bad['units'] if u['id']=='P170')
        if change=='annotation':unit.pop('rereview_overlay')
        elif change=='child':
            unit['acceptance_children'].remove('RR07')
            bad['readiness_graph']['accepted:P170'].remove('accepted:RR07')
        elif change=='source':unit['dependencies'].remove('RR07')
        else:bad['readiness_graph']['implementation:P170'].remove('implementation:RR07')
        try:validate(bad,root)
        except ValueError as e:require(expected in str(e),'Wrong rejection for omitted re-review '+change+': '+str(e))
        else:raise AssertionError('Omitted re-review '+change+' accepted')

    bad=copy.deepcopy(plan)
    next(u for u in bad['units'] if u['id']=='RR07').pop('coordinator_wiring')
    try:validate(bad,root)
    except ValueError as e:require('coordinator_wiring' in str(e),'Wrong rejection for omitted coordinator wiring')
    else:raise AssertionError('Omitted coordinator wiring accepted')

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--plan-dir',type=Path,default=Path(__file__).resolve().parent.parent);parser.add_argument('--check-completion',action='store_true');parser.add_argument('--self-test',action='store_true');args=parser.parse_args()
    root=args.plan_dir.resolve();plan=json.loads((root/'plan.json').read_text());result=validate(plan,root)
    if args.self_test:self_test(plan,root);result['negative_gate_checks']='PASS'
    if args.check_completion:
        state=json.loads((root/'execution.json').read_text());errors=completion(plan,state,root.parents[1]);result.update(parity='OPEN' if errors else 'ACCEPTED',open_gate_count=len(errors),first_open_gates=errors[:8]);print(json.dumps(result,indent=2));return 2 if errors else 0
    print(json.dumps(result,indent=2));return 0

if __name__=='__main__':raise SystemExit(main())

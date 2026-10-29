#!/usr/bin/env python3
"""Read-only plan and completion-ledger checks; no production runtime actions."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
from completion_contract import read_contract, validate_bindings, validate_repair_splits

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
    contract=read_contract(root)
    validate_bindings(contract,plan,root)
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
        keys = ['dependencies','paths','tasks','verification','state','owner_plan_units','budget']
        keys += ['coordinator_wiring'] if raw.get('coordinator_wiring') else []
        if raw['id'] in ['RR08', 'RR08a', 'RR08b']:
            keys += [key for key in ['acceptance_children', 'delegated_implementation', 'whole_parent_gates',
                      'split_parent', 'budget_counting', 'original_case_requirements', 'production_delta_estimate',
                      'source_regions', 'authored_or_touched_upper_estimate'] if key in raw]
        for key in keys:
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
    return {'static_contract_integrity':'PASS;behavioral_acceptance_remains_open','units':len(units),'source_graph_acyclic':True,'readiness_graph_nodes':len(stages),'readiness_graph_acyclic':True,'unordered_write_conflicts':0,'all_14_original_remaining_briefs_mapped':True,'source_assignment_counts_verified':True}

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
    # Meaningful refusal controls: input drift, omitted scope, and detached briefs.
    def rejects(action, expected):
        try:action()
        except ValueError as e:require(expected in str(e), 'Wrong contract rejection: '+str(e))
        else:raise AssertionError('Contract gate accepted '+expected)
    integrity_paths = ['COMPLETION-CONTRACT.md','acceptance-packet.schema.json',
                       'performance-policy.json','lanes/model.json','tools/build_plan.py']
    contract = read_contract(root)
    disposition = contract.get('legacy_prerequisite_budget_dispositions')
    amendment_paths = ([disposition['amendment']['path']] if disposition else [])
    amendment_paths += [policy['amendment']['path'] for policy in contract.get('repair_splits', {}).values()]
    for amendment_path in amendment_paths:
        rejects(lambda:read_contract(root, lambda name:(root/name).read_bytes() +
                (b'\n' if name==amendment_path else b'')), 'amendment hash mismatch')
    for path in integrity_paths:
        rejects(lambda:read_contract(root, lambda name:(root/name).read_bytes() +
                (b'\n' if name==path else b'')), 'hash mismatch')
    contract=read_contract(root)
    for mutation in ['omitted','unknown','duplicate']:
        changed=copy.deepcopy(contract)
        if mutation=='omitted':changed['families']['model'].pop()
        elif mutation=='unknown':changed['families']['model'].append('M-unknown')
        else:changed['families']['model'].append(changed['families']['model'][0])
        lock=json.loads((root/'completion-contract.lock.json').read_text())
        encoded=json.dumps(changed).encode()
        lock['sha256']['completion-contract.json']=hashlib.sha256(encoded).hexdigest()
        def read(name):
            if name=='completion-contract.json':return encoded
            if name=='completion-contract.lock.json':return json.dumps(lock).encode()
            return (root/name).read_bytes()
        rejects(lambda:read_contract(root,read), 'family coverage')
    bad=copy.deepcopy(plan);bad['units'][0].pop('completion_contract')
    rejects(lambda:validate(bad,root), 'unit contract binding')
    bad=copy.deepcopy(plan);bad['units'].pop()
    rejects(lambda:validate(bad,root), 'unit coverage')
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

    # Mutate both graph representations together, defeating superficial consistency checks.
    split_mutations = ['child-order', 'parent-order', 'acceptance', 'scope', 'parent-scope',
                       'lease', 'cap', 'counting', 'task', 'verification', 'gates', 'cases', 'estimate']
    for mutation in split_mutations:
        bad = copy.deepcopy(plan)
        byid = {unit['id']: unit for unit in bad['units']}
        first, second, parent = (byid[uid] for uid in ['RR08a', 'RR08b', 'RR08'])
        if mutation == 'child-order':
            second['dependencies'] = []
            bad['readiness_graph']['implementation:RR08b'] = []
            bad['unordered_write_conflicts'] = []
        elif mutation == 'parent-order':
            parent['dependencies'] = []
            bad['readiness_graph']['implementation:RR08'] = []
        elif mutation == 'acceptance':
            parent['acceptance_children'] = []
            bad['readiness_graph']['accepted:RR08'] = ['implementation:RR08']
        elif mutation == 'scope': first['paths'].append('rust/crates/syson-model/src/lib.rs')
        elif mutation == 'parent-scope': parent['paths'].append(first['paths'][0])
        elif mutation == 'lease': first['ownership'] = 'Parallel writers allowed'
        elif mutation == 'cap': first['budget']['production_lines_max'] = 401
        elif mutation == 'counting': first['budget_counting'] = 'added_only'
        elif mutation == 'task': parent['tasks'] = []
        elif mutation == 'verification': parent['verification'].pop(0)
        elif mutation == 'gates': parent['whole_parent_gates']['workload_points'].pop()
        elif mutation == 'cases': second['original_case_requirements'].pop()
        else: first['production_delta_estimate']['changed'] = 401
        rejects(lambda:validate(bad, root), 'RR08 split')
    contract_mutations = ['policy', 'unit', 'workloads', 'cases', 'exception', 'cap']
    for mutation in contract_mutations:
        bad = copy.deepcopy(contract)
        policy = bad['repair_splits']['RR08']
        if mutation == 'policy': bad.pop('repair_splits')
        elif mutation == 'unit': bad['unit_ids'].remove('P01')
        elif mutation == 'workloads': policy['whole_parent_gates']['workload_points'] = [100]
        elif mutation == 'cases': policy['whole_parent_gates']['original_rr08_cases'].pop()
        elif mutation == 'exception': bad['legacy_prerequisite_budget_dispositions']['RR08'] = {}
        else: policy['production_lines_max'] = True
        rejects(lambda:validate_repair_splits(bad), 'RR08 split')
    return len(split_mutations) + len(contract_mutations)


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--plan-dir',type=Path,default=Path(__file__).resolve().parent.parent);parser.add_argument('--check-completion',action='store_true');parser.add_argument('--self-test',action='store_true');args=parser.parse_args()
    root=args.plan_dir.resolve();plan=json.loads((root/'plan.json').read_text());result=validate(plan,root)
    if args.self_test:
        result['rr08_split_refusal_controls'] = self_test(plan,root)
        result['negative_gate_checks']='PASS'
    if args.check_completion:
        state=json.loads((root/'execution.json').read_text());errors=completion(plan,state,root.parents[1]);result.update(parity='OPEN' if errors else 'ACCEPTED',open_gate_count=len(errors),first_open_gates=errors[:8]);print(json.dumps(result,indent=2));return 2 if errors else 0
    print(json.dumps(result,indent=2));return 0

if __name__=='__main__':raise SystemExit(main())

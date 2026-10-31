#!/usr/bin/env python3
"""Read-only plan and completion-ledger checks; no production runtime actions."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
from source_next_splits import legacy_contract, legacy_plan
from completion_contract import read_contract, validate_bindings, validate_repair_splits, validate_layout_splits, validate_comparator_splits, validate_comparison_repairs, validate_routing_splits, validate_forest_splits

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
    units=plan['units'];ids={u['id'] for u in units};byid={u['id']:u for u in units}
    require(ids == set(contract['unit_ids']), 'Plan/contract unit coverage mismatch')
    require(len(ids)==len(units)==plan['unit_count'],'Duplicate/missing unit count')
    require(not plan['full_parity_claimed'],'A plan must not claim feature parity')
    for unit in units:
        require(unit.get('completion_contract', {}).get('id') == contract['contract_id'], 'Missing unit contract binding')
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
    # Keep precise legacy diagnostic guards first; the full verifier always checks all old records.
    validate_routing_splits(contract, plan)
    from team_policy import validate_teams
    expected = validate_teams(json.loads((root/'teams.json').read_bytes()), plan)
    require(json.loads((root/'team-dispatch.json').read_bytes()) == expected, 'Team dispatch drift')
    validate_bindings(contract,plan,root)
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
    amendment_paths += [policy['amendment']['path'] for policy in contract.get('layout_kernel_splits', {}).values()]
    amendment_paths += [policy['amendment']['path'] for policy in contract.get('tooling_comparator_splits', {}).values()]
    amendment_paths += [policy['amendment']['path'] for policy in contract.get('comparison_repairs', {}).values()]
    amendment_paths += [policy['amendment']['path'] for policy in contract.get('routing_kernel_splits', {}).values()]
    amendment_paths += [policy['amendment']['path'] for policy in contract.get('forest_kernel_splits', {}).values()]
    if contract.get('team_execution'):
        amendment_paths.append(contract['team_execution']['amendment']['path'])
    amendment_paths.append(contract['source_preparation_splits']['amendment']['path'])
    amendment_paths.append(contract['storage_source_splits']['amendment']['path'])
    amendment_paths.append(contract['source_next_splits']['amendment']['path'])
    integrity_paths += ['storage-policy-source.json', 'tools/storage_splits.py']
    integrity_paths += ['lanes/source-next.json', 'tools/source_next_splits.py',
                        'composition/source_next_prepare/guard.py',
                        'composition/source_next_prepare/construct.py']
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

    current_contract = contract
    plan = legacy_plan(current_contract, plan)
    contract = legacy_contract(current_contract)
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
        rejects(lambda:validate_repair_splits(contract, bad), 'RR08 split')
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
        # The new policy validates all old IDs before legacy split exclusions.
        expected = ('C09 split repair original unit coverage' if mutation == 'unit' and
                    contract.get('comparison_repairs') else 'RR08 split')
        rejects(lambda:validate_repair_splits(bad), expected)
    return len(split_mutations) + len(contract_mutations)


def layout_split_self_test(plan, root):
    """Refuse coherent layout split omissions, not merely stale input hashes."""
    plan = legacy_plan(read_contract(root), plan)
    def rejects(action, expected='U10 split'):
        try:
            action()
        except ValueError as error:
            require(expected in str(error), 'Wrong layout rejection: ' + str(error))
        else:
            raise AssertionError('U10 split mutation accepted')
    mutations = ['child-missing', 'source-order', 'parent-ready', 'parent-dependency',
                 'parent-acceptance', 'parent-task', 'parent-verification', 'parent-scope',
                 'child-scope', 'production-cap', 'authored-cap', 'counting', 'tests',
                 'test-source', 'wiring', 'parent-gates', 'estimate', 'child-task', 'kernel-dependency']
    for mutation in mutations:
        bad = copy.deepcopy(plan)
        byid = {unit['id']: unit for unit in bad['units']}
        first, second, parent = (byid[uid] for uid in ['U10a', 'U10b', 'U10'])
        if mutation == 'child-missing':
            bad['units'].remove(second)
        elif mutation == 'source-order':
            second['dependencies'] = []
            second['implementation_ready_dependencies'] = []
            bad['readiness_graph']['implementation:U10b'] = []
        elif mutation == 'parent-ready':
            parent['implementation_ready_dependencies'].remove('U10b')
            bad['readiness_graph']['implementation:U10'].remove('implementation:U10b')
        elif mutation == 'parent-dependency':
            parent['dependencies'].remove('U09')
            parent['implementation_ready_dependencies'].remove('U09')
            bad['readiness_graph']['implementation:U10'].remove('implementation:U09')
        elif mutation == 'parent-acceptance':
            parent['acceptance_children'].remove('U10b')
            bad['readiness_graph']['accepted:U10'].remove('accepted:U10b')
        elif mutation == 'parent-task': parent['tasks'].pop()
        elif mutation == 'parent-verification': parent['verification'].pop()
        elif mutation == 'parent-scope': parent['paths'].append('rust/crates/syson-initializer-graph/src/')
        elif mutation == 'child-scope': first['paths'].append('rust/crates/syson-web/src/lib.rs')
        elif mutation == 'production-cap': first['budget']['production_lines_max'] = 401
        elif mutation == 'authored-cap': second['budget']['authored_lines_max'] = 901
        elif mutation == 'counting': first['budget_counting'] = 'added_only'
        elif mutation == 'tests': second['original_case_requirements'].pop()
        elif mutation == 'test-source': second['retained_test_source']['sha256'] = '0' * 64
        elif mutation == 'wiring': first['coordinator_wiring'].append('rust/crates/syson-model/src/lib.rs')
        elif mutation == 'parent-gates': parent['kernel_parent_gates'].pop('actual_caller')
        elif mutation == 'estimate': second['production_delta_estimate']['changed_upper'] = 400
        elif mutation == 'kernel-dependency': first['kernel_dependency_policy'] = 'runtime ELK dependency'
        else: first['tasks'].pop()
        rejects(lambda: validate_layout_splits(legacy_contract(read_contract(root)), bad,
                json.loads((root / 'lanes/ui.json').read_text())))
    contract = legacy_contract(read_contract(root))
    contract_mutations = ['policy', 'old-unit', 'production-cap', 'authored-cap', 'tests',
                          'donor', 'paths', 'parent-proof', 'gates', 'order', 'measurement-interface', 'kernel-dependency']
    for mutation in contract_mutations:
        bad = copy.deepcopy(contract)
        policy = bad['layout_kernel_splits']['U10']
        if mutation == 'policy': bad.pop('layout_kernel_splits')
        elif mutation == 'old-unit': bad['unit_ids'].remove('P01')
        elif mutation == 'production-cap': policy['production_lines_max'] = True
        elif mutation == 'authored-cap': policy['authored_lines_max'] = 901
        elif mutation == 'tests': policy['donor']['original_cases'].pop()
        elif mutation == 'donor': policy['donor']['production_sha256'] = '0' * 64
        elif mutation == 'paths': policy['child_paths']['U10a'].append('rust/crates/syson-web/src/')
        elif mutation == 'parent-proof': policy['original_parent_sha256'] = '0' * 64
        elif mutation == 'gates': policy['whole_parent_gates'].pop('native_source_artifact')
        elif mutation == 'measurement-interface': policy['measurement_interface']['sha256'] = '0' * 64
        elif mutation == 'kernel-dependency': policy.pop('kernel_dependency_policy')
        else: policy['implementation_order'].reverse()
        # Missing old P01 is now rejected by the earlier shared inventory guard.
        expected = ('C09 split repair original unit coverage' if mutation == 'old-unit' and
                    contract.get('comparison_repairs') else 'U10 split')
        rejects(lambda: validate_layout_splits(bad), expected)
    return len(mutations) + len(contract_mutations)


def comparator_split_self_test(plan, root):
    """Refuse coherent module, RR15, source-epoch and composed-control losses."""
    plan = legacy_plan(read_contract(root), plan)
    def rejects(action):
        try:
            action()
        except ValueError as error:
            require('C09 split' in str(error), 'Wrong comparator rejection: ' + str(error))
        else:
            raise AssertionError('C09 split mutation accepted')
    contract = legacy_contract(read_contract(root))
    lane = json.loads((root / 'lanes/ui.json').read_text())
    mutations = ['missing-child', 'child-order', 'parent-ready', 'RR15-ready', 'RR15-acceptance',
                 'joint-input', 'expanded-input', 'parent-task', 'parent-verification', 'parent-path',
                 'parent-acceptance', 'child-path', 'production-cap', 'authored-cap', 'counting',
                 'counting-scope', 'child-task', 'control-list', 'control-source', 'epoch-list',
                 'helper-pin', 'receipt-limit', 'source-regions', 'wiring']
    for mutation in mutations:
        bad = copy.deepcopy(plan)
        byid = {unit['id']: unit for unit in bad['units']}
        first, second, parent = (byid[uid] for uid in ['C09a', 'C09b', 'C09'])
        if mutation == 'missing-child': bad['units'].remove(first)
        elif mutation == 'child-order':
            second['dependencies'] = second['implementation_ready_dependencies'] = []
            bad['readiness_graph']['implementation:C09b'] = []
        elif mutation == 'parent-ready':
            parent['implementation_ready_dependencies'].remove('C09b')
            bad['readiness_graph']['implementation:C09'].remove('implementation:C09b')
        elif mutation == 'RR15-ready':
            parent['dependencies'].remove('RR15')
            parent['implementation_ready_dependencies'].remove('RR15')
            bad['readiness_graph']['implementation:C09'].remove('implementation:RR15')
        elif mutation == 'RR15-acceptance':
            parent['acceptance_children'].remove('RR15')
            bad['readiness_graph']['accepted:C09'].remove('accepted:RR15')
        elif mutation == 'joint-input': parent['acceptance_implementation_inputs'].remove('B05')
        elif mutation == 'expanded-input':
            parent['expanded_implementation_inputs'].remove('B05')
            bad['readiness_graph']['accepted:C09'].remove('implementation:B05')
        elif mutation == 'parent-task': parent['tasks'].pop()
        elif mutation == 'parent-verification': parent['verification'].pop()
        elif mutation == 'parent-path': parent['paths'].append('rust/crates/syson-initializer-graph/src/')
        elif mutation == 'parent-acceptance':
            parent['acceptance_children'].remove('C09a')
            bad['readiness_graph']['accepted:C09'].remove('accepted:C09a')
        elif mutation == 'child-path': first['paths'].append('rust/validation/factory/')
        elif mutation == 'production-cap': first['budget']['production_lines_max'] = 401
        elif mutation == 'authored-cap': second['budget']['authored_lines_max'] = 901
        elif mutation == 'counting': first['budget_counting'] = 'added_only'
        elif mutation == 'counting-scope': first['budget']['counting_scope'] = 'All helper code excluded'
        elif mutation == 'child-task': first['tasks'].pop()
        elif mutation == 'control-list': second['original_control_requirements'].pop()
        elif mutation == 'control-source': second['retained_control_source']['control_count'] = 34
        elif mutation == 'epoch-list': first['source_epoch_requirements'].pop()
        elif mutation == 'helper-pin': first['prototype_requirements']['constructor_sha256'] = '0' * 64
        elif mutation == 'receipt-limit': first['prototype_requirements']['receipt_causality_limit'] = ''
        elif mutation == 'source-regions': second['source_regions']['controls'] = []
        else: first['coordinator_wiring'].append('rust/crates/syson-model/src/lib.rs')
        rejects(lambda: validate_comparator_splits(contract, bad, lane))
    contract_mutations = ['policy', 'old-unit', 'cap-bool', 'authored-cap', 'prototype',
                          'line-count', 'epoch', 'controls', 'source-regions', 'RR15-guard',
                          'parent-proof', 'legacy-exception', 'order', 'counting-scope', 'whole-gates']
    for mutation in contract_mutations:
        bad = copy.deepcopy(contract)
        policy = bad['tooling_comparator_splits']['C09']
        if mutation == 'policy': bad.pop('tooling_comparator_splits')
        elif mutation == 'old-unit': bad['unit_ids'].remove('P01')
        elif mutation == 'cap-bool': policy['production_lines_max'] = True
        elif mutation == 'authored-cap': policy['authored_lines_max'] = 901
        elif mutation == 'prototype': policy['frozen_prototype']['sha256'] = '0' * 64
        elif mutation == 'line-count': policy['frozen_prototype']['line_count'] = 685
        elif mutation == 'epoch': policy['frozen_prototype']['source_epochs'].pop()
        elif mutation == 'controls': policy['frozen_prototype']['original_controls'].pop()
        elif mutation == 'source-regions': policy['frozen_prototype']['source_regions']['capture'].pop()
        elif mutation == 'RR15-guard': policy['parent_dependencies'].remove('RR15')
        elif mutation == 'parent-proof': policy['original_parent_sha256'] = '0' * 64
        elif mutation == 'legacy-exception': bad['legacy_prerequisite_budget_dispositions']['C09a'] = {}
        elif mutation == 'order': policy['implementation_order'].reverse()
        elif mutation == 'counting-scope': policy['counting_scope'] = 'Ignore helper code'
        else: policy['whole_parent_gates'].pop('RR15_admission')
        rejects(lambda: validate_comparator_splits(bad))
    return len(mutations) + len(contract_mutations)


def comparison_repair_self_test(plan, root):
    """Reject coherent child/edge loss and changed partial-comparison requirements."""
    plan = legacy_plan(read_contract(root), plan)
    contract = legacy_contract(read_contract(root))
    lane = json.loads((root / 'lanes/ui.json').read_text())

    def rejects(action):
        try:
            action()
        except ValueError as error:
            require('C09 split repair' in str(error), 'Wrong repair rejection: ' + str(error))
        else:
            raise AssertionError('C09 split repair mutation accepted')

    mutations = ['missing-child', 'child-source', 'parent-source', 'parent-acceptance',
                 'child-scope', 'production-cap', 'authored-cap', 'original-control',
                 'new-control', 'parent-task', 'public-api-task', 'counting', 'wiring']
    for mutation in mutations:
        bad, raw = copy.deepcopy(plan), copy.deepcopy(lane)
        for records in [bad['units'], raw['units']]:
            byid = {unit['id']: unit for unit in records}
            first, second, parent = (byid[uid] for uid in ['C09b1', 'C09b2', 'C09b'])
            if mutation == 'missing-child': records.remove(first)
            elif mutation == 'child-source':
                second['dependencies'] = second['implementation_ready_dependencies'] = []
            elif mutation == 'parent-source':
                parent['dependencies'].remove('C09b2')
                parent['implementation_ready_dependencies'].remove('C09b2')
            elif mutation == 'parent-acceptance': parent['acceptance_children'].remove('C09b1')
            elif mutation == 'child-scope': second['paths'].append('rust/crates/syson-web/src/')
            elif mutation == 'production-cap': first['budget']['production_lines_max'] = 401
            elif mutation == 'authored-cap': second['budget']['authored_lines_max'] = 901
            elif mutation == 'original-control': second['original_control_requirements'].pop()
            elif mutation == 'new-control': second['additional_control_requirements'].pop()
            elif mutation == 'parent-task': parent['tasks'].pop()
            elif mutation == 'public-api-task': first['tasks'].pop(0)
            elif mutation == 'counting': first['budget_counting'] = 'added_only'
            else: second['coordinator_wiring'].append('rust/crates/syson-ast/src/lib.rs')
        if mutation == 'child-source': bad['readiness_graph']['implementation:C09b2'] = []
        elif mutation == 'parent-source':
            bad['readiness_graph']['implementation:C09b'].remove('implementation:C09b2')
        elif mutation == 'parent-acceptance':
            bad['readiness_graph']['accepted:C09b'].remove('accepted:C09b1')
        rejects(lambda: validate_comparison_repairs(contract, bad, raw))
    contract_mutations = ['missing-policy', 'old-unit', 'cap-bool', 'authored-cap',
                          'capture-pin', 'candidate', 'assignment', 'new-control',
                          'child-seal', 'raw-parent', 'normalized-parent', 'readiness', 'field-policy']
    for mutation in contract_mutations:
        bad = copy.deepcopy(contract)
        policy = bad['comparison_repairs']['C09b']
        if mutation == 'missing-policy': bad.pop('comparison_repairs')
        elif mutation == 'old-unit': bad['unit_ids'].remove('P01')
        elif mutation == 'cap-bool': policy['production_lines_max'] = True
        elif mutation == 'authored-cap': policy['authored_lines_max'] = 901
        elif mutation == 'capture-pin': policy['capture_module']['sha256'] = '0' * 64
        elif mutation == 'candidate': policy['candidate']['line_count'] = 399
        elif mutation == 'assignment': policy['source_assignment']['C09b1'].pop()
        elif mutation == 'new-control': policy['additional_controls'].pop()
        elif mutation == 'child-seal': policy['child_raw_sha256']['C09b2'] = '0' * 64
        elif mutation == 'raw-parent': policy['original_parent_sha256'] = '0' * 64
        elif mutation == 'normalized-parent': policy['normalized_parent_sha256'] = '0' * 64
        elif mutation == 'readiness': policy['readiness_policy'] = 'Require full feature acceptance'
        else: policy['partial_field_policy'] = 'Ignore references when anything is unavailable'
        rejects(lambda: validate_comparison_repairs(bad))
    return len(mutations) + len(contract_mutations)


def routing_split_self_test(plan, root):
    """Reject coherent routing omissions under normal and optimized Python."""
    plan = legacy_plan(read_contract(root), plan)
    contract = legacy_contract(read_contract(root))
    lane = json.loads((root / 'lanes/ui.json').read_text())

    def rejects(action):
        try:
            action()
        except ValueError as error:
            require('U11 split' in str(error), 'Wrong routing rejection: ' + str(error))
        else:
            raise AssertionError('U11 split mutation accepted')

    mutations = ['missing-child', 'child-source', 'parent-source', 'parent-acceptance',
                 'scope', 'production-cap', 'authored-cap', 'original-case', 'parent-task',
                 'opaque-continuation', 'wiring', 'counting', 'readiness-edge', 'acceptance-edge',
                 'old-unit-task', 'old-unit-verification', 'joint-input', 'joint-edge', 'unexpected-parent-field']
    for mutation in mutations:
        bad, raw = copy.deepcopy(plan), copy.deepcopy(lane)
        for records in [bad['units'], raw['units']]:
            byid = {unit['id']: unit for unit in records}
            first, second, parent = (byid[uid] for uid in ['U11a', 'U11b', 'U11'])
            if mutation == 'missing-child': records.remove(first)
            elif mutation == 'child-source':
                second['dependencies'] = second['implementation_ready_dependencies'] = []
            elif mutation == 'parent-source': parent['implementation_ready_dependencies'].remove('U11b')
            elif mutation == 'parent-acceptance': parent['acceptance_children'].remove('U11a')
            elif mutation == 'scope': first['paths'].append('rust/crates/syson-web/src/')
            elif mutation == 'production-cap': first['budget']['production_lines_max'] = 401
            elif mutation == 'authored-cap': second['budget']['authored_lines_max'] = 901
            elif mutation == 'original-case': second['original_case_requirements'].pop()
            elif mutation == 'parent-task': parent['tasks'].pop()
            elif mutation == 'opaque-continuation': first['tasks'].pop(1)
            elif mutation == 'wiring': first['coordinator_wiring'].append('rust/crates/syson-layout/src/lib.rs')
            elif mutation == 'counting': second['budget_counting'] = 'added_only'
            elif mutation in ['old-unit-task', 'old-unit-verification']:
                field = 'tasks' if mutation == 'old-unit-task' else 'verification'
                original = next(unit for unit in lane['units'] if unit['id'] == 'U02')[field][0]
                byid['U02'][field].remove(original)
            elif mutation == 'unexpected-parent-field': parent['unexpected_original_record_field'] = True
            elif mutation == 'joint-input' and records is bad['units']:
                byid['C04']['expanded_implementation_inputs'].remove('U11a')
        if mutation == 'joint-input':
            bad['readiness_graph']['accepted:C04'].remove('implementation:U11a')
        elif mutation == 'joint-edge':
            bad['readiness_graph']['accepted:C08'].remove('implementation:U11b')
        elif mutation in ['child-source', 'readiness-edge']:
            bad['readiness_graph']['implementation:U11b'] = []
        elif mutation == 'parent-source':
            bad['readiness_graph']['implementation:U11'].remove('implementation:U11b')
        elif mutation in ['parent-acceptance', 'acceptance-edge']:
            bad['readiness_graph']['accepted:U11'].remove('accepted:U11a')
        rejects(lambda: validate_routing_splits(contract, bad, raw))
    contract_mutations = ['missing-policy', 'old-unit', 'missing-child-id', 'cap-bool',
                          'donor', 'assignment', 'original-test', 'u10b-pin', 'readiness',
                          'child-seal', 'raw-parent', 'normalized-parent', 'continuation', 'old-policy']
    for mutation in contract_mutations:
        bad = copy.deepcopy(contract)
        policy = bad['routing_kernel_splits']['U11']
        if mutation == 'missing-policy': bad.pop('routing_kernel_splits')
        elif mutation == 'old-unit': bad['unit_ids'].remove('P01')
        elif mutation == 'missing-child-id': bad['unit_ids'].remove('U11b')
        elif mutation == 'cap-bool': policy['production_lines_max'] = True
        elif mutation == 'donor': policy['donor']['production_sha256'] = '0' * 64
        elif mutation == 'assignment': policy['source_assignment']['U11a'].pop()
        elif mutation == 'original-test': policy['donor']['original_cases'].pop()
        elif mutation == 'u10b-pin': policy['landed_u10b']['placement_sha256'] = '0' * 64
        elif mutation == 'readiness': policy['readiness_policy'] = 'Require full parent acceptance'
        elif mutation == 'child-seal': policy['child_raw_sha256']['U11b'] = '0' * 64
        elif mutation == 'raw-parent': policy['original_parent_sha256'] = '0' * 64
        elif mutation == 'normalized-parent': policy['normalized_parent_sha256'] = '0' * 64
        elif mutation == 'continuation': policy['opaque_continuation'] = 'Reset budget per phase'
        else: bad.pop('comparison_repairs')
        rejects(lambda: validate_routing_splits(bad))
    return len(mutations) + len(contract_mutations)


def forest_split_self_test(plan, root):
    """Reject coherent forest and historical-obligation omissions."""
    plan = legacy_plan(read_contract(root), plan)
    contract = legacy_contract(read_contract(root))
    lane = json.loads((root / 'lanes/ui.json').read_text())

    def rejects(action):
        try:
            action()
        except ValueError as error:
            require('U12 split' in str(error), 'Wrong forest rejection: ' + str(error))
        else:
            raise AssertionError('U12 split mutation accepted')

    mutations = ['missing-child', 'child-source', 'parent-source', 'parent-acceptance',
                 'scope', 'production-cap', 'authored-cap', 'original-case', 'parent-task',
                 'parent-verification', 'outer-budget', 'single-debit', 'wiring', 'counting',
                 'readiness-edge', 'acceptance-edge', 'old-unit-task', 'old-unit-verification',
                 'joint-input', 'joint-edge', 'unexpected-joint-input', 'unexpected-joint-edge',
                 'unexpected-parent-field']
    for mutation in mutations:
        bad, raw = copy.deepcopy(plan), copy.deepcopy(lane)
        for records in [bad['units'], raw['units']]:
            byid = {unit['id']: unit for unit in records}
            first, second, parent = (byid[uid] for uid in ['U12a', 'U12b', 'U12'])
            if mutation == 'missing-child': records.remove(first)
            elif mutation == 'child-source':
                second['dependencies'] = second['implementation_ready_dependencies'] = []
            elif mutation == 'parent-source': parent['implementation_ready_dependencies'].remove('U12b')
            elif mutation == 'parent-acceptance': parent['acceptance_children'].remove('U12a')
            elif mutation == 'scope': first['paths'].append('rust/crates/syson-web/src/')
            elif mutation == 'production-cap': first['budget']['production_lines_max'] = 401
            elif mutation == 'authored-cap': second['budget']['authored_lines_max'] = 901
            elif mutation == 'original-case': second['original_case_requirements'].pop()
            elif mutation == 'parent-task': parent['tasks'].pop()
            elif mutation == 'parent-verification': parent['verification'].pop()
            elif mutation == 'outer-budget': first['tasks'].pop(1)
            elif mutation == 'single-debit': second['tasks'].pop(1)
            elif mutation == 'wiring': first['coordinator_wiring'].append('rust/crates/syson-layout/src/types.rs')
            elif mutation == 'counting': second['budget_counting'] = 'added_only'
            elif mutation in ['old-unit-task', 'old-unit-verification']:
                field = 'tasks' if mutation == 'old-unit-task' else 'verification'
                original = next(unit for unit in lane['units'] if unit['id'] == 'U02')[field][0]
                byid['U02'][field].remove(original)
            elif mutation == 'unexpected-parent-field': parent['unexpected_original_record_field'] = True
            elif mutation == 'joint-input' and records is bad['units']:
                byid['C08']['expanded_implementation_inputs'].remove('U12a')
            elif mutation == 'unexpected-joint-input' and records is bad['units']:
                byid['C04']['expanded_implementation_inputs'].append('U12a')
        if mutation == 'joint-input':
            bad['readiness_graph']['accepted:C08'].remove('implementation:U12a')
        elif mutation == 'joint-edge':
            bad['readiness_graph']['accepted:C08'].remove('implementation:U12b')
        elif mutation == 'unexpected-joint-input':
            bad['readiness_graph']['accepted:C04'].append('implementation:U12a')
        elif mutation == 'unexpected-joint-edge':
            bad['readiness_graph']['accepted:C09'].append('implementation:U12b')
        elif mutation in ['child-source', 'readiness-edge']:
            bad['readiness_graph']['implementation:U12b'] = []
        elif mutation == 'parent-source':
            bad['readiness_graph']['implementation:U12'].remove('implementation:U12b')
        elif mutation in ['parent-acceptance', 'acceptance-edge']:
            bad['readiness_graph']['accepted:U12'].remove('accepted:U12a')
        rejects(lambda: validate_forest_splits(contract, bad, raw))
    contract_mutations = ['missing-policy', 'old-unit', 'missing-child-id', 'cap-bool',
                          'donor', 'assignment', 'original-test', 'u10b-pin', 'readiness',
                          'child-seal', 'raw-parent', 'normalized-parent', 'continuation',
                          'old-record-proof', 'estimate', 'joint-owner-scope', 'old-policy']
    for mutation in contract_mutations:
        bad = copy.deepcopy(contract)
        policy = bad['forest_kernel_splits']['U12']
        if mutation == 'missing-policy': bad.pop('forest_kernel_splits')
        elif mutation == 'old-unit': bad['unit_ids'].remove('P01')
        elif mutation == 'missing-child-id': bad['unit_ids'].remove('U12b')
        elif mutation == 'cap-bool': policy['production_lines_max'] = True
        elif mutation == 'donor': policy['donor']['production_sha256'] = '0' * 64
        elif mutation == 'assignment': policy['source_assignment']['U12a'].pop()
        elif mutation == 'original-test': policy['donor']['original_cases'].pop()
        elif mutation == 'u10b-pin': policy['landed_u10b']['placement_sha256'] = '0' * 64
        elif mutation == 'readiness': policy['readiness_policy'] = 'Require full parent acceptance'
        elif mutation == 'child-seal': policy['child_raw_sha256']['U12b'] = '0' * 64
        elif mutation == 'raw-parent': policy['original_parent_sha256'] = '0' * 64
        elif mutation == 'normalized-parent': policy['normalized_parent_sha256'] = '0' * 64
        elif mutation == 'continuation': policy['opaque_continuation'] = 'Reset budget per sibling'
        elif mutation == 'old-record-proof': policy['original_normalized_units_sha256'] = '0' * 64
        elif mutation == 'estimate': policy['authored_upper_estimates']['U12b'] = 901
        elif mutation == 'joint-owner-scope': policy['derived_joint_input_owners']['C04'] = 608
        else: bad.pop('routing_kernel_splits')
        rejects(lambda: validate_forest_splits(bad))
    return len(mutations) + len(contract_mutations)


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--plan-dir',type=Path,default=Path(__file__).resolve().parent.parent);parser.add_argument('--check-completion',action='store_true');parser.add_argument('--self-test',action='store_true');args=parser.parse_args()
    root=args.plan_dir.resolve();plan=json.loads((root/'plan.json').read_text());result=validate(plan,root)
    if args.self_test:
        result['rr08_split_refusal_controls'] = self_test(plan,root)
        result['layout_split_refusal_controls'] = layout_split_self_test(plan,root)
        result['comparator_split_refusal_controls'] = comparator_split_self_test(plan,root)
        result['comparison_repair_refusal_controls'] = comparison_repair_self_test(plan,root)
        result['routing_split_refusal_controls'] = routing_split_self_test(plan,root)
        result['forest_split_refusal_controls'] = forest_split_self_test(plan,root)
        result['negative_gate_checks']='PASS'
    if args.check_completion:
        state=json.loads((root/'execution.json').read_text());errors=completion(plan,state,root.parents[1]);result.update(parity='OPEN' if errors else 'ACCEPTED',open_gate_count=len(errors),first_open_gates=errors[:8]);print(json.dumps(result,indent=2));return 2 if errors else 0
    print(json.dumps(result,indent=2));return 0

if __name__=='__main__':raise SystemExit(main())

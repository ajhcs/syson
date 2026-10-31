"""Full captured baseline and coherent corruption controls; no application processes."""
import base64
import copy
import gzip
import hashlib
import os
from pathlib import Path
import unittest
from unittest import mock
import derive as N


class DeriveTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        root = Path(os.environ['SYSON_SOURCE_NEXT_FIXTURES'])
        expected = Path(N.__file__).resolve().parent / 'fixtures'
        if not root.is_absolute() or root.resolve() != root or root != expected:
            raise ValueError('canonical declared sibling fixture root required')
        plan = Path(N.__file__).resolve().parents[2]
        cls.args = tuple((root / name).read_bytes() for name in
                         ['baseline-1.0.10.json.gz', 'baseline-provenance.json', 'lane.json', 'amendment.md'])
        cls.args += ((plan / 'tools/build_plan.py').read_bytes(), (plan / N.OWNER).read_bytes())
        cls.P, _ = N.provider()
        cls.files = N.decode(cls.P, cls.args[0], cls.args[1], {})
        cls.old = cls.P.parse(cls.files['plan.json'])
        cls.result = N.derive(*cls.args)

    def artifacts(self):
        return self.result['artifacts']

    def repack(self, changes, lock_coherent=False):
        P = self.P
        files = dict(self.files, **changes)
        if lock_coherent:
            lock = P.parse(files['completion-contract.lock.json'])
            for name in changes:
                if name in lock['sha256']:
                    lock['sha256'][name] = P.sha(files[name])
            files['completion-contract.lock.json'] = P.encode(lock)
        proof = P.parse(self.args[1])
        rows = {}
        for name, raw in files.items():
            blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
            info = {'bytes': len(raw), 'sha256': P.sha(raw), 'git_blob_sha1': blob}
            proof['rows'][name] = info
            rows[name] = dict(info, base64=base64.b64encode(raw).decode())
        decoded = P.encode({'schema': 'SYSON_PLAN_BASELINE_CAPSULE/1',
                            'base_main': proof['base_main'], 'files': rows}, True)
        capsule = gzip.compress(decoded, mtime=0)
        provenance = P.encode(proof)
        patches = {'CAPSULE_SHA': P.sha(capsule), 'CAPSULE_BYTES': len(capsule),
                   'PROVENANCE_SHA': P.sha(provenance), 'EXPANDED_SHA': P.sha(decoded),
                   'EXPANDED_BYTES': len(decoded), 'RAW_BYTES': sum(map(len, files.values()))}
        return (capsule, provenance) + self.args[2:], patches

    def deny_baseline(self, changes, expected, coherent_lock=False):
        args, patches = self.repack(changes, coherent_lock)
        # Scope ONLY capture-envelope pins, never independent baseline/row/provider policy.
        with mock.patch.multiple(N, **patches), self.assertRaisesRegex(ValueError, expected):
            N.derive(*args)

    def test_full_positive_graph_schedule_families_and_bindings(self):
        P, a = self.P, self.artifacts()
        plan, contract = a['expected-plan.json'], a['completion-contract.json']
        self.assertEqual(self.result['status'], 'UNSEALED_PROPOSAL_ONLY')
        self.assertEqual((plan['unit_count'], len(plan['units']), len(plan['readiness_graph'])), (797, 797, 1594))
        self.assertEqual(P.digest(plan['readiness_graph']),
                         '246a60123ed84d3892239777fb85a8b1e7b4cb3ddf68706ddf45d042f9b79981')
        metadata = a['source-metadata.json']
        self.assertEqual(P.digest(metadata['source_graph']),
                         '00c0cabb1bac03c0e5ea644a5d8e71fe209b17f87842b25548aa8aa820507ab3')
        self.assertEqual(P.digest(plan['source_author_dispatch_waves']),
                         '1fd740f6a9051c7dda69ed7636ba2f2352523e416aff1c9302bef3e31f4336e1')
        self.assertEqual(len(plan['source_author_dispatch_waves']), 269)
        self.assertEqual(plan['unordered_write_conflicts'], [])
        self.assertEqual(plan['lane_counts'][N.FRAGMENT], 3)
        self.assertTrue(set(N.IDS[:2]) <= set(contract['families']['interchange']))
        self.assertIn('M06a2', contract['families']['model'])
        new = {u['id']: u for u in plan['units'] if u['id'] in N.IDS}
        for uid in N.IDS[:2]:
            self.assertIn('B06 immutable-persistence handoff', new[uid]['parent_acceptance_residuals'])
            self.assertIn('framework-history', new[uid]['parent_acceptance_residuals'])
        for uid in ['P152', 'P154', 'I03', 'I04']:
            self.assertIn(uid, new['M06a2']['parent_acceptance_residuals'])
            self.assertNotIn(uid, new['M06a2']['dependencies'])
        self.assertEqual(len(contract['unit_ids']), len(set(contract['unit_ids'])))
        self.assertEqual(a['acceptance-packet.schema.json']['properties']['contract_id']['const'], N.VERSION)
        hashes = self.result['input_sha256']
        for name, raw in self.files.items():
            self.assertEqual(hashes[name], P.sha(raw))
            self.assertEqual(hashes['logical-origin/' + name + '/via/physical/baseline-1.0.10.json.gz'], P.sha(raw))
        self.assertEqual(hashes['physical/tools/build_plan.py'], P.sha(self.args[4]))
        for key in [N.OWNER, 'physical/' + N.OWNER, 'current/' + N.OWNER,
                    'logical-origin/' + N.OWNER + '/via/physical/' + N.OWNER]:
            self.assertEqual(hashes[key], P.PRESERVED[N.OWNER])
        self.assertNotIn('logical-origin/' + N.OWNER + '/via/physical/baseline-1.0.10.json.gz', hashes)

    def test_all_old_records_and_edges_restore_including_m06a1(self):
        P, a = self.P, self.artifacts()
        plan, policy = copy.deepcopy(a['expected-plan.json']), a['policy.json']
        byid = {u['id']: u for u in plan['units']}
        old = {u['id']: u for u in self.old['units']}
        for uid, fields in policy['record_updates'].items():
            for key, change in fields.items():
                if change['before']['present']:
                    byid[uid][key] = copy.deepcopy(change['before']['value'])
                else:
                    del byid[uid][key]
        restored = [{k: v for k, v in byid[uid].items() if k not in P.OMIT} for uid in sorted(old)]
        self.assertEqual(P.digest(restored), N.BASE_UNITS)
        self.assertEqual(byid['M06a1']['tasks'], old['M06a1']['tasks'])
        self.assertEqual(byid['M06a1']['source_fragment_sha256'], old['M06a1']['source_fragment_sha256'])
        graph = plan['readiness_graph']
        for node, edges in policy['graph_additions'].items():
            if node.split(':', 1)[1] in N.IDS:
                del graph[node]
            else:
                graph[node] = [e for e in graph[node] if e not in edges]
        self.assertEqual(P.digest(graph), N.BASE_GRAPH)
        for uid in ['C04', 'C05', 'C06', 'C07', 'C08', 'C09']:
            self.assertTrue(set(N.IDS) <= set(a['expected-plan.json']['units'][
                next(i for i, u in enumerate(a['expected-plan.json']['units']) if u['id'] == uid)
            ]['expanded_implementation_inputs']))
        self.assertEqual(self.result['input_sha256']['execution.json'], P.BASE_EXECUTION)
        self.assertEqual(policy['historical_unit_count'], 55)
        self.assertEqual(policy['historical_units_sha256'],
                         '082af527d8dd4140a8d4104f45c62a62b777d24c68837ed03554d14deec25321')

    def test_required_current_generator_bytes_refresh_oracle_without_execution(self):
        args = self.args[:4] + (self.args[4] + b'\n# changed current generator data; never executed\n', self.args[5])
        result = N.derive(*args)
        before = next(u for u in self.artifacts()['expected-plan.json']['units'] if u['id'] == 'C99')
        after = next(u for u in result['artifacts']['expected-plan.json']['units'] if u['id'] == 'C99')
        self.assertNotEqual(before['source_fragment_sha256'], after['source_fragment_sha256'])
        self.assertEqual(after['source_fragment_sha256'], self.P.sha(args[4]))
        self.assertEqual(result['input_sha256']['current/tools/build_plan.py'], self.P.sha(args[4]))
        self.assertEqual(self.artifacts()['policy.json'], result['artifacts']['policy.json'])
        self.assertEqual(self.artifacts()['expected-plan.json']['readiness_graph'],
                         result['artifacts']['expected-plan.json']['readiness_graph'])
        with self.assertRaises(TypeError):
            N.derive(*self.args[:4], ownership=self.args[5])

    def test_required_current_owner_refuses_before_decode(self):
        with self.assertRaises(TypeError):
            N.derive(*self.args[:5])
        with mock.patch.object(N, 'decode', side_effect=AssertionError('owner must refuse first')):
            for owner in [b'', self.args[5] + b'\n', b'x' * 65537]:
                with self.subTest(size=len(owner)), self.assertRaisesRegex(ValueError, 'current ownership'):
                    N.derive(*self.args[:5], owner)

    def test_fixed_capture_outer_pins_and_strict_formats(self):
        for index in [0, 1, 2, 3]:
            args = list(self.args)
            args[index] += b'\n'
            with self.subTest(index=index), self.assertRaises(ValueError):
                N.derive(*args)
        bad = b'not gzip'
        with mock.patch.multiple(N, CAPSULE_SHA=self.P.sha(bad), CAPSULE_BYTES=len(bad)):
            with self.assertRaisesRegex(ValueError, 'compressed'):
                N.derive(bad, *self.args[1:])
        for text in [b'{"a":1,"a":2}', b'NaN', b'Infinity', b'{"a":1e999}']:
            with self.subTest(raw=text), self.assertRaises(ValueError):
                self.P.parse(text)

    def test_coherent_old_record_graph_history_and_hold_corruptions(self):
        for mutation in ['record', 'node', 'integration', 'history', 'held']:
            if mutation in ['record', 'node']:
                value = copy.deepcopy(self.old)
                if mutation == 'record':
                    next(u for u in value['units'] if u['id'] == 'M06a1')['tasks'] = []
                else:
                    del value['readiness_graph']['accepted:M06a1']
                changes, expected = {'plan.json': self.P.encode(value)}, 'baseline records/edges'
            elif mutation in ['integration', 'history']:
                state = self.P.parse(self.files['execution.json'])
                if mutation == 'integration':
                    state['current_integration']['unknown_attribution'] = 'must not disappear'
                else:
                    state['units'][next(iter(state['units']))]['unknown_future_field'] = 'retained'
                changes, expected = {'execution.json': self.P.encode(state)}, 'contract/history'
            else:
                changes, expected = {'performance-policy.json': self.files['performance-policy.json'] + b'\n'}, 'held policy'
            with self.subTest(mutation=mutation):
                self.deny_baseline(changes, expected, coherent_lock=True)

    def test_coherent_lane_mutations_do_not_replace_independent_row_authority(self):
        for mutation in ['float', 'bool', 'family', 'tasks', 'unknown', 'duplicate']:
            lane = self.P.parse(self.args[2])
            row = lane['units'][0]
            if mutation in ['float', 'bool']:
                row['budget']['production_lines_max'] = 400.0 if mutation == 'float' else True
            elif mutation == 'family':
                row['primary_family'] = 'migrations'
            elif mutation == 'tasks':
                row['tasks'] = []
            elif mutation == 'unknown':
                row['new_unreviewed_field'] = {'must': 'not be ignored'}
            else:
                lane['units'].append(copy.deepcopy(row))
            raw = self.P.encode(lane)
            args = self.args[:2] + (raw,) + self.args[3:]
            with self.subTest(mutation=mutation), mock.patch.object(N, 'LANE_SHA', self.P.sha(raw)):
                with self.assertRaises(ValueError):
                    N.derive(*args)

    def test_coherent_capsule_coverage_names_row_fields_and_base64_refuse(self):
        for mutation in ['missing', 'namespace', 'fields', 'base64', 'bool']:
            data = self.P.parse(gzip.decompress(self.args[0]))
            proof = self.P.parse(self.args[1])
            name = next(iter(data['files']))
            row = data['files'][name]
            if mutation == 'missing':
                del data['files'][name]
            elif mutation == 'namespace':
                data['files']['physical/collision'] = data['files'].pop(name)
                proof['rows']['physical/collision'] = proof['rows'].pop(name)
            elif mutation == 'fields':
                row['unknown_extra'] = 'not ignored'
            elif mutation == 'base64':
                row['base64'] = '!' + row['base64'][1:]
            else:
                row['bytes'] = True
            expanded = self.P.encode(data, True)
            capsule = gzip.compress(expanded, mtime=0)
            provenance = self.P.encode(proof)
            patches = {'CAPSULE_SHA': self.P.sha(capsule), 'CAPSULE_BYTES': len(capsule),
                       'PROVENANCE_SHA': self.P.sha(provenance),
                       'EXPANDED_SHA': self.P.sha(expanded), 'EXPANDED_BYTES': len(expanded)}
            with self.subTest(mutation=mutation), mock.patch.multiple(N, **patches):
                with self.assertRaises(ValueError):
                    N.derive(capsule, provenance, *self.args[2:])

    def test_source_and_typed_cycles_are_distinct_and_no_unordered_overlap(self):
        units = copy.deepcopy(self.artifacts()['expected-plan.json']['units'])
        next(u for u in units if u['id'] == 'M06a1')['dependencies'] = ['M06']
        with self.assertRaisesRegex(ValueError, 'source dependency cycle'):
            N.source_metadata(self.P, units)
        units = copy.deepcopy(self.artifacts()['expected-plan.json']['units'])
        next(u for u in units if u['id'] == 'M06a1')['implementation_ready_dependencies'] = ['M06']
        joint = self.P.parse(self.files['cross-lane-review.json'])['joint_acceptance_gates']
        with self.assertRaisesRegex(ValueError, 'typed cycle'):
            self.P._graph(units, joint)
        units = copy.deepcopy(self.artifacts()['expected-plan.json']['units'])
        next(u for u in units if u['id'] == 'B25d2')['paths'] += ['unique/conflict']
        next(u for u in units if u['id'] == 'M06a2')['paths'] += ['unique/conflict']
        with self.assertRaisesRegex(ValueError, 'unordered write conflict'):
            N.source_metadata(self.P, units)

    def test_early_bounds_names_passive_import_and_no_provider_global_overrides(self):
        for name in ['prior', 'prior/x', 'physical', 'physical/x', 'logical-origin', 'logical-origin/x', '../x', '/x', 'x//y']:
            with self.subTest(name=name), self.assertRaises(ValueError):
                N.names(self.P, [name])
        huge = b'x' * (N.LIMIT + 1)
        with mock.patch.object(N, 'decode', side_effect=AssertionError('must refuse before parse')):
            with self.assertRaisesRegex(ValueError, 'aggregate input'):
                N.derive(huge, *self.args[1:])
        for generator in [b'', b'x' * 262145]:
            with self.subTest(size=len(generator)), self.assertRaisesRegex(ValueError, 'bounded actual generator'):
                N.derive(*self.args[:4], generator, self.args[5])
        source = Path(N.__file__).read_bytes()
        with mock.patch.object(Path, 'open', side_effect=AssertionError('passive import')):
            namespace = {'__file__': N.__file__}
            exec(compile(source, N.__file__, 'exec'), namespace)
        self.assertTrue(callable(namespace['derive']))
        self.assertEqual(self.P.CHILDREN, ['RR13a', 'RR13b'])
        self.assertEqual(self.P.NEW_CONTRACT, 'SYSON-RUST-COMPLETION/1.0.10')


if __name__ == '__main__':
    unittest.main()

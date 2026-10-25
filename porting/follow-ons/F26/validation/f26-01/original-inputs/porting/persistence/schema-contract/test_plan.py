#!/usr/bin/env python3
# SPDX-License-Identifier: EPL-2.0
"""Source-plan tests only; no Liquibase, JDBC driver or DBMS is invoked."""
from copy import deepcopy
import unittest
import plan


def xml(body, attrs=''):
    return (f'<databaseChangeLog xmlns="{plan.NS}" {attrs}>{body}</databaseChangeLog>').encode()


def fixture():
    return {
        plan.MASTER: xml('<include file="db/a.xml"/><include file="db/b.xml"/>'),
        plan.RESOURCE + 'db/a.xml': xml('<changeSet id="first.xml" author="one"><sql dbms="postgresql">SELECT 1</sql></changeSet>'),
        plan.RESOURCE + 'db/b.xml': xml('<changeSet id="second" author="two" context="synthetic" runInTransaction="false"><preConditions onFail="MARK_RAN"><dbms type="postgresql"/></preConditions><createTable tableName="synthetic"/><rollback><dropTable tableName="synthetic"/></rollback><validCheckSum>1:any</validCheckSum></changeSet>'),
    }


class Plans(unittest.TestCase):
    def test_frozen_candidate_graph(self):
        result = plan.candidate_plan()
        self.assertEqual((len(result['document_order']), len(result['include_order']), result['declaration_count']), (29, 28, 20))
        self.assertEqual(result['changesets'][0]['identity'][1:], ['01-initial-schema', 'sbegaudeau'])
        self.assertEqual(result['changesets'][-1]['identity'][1], '05-make-representations-identifiers-semantic-data-aware')
        self.assertEqual(result['artifact_correspondence'], 'UNPROVEN')
        self.assertEqual(result['applied_schema'], 'UNKNOWN')
        self.assertTrue(result['execution_blockers'])

    def test_depth_first_include_order_preserves_literal_identity(self):
        result = plan.build_plan(fixture())
        self.assertEqual([r['identity'][1] for r in result['changesets']], ['first.xml', 'second'])
        self.assertEqual([r['sequence'] for r in result['changesets']], [0, 1])
        self.assertEqual([r['target'] for r in result['include_order']], [plan.RESOURCE + 'db/a.xml', plan.RESOURCE + 'db/b.xml'])

    def test_relative_include_uses_declaring_file_directory(self):
        data = fixture()
        data[plan.MASTER] = xml('<include file="a.xml" relativeToChangelogFile="true"/><include file="db/b.xml"/>')
        self.assertEqual(len(plan.build_plan(data)['changesets']), 2)

    def test_nested_order_is_not_lexical_file_order(self):
        data = fixture()
        data[plan.MASTER] = xml('<include file="db/b.xml"/><include file="db/a.xml"/>')
        self.assertEqual([r['identity'][1] for r in plan.build_plan(data)['changesets']], ['second', 'first.xml'])

    def test_controls_are_retained_never_evaluated(self):
        result = plan.build_plan(fixture())
        first, second = result['changesets']
        self.assertEqual(first['declarations'][0]['attributes']['dbms'], 'postgresql')
        self.assertEqual(first['controls'], [])
        self.assertEqual(second['attributes']['runInTransaction'], 'false')
        self.assertEqual(second['attributes']['context'], 'synthetic')
        self.assertEqual([r['tag'] for r in second['controls']], ['preConditions', 'rollback', 'validCheckSum'])
        self.assertEqual(second['controls'][0]['attributes']['onFail'], 'MARK_RAN')
        self.assertEqual(second['runtime_decision'], 'UNEVALUATED')
        self.assertEqual(second['liquibase_checksum'], 'NOT_COMPUTED')

    def test_global_preconditions_are_retained(self):
        data = {plan.MASTER: xml('<preConditions onFail="HALT"><dbms type="postgresql"/></preConditions>')}
        result = plan.build_plan(data)
        self.assertEqual(result['global_controls'][0]['declaration']['attributes']['onFail'], 'HALT')

    def test_cycle_is_rejected(self):
        data = fixture()
        data[plan.RESOURCE + 'db/a.xml'] = xml('<include file="db/db.changelog-master.xml"/>')
        with self.assertRaisesRegex(ValueError, 'include cycle'):
            plan.build_plan(data)

    def test_missing_and_unreachable_documents_rejected(self):
        for mode in ['missing', 'extra']:
            data = fixture()
            if mode == 'missing':
                del data[plan.RESOURCE + 'db/a.xml']
            else:
                data[plan.RESOURCE + 'db/extra.xml'] = xml('')
            with self.assertRaisesRegex(ValueError, 'missing included|unreachable supplied'):
                plan.build_plan(data)

    def test_repeated_include_is_not_silently_deduplicated(self):
        data = {plan.MASTER: xml('<include file="db/a.xml"/><include file="db/a.xml"/>'), plan.RESOURCE + 'db/a.xml': xml('')}
        with self.assertRaisesRegex(ValueError, 'repeated include unsupported'):
            plan.build_plan(data)

    def test_duplicate_changeset_identity_rejected_but_same_id_other_file_allowed(self):
        data = {plan.MASTER: xml('<changeSet id="same" author="a"/><changeSet id="same" author="a"/>')}
        with self.assertRaisesRegex(ValueError, 'duplicate changeset'):
            plan.build_plan(data)
        data = fixture()
        data[plan.RESOURCE + 'db/b.xml'] = xml('<changeSet id="first.xml" author="one"/>')
        self.assertEqual(len(plan.build_plan(data)['changesets']), 2)

    def test_paths_cannot_escape_classpath_or_change_resolution(self):
        for name in ['../outside.xml', '/tmp/db.xml', 'db/../a.xml', 'db\\a.xml']:
            with self.assertRaisesRegex(ValueError, 'resource path'):
                plan.build_plan({plan.MASTER: xml(f'<include file="{name}"/>')})

    def test_include_controls_and_include_all_are_not_guessed(self):
        for body in ['<include file="db/a.xml" contextFilter="x"/>', '<includeAll path="db"/>', '<include file="a.xml" relativeToChangelogFile="maybe"/>']:
            with self.assertRaises(ValueError):
                plan.build_plan({plan.MASTER: xml(body)})

    def test_properties_and_logical_identity_overrides_are_unsupported(self):
        for body, attrs in [('<property name="t" value="x"/>', ''), ('<changeSet id="x" author="a"><sql>${statement}</sql></changeSet>', ''), ('', 'logicalFilePath="elsewhere.xml"'), ('<changeSet id="x" author="a" logicalFilePath="elsewhere.xml"/>', '')]:
            with self.assertRaisesRegex(ValueError, 'unsupported'):
                plan.build_plan({plan.MASTER: xml(body, attrs)})

    def test_external_payloads_and_custom_execution_are_not_byte_closed(self):
        for operation in ['<sqlFile path="external.sql"/>', '<loadData file="data.csv"/>', '<customChange class="example.Custom"/>', '<createProcedure path="external.sql"/>', '<executeCommand executable="sh"/>']:
            with self.assertRaisesRegex(ValueError, 'external payload|custom execution'):
                plan.build_plan({plan.MASTER: xml('<changeSet id="x" author="a">' + operation + '</changeSet>')})

    def test_namespace_dtd_and_missing_identity_rejected(self):
        for raw in [b'<databaseChangeLog/>', b'<!DOCTYPE x><databaseChangeLog/>', xml('<changeSet id="x"/>')]:
            with self.assertRaises(ValueError):
                plan.build_plan({plan.MASTER: raw})

    def test_source_record_hash_revision_and_duplicates_are_checked(self):
        raw = xml('')
        record = {'path': plan.MASTER, 'source_commit': plan.PIN, 'xml': raw.decode(), 'sha256': plan.sha(raw)}
        self.assertEqual(plan.documents([record]), {plan.MASTER: raw})
        for key, value in [('sha256', '0' * 64), ('source_commit', '0' * 40)]:
            changed = deepcopy(record)
            changed[key] = value
            with self.assertRaises(ValueError):
                plan.documents([changed])
        with self.assertRaisesRegex(ValueError, 'duplicate source document'):
            plan.documents([record, record])

    def test_duplicate_json_keys_are_rejected(self):
        with self.assertRaisesRegex(ValueError, 'duplicate JSON key'):
            plan.strict_json('{"id":"a","id":"b"}')


if __name__ == '__main__':
    unittest.main()

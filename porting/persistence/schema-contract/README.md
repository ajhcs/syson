# F26: candidate schema declaration plan

The planner freezes one retained Liquibase XML graph: 29 documents, 28 includes
and 20 changeSets attributed to Sirius candidate
`3191e2b8990373afee3415f6b26f8c5039a28321`. It traverses includes in XML encounter
order and preserves file/id/author identity, operation trees, attributes, SQL text
and unevaluated controls. Character text and child tails are retained; comments
and XML prefix spellings remain in the original retained XML capsule. It does not compute a final schema or select changes
for execution.

The frozen [candidate plan](candidate-plan.json) is generated from byte-retained
XML already tracked at integration `ad814c232ee24109639c1d52239cbeec6f4a3e92`.
The [source manifest](source-manifest.json) pins four integration Git objects:
that XML capsule, the inherited Maven declarations, the dependency source audit
and SysON's POM. The checker proves supplied bytes match those Git objects and
per-document hashes. The candidate revision association remains captured source
metadata; no source JAR or deployed artifact was independently matched here.
The similarly named schema summaries in the existing DB inventory are retained
unchanged, rather than promoted to applied-schema evidence.

## Declaration behavior and identity

This finite family admits explicit `include` paths only. Without
`relativeToChangelogFile`, a path is resolved against the captured classpath
resource root; an explicit true resolves against the declaring XML directory.
Traversal never searches disk or the network. Files must use canonical paths
within that resource root. Parent traversal and repeated includes are rejected as
conservative planner admission rules, not claims about every Liquibase version.
Cycles, missing/unreachable documents, duplicate documents and duplicate
file/id/author tuples fail. An identical id/author in a different file remains
a distinct declaration. Literal punctuation is preserved: the source id
`01-add-is-read-only-column-in-document-table.xml` is not stripped or repaired.

Unknown include attributes, includeAll, properties/substitution and
logicalFilePath overrides are outside this family. External payload references
such as sqlFile/loadData/createProcedure paths and custom execution are rejected;
XML closure alone cannot prove their bytes. Inline SQL remains opaque; referenced
functions, objects or external files require separate runtime/source review. Namespace errors and DTD/entity
input are not admitted. The checks cover a declaration plan, not arbitrary
Liquibase XML schema validation.

The captured graph has no preConditions or rollback elements and no changeSet
DBMS/context/label/runAlways/runOnChange/runInTransaction attributes. One inline
SQL operation declares `dbms="postgresql"` for `CREATE EXTENSION ... pgcrypto`.
These are source facts only: missing explicit rollback is not proof that
Liquibase cannot infer rollback for an operation, and an SQL-level DBMS attribute
is not proof that other operations support every database. The synthetic plan
and tests retain preconditions, context/transaction attributes, rollback and
validCheckSum data, with runtime decisions still **UNEVALUATED**.

SHA-256 hashes identify supplied source bytes. They are not Liquibase's stored
MD5SUM/checksum algorithm, DATABASECHANGELOG history or evidence that an existing
migration is safe to change. Defaults such as transaction policy remain runtime
questions; the planner supplies none.

## Artifact correspondence and execution gate

The SysON POM declares Sirius `2026.9.3` and EMF JSON `2.5.5-SNAPSHOT`; the inherited
Maven data does not establish effective resolution. The SNAPSHOT timestamp/build,
artifact/source JAR digests, transitive resolution, exact Liquibase/JDBC versions
and deployed source correspondence remain unknown. The plan always reports
`artifact_correspondence=UNPROVEN`, `applied_schema=UNKNOWN` and database execution
`NOT_RUN`. Source checks passing cannot close those blockers.

[execution-review.json](execution-review.json) records the ordered command
**templates for a separate execution review** and the exact unresolved bindings.
No executable command is frozen while its binary/classpath/JDBC target and
history/extension baseline are unknown. No database, migration tool, Maven/npm
lifecycle, container, service, extension installation or credential action was
run. A future execution request must supply and review an immutable artifact/tool
manifest, a disposable workspace-backed synthetic DB target, history/checksum
state, required PostgreSQL extension permissions and a rollback/retention plan.
No production URL or credential is part of this packet.

## Local checks

From the repository root with its pinned Git objects available:

```sh
python3 -B porting/persistence/schema-contract/plan.py
python3 -B -m unittest discover -s porting/persistence/schema-contract -v
```

`plan.py --write` regenerates only candidate-plan.json from the frozen source;
it never invokes Liquibase or a database. The [synthetic plan](synthetic-plan.json)
is a separate two-changeSet example, not a projection of a real database. Tests
exercise ordering, relative paths, identity, retained controls and negative
closure/admission cases. Actual captures and their input hashes are under
`porting/follow-ons/F26/validation/`. Whole DB parity, applied schema, SQL semantics,
transactionality, rollback, crash recovery and a resolved-artifact schema remain
unqualified.

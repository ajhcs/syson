# SPDX-License-Identifier: EPL-2.0
"""Private fixed-operation SQLite bridge; candidate format, not a SysON schema."""
import json
import pathlib
import sqlite3
import sys

LIMIT = 8 * 1024 * 1024
APP = 0x46323731
DDL = "CREATE TABLE f27_documents(project TEXT NOT NULL, document TEXT NOT NULL, name TEXT NOT NULL, content TEXT NOT NULL CHECK(length(CAST(content AS BLOB)) <= 1048576), read_only INTEGER NOT NULL CHECK(read_only IN (0,1)), revision INTEGER NOT NULL CHECK(revision > 0 AND typeof(revision) = 'integer'), PRIMARY KEY(project, document)) WITHOUT ROWID"

CLOCK = "CREATE TABLE f27_clock(singleton INTEGER PRIMARY KEY CHECK(singleton=1), value INTEGER NOT NULL CHECK(value >= 0 AND typeof(value) = 'integer'))"

def require(value, error):
    if not value:
        raise ValueError(error)


def main():
    raw = sys.stdin.buffer.read(LIMIT + 1)
    require(len(raw) <= LIMIT, 'limit')
    request = json.loads(raw)
    require(set(request) == {'action', 'value'}, 'protocol')
    action, value = request['action'], request['value']
    require(action in {'initialize', 'load', 'transact'}, 'protocol')
    path = pathlib.Path(sys.argv[1])
    require(path.is_absolute() and path.is_file() and not path.is_symlink(), 'path')
    uri = path.as_uri() + ('?mode=ro' if action == 'load' else '?mode=rw')
    with sqlite3.connect(uri, uri=True, timeout=1, isolation_level=None) as db:
        db.execute('PRAGMA temp_store=MEMORY')
        schema = db.execute("SELECT name, type, sql FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY name").fetchall()
        if action == 'initialize':
            require(not schema and db.execute('PRAGMA application_id').fetchone()[0] == 0, 'format')
            db.execute('PRAGMA journal_mode=DELETE')
            db.execute('PRAGMA synchronous=FULL')
            db.execute('BEGIN IMMEDIATE')
            try:
                db.execute(DDL)
                db.execute(CLOCK)
                db.execute('INSERT INTO f27_clock VALUES(1,0)')
                db.execute(f'PRAGMA application_id={APP}')
                db.execute('PRAGMA user_version=1')
                db.execute('COMMIT')
            except Exception:
                db.execute('ROLLBACK')
                raise
            return {'ok': True}
        require(db.execute('PRAGMA application_id').fetchone()[0] == APP
                and db.execute('PRAGMA user_version').fetchone()[0] == 1
                and schema == [('f27_clock', 'table', CLOCK), ('f27_documents', 'table', DDL)], 'format')
        if action == 'load':
            require(isinstance(value, list) and len(value) == 2, 'protocol')
            row = db.execute('SELECT name, content, read_only, revision FROM f27_documents WHERE project=? AND document=?', value).fetchone()
            return {'row': row}
        require(isinstance(value, list) and 0 < len(value) <= 64, 'limit')
        db.execute('PRAGMA synchronous=FULL')
        db.execute('BEGIN IMMEDIATE')
        try:
            for operation in value:
                require(set(operation) == {'kind', 'key', 'name', 'content', 'revision', 'changed', 'read_only'}, 'protocol')
                key, kind = operation['key'], operation['kind']
                require(isinstance(key, list) and len(key) == 2, 'protocol')
                require(kind in {'create', 'update', 'delete'}, 'protocol')
                row = db.execute('SELECT read_only, revision FROM f27_documents WHERE project=? AND document=?', key).fetchone()
                if kind == 'create':
                    db.execute('UPDATE f27_clock SET value=value+1 WHERE singleton=1')
                    revision = db.execute('SELECT value FROM f27_clock WHERE singleton=1').fetchone()[0]
                    db.execute('INSERT INTO f27_documents VALUES(?,?,?,?,?,?)', [*key, operation['name'], operation['content'], int(operation['read_only']), revision])
                else:
                    require(row is not None, 'missing')
                    require(not row[0], 'read-only')
                    require(row[1] == operation['revision'], 'stale')
                    if kind == 'delete':
                        db.execute('DELETE FROM f27_documents WHERE project=? AND document=?', key)
                    elif operation['changed']:
                        db.execute('UPDATE f27_clock SET value=value+1 WHERE singleton=1')
                        revision = db.execute('SELECT value FROM f27_clock WHERE singleton=1').fetchone()[0]
                        db.execute('UPDATE f27_documents SET name=?, content=?, read_only=0, revision=? WHERE project=? AND document=?', [operation['name'], operation['content'], revision, *key])
            db.execute('COMMIT')
        except Exception:
            db.execute('ROLLBACK')
            raise
        return {'ok': True}


try:
    result = main()
except ValueError as error:
    result = {'error': str(error) if str(error) in {'limit', 'protocol', 'path', 'format', 'missing', 'read-only', 'stale'} else 'protocol'}
except sqlite3.Error:
    result = {'error': 'sql'}
except Exception:
    result = {'error': 'driver'}
encoded = json.dumps(result, ensure_ascii=False).encode()
sys.stdout.buffer.write(encoded if len(encoded) <= LIMIT else b'{"error":"limit"}')

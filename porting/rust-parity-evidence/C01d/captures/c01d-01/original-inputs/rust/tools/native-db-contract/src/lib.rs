// SPDX-License-Identifier: EPL-2.0
//! Projection of a coordinator-captured PostgreSQL catalog, never a DB client.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::path::Path;

pub const MAX_BYTES: usize = 8 * 1024 * 1024;
/// Candidate tool limit, not a PostgreSQL identifier-width assertion.
pub const MAX_IDENTITY_BYTES: usize = 4096;
type Result<T> = std::result::Result<T, String>;
fn need(ok: bool, message: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(message.into()) }
}
fn fields(value: &Value, names: &[&str]) -> Result<()> {
    let object = value.as_object().ok_or("expected object")?;
    need(
        object.len() == names.len() && names.iter().all(|n| object.contains_key(*n)),
        "unknown/missing descriptor fields",
    )
}
fn text(value: &Value) -> Result<&str> {
    value
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("expected nonempty string".into())
}
fn name(value: &Value) -> Result<&str> {
    let value = text(value)?;
    need(
        value.len() <= MAX_IDENTITY_BYTES,
        "candidate identity byte limit",
    )?;
    Ok(value)
}
struct Counter(usize);
impl Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_BYTES)
            .ok_or_else(|| std::io::Error::other("contract output byte limit"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn reserve(budget: &mut Counter, value: &Value) -> Result<()> {
    serde_json::to_writer(&mut *budget, value).map_err(|e| e.to_string())?;
    budget.write_all(b",").map_err(|e| e.to_string())
}
fn strings(value: &Value, names: &[&str]) -> Result<()> {
    for name in names {
        text(&value[*name])?;
    }
    Ok(())
}
fn nullable_strings(value: &Value, names: &[&str]) -> Result<()> {
    for name in names {
        need(
            value[*name].is_null() || value[*name].is_string(),
            "expected nullable string",
        )?;
    }
    Ok(())
}
fn flags(value: &Value, names: &[&str]) -> Result<()> {
    for name in names {
        need(value[*name].is_boolean(), "expected boolean")?;
    }
    Ok(())
}
fn number(value: &Value) -> Result<u64> {
    value.as_u64().ok_or("expected unsigned integer".into())
}
fn oid(value: &Value) -> Result<u64> {
    let n = number(value)?;
    need(n > 0 && n <= u32::MAX as u64, "invalid PostgreSQL OID")?;
    Ok(n)
}
fn optional_oid(value: &Value) -> Result<Option<u64>> {
    if value.is_null() {
        Ok(None)
    } else {
        oid(value).map(Some)
    }
}
fn array(value: &Value, max: usize) -> Result<&Vec<Value>> {
    let rows = value.as_array().ok_or("expected array")?;
    need(rows.len() <= max, "descriptor count limit")?;
    Ok(rows)
}
fn unique<T: Ord>(set: &mut BTreeSet<T>, key: T) -> Result<()> {
    need(set.insert(key), "duplicate descriptor identity")
}
fn hash(value: &Value, size: usize) -> Result<()> {
    let s = text(value)?;
    need(
        s.len() == size
            && s.bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
        "invalid lowercase digest",
    )
}
fn relation(tables: &BTreeMap<u64, Vec<String>>, value: &Value) -> Result<Vec<String>> {
    tables
        .get(&oid(value)?)
        .cloned()
        .ok_or("dangling table reference".into())
}
fn column_refs(
    value: &Value,
    table: u64,
    columns: &BTreeMap<u64, BTreeMap<u64, String>>,
    expressions: bool,
) -> Result<()> {
    for item in array(value, 32767)? {
        let n = number(item)?;
        need(
            (expressions && n == 0) || columns.get(&table).is_some_and(|c| c.contains_key(&n)),
            "dangling column reference",
        )?;
    }
    Ok(())
}
fn record(
    rows: &mut Vec<Value>,
    budget: &mut Counter,
    kind: &str,
    key: Vec<String>,
    row: &Value,
) -> Result<()> {
    let identity = serde_json::to_string(&key).map_err(|e| e.to_string())?;
    let entry = json!({
        "id":format!("B05/native/{kind}/{identity}"),
        "owner":"B05", "kind":kind, "key":key, "observed":row,
        "state":"NATIVE_CAPTURED_RUST_COMPARISON_PENDING"
    });
    reserve(budget, &entry)?;
    rows.push(entry);
    Ok(())
}

/// Bounded regular-file input. This reads supplied bytes; capture provenance is
/// established by the coordinator's retained SQL, runner and native raw output.
pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    need(
        std::fs::symlink_metadata(path)
            .map_err(|e| e.to_string())?
            .is_file(),
        "input is not a regular nonsymlink file",
    )?;
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    need(
        file.metadata().map_err(|e| e.to_string())?.is_file(),
        "input is not regular",
    )?;
    let mut bytes = Vec::new();
    file.take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    need(bytes.len() <= MAX_BYTES, "catalog byte limit")?;
    Ok(bytes)
}

/// Preserve opaque PostgreSQL descriptors, check relational integrity and emit
/// stable named B05 obligations. Success is not native/Rust schema equivalence.
pub fn project(bytes: &[u8]) -> Result<Value> {
    need(bytes.len() <= MAX_BYTES, "catalog byte limit")?;
    let input: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    fields(
        &input,
        &[
            "schema_version",
            "capture",
            "tables",
            "columns",
            "constraints",
            "indexes",
            "changelog",
        ],
    )?;
    need(input["schema_version"] == 1, "unsupported schema version")?;
    let capture = &input["capture"];
    fields(
        capture,
        &[
            "artifact_sha256",
            "target_revision",
            "source_label_revision",
            "query_sha256",
            "database",
            "server_version",
            "server_version_num",
            "schemas",
            "selected_relation_kinds",
            "search_path",
            "transaction_isolation",
            "transaction_read_only",
            "changelog_table",
            "captured_at",
        ],
    )?;
    for name in ["artifact_sha256", "query_sha256"] {
        hash(&capture[name], 64)?;
    }
    for name in ["target_revision", "source_label_revision"] {
        hash(&capture[name], 40)?;
    }
    strings(
        capture,
        &["database", "server_version", "search_path", "captured_at"],
    )?;
    need(
        number(&capture["server_version_num"])? > 0,
        "invalid server version",
    )?;
    need(
        capture["selected_relation_kinds"] == json!(["r", "p"]),
        "unexpected relation-kind scope",
    )?;
    need(
        capture["transaction_isolation"] == "repeatable read"
            && capture["transaction_read_only"] == "on",
        "capture must declare a read-only repeatable-read snapshot",
    )?;
    let mut schemas = BTreeSet::new();
    for schema in array(&capture["schemas"], 512)? {
        unique(&mut schemas, name(schema)?.to_owned())?;
    }
    let history_table = &capture["changelog_table"];
    fields(history_table, &["schema", "name"])?;
    strings(history_table, &["schema", "name"])?;
    let mut output = json!({
        "schema_version":1, "kind":"syson-native-postgresql-contract", "capture":capture,
        "obligations":[], "migration_order":[], "parent_C01":"OPEN", "whole_build_source_binding":"OPEN",
        "schema_parity":"NOT_ESTABLISHED",
        "remaining_scope":["sequences","views","functions","triggers","policies","privileges","extension/type definitions","effective transaction behavior","Rust comparison"]
    });
    let mut budget = Counter(0);
    reserve(&mut budget, &output)?;
    let mut tables = BTreeMap::new();
    let mut table_names = BTreeSet::new();
    let mut obligations = Vec::new();
    for row in array(&input["tables"], 512)? {
        fields(row, &["oid", "schema", "name", "relkind", "relpersistence"])?;
        strings(row, &["schema", "name", "relkind", "relpersistence"])?;
        need(
            schemas.contains(name(&row["schema"])?),
            "table outside captured schemas",
        )?;
        need(
            row["relkind"] == "r" || row["relkind"] == "p",
            "unexpected table relation kind",
        )?;
        let key = vec![name(&row["schema"])?.into(), name(&row["name"])?.into()];
        unique(&mut table_names, key.clone())?;
        need(
            tables.insert(oid(&row["oid"])?, key.clone()).is_none(),
            "duplicate table OID",
        )?;
        record(&mut obligations, &mut budget, "table", key, row)?;
    }
    need(
        table_names.contains(&vec![
            name(&history_table["schema"])?.into(),
            name(&history_table["name"])?.into(),
        ]),
        "migration-history table missing",
    )?;
    let mut columns: BTreeMap<u64, BTreeMap<u64, String>> = BTreeMap::new();
    let mut column_names = BTreeSet::new();
    for row in array(&input["columns"], 16384)? {
        fields(
            row,
            &[
                "table_oid",
                "attnum",
                "name",
                "type_oid",
                "formatted_type",
                "not_null",
                "default_expr",
                "identity",
                "generated",
                "collation",
            ],
        )?;
        strings(row, &["name", "formatted_type"])?;
        nullable_strings(row, &["default_expr", "collation"])?;
        need(
            row["identity"].is_string() && row["generated"].is_string(),
            "invalid identity/generated descriptor",
        )?;
        flags(row, &["not_null"])?;
        oid(&row["type_oid"])?;
        let table = oid(&row["table_oid"])?;
        let mut key = relation(&tables, &row["table_oid"])?;
        key.push(name(&row["name"])?.into());
        unique(&mut column_names, key.clone())?;
        let attnum = number(&row["attnum"])?;
        need(
            attnum > 0 && attnum <= i16::MAX as u64,
            "invalid attribute number",
        )?;
        need(
            columns
                .entry(table)
                .or_default()
                .insert(attnum, name(&row["name"])?.into())
                .is_none(),
            "duplicate attribute number",
        )?;
        record(&mut obligations, &mut budget, "column", key, row)?;
    }
    let mut indexes = BTreeMap::new();
    let mut index_names = BTreeSet::new();
    for row in array(&input["indexes"], 8192)? {
        fields(
            row,
            &[
                "oid",
                "table_oid",
                "schema",
                "name",
                "access_method",
                "definition",
                "unique",
                "primary",
                "valid",
                "ready",
                "predicate",
                "expressions",
                "key_attnums",
                "key_count",
            ],
        )?;
        strings(row, &["schema", "name", "access_method", "definition"])?;
        nullable_strings(row, &["predicate", "expressions"])?;
        flags(row, &["unique", "primary", "valid", "ready"])?;
        let table = oid(&row["table_oid"])?;
        let table_key = relation(&tables, &row["table_oid"])?;
        need(
            name(&row["schema"])? == table_key[0],
            "index/table schema mismatch",
        )?;
        let key = vec![name(&row["schema"])?.into(), name(&row["name"])?.into()];
        unique(&mut index_names, key.clone())?;
        need(
            !table_names.contains(&key),
            "index/table relation-name collision",
        )?;
        let index = oid(&row["oid"])?;
        need(
            !tables.contains_key(&index) && indexes.insert(index, table).is_none(),
            "duplicate/colliding index OID",
        )?;
        column_refs(&row["key_attnums"], table, &columns, true)?;
        let count = number(&row["key_count"])?;
        need(
            count > 0 && count <= array(&row["key_attnums"], 32767)?.len() as u64,
            "invalid index key/include count",
        )?;
        record(&mut obligations, &mut budget, "index", key, row)?;
    }
    let mut constraint_ids = BTreeSet::new();
    let mut constraint_names = BTreeSet::new();
    for row in array(&input["constraints"], 8192)? {
        fields(
            row,
            &[
                "oid",
                "table_oid",
                "name",
                "contype",
                "definition",
                "validated",
                "deferrable",
                "initially_deferred",
                "referenced_table_oid",
                "index_oid",
                "columns",
                "referenced_columns",
            ],
        )?;
        strings(row, &["name", "contype", "definition"])?;
        flags(row, &["validated", "deferrable", "initially_deferred"])?;
        let table = oid(&row["table_oid"])?;
        let mut key = relation(&tables, &row["table_oid"])?;
        key.push(name(&row["name"])?.into());
        unique(&mut constraint_names, key.clone())?;
        unique(&mut constraint_ids, oid(&row["oid"])?)?;
        column_refs(&row["columns"], table, &columns, row["contype"] == "x")?;
        let referenced = optional_oid(&row["referenced_table_oid"])?;
        if let Some(target) = referenced {
            relation(&tables, &json!(target))?;
            column_refs(&row["referenced_columns"], target, &columns, false)?;
            need(
                array(&row["columns"], 32767)?.len()
                    == array(&row["referenced_columns"], 32767)?.len(),
                "foreign-key column count mismatch",
            )?;
        } else {
            need(
                array(&row["referenced_columns"], 32767)?.is_empty(),
                "referenced columns without relation",
            )?;
        }
        if let Some(index) = optional_oid(&row["index_oid"])? {
            let owner = indexes
                .get(&index)
                .ok_or("dangling backing-index reference")?;
            need(
                *owner == referenced.unwrap_or(table),
                "backing-index table mismatch",
            )?;
        }
        record(&mut obligations, &mut budget, "constraint", key, row)?;
    }
    let history = array(&input["changelog"], 8192)?;
    need(!history.is_empty(), "applied migration history empty")?;
    let mut history_ids = BTreeSet::new();
    let mut positions = BTreeMap::new();
    for row in history {
        fields(
            row,
            &[
                "position",
                "id",
                "author",
                "filename",
                "dateexecuted",
                "orderexecuted",
                "exectype",
                "md5sum",
                "description",
                "comments",
                "tag",
                "liquibase",
                "contexts",
                "labels",
                "deployment_id",
            ],
        )?;
        strings(
            row,
            &["id", "author", "filename", "dateexecuted", "exectype"],
        )?;
        nullable_strings(
            row,
            &[
                "md5sum",
                "description",
                "comments",
                "tag",
                "liquibase",
                "contexts",
                "labels",
                "deployment_id",
            ],
        )?;
        need(
            row["orderexecuted"].as_i64().is_some(),
            "invalid migration execution order",
        )?;
        let position = number(&row["position"])?;
        need(position < history.len() as u64, "migration position gap")?;
        let key = vec![
            name(&row["id"])?.into(),
            name(&row["author"])?.into(),
            name(&row["filename"])?.into(),
        ];
        unique(&mut history_ids, key.clone())?;
        let id = format!(
            "B05/native/migration/{}",
            serde_json::to_string(&key).map_err(|e| e.to_string())?
        );
        reserve(&mut budget, &json!(id))?;
        need(
            positions.insert(position, id).is_none(),
            "duplicate migration position",
        )?;
        record(&mut obligations, &mut budget, "migration", key, row)?;
    }
    obligations.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    output["obligations"] = Value::Array(obligations);
    output["migration_order"] = Value::Array(positions.into_values().map(Value::String).collect());
    serde_json::to_writer(Counter(0), &output).map_err(|e| e.to_string())?;
    Ok(output)
}

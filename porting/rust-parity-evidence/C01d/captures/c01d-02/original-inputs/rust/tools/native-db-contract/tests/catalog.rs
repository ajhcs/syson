// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use syson_native_db_contract::{MAX_BYTES, project};

// Deliberately synthetic descriptors; native observations are separate evidence.
fn fixture() -> Value {
    let column = |table, attnum, name, data_type, default| {
        json!({
            "table_oid":table,"attnum":attnum,"name":name,"type_oid":25,
            "formatted_type":data_type,"not_null":true,"default_expr":default,
            "identity":"","generated":"","collation":null
        })
    };
    json!({
        "schema_version":1,
        "capture":{
            "artifact_sha256":"a".repeat(64),"target_revision":"b".repeat(40),
            "source_label_revision":"c".repeat(40),"query_sha256":"d".repeat(64),
            "database":"owned-fixture","server_version":"17.6","server_version_num":170006,
            "schemas":["public"],"selected_relation_kinds":["r","p"],"search_path":"pg_catalog, public",
            "transaction_isolation":"repeatable read","transaction_read_only":"on",
            "changelog_table":{"schema":"public","name":"databasechangelog"},"captured_at":"2026-10-26 12:00:00+00"
        },
        "tables":[
            {"oid":10,"schema":"public","name":"databasechangelog","relkind":"r","relpersistence":"p"},
            {"oid":20,"schema":"public","name":"project","relkind":"r","relpersistence":"p"},
            {"oid":30,"schema":"public","name":"representation","relkind":"p","relpersistence":"p"}
        ],
        "columns":[
            column(10,1,"id","character varying(255)",Value::Null),
            column(20,1,"id","uuid",json!("gen_random_uuid()")),
            column(20,2,"name","text COLLATE \"odd.locale\"",json!("'quoted ''Ω''; -- opaque'::text")),
            column(30,1,"id","uuid",Value::Null),
            column(30,2,"project_id","uuid",Value::Null),
            column(30,3,"body","custom_schema.\"strange type\"[]",Value::Null)
        ],
        "indexes":[
            {"oid":21,"table_oid":20,"schema":"public","name":"project_pkey","access_method":"btree",
             "definition":"CREATE UNIQUE INDEX project_pkey ON public.project USING btree (id)",
             "unique":true,"primary":true,"valid":true,"ready":true,"predicate":null,"expressions":null,
             "key_attnums":[1],"key_count":1},
            {"oid":31,"table_oid":30,"schema":"public","name":"body_expression","access_method":"btree",
             "definition":"CREATE INDEX body_expression ON public.representation ((lower(body))) INCLUDE (project_id)",
             "unique":false,"primary":false,"valid":false,"ready":false,"predicate":"body IS NOT NULL",
             "expressions":"lower(body)","key_attnums":[0,2],"key_count":1}
        ],
        "constraints":[
            {"oid":100,"table_oid":20,"name":"project_pkey","contype":"p","definition":"PRIMARY KEY (id)",
             "validated":true,"deferrable":false,"initially_deferred":false,"referenced_table_oid":null,
             "index_oid":21,"columns":[1],"referenced_columns":[]},
            {"oid":101,"table_oid":30,"name":"fk_project","contype":"f",
             "definition":"FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE",
             "validated":false,"deferrable":true,"initially_deferred":true,"referenced_table_oid":20,
             "index_oid":21,"columns":[2],"referenced_columns":[1]}
        ],
        "changelog":[
            {"position":0,"id":"initial","author":"native","filename":"db/initial.xml",
             "dateexecuted":"2026-10-26 12:00:00","orderexecuted":1,"exectype":"EXECUTED",
             "md5sum":null,"description":"raw description","comments":"","tag":null,"liquibase":"4.x",
             "contexts":null,"labels":null,"deployment_id":"123"},
            {"position":1,"id":"later","author":"native","filename":"db/later.xml",
             "dateexecuted":"2026-10-26 12:00:01","orderexecuted":1,"exectype":"MARK_RAN",
             "md5sum":"9:opaque-checksum","description":null,"comments":null,"tag":"fixture",
             "liquibase":null,"contexts":"(local OR test)","labels":"a,b","deployment_id":null}
        ]
    })
}
fn run(input: &Value) -> Result<Value, String> {
    project(&serde_json::to_vec(input).unwrap())
}
fn observed<'a>(output: &'a Value, kind: &str, key: &[&str]) -> &'a Value {
    &output["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["kind"] == kind && o["key"] == json!(key))
        .unwrap()["observed"]
}
fn refuses(mut input: Value, change: impl FnOnce(&mut Value), expected: &str) {
    change(&mut input);
    let error = run(&input).unwrap_err();
    assert!(
        error.contains(expected),
        "{error:?} did not contain {expected:?}"
    );
}

#[test]
fn preserves_actual_descriptors_without_type_or_expression_normalization() {
    let input = fixture();
    let output = run(&input).unwrap();
    assert_eq!(output["capture"], input["capture"]);
    assert_eq!(output["obligations"].as_array().unwrap().len(), 15);
    assert_eq!(
        observed(&output, "column", &["public", "project", "name"]),
        &input["columns"][2]
    );
    assert_eq!(
        observed(&output, "column", &["public", "representation", "body"]),
        &input["columns"][5]
    );
    assert_eq!(
        observed(&output, "index", &["public", "body_expression"]),
        &input["indexes"][1]
    );
    assert_eq!(
        observed(
            &output,
            "constraint",
            &["public", "representation", "fk_project"]
        ),
        &input["constraints"][1]
    );
    assert_eq!(
        output["migration_order"],
        json!([
            "B05/native/migration/[\"initial\",\"native\",\"db/initial.xml\"]",
            "B05/native/migration/[\"later\",\"native\",\"db/later.xml\"]"
        ])
    );
    assert_eq!(output["schema_parity"], "NOT_ESTABLISHED");
    assert_eq!(output["parent_C01"], "OPEN");
    assert!(output["obligations"].as_array().unwrap().iter().all(|o| o["owner"]=="B05" && o["state"]=="NATIVE_CAPTURED_RUST_COMPARISON_PENDING"));
}

#[test]
fn canonical_order_ignores_capture_row_order_and_uses_unambiguous_keys() {
    let input = fixture();
    let mut reversed = input.clone();
    for name in ["tables", "columns", "indexes", "constraints", "changelog"] {
        reversed[name].as_array_mut().unwrap().reverse();
    }
    assert_eq!(run(&input).unwrap(), run(&reversed).unwrap());
    let mut quoted = fixture();
    quoted["tables"][1]["name"] = json!("project/\".Ω");
    let output = run(&quoted).unwrap();
    let row = output["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["key"] == json!(["public", "project/\".Ω"]))
        .unwrap();
    assert_eq!(
        row["id"],
        format!(
            "B05/native/table/{}",
            serde_json::to_string(&vec!["public", "project/\".Ω"]).unwrap()
        )
    );
}

#[test]
fn rejects_duplicate_catalog_and_history_identities() {
    for name in ["tables", "columns", "indexes", "constraints", "changelog"] {
        refuses(
            fixture(),
            |v| {
                let row = v[name][0].clone();
                v[name].as_array_mut().unwrap().push(row);
            },
            "duplicate",
        );
    }
    refuses(
        fixture(),
        |v| v["tables"][1]["oid"] = json!(10),
        "duplicate table OID",
    );
    refuses(
        fixture(),
        |v| v["columns"][2]["attnum"] = json!(1),
        "duplicate attribute",
    );
    refuses(
        fixture(),
        |v| v["indexes"][1]["oid"] = json!(21),
        "index OID",
    );
    refuses(
        fixture(),
        |v| v["constraints"][1]["oid"] = json!(100),
        "duplicate",
    );
    refuses(
        fixture(),
        |v| v["changelog"][1]["position"] = json!(0),
        "duplicate migration position",
    );
}

#[test]
fn rejects_dangling_table_column_and_index_references() {
    for name in ["columns", "constraints", "indexes"] {
        refuses(
            fixture(),
            |v| v[name][0]["table_oid"] = json!(999),
            "dangling table",
        );
    }
    refuses(
        fixture(),
        |v| v["constraints"][1]["columns"] = json!([99]),
        "dangling column",
    );
    refuses(
        fixture(),
        |v| v["constraints"][1]["referenced_table_oid"] = json!(999),
        "dangling table",
    );
    refuses(
        fixture(),
        |v| v["constraints"][1]["referenced_columns"] = json!([99]),
        "dangling column",
    );
    refuses(
        fixture(),
        |v| v["constraints"][0]["index_oid"] = json!(999),
        "dangling backing-index",
    );
    refuses(
        fixture(),
        |v| v["indexes"][1]["key_attnums"] = json!([0, 99]),
        "dangling column",
    );
}

#[test]
fn foreign_index_belongs_to_target_and_expression_include_references_remain_valid() {
    assert!(run(&fixture()).is_ok()); // FK conindid points to project, not representation.
    refuses(
        fixture(),
        |v| v["constraints"][1]["index_oid"] = json!(31),
        "backing-index table mismatch",
    );
    refuses(
        fixture(),
        |v| v["constraints"][0]["index_oid"] = json!(31),
        "backing-index table mismatch",
    );
    refuses(
        fixture(),
        |v| v["constraints"][1]["referenced_columns"] = json!([1, 2]),
        "column count mismatch",
    );
    refuses(
        fixture(),
        |v| v["constraints"][0]["referenced_columns"] = json!([1]),
        "without relation",
    );
    refuses(
        fixture(),
        |v| v["indexes"][1]["key_count"] = json!(3),
        "key/include count",
    );
    refuses(
        fixture(),
        |v| v["indexes"][1]["key_count"] = json!(0),
        "key/include count",
    );
    let mut excluded = fixture();
    excluded["constraints"].as_array_mut().unwrap().push(json!({
        "oid":102,"table_oid":30,"name":"expression_exclusion","contype":"x","definition":"EXCLUDE USING gist ((lower(body)) WITH =)",
        "validated":true,"deferrable":false,"initially_deferred":false,"referenced_table_oid":null,
        "index_oid":31,"columns":[0],"referenced_columns":[]
    }));
    assert!(run(&excluded).is_ok());
}

#[test]
fn rejects_missing_history_scope_and_malformed_descriptors() {
    refuses(fixture(), |v| v["changelog"] = json!([]), "history empty");
    refuses(
        fixture(),
        |v| v["changelog"][1]["position"] = json!(3),
        "position gap",
    );
    refuses(
        fixture(),
        |v| v["capture"]["changelog_table"]["name"] = json!("missing"),
        "history table missing",
    );
    refuses(
        fixture(),
        |v| v["capture"]["transaction_read_only"] = json!("off"),
        "read-only repeatable-read",
    );
    refuses(
        fixture(),
        |v| v["capture"]["selected_relation_kinds"] = json!(["r", "v"]),
        "relation-kind",
    );
    refuses(fixture(), |v| v["columns"][0]["type_oid"] = json!(0), "OID");
    refuses(
        fixture(),
        |v| v["columns"][0]["not_null"] = json!("true"),
        "boolean",
    );
    refuses(
        fixture(),
        |v| v["columns"][0]["default_expr"] = json!(7),
        "nullable string",
    );
    refuses(
        fixture(),
        |v| v["tables"][0]["extra"] = json!(true),
        "descriptor fields",
    );
    refuses(
        fixture(),
        |v| {
            v["constraints"][0]
                .as_object_mut()
                .unwrap()
                .remove("definition");
        },
        "descriptor fields",
    );
    refuses(
        fixture(),
        |v| v["indexes"][0]["oid"] = json!(20),
        "colliding index OID",
    );
}

#[test]
fn byte_and_row_budgets_refuse_before_contract_publication() {
    assert!(
        project(&vec![b' '; MAX_BYTES + 1])
            .unwrap_err()
            .contains("byte limit")
    );
    refuses(
        fixture(),
        |v| v["tables"] = json!(vec![v["tables"][0].clone(); 513]),
        "count limit",
    );
    let mut input = fixture();
    let baseline_len = serde_json::to_vec(&input).unwrap().len();
    input["columns"][0]["default_expr"] = json!("x".repeat(MAX_BYTES - baseline_len - 32));
    let bytes = serde_json::to_vec(&input).unwrap();
    assert!(bytes.len() < MAX_BYTES);
    assert!(project(&bytes).unwrap_err().contains("output byte limit"));
    assert!(project(br#"{"schema_version":1}"#).is_err());
    assert!(project(b"not JSON").is_err());
}

#[test]
fn cli_projects_real_file_without_modification_and_refuses_without_stdout() {
    use std::{fs, path::PathBuf, process::Command};
    let root = PathBuf::from(
        std::env::var_os("SYSON_DB_CONTRACT_TEST_ROOT")
            .expect("coordinator-owned fixture root required"),
    );
    assert!(root.is_absolute() && root.is_dir());
    let directory = root.join(format!("native-db-contract-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let file = directory.join("synthetic-catalog.json");
    let bytes = serde_json::to_vec(&fixture()).unwrap();
    fs::write(&file, &bytes).unwrap();
    let binary = env!("CARGO_BIN_EXE_syson-native-db-contract");
    let result = Command::new(binary).arg(&file).output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stderr.is_empty());
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout).unwrap(),
        project(&bytes).unwrap()
    );
    assert_eq!(fs::read(&file).unwrap(), bytes);
    #[cfg(unix)]
    {
        let link = directory.join("symlink.json");
        std::os::unix::fs::symlink(&file, &link).unwrap();
        let result = Command::new(binary).arg(&link).output().unwrap();
        assert!(!result.status.success() && result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("nonsymlink"));
    }
    fs::write(&file, b"{}").unwrap();
    for args in [
        vec![file.as_os_str()],
        vec![],
        vec![file.as_os_str(), file.as_os_str()],
        vec![directory.as_os_str()],
    ] {
        let result = Command::new(binary).args(args).output().unwrap();
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("projection refused"));
    }
    fs::write(&file, vec![b' '; MAX_BYTES + 1]).unwrap();
    let result = Command::new(binary).arg(&file).output().unwrap();
    assert!(!result.status.success() && result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("byte limit"));
}

#[test]
fn repeated_long_prefixes_are_refused_within_aggregate_output_budget() {
    use syson_native_db_contract::MAX_IDENTITY_BYTES;
    let mut input = fixture();
    input["tables"][1]["name"] = json!("prefix".repeat(MAX_IDENTITY_BYTES / 6));
    let column = input["columns"][1].clone();
    for n in 3..1500 {
        let mut row = column.clone();
        row["attnum"] = json!(n);
        row["name"] = json!(format!("column_{n}"));
        input["columns"].as_array_mut().unwrap().push(row);
    }
    // Refusal must occur before validating the final row, not after retention.
    let last = input["columns"].as_array().unwrap().len() - 1;
    input["columns"][last]["not_null"] = json!("invalid-late-value");
    let bytes = serde_json::to_vec(&input).unwrap();
    assert!(bytes.len() < MAX_BYTES / 4);
    assert!(project(&bytes).unwrap_err().contains("output byte limit"));
    // A larger prefix is refused before any relation-prefix clone per column.
    input["tables"][1]["name"] = json!("x".repeat(MAX_IDENTITY_BYTES + 1));
    assert!(run(&input).unwrap_err().contains("identity byte limit"));
}

#[test]
fn refuses_incomplete_foreign_keys_including_real_native_catalog_mutation() {
    for field in ["referenced_table_oid", "index_oid"] {
        refuses(
            fixture(),
            |v| v["constraints"][1][field] = Value::Null,
            "incomplete foreign-key",
        );
    }
    refuses(
        fixture(),
        |v| {
            v["constraints"][1]["columns"] = json!([]);
            v["constraints"][1]["referenced_columns"] = json!([]);
        },
        "incomplete foreign-key",
    );
    let raw = include_bytes!("fixtures/native-01-catalog.json");
    let native = project(raw).unwrap();
    assert_eq!(native["obligations"].as_array().unwrap().len(), 164);
    let mut invalid: Value = serde_json::from_slice(raw).unwrap();
    let foreign = invalid["constraints"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["contype"] == "f")
        .unwrap();
    foreign["referenced_table_oid"] = Value::Null;
    foreign["referenced_columns"] = json!([]);
    foreign["index_oid"] = Value::Null;
    assert!(
        run(&invalid)
            .unwrap_err()
            .contains("incomplete foreign-key")
    );
}

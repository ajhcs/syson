// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use syson_native_project_templates::{Catalogue, Choice, MAX_BYTES};
const RAW: &[u8] = include_bytes!("fixtures/native-templates.json");
fn native() -> Value {
    serde_json::from_slice(RAW).unwrap()
}
fn read(v: &Value) -> Result<Catalogue, String> {
    Catalogue::from_capture(&serde_json::to_vec(v).unwrap())
}
fn body(v: &mut Value, id: &str, change: impl FnOnce(&mut Value)) {
    let row = v["observations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|o| o["id"] == id)
        .unwrap();
    let mut parsed: Value = serde_json::from_str(row["body_utf8"].as_str().unwrap()).unwrap();
    change(&mut parsed);
    row["body_utf8"] = json!(serde_json::to_string(&parsed).unwrap());
}
fn refusal(change: impl FnOnce(&mut Value), expected: &str) {
    let mut v = native();
    change(&mut v);
    let e = read(&v).unwrap_err();
    assert!(e.contains(expected), "{e:?} did not contain {expected:?}");
}
#[test]
fn actual_native_catalogues_and_error_observations_remain_exact() {
    let c = Catalogue::from_capture(RAW).unwrap();
    let output = c.projection(None).unwrap();
    assert_eq!(c.envelope(), &native());
    assert_eq!(output["capture"], native());
    let observed = output["observations"].as_array().unwrap();
    let errors: Vec<_> = observed
        .iter()
        .filter(|o| o["outcome"] == "GRAPHQL_ERROR")
        .map(|o| o["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        errors,
        vec![
            "browser-small-first",
            "browser-small-next",
            "invalid-context",
            "invalid-page-type"
        ]
    );
    assert!(
        observed
            .iter()
            .filter(|o| o["outcome"] == "GRAPHQL_ERROR")
            .all(|o| o["choices"].is_null())
    );
    let empty = observed
        .iter()
        .find(|o| o["id"] == "modal-small-next")
        .unwrap();
    assert_eq!(empty["outcome"], "SUCCESS");
    assert_eq!(empty["choices"], json!([]));
    assert_eq!(output["C01"], "OPEN");
    assert_eq!(output["B23"], "OPEN");
    assert_eq!(output["U03"], "OPEN");
    assert_eq!(
        c.lookup("all-templates", "batmobile-template").unwrap(),
        Choice::Template("batmobile-template".into())
    );
    assert_eq!(
        c.lookup("browser-large", "blank-project").unwrap(),
        Choice::Template("blank-project".into())
    );
    assert_eq!(
        c.lookup("browser-large", "upload-project").unwrap(),
        Choice::UploadProject
    );
    assert_eq!(
        c.lookup("browser-large", "browse-all-project-templates")
            .unwrap(),
        Choice::BrowseTemplates
    );
}
#[test]
fn exact_controls_are_browser_only_and_empty_labels_are_ordinary() {
    let mut v = native();
    body(&mut v, "all-templates", |b| {
        b["data"]["viewer"]["allProjectTemplates"][0]["id"] = json!("upload-project");
        b["data"]["viewer"]["allProjectTemplates"][0]["label"] = json!("");
    });
    body(&mut v, "modal-large", |b| {
        b["data"]["viewer"]["projectTemplates"]["edges"][0]["node"]["id"] =
            json!("browse-all-project-templates")
    });
    body(&mut v, "browser-large", |b| {
        let node = &mut b["data"]["viewer"]["projectTemplates"]["edges"][0]["node"];
        node["id"] = json!("upload-project-suffix");
        node["label"] = json!("");
    });
    let c = read(&v).unwrap();
    for (observation, id) in [
        ("all-templates", "upload-project"),
        ("modal-large", "browse-all-project-templates"),
        ("browser-large", "upload-project-suffix"),
    ] {
        assert_eq!(
            c.lookup(observation, id).unwrap(),
            Choice::Template(id.into())
        );
    }
}
#[test]
fn unknown_ambiguous_and_failed_observation_lookup_refuses() {
    let c = Catalogue::from_capture(RAW).unwrap();
    assert!(
        c.lookup("missing", "batmobile-template")
            .unwrap_err()
            .contains("unknown observation")
    );
    assert!(
        c.lookup("all-templates", "missing")
            .unwrap_err()
            .contains("unknown template")
    );
    assert!(
        c.lookup("browser-small-first", "batmobile-template")
            .unwrap_err()
            .contains("GRAPHQL_ERROR")
    );
    let mut v = native();
    body(&mut v, "all-templates", |b| {
        let rows = b["data"]["viewer"]["allProjectTemplates"]
            .as_array_mut()
            .unwrap();
        rows.push(rows[0].clone());
    });
    let c = read(&v).unwrap();
    assert!(c.projection(None).is_ok());
    assert!(
        c.lookup("all-templates", "batmobile-template")
            .unwrap_err()
            .contains("ambiguous")
    );
}
#[test]
fn unknown_fields_numbers_array_order_and_raw_page_info_are_preserved() {
    let mut v = native();
    v["context"]["unknown"]=serde_json::from_str(r#"{"big":18446744073709551616000000000000001,"tiny":1e-900,"negativeZero":-0,"ordered":[false,null,{},[],2]}"#).unwrap();
    body(&mut v, "modal-small-next", |b| {
        let page = &mut b["data"]["viewer"]["projectTemplates"]["pageInfo"];
        page["count"] = json!(2147483647);
        page["unknown"] = json!(["z", "a", null]);
    });
    let text = &mut v["observations"][0]["body_utf8"];
    *text = json!(format!(" \n{}\t", text.as_str().unwrap()));
    let c = read(&v).unwrap();
    let projected = c.projection(None).unwrap();
    assert_eq!(projected["capture"], v);
    assert_eq!(
        projected["capture"]["observations"][0]["body_utf8"],
        v["observations"][0]["body_utf8"]
    );
    assert_eq!(
        serde_json::to_string(&projected["capture"]["context"]["unknown"]).unwrap(),
        serde_json::to_string(&v["context"]["unknown"]).unwrap()
    );
}
#[test]
fn partial_graphql_data_and_http_failures_do_not_become_empty_success() {
    let mut v = native();
    body(
        &mut v,
        "all-templates",
        |b| {
            b["errors"] =
                json!([{"message":"synthetic partial result","extensions":{"opaque":[1,null]}}])
        },
    );
    v["observations"][1]["status"] = json!(503);
    let c = read(&v).unwrap();
    let output = c.projection(None).unwrap();
    assert_eq!(output["capture"], v);
    assert_eq!(output["observations"][0]["outcome"], "GRAPHQL_ERROR");
    assert_eq!(output["observations"][1]["outcome"], "HTTP_ERROR");
    assert!(output["observations"][0]["choices"].is_null());
    assert!(output["observations"][1]["choices"].is_null());
    assert!(c.lookup("all-templates", "batmobile-template").is_err());
}
#[test]
fn canonical_requests_complete_ids_and_known_field_types_are_validated() {
    refusal(|v| v["context"]["anonymous"] = json!(false), "anonymous");
    refusal(
        |v| {
            v["observations"].as_array_mut().unwrap().pop();
        },
        "incomplete",
    );
    refusal(
        |v| v["observations"][1]["id"] = json!("all-templates"),
        "duplicate",
    );
    refusal(
        |v| v["observations"][0]["id"] = json!("unknown"),
        "canonical observation",
    );
    refusal(
        |v| v["observations"][0]["request"]["query"] = json!("query { viewer { __typename } }"),
        "canonical manifest",
    );
    refusal(
        |v| v["observations"][0]["headers"][0] = json!(["one"]),
        "header pair",
    );
    refusal(
        |v| v["observations"][0]["body_utf8"] = json!("not JSON"),
        "JSON response",
    );
    refusal(
        |v| {
            body(v, "all-templates", |b| {
                b["data"]["viewer"]["allProjectTemplates"][0]["id"] = json!(42)
            })
        },
        "string required",
    );
    refusal(
        |v| {
            body(v, "modal-large", |b| {
                b["data"]["viewer"]["projectTemplates"]["pageInfo"]["count"] = json!(-1)
            })
        },
        "PageInfo count",
    );
    refusal(
        |v| {
            body(v, "modal-large", |b| {
                b["data"]["viewer"]["projectTemplates"]["pageInfo"]["hasNextPage"] = Value::Null
            })
        },
        "PageInfo flag",
    );
}
#[test]
fn candidate_limits_and_aggregate_projection_are_bounded() {
    assert!(
        Catalogue::from_capture(&vec![b' '; MAX_BYTES + 1])
            .unwrap_err()
            .contains("capture byte limit")
    );
    refusal(
        |v| {
            body(v, "all-templates", |b| {
                let row = b["data"]["viewer"]["allProjectTemplates"][0].clone();
                b["data"]["viewer"]["allProjectTemplates"] = json!(vec![row; 513]);
            })
        },
        "catalogue item limit",
    );
    refusal(
        |v| {
            body(v, "all-templates", |b| {
                b["data"]["viewer"]["allProjectTemplates"][0]["id"] = json!("x".repeat(4097))
            })
        },
        "ID byte limit",
    );
    let mut v = native();
    let base = serde_json::to_vec(&v).unwrap().len();
    v["context"]["padding"] = json!("x".repeat(MAX_BYTES - base - 64));
    let bytes = serde_json::to_vec(&v).unwrap();
    assert!(bytes.len() < MAX_BYTES);
    let c = Catalogue::from_capture(&bytes).unwrap();
    assert!(
        c.projection(None)
            .unwrap_err()
            .contains("projection byte limit")
    );
}
#[test]
fn cli_roundtrip_selection_and_no_partial_stdout_on_refusal() {
    use std::{fs, path::PathBuf, process::Command};
    let root = PathBuf::from(
        std::env::var_os("SYSON_PROJECT_TEMPLATES_TEST_ROOT").expect("owned scratch required"),
    );
    assert!(root.is_absolute() && root.is_dir());
    let dir = root.join(format!("templates-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let file = dir.join("native.json");
    fs::write(&file, RAW).unwrap();
    let program = env!("CARGO_BIN_EXE_syson-native-project-templates");
    let result = Command::new(program).arg(&file).output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout).unwrap()["capture"],
        native()
    );
    assert_eq!(fs::read(&file).unwrap(), RAW);
    let result = Command::new(program)
        .arg(&file)
        .args(["browser-large", "upload-project"])
        .output()
        .unwrap();
    assert!(result.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout).unwrap()["selection"],
        json!({"id":"upload-project","kind":"upload"})
    );
    for args in [
        vec!["browser-small-first", "batmobile-template"],
        vec!["all-templates", "missing"],
        vec!["extra"],
    ] {
        let r = Command::new(program)
            .arg(&file)
            .args(args)
            .output()
            .unwrap();
        assert!(!r.status.success() && r.stdout.is_empty());
    }
    let r = Command::new(program).arg(&dir).output().unwrap();
    assert!(!r.status.success() && r.stdout.is_empty());
    #[cfg(unix)]
    {
        let link = dir.join("link.json");
        std::os::unix::fs::symlink(&file, &link).unwrap();
        let r = Command::new(program).arg(link).output().unwrap();
        assert!(!r.status.success() && r.stdout.is_empty());
    }
}

#[test]
fn regression_projection_binds_selection_to_validated_observation() {
    let c = Catalogue::from_capture(RAW).unwrap();
    for selection in [
        ("all-templates", "missing"),
        ("missing", "batmobile-template"),
        ("browser-small-first", "batmobile-template"),
        ("all-templates", "upload-project"),
    ] {
        assert!(c.projection(Some(selection)).is_err(), "{selection:?}");
    }
    assert_eq!(
        c.projection(Some(("browser-large", "upload-project")))
            .unwrap()["selection"],
        json!({"id":"upload-project","kind":"upload"})
    );
    assert_eq!(
        c.projection(Some(("browser-large", "blank-project")))
            .unwrap()["selection"],
        json!({"id":"blank-project","kind":"template"})
    );
}
#[test]
fn regression_selected_nullable_cursors_must_exist() {
    for field in ["startCursor", "endCursor"] {
        refusal(
            |v| {
                body(v, "modal-small-next", |b| {
                    b["data"]["viewer"]["projectTemplates"]["pageInfo"]
                        .as_object_mut()
                        .unwrap()
                        .remove(field);
                })
            },
            "missing selected PageInfo cursor",
        );
    }
    // Actual successful empty page has explicit null cursors and remains valid.
    assert!(
        Catalogue::from_capture(RAW)
            .unwrap()
            .projection(None)
            .is_ok()
    );
}

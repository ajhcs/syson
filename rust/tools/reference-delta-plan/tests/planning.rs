// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use syson_reference_delta_plan::{MAX_BYTES, plan};
const RAW: &[u8] = include_bytes!("fixtures/packet-final.json");
fn packet() -> Value {
    serde_json::from_slice(RAW).unwrap()
}
fn run(v: &Value) -> Result<Value, String> {
    plan(&serde_json::to_vec(v).unwrap())
}
fn refuse(change: impl FnOnce(&mut Value), message: &str) {
    let mut v = packet();
    change(&mut v);
    let error = run(&v).unwrap_err();
    assert!(
        error.contains(message),
        "expected {message:?}, got {error:?}"
    );
}
fn resource(v: &Value, status: &str) -> usize {
    v["delta"]
        .as_array()
        .unwrap()
        .iter()
        .position(|r| r["status"] == status && r["path"].as_str().unwrap().ends_with(".svg"))
        .unwrap()
}
fn selected(v: &mut Value) -> &mut Value {
    v["containers"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| !c["members"].is_null())
        .unwrap()
}
#[test]
fn actual_sources_compile_inventory_and_memberships_produce_exact_bounded_recipe() {
    let output = plan(RAW).unwrap();
    assert_eq!(output["resources"].as_array().unwrap().len(), 114);
    assert_eq!(
        output["resources"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["old_sha256"].is_null())
            .count(),
        7
    );
    assert_eq!(output["families"].as_array().unwrap().len(), 7);
    assert!(
        output["families"]
            .as_array()
            .unwrap()
            .iter()
            .all(|f| f["old_members"].as_array().unwrap().len() == 1
                && f["new_members"].as_array().unwrap().len() == 1)
    );
    assert_eq!(output["docs"].as_array().unwrap().len(), 4);
    assert_eq!(
        output["unchanged_containers"].as_array().unwrap().len(),
        309
    );
    assert_eq!(
        output["non_signature_markers"][0]["disposition"],
        "PRESERVE_UNCHANGED_EMPTY_BOOT_MARKER"
    );
    assert_eq!(output["claim"], "SOURCE_DELTA_DERIVED_REFERENCE_ONLY");
    assert_eq!(output["C01"], "OPEN");
    assert_eq!(output["whole_build_source_binding"], "OPEN");
    assert_eq!(output["independent_build"], false);
}
#[test]
fn input_inventory_order_does_not_change_recipe() {
    let mut reversed = packet();
    for name in ["delta", "containers", "hits", "compiled"] {
        reversed[name].as_array_mut().unwrap().reverse();
    }
    for c in reversed["containers"].as_array_mut().unwrap() {
        if let Some(m) = c["members"].as_array_mut() {
            m.reverse();
        }
    }
    for name in ["containers", "queries"] {
        reversed["scan"][name].as_array_mut().unwrap().reverse();
    }
    assert_eq!(run(&reversed).unwrap(), plan(RAW).unwrap());
}
#[test]
fn pins_package_declarations_and_stale_label_payloads_refuse() {
    refuse(
        |v| v["artifact"]["target_revision"] = json!("0".repeat(40)),
        "pins",
    );
    refuse(
        |v| {
            let d = v["delta"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|d| d["path"].as_str().unwrap().ends_with(".java"))
                .unwrap();
            let code = d["target"]["text"]
                .as_str()
                .unwrap()
                .replace("package ", "package wrong.");
            d["target"]["bytes"] = json!(code.len());
            d["target"]["text"] = json!(code);
        },
        "package/binary path",
    );
    refuse(
        |v| {
            let i = resource(v, "M");
            v["delta"][i]["label"]["sha256"] = json!("0".repeat(64));
        },
        "label SVG",
    );
    refuse(
        |v| v["hits"][0]["sha256"] = json!("0".repeat(64)),
        "stale member hit metadata",
    );
}
#[test]
fn new_resource_collision_and_global_container_collision_refuse() {
    refuse(
        |v| {
            let p = v["hits"][0]["container"].clone();
            v["hits"][0]["container"] = json!("BOOT-INF/classes");
            assert_ne!(p, "BOOT-INF/classes");
        },
        "colliding member container",
    );
    refuse(
        |v| {
            let i = resource(v, "A");
            let path = v["delta"][i]["path"].as_str().unwrap();
            let entry = path
                .split("/src/main/resources/")
                .nth(1)
                .unwrap()
                .to_owned();
            let c = selected(v);
            let mut m = c["members"][0].clone();
            m["entry"] = json!(entry);
            c["members"].as_array_mut().unwrap().push(m);
        },
        "added SVG already exists",
    );
}
#[test]
fn class_family_missing_unexpected_and_multirelease_outputs_refuse() {
    refuse(
        |v| {
            v["compiled"].as_array_mut().unwrap().remove(0);
        },
        "missing compiled family",
    );
    refuse(
        |v| v["compiled"][0]["entry"] = json!("unexpected/Injected.class"),
        "unexpected compiled output",
    );
    refuse(|v| v["compiled"][0]["major"] = json!(66), "class version");
    refuse(
        |v| {
            let p = v["compiled"][0]["source_path"].clone();
            let hits = v["hits"].as_array_mut().unwrap();
            let i = hits.iter().position(|h| h["source_path"] == p).unwrap();
            hits.remove(i);
        },
        "incomplete old class family",
    );
    refuse(
        |v| {
            let c = selected(v);
            let mut m = c["members"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| {
                    m["entry"]
                        .as_str()
                        .unwrap()
                        .ends_with("ItemUsageItemProvider.class")
                })
                .unwrap()
                .clone();
            m["entry"] = json!(format!(
                "META-INF/versions/21/{}",
                m["entry"].as_str().unwrap()
            ));
            c["members"].as_array_mut().unwrap().push(m);
        },
        "multi-release family",
    );
}
#[test]
fn cryptographic_candidates_and_any_other_empty_marker_refuse() {
    refuse(
        |v| v["artifact"]["signatures"] = json!(["META-INF/OTHER.SF"]),
        "signed/digest-bearing",
    );
    refuse(
        |v| v["artifact"]["manifest_digests"] = json!(["META-INF/MANIFEST.MF#SHA-256-Digest"]),
        "signed/digest-bearing",
    );
    refuse(
        |v| selected(v)["signatures"] = json!(["META-INF/X.RSA"]),
        "signed/digest-bearing",
    );
    refuse(
        |v| v["artifact"]["non_signature_markers"][0]["entry"] = json!("META-INF/OTHER.SF"),
        "exact empty BOOT.SF",
    );
    refuse(
        |v| v["artifact"]["non_signature_markers"][0]["bytes"] = json!(1),
        "exact empty BOOT.SF",
    );
    refuse(
        |v| v["artifact"]["non_signature_markers"][0]["sha256"] = json!("0".repeat(64)),
        "exact empty BOOT.SF",
    );
}
#[test]
fn duplicate_source_scan_member_and_compiler_identities_refuse() {
    refuse(
        |v| v["delta"][1] = v["delta"][0].clone(),
        "duplicate source delta",
    );
    refuse(
        |v| v["containers"][1]["entry"] = v["containers"][0]["entry"].clone(),
        "duplicate container",
    );
    refuse(
        |v| {
            let c = selected(v);
            let m = c["members"][0].clone();
            c["members"].as_array_mut().unwrap().push(m);
        },
        "duplicate/oversized member",
    );
    refuse(
        |v| {
            let h = v["hits"][0].clone();
            v["hits"].as_array_mut().unwrap().push(h);
        },
        "duplicate member hit",
    );
    refuse(
        |v| {
            let c = v["compiled"][0].clone();
            v["compiled"].as_array_mut().unwrap().push(c);
        },
        "colliding compiled output",
    );
    refuse(
        |v| {
            v["scan"]["queries"].as_array_mut().unwrap().pop();
        },
        "incomplete selected member queries",
    );
}
#[test]
fn limits_paths_storage_modes_and_unknown_fields_refuse() {
    assert!(
        plan(&vec![b' '; MAX_BYTES + 1])
            .unwrap_err()
            .contains("byte limit")
    );
    refuse(
        |v| v["delta"][0]["path"] = json!("../escape.svg"),
        "invalid archive/source path",
    );
    refuse(|v| v["containers"][0]["compression"] = json!(8), "STORED");
    refuse(
        |v| v["delta"][0]["target"]["mode"] = json!("120000"),
        "source mode",
    );
    refuse(|v| v["extra"] = json!(true), "packet fields");
    refuse(
        |v| {
            v["delta"].as_array_mut().unwrap().pop();
        },
        "selected source delta",
    );
}
#[test]
fn actual_cli_keeps_input_unchanged_and_refuses_before_stdout() {
    use std::{fs, path::PathBuf, process::Command};
    let root = PathBuf::from(
        std::env::var_os("SYSON_REFERENCE_DELTA_TEST_ROOT").expect("owned test root required"),
    );
    assert!(root.is_absolute() && root.is_dir());
    let dir = root.join(format!("delta-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let file = dir.join("actual-packet.json");
    fs::write(&file, RAW).unwrap();
    let binary = env!("CARGO_BIN_EXE_syson-reference-delta-plan");
    let output = Command::new(binary).arg(&file).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        plan(RAW).unwrap()
    );
    assert_eq!(fs::read(&file).unwrap(), RAW);
    fs::write(&file, b"{}").unwrap();
    for args in [vec![], vec![file.as_os_str()], vec![dir.as_os_str()]] {
        let output = Command::new(binary).args(args).output().unwrap();
        assert!(!output.status.success() && output.stdout.is_empty());
    }
}

#[test]
fn selected_svg_multi_release_shadow_is_refused() {
    refuse(
        |v| {
            let i = resource(v, "M");
            let path = v["delta"][i]["path"].as_str().unwrap();
            let entry = path
                .split("/src/main/resources/")
                .nth(1)
                .unwrap()
                .to_owned();
            let c = selected(v);
            let mut shadow = c["members"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["entry"] == entry)
                .unwrap()
                .clone();
            shadow["entry"] = json!(format!("META-INF/versions/21/{entry}"));
            c["members"].as_array_mut().unwrap().push(shadow);
        },
        "multi-release SVG shadow",
    );
}

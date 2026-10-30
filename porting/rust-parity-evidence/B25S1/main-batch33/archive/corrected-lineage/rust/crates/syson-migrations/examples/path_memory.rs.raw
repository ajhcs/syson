// SPDX-License-Identifier: EPL-2.0
use serde_json::{Map, Value, json};
use syson_migrations::{Removal, SYSML_URI, prepare};
fn main() {
    let mut typed = Map::new();
    typed.insert("eClass".into(), json!("sysml:Annotation"));
    for n in 0..256 {
        typed.insert(
            format!("slot{n}"),
            json!({"annotatingElement":"old","keep":n}),
        );
    }
    let mut wrapper = Map::new();
    wrapper.insert("k".repeat(128 * 1024), Value::Object(typed));
    let input = json!({"ns":{"sysml":SYSML_URI},"content":wrapper}).to_string();
    let start = std::time::Instant::now();
    let result = prepare(
        &input,
        "probe",
        &[Removal {
            class: "Annotation",
            features: &["annotatingElement"],
        }],
    )
    .unwrap();
    let micros = start.elapsed().as_micros();
    let typed = result.json["content"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap();
    for n in 0..256 {
        assert!(typed[format!("slot{n}")].get("annotatingElement").is_none());
        assert_eq!(typed[format!("slot{n}")]["keep"], n);
    }
    println!(
        "{}",
        json!({"input_bytes":input.len(),"ancestor_key_bytes":128*1024,"matching_object_children":256,"all_removals_verified":true,"preparation_micros":micros})
    );
}

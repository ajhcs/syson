// SPDX-License-Identifier: EPL-2.0
//! Development-only raw-sample observer; fixture parsing/output are not timed.
use serde_json::{Value, json};
use std::{collections::HashMap, io::Read, time::Instant};
use syson_migration_pipeline::Resolver;
use syson_persistence::ResolvedElement;
use syson_project_preparation::{
    ProjectInput, RepresentationInput, SemanticInput, SemanticVersions, prepare,
};
fn text(v: &Value) -> Result<String, String> {
    Ok(v.as_str().ok_or("fixture string required")?.into())
}
fn optional(v: &Value) -> Result<Option<String>, String> {
    (!v.is_null()).then(|| text(v)).transpose()
}
fn rows(v: &Value) -> Result<&Vec<Value>, String> {
    v.as_array().ok_or_else(|| "fixture array required".into())
}
fn decode(v: &Value) -> Result<ProjectInput, String> {
    let mut out = ProjectInput::default();
    for d in rows(&v["documents"])? {
        let s = &d["versions"];
        out.documents.push(SemanticInput {
            json: text(&d["json"])?,
            references_version: optional(&d["references_version"])?,
            versions: SemanticVersions {
                connector: optional(&s["connector"])?,
                annotation: optional(&s["annotation"])?,
                view: optional(&s["view"])?,
            },
        });
    }
    for r in rows(&v["representations"])? {
        out.representations.push(RepresentationInput {
            kind: text(&r["kind"])?,
            json: text(&r["json"])?,
            control_version: optional(&r["control_version"])?,
            diagram_version: optional(&r["diagram_version"])?,
        });
    }
    Ok(out)
}
fn encode(p: &ProjectInput) -> Value {
    let documents:Vec<_>=p.documents.iter().map(|d|json!({"json":d.json,"references_version":d.references_version,
        "versions":{"connector":d.versions.connector,"annotation":d.versions.annotation,"view":d.versions.view}})).collect();
    let representations:Vec<_>=p.representations.iter().map(|r|json!({"kind":r.kind,"json":r.json,"control_version":r.control_version,"diagram_version":r.diagram_version})).collect();
    json!({"documents":documents,"representations":representations})
}
#[derive(Default)]
struct Snapshot(usize);
impl Resolver for Snapshot {
    fn resolve(&mut self, _: &str) -> Result<Option<ResolvedElement>, String> {
        self.0 += 1;
        Ok(None)
    }
}
fn run() -> Result<Value, String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(1..=2).contains(&args.len()) {
        return Err("usage: observe ONE_PROJECT_PACKET [ITERATIONS]".into());
    }
    let mut raw = Vec::new();
    std::fs::File::open(&args[0])
        .map_err(|e| e.to_string())?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    if raw.len() > 16 * 1024 * 1024 {
        return Err("fixture envelope byte limit".into());
    }
    let packet: Value = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    let iterations = args
        .get(1)
        .and_then(|v| v.to_str())
        .map(str::parse::<u64>)
        .transpose()
        .map_err(|e| e.to_string())?
        .unwrap_or(1);
    if !(1..=20).contains(&iterations) {
        return Err("iterations must be 1..20".into());
    }
    // Extract a corpus case before invoking this single-project observer.
    let input = decode(packet.get("input").unwrap_or(&packet))?;
    let ids = HashMap::from([
        ("old".into(), "new".into()),
        ("new".into(), "wrong-replay".into()),
    ]);
    let mut samples = Vec::new();
    let mut wire = None;
    let mut error = None;
    for _ in 0..iterations {
        let mut resolver = Snapshot::default();
        let start = Instant::now();
        let result = prepare(&input, &ids, &mut resolver);
        let elapsed = start.elapsed().as_micros();
        match result {
            Ok(p) => wire = Some(encode(p.wire())),
            Err(e) => error = Some(format!("{e:?}")),
        }
        samples.push(json!({"elapsed_us":elapsed,"resolver_calls":resolver.0}));
    }
    Ok(
        json!({"scope":"One selected local project; prepare-only raw samples, no general/native performance qualification","samples":samples,"wire":wire,"error":error}),
    )
}
fn main() -> Result<(), String> {
    println!("{}", run()?);
    Ok(())
}

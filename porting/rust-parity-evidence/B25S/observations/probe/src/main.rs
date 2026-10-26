// SPDX-License-Identifier: EPL-2.0
//! Development-only observations of the actual frozen participant modules.
use serde_json::{Value, json};
pub use syson_migrations::{Error, Prepared, Removal, prepare};
#[path = "/workspace/worktrees/builderbob/syson/rp-b25a/rust/crates/syson-migrations/b25a.rs"]
mod b25a;
#[path = "/workspace/worktrees/builderbob/syson/rp-b25b/rust/crates/syson-migrations/b25b.rs"]
mod b25b;
#[path = "/workspace/worktrees/builderbob/syson/rp-b25c/rust/crates/syson-migrations/b25c.rs"]
mod b25c;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["schema"] {
        use syson_metamodel::{Package, all, is_subtype, lookup};
        let mut output = json!({"namespace":syson_migrations::SYSML_URI,"classifiers":all().filter(|x|x.package==Package::Domain).count()});
        for root in [
            "Annotation",
            "AnnotatingElement",
            "ConnectorAsUsage",
            "ViewUsage",
        ] {
            let parent = lookup(Package::Domain, root).unwrap();
            let rows: Vec<_> = all()
                .filter(|x| x.package == Package::Domain && is_subtype(x.id, parent))
                .map(|x| x.name())
                .collect();
            output[root] = json!(rows);
        }
        println!("{output}");
        return Ok(());
    }
    if args.len() != 2 {
        return Err("unit and packet required".into());
    }
    if args[0] == "sequence" {
        let packet: Value = serde_json::from_slice(&std::fs::read(&args[1])?)?;
        let mut results = Vec::new();
        for case in packet["cases"].as_array().unwrap() {
            let mut input = case["input"].as_str().unwrap().to_owned();
            let mut predicates = Vec::new();
            for run in [b25b::prepare, b25a::prepare, b25c::prepare] {
                let result = run(&input).map_err(|e| format!("{e:?}"))?;
                predicates.push(result.matched_resource);
                input = result.to_json().map_err(|e| format!("{e:?}"))?;
            }
            results.push(json!({"id":case["id"],"json":serde_json::from_str::<Value>(&input)?,"matched_resources":predicates,"versions":[b25b::VERSION,b25a::VERSION,b25c::VERSION],"serialized":input}));
        }
        println!("{}", json!({"results":results}));
        return Ok(());
    }
    let (version, run): (&str, fn(&str) -> Result<Prepared, Error>) = match args[0].as_str() {
        "B25a" => (b25a::VERSION, b25a::prepare),
        "B25b" => (b25b::VERSION, b25b::prepare),
        "B25c" => (b25c::VERSION, b25c::prepare),
        _ => return Err("invalid unit".into()),
    };
    let packet: Value = serde_json::from_slice(&std::fs::read(&args[1])?)?;
    let cases = packet["cases"].as_array().ok_or("cases required")?;
    let mut results = Vec::new();
    for case in cases {
        let input = case["input"].as_str().ok_or("raw input required")?;
        let mut row = json!({"id":case["id"],"version":version});
        match run(input) {
            Ok(prepared) => {
                row["status"] = json!("PREPARED");
                row["matched_resource"] = json!(prepared.matched_resource);
                row["serialized"] = json!(prepared.to_json().map_err(|e| format!("{e:?}"))?);
                row["json"] = prepared.json;
            }
            Err(error) => {
                row["status"] = json!("REFUSED");
                row["error"] = json!(format!("{error:?}"));
            }
        }
        results.push(row);
    }
    println!("{}", json!({"unit":args[0],"results":results}));
    Ok(())
}

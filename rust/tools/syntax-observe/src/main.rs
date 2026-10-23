// SPDX-License-Identifier: EPL-2.0
//! Native syntax observation only. This tool never executes the reference parser.
use serde_json::json;
use std::io::Read;
use std::path::Path;
use syson_syntax::{Grammar, Language, Options};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        return Err("expected exactly one reviewed fixture path".into());
    }
    let path = Path::new(&args[1]);
    let language = match path.extension().and_then(|s| s.to_str()) {
        Some("sysml") => Language::SysML,
        Some("kerml") => Language::KerML,
        _ => return Err("unsupported fixture suffix".into()),
    };
    if path.metadata()?.len() > 1024 {
        return Err("observation tool accepts at most 1024 fixture bytes".into());
    }
    let mut input = String::new();
    std::fs::File::open(path)?.take(1025).read_to_string(&mut input)?;
    if input.len() > 1024 {
        return Err("observation tool accepts at most 1024 fixture bytes".into());
    }
    let grammar = Grammar::load(language).map_err(|e| format!("grammar load failed: {e:?}"))?;
    let mut output = json!({"language":format!("{language:?}"),"input_bytes":input.len(),"input_utf16_units":input.encode_utf16().count(),"ast_status":"NotPerformed","semantic_status":"NotPerformed"});
    match grammar.recognize(&input, Options::default()) {
        Ok(r) => {
            output["syntax_recognized"] = json!(true);
            output["tokens"] = json!(r.tokens.len());
            output["visible_tokens"] = json!(r.tokens.iter().filter(|t| !t.hidden).count());
            output["cst_nodes"] = json!(r.cst.len());
            output["root_byte_start"] = json!(r.cst[r.root].span.start.byte);
            output["root_byte_end"] = json!(r.cst[r.root].span.end.byte);
            output["state_history_merges"] = json!(r.ambiguous_state_merges);
            output["semantic_status"] = json!(format!("{:?}", r.semantics));
        }
        Err(e) => {
            output["syntax_recognized"] = json!(false);
            output["error_debug"] = json!(format!("{e:?}"));
        }
    }
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

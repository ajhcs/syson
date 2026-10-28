// SPDX-License-Identifier: EPL-2.0
//! Emit a native observation for the exact small reference fixture. No semantic AST.
use serde_json::json;
use syson_syntax::{Grammar, Language, Options, Point};
fn point(p: Point) -> serde_json::Value {
    json!({"byte":p.byte,"utf16":p.utf16,"line":p.line,"column_utf16":p.column_utf16})
}
fn main() {
    let input = include_str!("../../../../porting/validation/fixtures/parser-oracle.sysml");
    let result = Grammar::load(Language::SysML)
        .expect("pinned grammar")
        .recognize(input, Options::default())
        .expect("complete syntax recognition");
    println!("{}",serde_json::to_string_pretty(&json!({"language":"SysML","syntax_recognized":true,"consumed_input_bytes":input.len(),"semantic_status":format!("{:?}",result.semantics),"ast_status":"NotPerformed","root":result.root,"ambiguous_state_merges":result.ambiguous_state_merges,"tokens":result.tokens.iter().map(|t|json!({"kind":format!("{:?}",t.kind),"hidden":t.hidden,"text":t.text(input),"start":point(t.span.start),"end":point(t.span.end)})).collect::<Vec<_>>(),"cst":result.cst.iter().map(|n|json!({"kind":format!("{:?}",n.kind),"children":n.children,"start":point(n.span.start),"end":point(n.span.end)})).collect::<Vec<_>>()})).unwrap());
}

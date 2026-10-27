#[path = "../../allocation.rs"] mod allocation;
use syson_json_interchange::{Bundle,Input,MAX_RESOURCE_BYTES};
use syson_references::FragmentIdentity;
fn main() {
    const NESTING:usize=60; const NULLS:usize=100_000;
    let mut opaque=String::new();for _ in 0..NESTING {opaque.push_str("{\"x\":");}
    opaque.push('[');for i in 0..NULLS {if i!=0 {opaque.push(',');}opaque.push_str("null");}opaque.push(']');
    for _ in 0..NESTING {opaque.push('}');}
    let json=format!(r#"{{"json":{{"version":"1.0","encoding":"utf-8"}},"ns":{{"sysml":"http://www.eclipse.org/syson/sysml"}},"content":[],"vendor":{opaque}}}"#);
    let bytes=json.len();assert!(bytes<MAX_RESOURCE_BYTES);
    let input=Input{key:"opaque-budget-probe".into(),json,fragments:FragmentIdentity::JsonNodeId};
    let inputs=vec![input];let base=allocation::begin();let result=Bundle::load(inputs);let (peak,max_one)=allocation::end(base);
    let outcome=match result {Err(syson_json_interchange::Error::Limit)=>"Limit",Err(_)=>"OtherError",Ok(_)=>"Accepted"};
    println!("{{\"case\":\"F33-pretty-before-budget\",\"input_bytes\":{bytes},\"output_limit\":{MAX_RESOURCE_BYTES},\"peak_additional_live_bytes\":{peak},\"max_single_allocation\":{max_one},\"outcome\":\"{outcome}\"}}");
}

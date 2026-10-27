#[path = "../../allocation.rs"] mod allocation;
use syson_model::Document;
use syson_text_export::{export,Options,Error};
fn main() {
    const NAME_BYTES:usize=2*1024*1024;
    let name="A".repeat(NAME_BYTES);
    let json=format!(r#"{{"json":{{"version":"1.0","encoding":"utf-8"}},"ns":{{"sysml":"http://www.eclipse.org/syson/sysml"}},"content":[{{"id":"10000000-0000-4000-8000-000000000001","eClass":"sysml:Package","data":{{"elementId":"20000000-0000-4000-8000-000000000001","declaredName":"{name}"}}}}]}}"#);
    let bytes=json.len();let document=Document::from_json(&json).expect("valid selected package");
    let base=allocation::begin();let result=export(&document,Options::default());let (peak,max_one)=allocation::end(base);
    let outcome=match result {Err(Error::Limit)=>"Limit",Err(_)=>"OtherError",Ok(_)=>"Accepted"};
    println!("{{\"case\":\"F34-name-before-budget\",\"input_bytes\":{bytes},\"name_bytes\":{NAME_BYTES},\"output_limit\":65536,\"peak_additional_live_bytes\":{peak},\"max_single_allocation\":{max_one},\"outcome\":\"{outcome}\"}}");
}

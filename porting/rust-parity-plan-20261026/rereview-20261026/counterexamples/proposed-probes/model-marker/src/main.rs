use syson_model::Document;
fn main(){
let raw=r#"{"json":{"version":"1.0","encoding":"utf-8"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"id":"10000000-0000-4000-8000-000000000001","eClass":"sysml:Package","data":{"elementId":"20000000-0000-4000-8000-000000000001","declaredName":"P","vendor":{"$serde_json::private::Number":"123"}}}]}"#;
let doc=Document::from_json(raw).unwrap();let output=doc.to_json().unwrap();let v:serde_json::Value=serde_json::from_str(&output).unwrap();
println!("{}",serde_json::json!({"case":"opaque_private_number_key","expected":"object","actual_vendor":v["content"][0]["data"]["vendor"],"preserved_object":v["content"][0]["data"]["vendor"].is_object()}));
}

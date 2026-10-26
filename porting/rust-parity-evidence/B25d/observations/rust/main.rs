// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value,json};
use syson_migrations::b25d;
fn main()->Result<(),Box<dyn std::error::Error>>{
 let path=std::env::args().nth(1).ok_or("packet path required")?;
 let packet:Value=serde_json::from_slice(&std::fs::read(path)?)?;let mut results=Vec::new();
 for case in packet["cases"].as_array().ok_or("cases required")?{
  let mut row=json!({"id":case["id"],"version":b25d::VERSION,"kind":b25d::KIND});
  let input=case["input"].as_str().ok_or("raw input required")?;
  if case["mode"]=="callback"{
   let mut root:Value=serde_json::from_str(input)?;
   let changed=b25d::replace_json_node(root.as_object_mut().ok_or("object required")?,case["current_attribute"].as_str().ok_or("attribute required")?,&case["current_value"]);
   row["status"]=json!("PREPARED");row["changed"]=json!(changed);row["json"]=root;
  }else{
   match b25d::prepare(b25d::KIND,input,None){
    Ok(prepared)=>{row["status"]=json!("PREPARED");row["changed"]=json!(prepared.changed);row["json"]=prepared.json;},
    Err(error)=>{row["status"]=json!("REFUSED");row["error"]=json!(format!("{error:?}"));}
   }
  }
  results.push(row);
 }
 println!("{}",json!({"results":results}));Ok(())
}

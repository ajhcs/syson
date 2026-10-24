// SPDX-License-Identifier: EPL-2.0
//! Independent metadata observation adapter. Does not execute model operations.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use syson_metamodel::*;
fn key(id: ClassifierId) -> String {
    let c = id.metadata();
    format!("{:?}::{}", c.package, c.name())
}
fn attributes(values: &[(&str, &str)]) -> BTreeMap<String, String> {
    values
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}
fn node(n: &MetadataNode) -> Value {
    json!({"tag":n.tag,"attributes":attributes(n.attributes),"children":n.children.iter().map(node).collect::<Vec<_>>()})
}
fn reference(r: TypeReference) -> Value {
    json!({"raw":r.raw(),"local":r.local().map(key),"resolution":match r{TypeReference::Local{..}=>"local",TypeReference::External{..}=>"external",TypeReference::UnresolvedLocal{..}=>"unresolved_local"}})
}
fn typed(t: TypedElement) -> Value {
    json!({"name":t.name(),"type":t.type_reference().map(reference),"lower":t.lower_bound(),"upper":t.upper_bound(),"ordered":t.ordered(),"unique":t.unique()})
}
fn main() {
    let names: BTreeSet<_> = all().flat_map(|c| c.features().map(|f| f.name())).collect();
    let classes=all().map(|c|{
  let mut inherited=BTreeMap::new();
  for name in &names {
   let found=inherited_feature(c.id,name);
   inherited.insert(*name,json!({"candidates":found.candidates.iter().map(|f|format!("{}.{}",key(f.owner),f.name())).collect::<Vec<_>>(),"unresolved":found.unresolved_supertypes.iter().map(|r|r.raw()).collect::<Vec<_>>(),"unique":found.unique().map(|f|format!("{}.{}",key(f.owner),f.name()))}));
  }
  json!({"key":key(c.id),"kind":format!("{:?}",c.kind),"abstract":c.is_abstract(),"interface":c.is_interface(),"node":node(&c.node),"supertypes":c.supertypes().map(reference).collect::<Vec<_>>(),"features":c.features().map(|f|json!({"typed":typed(f.typed()),"kind":format!("{:?}",f.kind()),"containment":f.containment(),"resolve_proxies":f.resolve_proxies(),"derived":f.derived(),"volatile":f.volatile(),"transient":f.transient(),"changeable":f.changeable(),"unsettable":f.unsettable(),"is_id":f.is_id(),"opposite":f.opposite(),"default":f.explicit_default_literal()})).collect::<Vec<_>>(),"operations":c.operations().map(|o|json!({"typed":typed(o.typed()),"parameters":o.parameters().map(typed).collect::<Vec<_>>()})).collect::<Vec<_>>(),"enum_literals":c.enum_literals().map(|e|json!({"name":e.name,"value":e.value,"literal":e.literal})).collect::<Vec<_>>(),"local_ancestors":all().filter(|parent|is_subtype(c.id,parent.id)).map(|p|key(p.id)).collect::<Vec<_>>(),"unresolved_ancestors":unresolved_supertypes(c.id).iter().map(|r|r.raw()).collect::<Vec<_>>(),"inherited_features":inherited})
 }).collect::<Vec<_>>();
    let sources=SOURCES.iter().map(|s|json!({"package":format!("{:?}",s.package),"path":s.path,"revision":s.revision,"sha256":s.sha256,"attributes":attributes(s.attributes)})).collect::<Vec<_>>();
    println!("{}", json!({"sources":sources,"classifiers":classes}));
}

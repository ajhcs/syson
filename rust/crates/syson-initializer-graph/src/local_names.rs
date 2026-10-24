// SPDX-License-Identifier: EPL-2.0
// Source: pinned FeatureImpl.getOwnedRedefinition/namingFeature/effectiveName/
// effectiveShortName and RedefinitionImpl.getRedefinedFeature. See provenance.
use super::*;

/// Explicit wire interpretation for this local adapter. SemanticElementIds is
/// the Rust document adapter convention, not a general EMF JSON import claim.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReferenceEncoding {
    #[default]
    Disabled,
    SemanticElementIds,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameKind { Name, ShortName }
impl Index {
    /// Allocation-free local traversal in source containment order. A cycle is
    /// refused rather than emulating Java recursion until stack exhaustion.
    /// Source-declared null and unresolved reference identity stay distinct.
    pub fn effective_name_with_references(&self,id:&str,kind:NameKind,encoding:ReferenceEncoding)->Result<Option<String>,Error> {
        let mut current=id;
        let mut visited=std::collections::BTreeSet::new();
        loop {
            if !visited.insert(current) { return Err(gap("namingFeature","local naming cycle; Java recursive getter not executed")); }
            let node=self.node(current)?;
            if !mm::is_subtype(node.classifier,C::Feature) {
                return Err(gap("effectiveName","non-Feature implementation dispatch unsupported"));
            }
            let name=node.data.get("declaredName").filter(|v|!v.is_null());
            let short=node.data.get("declaredShortName").filter(|v|!v.is_null());
            for value in [name,short].into_iter().flatten() {
                if !value.is_string() { return Err(gap("effectiveName","declared name is not a string")); }
            }
            if name.is_some() || short.is_some() {
                return Ok(match kind {NameKind::Name=>name,NameKind::ShortName=>short}.and_then(Value::as_str).map(str::to_owned));
            }
            // Scan the actual JSON list rather than BTreeMap identity order.
            let mut first=None;
            if let Some(children)=node.data.get("ownedRelationship") {
                for child in children.as_array().ok_or_else(||gap("ownedRedefinition","invalid relationship list"))? {
                    let relation=self.node(child["data"]["elementId"].as_str().ok_or_else(||gap("ownedRedefinition","missing semantic identity"))?)?;
                    if mm::is_subtype(relation.classifier,C::Redefinition) { first=Some(relation); break; }
                }
            }
            let Some(relation)=first else { return Ok(None) };
            if encoding==ReferenceEncoding::Disabled {
                return Err(gap("redefinedFeature","reference encoding was not selected"));
            }
            let target=match relation.data.get("redefinedFeature") {
                None|Some(Value::Null)=>return Ok(None),
                Some(Value::String(target))=>target.as_str(),
                Some(_)=>return Err(gap("redefinedFeature","unsupported reference wire shape")),
            };
            if target==current { return Ok(None) }
            let Some(target_node)=self.nodes.get(target) else {
                return Err(Error::Unresolved(ContextGap {contract:"redefinedFeature",reason:format!("semantic elementId not present in this document: {target}")}));
            };
            if !mm::is_subtype(target_node.classifier,C::Feature) { return Err(gap("redefinedFeature","target is not a Feature")); }
            current=&target_node.element_id;
        }
    }
}

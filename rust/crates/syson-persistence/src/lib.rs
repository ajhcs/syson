// Copyright (c) 2024, 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
// Contributors: Obeo - initial API and implementation; Rust port contributors.
//! Pure standard-library ID migration rules, not a database implementation.
//! Source baseline: 6ba2ac64e6b16ee2b05752ad55395eebb87d4910.
use std::collections::HashMap;

pub const REFERENCES_VERSION: &str = "2025.8.0-202506301600";
pub const DIAGRAM_VERSION: &str = "2025.8.0-202506301700";
pub const DIAGRAM_KIND: &str = "siriusComponents://representation?type=Diagram";
const SYSML: &str = "sysmllibrary";
const KERML: &str = "kermllibrary";

/// Java String.trim removes code points <= U+0020, unlike Rust str::trim.
fn java_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c <= '\u{20}')
}
/// Port of StandardLibrariesIdsResourceToMapParser.resourceToMap's line rules.
/// Java split("=") discards trailing empty parts; duplicate keys overwrite.
/// UTF-8 text acquisition and I/O policy are explicit caller responsibilities.
pub fn parse_id_map(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in text.split(['\n', '\r']) {
        let mut parts: Vec<&str> = line.split('=').collect();
        while parts.last() == Some(&"") {
            parts.pop();
        }
        if parts.len() == 2 {
            map.insert(java_trim(parts[0]).into(), java_trim(parts[1]).into());
        }
    }
    map
}
/// ElementUtil.isStandardLibraryResource's URI-prefix check. Missing resource
/// and missing resource URI both map to None. Prefix is intentionally not a
/// parsed URI-scheme equality check, matching the upstream implementation.
pub fn is_standard_library_resource(uri: Option<&str>) -> bool {
    uri.is_some_and(|u| u.starts_with(SYSML) || u.starts_with(KERML))
}
/// SysMLv2EditingContextPersistenceFilter excludes standard libraries from downloads.
pub fn should_persist_in_project_download(uri: Option<&str>) -> bool {
    !is_standard_library_resource(uri)
}

/// Port of getEObjectUri. Resolver must reproduce ResourceSet.getEObject(uri,
/// false) and return Some only for an actual SysML Element. It must not load a
/// missing resource as a side effect. No EMF URI parser is supplied here.
pub fn migrate_reference_uri(
    uri: &str,
    containment: bool,
    ids: &HashMap<String, String>,
    mut resolve_element_id: impl FnMut(&str) -> Option<String>,
) -> String {
    if containment || !(uri.contains(SYSML) || uri.contains(KERML)) {
        return uri.into();
    }
    let lookup = if uri.starts_with("sysml:") {
        // Upstream prioritizes SysML when both substrings occur.
        let index = uri.find(SYSML).or_else(|| uri.find(KERML)).unwrap();
        &uri[index..]
    } else {
        uri
    };
    let (prefix, old_id) = match uri.find('#') {
        Some(index) => (&uri[..=index], &uri[index + 1..]),
        None => ("", uri),
    };
    if let Some(id) = resolve_element_id(lookup) {
        format!("{prefix}{id}")
    } else if let Some(id) = ids.get(old_id) {
        format!("{prefix}{id}")
    } else {
        uri.into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedElement {
    pub element_id: String,
    pub resource_uri: Option<String>,
}
/// Port of replaceJsonNode's targetObjectId string case. Return None for no
/// replacement. Caller performs actual JSON traversal/write; other attribute
/// types are deliberately excluded by the Option<&str> boundary.
pub fn migrate_diagram_target(
    attribute: &str,
    string_value: Option<&str>,
    ids: &HashMap<String, String>,
    mut resolve: impl FnMut(&str) -> Option<ResolvedElement>,
) -> Option<String> {
    if attribute != "targetObjectId" {
        return None;
    }
    let old_id = string_value?;
    if let Some(element) = resolve(old_id) {
        if is_standard_library_resource(element.resource_uri.as_deref()) {
            Some(element.element_id)
        } else {
            None
        }
    } else {
        ids.get(old_id).cloned()
    }
}

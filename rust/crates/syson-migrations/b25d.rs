// Copyright (c) 2026 Obeo.
// SPDX-License-Identifier: EPL-2.0
//! Selected action/control representation migration; no framework/history host.
use serde_json::{Map, Value};
use std::io::Write;

use crate::{MAX_BYTES, MAX_DEPTH, MAX_VALUES};

pub const VERSION: &str = "2026.7.0-202606240000";
pub const KIND: &str = "siriusComponents://representation?type=Diagram";
pub const DESCRIPTION_ID: &str = "descriptionId";
pub const OLD_START_DESCRIPTION_ID: &str = "siriusComponents://nodeDescription?sourceKind=view&sourceId=8dcd14b0-6259-3193-ad2c-743f394c68e4&sourceElementId=bf75f645-8a6d-30d9-9514-4e9738e212a3";
pub const NEW_START_DESCRIPTION_ID: &str = "siriusComponents://nodeDescription?sourceKind=view&sourceId=8dcd14b0-6259-3193-ad2c-743f394c68e4&sourceElementId=596f7e8d-ba35-38d6-937f-011e05b74063";
pub const OLD_DONE_DESCRIPTION_ID: &str = "siriusComponents://nodeDescription?sourceKind=view&sourceId=8dcd14b0-6259-3193-ad2c-743f394c68e4&sourceElementId=03789d5f-51d6-3f66-a496-146b1685930c";
pub const NEW_DONE_DESCRIPTION_ID: &str = "siriusComponents://nodeDescription?sourceKind=view&sourceId=8dcd14b0-6259-3193-ad2c-743f394c68e4&sourceElementId=e7465fdc-fec8-38da-b7c0-26ba953b96de";
pub const OLD_DECISION_DESCRIPTION_ID: &str = "siriusComponents://nodeDescription?sourceKind=view&sourceId=8dcd14b0-6259-3193-ad2c-743f394c68e4&sourceElementId=d6e7c035-98ae-3fcf-a5a8-a8e502cd447e";
pub const NEW_DECISION_DESCRIPTION_ID: &str = "siriusComponents://nodeDescription?sourceKind=view&sourceId=8dcd14b0-6259-3193-ad2c-743f394c68e4&sourceElementId=b2f1248d-2a25-37a2-8822-9e21ed709c72";
pub const OLD_FORK_DESCRIPTION_ID: &str = "siriusComponents://nodeDescription?sourceKind=view&sourceId=8dcd14b0-6259-3193-ad2c-743f394c68e4&sourceElementId=46dc7020-1788-3413-967d-e1e3053349ca";
pub const NEW_FORK_DESCRIPTION_ID: &str = "siriusComponents://nodeDescription?sourceKind=view&sourceId=8dcd14b0-6259-3193-ad2c-743f394c68e4&sourceElementId=e940b643-cce5-3904-b785-fb3814493642";
pub const OLD_JOIN_DESCRIPTION_ID: &str = "siriusComponents://nodeDescription?sourceKind=view&sourceId=8dcd14b0-6259-3193-ad2c-743f394c68e4&sourceElementId=ae252080-4a68-3129-970b-2d049bb3e5a3";
pub const NEW_JOIN_DESCRIPTION_ID: &str = "siriusComponents://nodeDescription?sourceKind=view&sourceId=8dcd14b0-6259-3193-ad2c-743f394c68e4&sourceElementId=963e573a-d0ff-3134-9e96-9ce55e3384ec";
pub const OLD_MERGE_DESCRIPTION_ID: &str = "siriusComponents://nodeDescription?sourceKind=view&sourceId=8dcd14b0-6259-3193-ad2c-743f394c68e4&sourceElementId=91f597c3-96b8-3a9e-8b30-99e5401952fa";
pub const NEW_MERGE_DESCRIPTION_ID: &str = "siriusComponents://nodeDescription?sourceKind=view&sourceId=8dcd14b0-6259-3193-ad2c-743f394c68e4&sourceElementId=d47553da-9705-3eca-99ea-dc05ff972f55";

pub const MAPPINGS: [(&str, &str); 6] = [
    (OLD_START_DESCRIPTION_ID, NEW_START_DESCRIPTION_ID),
    (OLD_DONE_DESCRIPTION_ID, NEW_DONE_DESCRIPTION_ID),
    (OLD_DECISION_DESCRIPTION_ID, NEW_DECISION_DESCRIPTION_ID),
    (OLD_FORK_DESCRIPTION_ID, NEW_FORK_DESCRIPTION_ID),
    (OLD_JOIN_DESCRIPTION_ID, NEW_JOIN_DESCRIPTION_ID),
    (OLD_MERGE_DESCRIPTION_ID, NEW_MERGE_DESCRIPTION_ID),
];

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Input(String),
    Limit,
    UnsupportedVersion(String),
    NestedArray,
}
#[derive(Debug, PartialEq)]
pub struct Prepared {
    pub json: Value,
    /// Selected wrapper watermark, separate from native framework history and
    /// B01's library-ID diagram watermark. Foreign kinds retain the input value.
    pub participant_version: Option<String>,
    pub matched_kind: bool,
    pub changed: bool,
}
fn replacement(attribute: &str, current_value: &Value) -> Option<&'static str> {
    if attribute != DESCRIPTION_ID {
        return None;
    }
    let current = current_value.as_str()?;
    MAPPINGS
        .iter()
        .find_map(|(old, new)| (*old == current).then_some(*new))
}
/// Exact attribute callback: current_value is the supplied callback value, not
/// a lookup in root. Native editingContext (including read-only state) is unused.
/// Returns whether the JSON value changed; native's callback itself returns void.
/// This low-level mutable callback performs no admission or persistence checks.
pub fn replace_json_node(
    root: &mut Map<String, Value>,
    current_attribute: &str,
    current_value: &Value,
) -> bool {
    let Some(new) = replacement(current_attribute, current_value) else {
        return false;
    };
    let changed = root.get(DESCRIPTION_ID).and_then(Value::as_str) != Some(new);
    root.insert(DESCRIPTION_ID.into(), Value::String(new.into()));
    changed
}
fn bounded(value: &Value, depth: usize, count: &mut usize) -> Result<(), Error> {
    *count += 1;
    if depth > MAX_DEPTH || *count > MAX_VALUES {
        return Err(Error::Limit);
    }
    match value {
        Value::Object(o) => {
            for v in o.values() {
                bounded(v, depth + 1, count)?;
            }
        }
        Value::Array(a) => {
            for v in a {
                bounded(v, depth + 1, count)?;
            }
        }
        _ => (),
    }
    Ok(())
}
fn literal_error(error: syson_json_value::Error) -> Error {
    match error {
        syson_json_value::Error::Limit(_) => Error::Limit,
        other => Error::Input(other.to_string()),
    }
}
struct Counter(usize);
impl Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_BYTES)
            .ok_or_else(|| std::io::Error::other("representation byte budget"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Prepared {
    /// Serialize the complete output, rechecking public edited JSON before any
    /// output allocation. Non-namespace object order is not a wire-byte promise.
    pub fn to_json(&self) -> Result<String, Error> {
        if self.matched_kind && !self.json.is_object() {
            return Err(Error::Input("diagram root must be an object".into()));
        }
        bounded(&self.json, 0, &mut 0)?;
        serde_json::to_writer(&mut Counter(0), &self.json).map_err(|_| Error::Limit)?;
        serde_json::to_string(&self.json).map_err(|e| Error::Input(e.to_string()))
    }
}
fn walk(object: &mut Map<String, Value>) -> Result<bool, Error> {
    let mut changed = false;
    for (attribute, current_value) in object {
        match current_value {
            Value::Object(child) => changed |= walk(child)?,
            Value::Array(children) => {
                for child in children {
                    match child {
                        Value::Object(child) => changed |= walk(child)?,
                        // Native casts any container element to ObjectNode.
                        // A nested array therefore fails instead of recursing.
                        Value::Array(_) => return Err(Error::NestedArray),
                        _ => (),
                    }
                }
            }
            _ => (),
        }
        // Same explicit-value callback after child traversal. Only an existing
        // descriptionId scalar can match, so replacing in place has identical
        // final effects without cloning whole child trees or ancestor paths.
        if let Some(new) = replacement(attribute, current_value) {
            *current_value = Value::String(new.into());
            changed = true;
        }
    }
    Ok(changed)
}
/// Bounded immutable wrapper around one native participant and its inherited
/// traversal. Diagram roots must be objects. None applies this selected unit;
/// exact VERSION skips replay; another matched-kind version refuses. Foreign
/// kinds retain JSON and selected watermark without invoking the participant.
/// This selection policy is local, not native migration-framework activation.
/// Native traversal may mutate before a nested-array exception; this wrapper
/// intentionally publishes no partial output on any error.
pub fn prepare(
    kind: &str,
    input: &str,
    participant_version: Option<&str>,
) -> Result<Prepared, Error> {
    if input.len() > MAX_BYTES || participant_version.is_some_and(|v| v.len() > 4096) {
        return Err(Error::Limit);
    }
    let matched_kind = kind == KIND;
    if matched_kind && participant_version.is_some_and(|v| v != VERSION) {
        return Err(Error::UnsupportedVersion(
            participant_version.unwrap().into(),
        ));
    }
    let admitted = crate::literal(input).map_err(literal_error)?;
    let (mut json, _) = admitted.into_value(|| false).map_err(literal_error)?;
    bounded(&json, 0, &mut 0)?;
    let mut changed = false;
    if matched_kind {
        let object = json
            .as_object_mut()
            .ok_or_else(|| Error::Input("diagram root must be an object".into()))?;
        if participant_version.is_none() {
            changed = walk(object)?;
        }
    }
    let prepared = Prepared {
        json,
        participant_version: if matched_kind {
            Some(VERSION.into())
        } else {
            participant_version.map(str::to_owned)
        },
        matched_kind,
        changed,
    };
    prepared.to_json()?;
    Ok(prepared)
}

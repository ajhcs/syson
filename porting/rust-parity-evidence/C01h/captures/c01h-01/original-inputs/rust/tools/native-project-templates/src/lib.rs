// SPDX-License-Identifier: EPL-2.0
//! Read native template observations and derive only the source-defined choices.
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::Path,
};
pub const MAX_BYTES: usize = 8 * 1024 * 1024;
pub const QUERIES: &str =
    include_str!("../../../../porting/rust-parity-evidence/C01h/queries.json");
type Result<T> = std::result::Result<T, String>;
fn need(ok: bool, why: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(why.into()) }
}
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or("string required".into())
}
fn array(v: &Value, limit: usize) -> Result<&Vec<Value>> {
    let a = v.as_array().ok_or("array required")?;
    need(a.len() <= limit, "catalogue item limit")?;
    Ok(a)
}
fn digest(v: &Value, n: usize) -> Result<()> {
    let s = text(v)?;
    need(
        s.len() == n
            && s.bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
        "invalid capture digest",
    )
}
struct Counter(usize);
impl Write for Counter {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(b.len())
            .filter(|n| *n <= MAX_BYTES)
            .ok_or_else(|| std::io::Error::other("projection byte limit"))?;
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn count(budget: &mut Counter, v: &Value) -> Result<()> {
    serde_json::to_writer(&mut *budget, v).map_err(|e| e.to_string())?;
    budget.write_all(b",").map_err(|e| e.to_string())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    Template(String),
    UploadProject,
    BrowseTemplates,
}
impl Choice {
    pub fn id(&self) -> &str {
        match self {
            Self::Template(id) => id,
            Self::UploadProject => "upload-project",
            Self::BrowseTemplates => "browse-all-project-templates",
        }
    }
    fn json(&self) -> Value {
        let kind = match self {
            Self::Template(_) => "template",
            Self::UploadProject => "upload",
            Self::BrowseTemplates => "browse",
        };
        json!({"id":self.id(),"kind":kind})
    }
}
#[derive(Debug)]
struct Observed {
    id: String,
    outcome: &'static str,
    choices: Option<Vec<Choice>>,
}
#[derive(Debug)]
pub struct Catalogue {
    envelope: Value,
    observed: Vec<Observed>,
}
fn choice(node: &Value, browser: bool) -> Result<Choice> {
    let id = text(&node["id"])?;
    need(id.len() <= 4096, "template ID byte limit")?;
    text(&node["label"])?;
    text(&node["imageURL"])?;
    for library in array(&node["requiredLibraries"], 512)? {
        for field in ["id", "namespace", "name", "version"] {
            text(&library[field])?;
        }
    }
    Ok(match (browser, id) {
        (true, "upload-project") => Choice::UploadProject,
        (true, "browse-all-project-templates") => Choice::BrowseTemplates,
        _ => Choice::Template(id.into()),
    })
}
fn page_info(v: &Value) -> Result<()> {
    need(
        v["count"].as_u64().is_some_and(|n| n <= i32::MAX as u64),
        "invalid PageInfo count",
    )?;
    for field in ["hasPreviousPage", "hasNextPage"] {
        need(v[field].is_boolean(), "invalid PageInfo flag")?;
    }
    for field in ["startCursor", "endCursor"] {
        need(
            v[field].is_null() || v[field].is_string(),
            "invalid PageInfo cursor",
        )?;
    }
    Ok(())
}
impl Catalogue {
    /// Metadata provenance is established by the coordinator, not by this parser.
    /// Raw body strings, unknown fields, numbers and every array remain untouched.
    pub fn from_capture(bytes: &[u8]) -> Result<Self> {
        need(bytes.len() <= MAX_BYTES, "capture byte limit")?;
        let envelope: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        need(
            envelope["context"]["anonymous"] == true,
            "anonymous capture required",
        )?;
        for key in ["artifact_sha256", "configuration_sha256"] {
            digest(&envelope["context"][key], 64)?;
        }
        for key in ["source_label_revision", "target_revision"] {
            digest(&envelope["context"][key], 40)?;
        }
        let manifest: Value = serde_json::from_str(QUERIES).map_err(|e| e.to_string())?;
        let specifications = array(&manifest["observations"], 32)?;
        let captures = array(&envelope["observations"], 32)?;
        need(
            captures.len() == specifications.len(),
            "incomplete observation set",
        )?;
        let mut by_id = BTreeMap::new();
        for row in captures {
            need(
                by_id.insert(text(&row["id"])?, row).is_none(),
                "duplicate observation ID",
            )?;
        }
        let mut observed = Vec::new();
        for spec in specifications {
            let id = text(&spec["id"])?;
            let row = by_id.remove(id).ok_or("missing canonical observation")?;
            need(
                row["request"] == spec["request"],
                "request differs from canonical manifest",
            )?;
            let status = row["status"]
                .as_u64()
                .filter(|n| (100..=599).contains(n))
                .ok_or("invalid HTTP status")?;
            for header in array(&row["headers"], 256)? {
                let pair = array(header, 2)?;
                need(pair.len() == 2, "header pair required")?;
                text(&pair[0])?;
                text(&pair[1])?;
            }
            let body: Value = serde_json::from_str(text(&row["body_utf8"])?)
                .map_err(|e| format!("{id}: JSON response: {e}"))?;
            let errors = if body["errors"].is_null() {
                false
            } else {
                !array(&body["errors"], 512)?.is_empty()
            };
            let outcome = if errors {
                "GRAPHQL_ERROR"
            } else if !(200..300).contains(&status) {
                "HTTP_ERROR"
            } else {
                "SUCCESS"
            };
            need(
                spec["expectation"] != "GRAPHQL_ERROR" || errors,
                "expected GraphQL refusal missing",
            )?;
            let choices = if outcome != "SUCCESS" {
                None
            } else {
                let viewer = &body["data"]["viewer"];
                let mut choices = Vec::new();
                if spec["variant"] == "all" {
                    for node in array(&viewer["allProjectTemplates"], 512)? {
                        choices.push(choice(node, false)?);
                    }
                } else {
                    let connection = &viewer["projectTemplates"];
                    page_info(&connection["pageInfo"])?;
                    let browser = spec["request"]["variables"]["context"] == "PROJECT_BROWSER";
                    for edge in array(&connection["edges"], 512)? {
                        choices.push(choice(&edge["node"], browser)?);
                    }
                }
                Some(choices)
            };
            observed.push(Observed {
                id: id.into(),
                outcome,
                choices,
            });
        }
        need(by_id.is_empty(), "unknown observation ID")?;
        Ok(Self { envelope, observed })
    }
    pub fn envelope(&self) -> &Value {
        &self.envelope
    }
    /// Duplicate native IDs are preserved in the observation, but cannot be
    /// selected ambiguously. No fallback template or instantiation is invented.
    pub fn lookup(&self, observation: &str, id: &str) -> Result<Choice> {
        need(id.len() <= 4096, "template ID byte limit")?;
        let view = self
            .observed
            .iter()
            .find(|o| o.id == observation)
            .ok_or("unknown observation")?;
        let choices = view
            .choices
            .as_ref()
            .ok_or_else(|| format!("observation failed: {}", view.outcome))?;
        let mut found = choices.iter().filter(|c| c.id() == id);
        let first = found.next().ok_or("unknown template/control ID")?;
        need(found.next().is_none(), "ambiguous template/control ID")?;
        Ok(first.clone())
    }
    pub fn projection(&self, selection: Option<Choice>) -> Result<Value> {
        let mut output = json!({"capture":self.envelope,"observations":[],"selection":selection.map(|c|c.json()),"C01":"OPEN","B23":"OPEN","U03":"OPEN","claim":"OBSERVED_TEMPLATE_CHOICES_ONLY"});
        let mut budget = Counter(0);
        count(&mut budget, &output)?;
        let mut records = Vec::new();
        for view in &self.observed {
            let choices = view
                .choices
                .as_ref()
                .map(|c| c.iter().map(Choice::json).collect::<Vec<_>>());
            let value = json!({"id":view.id,"outcome":view.outcome,"choices":choices});
            count(&mut budget, &value)?;
            records.push(value);
        }
        output["observations"] = Value::Array(records);
        serde_json::to_writer(Counter(0), &output).map_err(|e| e.to_string())?;
        Ok(output)
    }
}
pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    need(
        std::fs::symlink_metadata(path)
            .map_err(|e| e.to_string())?
            .is_file(),
        "regular nonsymlink capture required",
    )?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    need(bytes.len() <= MAX_BYTES, "capture byte limit")?;
    Ok(bytes)
}

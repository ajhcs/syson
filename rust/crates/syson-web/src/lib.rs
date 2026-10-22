// SPDX-License-Identifier: EPL-2.0
//! Bounded browser authoring UI. Model semantics remain in syson-model.
#![deny(unsafe_op_in_unsafe_fn)]
use serde_json::Value;
use std::{cell::RefCell, collections::BTreeMap};
use syson_editor::{Diagram, Node, Point, prepare_rotation};
use syson_model::{Document, Editor, NodeIds, Transaction};

const LIMIT: usize = 1_048_576;
struct App {
    editor: Editor,
    pending: Option<Transaction>,
    selected: String,
    message: String,
    error: bool,
    geometry: BTreeMap<String, (f64, f64)>,
}
impl Default for App {
    fn default() -> Self {
        Self {
            editor: Editor::new(Document::empty()),
            pending: None,
            selected: String::new(),
            message: "Create a package to begin. Changes are held in this browser tab.".into(),
            error: false,
            geometry: BTreeMap::new(),
        }
    }
}
fn field<'a>(request: &'a Value, key: &str) -> &'a str {
    request["fields"][key].as_str().unwrap_or("")
}
fn uuid(bytes: &[u8]) -> String {
    let mut b: [u8; 16] = bytes.try_into().expect("checked entropy chunk");
    b[6] = (b[6] & 15) | 64;
    b[8] = (b[8] & 63) | 128;
    let hex: String = b.iter().map(|v| format!("{v:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    )
}
fn ids(request: &Value) -> Result<Vec<NodeIds>, String> {
    let raw = request["entropy"]
        .as_array()
        .ok_or("Browser random bytes missing")?;
    if raw.len() != 96 {
        return Err("Expected 96 browser random bytes".into());
    }
    let bytes: Vec<u8> = raw
        .iter()
        .map(|v| {
            v.as_u64()
                .filter(|n| *n <= 255)
                .map(|n| n as u8)
                .ok_or("Invalid random byte".to_string())
        })
        .collect::<Result<_, _>>()?;
    (0..3)
        .map(|i| {
            NodeIds::new(
                &uuid(&bytes[i * 32..i * 32 + 16]),
                &uuid(&bytes[i * 32 + 16..i * 32 + 32]),
            )
            .map_err(|e| e.to_string())
        })
        .collect()
}
impl App {
    fn handle(&mut self, request: &Value) -> Result<String, String> {
        let action = request["action"].as_str().ok_or("Action missing")?;
        match action {
            "select" => {
                self.selected = field(request, "value").into();
                Ok("Selection updated.".into())
            }
            "stage" => {
                if self.pending.is_some() {
                    return Err("Commit or cancel the current staged change first.".into());
                }
                let mut allocated = ids(request)?.into_iter();
                let element = allocated.next().unwrap();
                let membership = allocated.next().unwrap();
                let typing = allocated.next().unwrap();
                let parent = field(request, "parent");
                let name = field(request, "name");
                let mut tx = self.editor.begin();
                let created = element.element.clone();
                match field(request, "kind") {
                    "package" => tx.create_package(
                        if parent.is_empty() {
                            None
                        } else {
                            Some(parent)
                        },
                        element,
                        if parent.is_empty() {
                            None
                        } else {
                            Some(membership)
                        },
                        name,
                    ),
                    "part_definition" => {
                        tx.create_part_definition(parent, element, membership, name)
                    }
                    "part_usage" => tx.create_part_usage(parent, element, membership, name),
                    _ => return Err("Choose a supported element kind.".into()),
                }
                .map_err(|e| e.to_string())?;
                if field(request, "kind") == "part_usage"
                    && !field(request, "definition").is_empty()
                {
                    tx.type_part_usage(&created, field(request, "definition"), typing)
                        .map_err(|e| e.to_string())?;
                }
                self.pending = Some(tx);
                Ok("Change staged. Review the preview, then commit or cancel.".into())
            }
            "rename" => {
                if self.pending.is_some() {
                    return Err("Commit or cancel the staged change first.".into());
                }
                let mut tx = self.editor.begin();
                tx.rename(&self.selected, field(request, "name"))
                    .map_err(|e| e.to_string())?;
                self.pending = Some(tx);
                Ok("Rename staged. Commit or cancel.".into())
            }
            "commit" => {
                let tx = self.pending.take().ok_or("No staged change")?;
                self.editor.commit(tx).map_err(|e| e.to_string())?;
                Ok("Change committed. Undo is available.".into())
            }
            "cancel" => {
                if let Some(tx) = self.pending.take() {
                    tx.cancel();
                }
                Ok("Staged change cancelled.".into())
            }
            "undo" | "redo" => {
                if self.pending.is_some() {
                    return Err("Commit or cancel the staged change first.".into());
                }
                let changed = if action == "undo" {
                    self.editor.undo()
                } else {
                    self.editor.redo()
                };
                Ok(if changed {
                    "History applied."
                } else {
                    "No history available."
                }
                .into())
            }
            "import" => {
                if self.pending.is_some() {
                    return Err("Commit or cancel the staged change first.".into());
                }
                let doc = Document::from_json(field(request, "json")).map_err(|e| e.to_string())?;
                self.editor = Editor::new(doc);
                self.selected.clear();
                self.geometry.clear();
                Ok("Imported a new document. Previous document history was cleared.".into())
            }
            "rotate" => {
                if self.selected.is_empty() {
                    return Err("Select an element first.".into());
                }
                let (width, height) = self
                    .geometry
                    .get(&self.selected)
                    .copied()
                    .unwrap_or((180., 70.));
                let diagram: Diagram<(), ()> = Diagram {
                    nodes: vec![Node {
                        id: self.selected.clone(),
                        position: Point::default(),
                        width: Some(width),
                        height: Some(height),
                        resized_by_user: false,
                        connection_handles: vec![],
                        payload: (),
                    }],
                    edges: vec![],
                };
                let rotated = prepare_rotation(&diagram, std::slice::from_ref(&self.selected));
                self.geometry.insert(
                    self.selected.clone(),
                    (
                        rotated.nodes[0].width.unwrap(),
                        rotated.nodes[0].height.unwrap(),
                    ),
                );
                Ok("Preview geometry rotated. Model data is unchanged.".into())
            }
            _ => Err("Unsupported action.".into()),
        }
    }
    fn render(&self) -> String {
        let doc = self
            .pending
            .as_ref()
            .map(Transaction::document)
            .unwrap_or_else(|| self.editor.document());
        let elements: Vec<_> = doc
            .elements()
            .into_iter()
            .filter(|e| {
                matches!(
                    e.class.as_str(),
                    "Package" | "Namespace" | "PartDefinition" | "PartUsage"
                )
            })
            .collect();
        let mut tree = String::new();
        let mut parent_options =
            String::from("<option value=\"\">Document root (package only)</option>");
        let mut definitions = String::from("<option value=\"\">No explicit definition</option>");
        let mut diagram = String::new();
        let mut y = 24.;
        for element in &elements {
            let id = escape(&element.element_id);
            let name = escape(element.name.as_deref().unwrap_or("(unnamed)"));
            let class = escape(&element.class);
            tree.push_str(&format!("<li><button type=\"button\" data-action=\"select\" data-value=\"{id}\" aria-pressed=\"{}\">{name}<span class=\"kind\">{class}</span></button></li>",self.selected==element.element_id));
            parent_options.push_str(&format!("<option value=\"{id}\">{name} · {class}</option>"));
            if element.class == "PartDefinition" {
                definitions.push_str(&format!("<option value=\"{id}\">{name}</option>"));
            }
            let (width, height) = self
                .geometry
                .get(&element.element_id)
                .copied()
                .unwrap_or((180., 70.));
            let stroke = if self.selected == element.element_id {
                "#1768b5"
            } else {
                "#8193a4"
            };
            diagram.push_str(&format!("<g><rect x=\"24\" y=\"{y}\" width=\"{width}\" height=\"{height}\" fill=\"white\" stroke=\"{stroke}\" stroke-width=\"2\"/><text x=\"36\" y=\"{}\" font-size=\"12\" fill=\"#465b6d\">{class}</text><text x=\"36\" y=\"{}\" font-size=\"14\">{name}</text></g>",y+22.,y+45.));
            y += height + 24.;
        }
        if tree.is_empty() {
            tree.push_str("<li><p>No model elements yet.</p></li>");
        }
        let selected = elements.iter().find(|e| e.element_id == self.selected);
        let selected_name = escape(selected.and_then(|e| e.name.as_deref()).unwrap_or(""));
        let inspector = if let Some(element) = selected {
            format!(
                "<h3>{}</h3><p><small>Stable element ID<br>{}</small></p><form data-action=\"rename\"><label for=\"rename\">Element name</label><input id=\"rename\" name=\"name\" value=\"{selected_name}\" required><div class=\"form-actions\"><button type=\"submit\">Stage rename</button></div></form><p><button type=\"button\" data-action=\"rotate\">Rotate preview</button></p>",
                escape(&element.class),
                escape(&element.element_id)
            )
        } else {
            "<p>Select an element in the model list.</p>".into()
        };
        let pending = if self.pending.is_some() {
            "<div class=\"form-actions\"><button class=\"primary\" type=\"button\" data-action=\"commit\">Commit staged change</button><button type=\"button\" data-action=\"cancel\">Cancel staged change</button></div>"
        } else {
            ""
        };
        let json = escape(
            &self
                .editor
                .document()
                .to_json()
                .unwrap_or_else(|e| e.to_string()),
        );
        format!(
            "<header><h1>SysON model editor</h1><nav class=\"toolbar\" aria-label=\"Model history\"><button type=\"button\" data-action=\"undo\">Undo</button><button type=\"button\" data-action=\"redo\">Redo</button></nav></header><p class=\"status {}\" role=\"{}\">{}</p><div class=\"layout\"><section class=\"panel\" aria-label=\"Model elements\"><h2>Model</h2><ul class=\"tree\">{tree}</ul></section><section class=\"canvas\"><h2>Diagram preview</h2><small>Local preview positions; upstream diagram layout is not implemented.</small><svg viewBox=\"0 0 420 {}\" aria-hidden=\"true\">{diagram}</svg></section><section class=\"panel inspector\"><h2>Create element</h2><form data-action=\"stage\"><label for=\"kind\">Element kind</label><select id=\"kind\" name=\"kind\"><option value=\"package\">Package</option><option value=\"part_definition\">Part definition</option><option value=\"part_usage\">Part usage</option></select><label for=\"parent\">Parent</label><select id=\"parent\" name=\"parent\">{parent_options}</select><label for=\"name\">Name</label><input id=\"name\" name=\"name\" required><label for=\"definition\">Definition for part usage</label><select id=\"definition\" name=\"definition\">{definitions}</select><div class=\"form-actions\"><button class=\"primary\" type=\"submit\">Stage creation</button></div></form>{pending}<hr><h2>Selection</h2>{inspector}</section></div><section class=\"file-panel\"><h2>Document JSON</h2><p>Copy the committed EMF JSON below to export. Import opens a new document and clears the current history.</p><label for=\"export-json\">Committed document export</label><textarea id=\"export-json\" readonly spellcheck=\"false\">{json}</textarea><details><summary>Import another document</summary><form data-action=\"import\"><label for=\"import-json\">EMF JSON document</label><textarea id=\"import-json\" name=\"json\" required spellcheck=\"false\"></textarea><div class=\"form-actions\"><button type=\"submit\">Import as new document and clear history</button></div></form></details></section>",
            if self.error { "error" } else { "" },
            if self.error { "alert" } else { "status" },
            escape(&self.message),
            y.max(250.)
        )
    }
}
fn escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
thread_local! {static STATE:RefCell<App>=RefCell::new(App::default());static INPUT:RefCell<Vec<u8>>=const{RefCell::new(Vec::new())};static OUTPUT:RefCell<String>=const{RefCell::new(String::new())};}
fn paint() {
    STATE.with(|s| OUTPUT.with(|o| {
        let html=s.borrow().render();
        *o.borrow_mut()=if html.len() > 16 * LIMIT {
            "<h1>SysON model editor</h1><p role=\"alert\">This model exceeds the 16 MiB preview output limit. Model data remains in tab memory.</p><button type=\"button\" data-action=\"undo\">Undo last edit</button>".into()
        } else {html};
    }));
}
#[unsafe(no_mangle)]
pub extern "C" fn initialize() {
    STATE.with(|s| *s.borrow_mut() = App::default());
    paint();
}
#[unsafe(no_mangle)]
pub extern "C" fn input_reset() {
    INPUT.with(|i| i.borrow_mut().clear());
}
#[unsafe(no_mangle)]
pub extern "C" fn input_byte(byte: u32) {
    INPUT.with(|i| {
        let mut input = i.borrow_mut();
        if input.len() <= LIMIT {
            input.push(if byte <= 255 { byte as u8 } else { 0xff });
        }
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn dispatch() {
    let request = INPUT.with(|i| {
        let bytes = std::mem::take(&mut *i.borrow_mut());
        if bytes.len() > LIMIT {
            Err("Input exceeds 1 MiB".into())
        } else {
            serde_json::from_slice::<Value>(&bytes)
                .map_err(|e| e.to_string())
                .and_then(|value| {
                    if !value.is_object() || !value["action"].is_string() {
                        return Err("Expected an action object".into());
                    }
                    if let Some(fields) = value.get("fields")
                        && (!fields.is_object()
                            || !fields.as_object().unwrap().values().all(Value::is_string))
                    {
                        return Err("Action fields must be strings".into());
                    }
                    Ok(value)
                })
        }
    });
    STATE.with(|s| {
        let mut app = s.borrow_mut();
        let result = request.and_then(|r| app.handle(&r));
        match result {
            Ok(message) => {
                app.message = message;
                app.error = false;
            }
            Err(message) => {
                app.message = message;
                app.error = true;
            }
        }
    });
    paint();
}
/// Pointer and length remain valid until the next initialize/dispatch call.
/// The loader must copy/decode synchronously before changing Rust state.
#[unsafe(no_mangle)]
pub extern "C" fn output_ptr() -> usize {
    OUTPUT.with(|o| o.borrow().as_ptr() as usize)
}
#[unsafe(no_mangle)]
pub extern "C" fn output_len() -> usize {
    OUTPUT.with(|o| o.borrow().len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn create(seed: u8, name: &str) -> Value {
        json!({"action":"stage","fields":{"kind":"package","name":name,"parent":""},"entropy":(0..96).map(|i|(i as u8).wrapping_add(seed)).collect::<Vec<_>>()})
    }
    #[test]
    fn staged_creation_cancel_commit_undo_redo_use_shared_model_history() {
        let mut app = App::default();
        app.handle(&create(0, "Vehicle")).unwrap();
        assert_eq!(app.editor.document().elements().len(), 0);
        assert_eq!(app.pending.as_ref().unwrap().document().elements().len(), 1);
        app.handle(&json!({"action":"cancel"})).unwrap();
        assert!(app.pending.is_none());
        assert_eq!(app.editor.document().elements().len(), 0);
        app.handle(&create(0, "Vehicle")).unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        assert_eq!(app.editor.document().elements().len(), 1);
        app.handle(&json!({"action":"undo"})).unwrap();
        assert_eq!(app.editor.document().elements().len(), 0);
        app.handle(&json!({"action":"redo"})).unwrap();
        assert_eq!(app.editor.document().elements().len(), 1);
    }
    #[test]
    fn import_error_preserves_current_model_and_staged_edits_prevent_import() {
        let mut app = App::default();
        app.handle(&create(0, "Vehicle")).unwrap();
        assert!(
            app.handle(&json!({"action":"import","fields":{"json":"{}"}}))
                .is_err()
        );
        assert!(app.pending.is_some());
        app.handle(&json!({"action":"commit"})).unwrap();
        let before = app.editor.document().to_json().unwrap();
        assert!(
            app.handle(&json!({"action":"import","fields":{"json":"{}"}}))
                .is_err()
        );
        assert_eq!(app.editor.document().to_json().unwrap(), before);
    }
    #[test]
    fn html_is_escaped_before_model_names_enter_dom_and_export_text() {
        assert_eq!(escape("<&\"'>"), "&lt;&amp;&quot;&#39;&gt;");
        let mut app = App::default();
        app.handle(&create(0, "<script>alert(1)</script>")).unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        let html = app.render();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }
    #[test]
    fn typed_vehicle_workflow_uses_model_type_and_membership_contracts() {
        let mut app = App::default();
        app.handle(&create(0, "Vehicle")).unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        let package = app.editor.document().elements()[0].element_id.clone();
        let mut definition = create(1, "Engine");
        definition["fields"]["kind"] = json!("part_definition");
        definition["fields"]["parent"] = json!(package);
        app.handle(&definition).unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        let definition = app
            .editor
            .document()
            .elements()
            .into_iter()
            .find(|e| e.class == "PartDefinition")
            .unwrap()
            .element_id;
        let mut usage = create(2, "engine");
        usage["fields"]["kind"] = json!("part_usage");
        usage["fields"]["parent"] = json!(package);
        usage["fields"]["definition"] = json!(definition);
        app.handle(&usage).unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        let elements = app.editor.document().elements();
        assert!(elements.iter().any(|e| e.class == "FeatureTyping"));
        assert!(
            elements
                .iter()
                .any(|e| e.class == "PartUsage" && e.name.as_deref() == Some("engine"))
        );
        let export = app.editor.document().to_json().unwrap();
        assert_eq!(
            Document::from_json(&export).unwrap(),
            *app.editor.document()
        );
    }
    #[test]
    fn byte_abi_reports_invalid_json_and_bounds_input_without_pointer_allocation() {
        initialize();
        input_reset();
        input_byte(b'{' as u32);
        dispatch();
        OUTPUT.with(|o| assert!(o.borrow().contains("role=\"alert\"")));
        input_reset();
        for _ in 0..LIMIT + 5 {
            input_byte(b'a' as u32);
        }
        dispatch();
        OUTPUT.with(|o| assert!(o.borrow().contains("Input exceeds 1 MiB")));
    }
}

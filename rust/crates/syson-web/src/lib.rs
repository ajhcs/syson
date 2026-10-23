// SPDX-License-Identifier: EPL-2.0
//! Bounded browser authoring UI. Model semantics remain in syson-model.
#![deny(unsafe_op_in_unsafe_fn)]
use serde_json::Value;
use std::{cell::RefCell, collections::BTreeMap};
use syson_editor::{Diagram, Node, Point, prepare_rotation};
use syson_metamodel::ClassifierId;
use syson_model::{
    AttributeBlock, AttributeDescriptor, AttributeError, Document, Editor, NodeIds, ScalarType,
    Transaction,
};

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
                if !parent.is_empty() && !self.editor.document().can_create_in(parent) {
                    return Err("This element cannot contain new elements in this editor.".into());
                }
                let definition = field(request, "definition");
                if field(request, "kind") == "part_usage"
                    && !definition.is_empty()
                    && !self
                        .editor
                        .document()
                        .is_instance_of(definition, ClassifierId::PartDefinition)
                {
                    return Err("Choose a supported part definition.".into());
                }
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
            "attribute" => {
                if self.pending.is_some() {
                    return Err("Commit or cancel the staged change first.".into());
                }
                let inspection = self
                    .editor
                    .document()
                    .inspect_node(&self.selected)
                    .map_err(|_| "Properties are unavailable for this element.".to_string())?;
                let descriptor = inspection
                    .writable_attributes()
                    .find(|a| a.name == field(request, "attribute"))
                    .ok_or("This property is read only or unavailable.")?;
                let value =
                    parse_scalar(descriptor, field(request, "mode"), field(request, "value"))?;
                let mut tx = self.editor.begin();
                tx.set_attribute(&self.selected, descriptor.name, value)
                    .map_err(|_| {
                        "The model rejected this property value. No change was staged.".to_string()
                    })?;
                self.pending = Some(tx);
                Ok("Property change staged. Review it, then commit or cancel.".into())
            }
            "rename" => {
                if self.pending.is_some() {
                    return Err("Commit or cancel the staged change first.".into());
                }
                if !can_rename(self.editor.document(), &self.selected) {
                    return Err("Renaming is unavailable for this element type.".into());
                }
                let name = field(request, "name");
                if name.is_empty() {
                    return Err("Enter a name.".into());
                }
                let value = preserve_unchanged_text(
                    self.editor
                        .document()
                        .element(&self.selected)
                        .and_then(|node| node["data"]["declaredName"].as_str()),
                    name,
                );
                let mut tx = self.editor.begin();
                tx.set_attribute(&self.selected, "declaredName", Value::String(value))
                    .map_err(|_| {
                        "The model rejected this name. No change was staged.".to_string()
                    })?;
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
        // Every imported element remains selectable, including literal values and
        // relationship objects. Only the existing bounded kinds get SVG previews.
        let elements = doc.elements();
        let mut tree = String::new();
        let mut parent_options =
            String::from("<option value=\"\">Document root (package only)</option>");
        let mut definitions = String::from("<option value=\"\">No explicit definition</option>");
        let mut diagram = String::new();
        let mut y = 24.;
        for (index, element) in elements.iter().enumerate() {
            let id = escape(&element.element_id);
            let name = escape(element.name.as_deref().unwrap_or("(unnamed)"));
            let class = escape(&element.class);
            tree.push_str(&format!("<li><button type=\"button\" data-action=\"select\" data-value=\"{id}\" aria-pressed=\"{}\">{name}<span class=\"kind\">{class}</span></button></li>",self.selected==element.element_id));
            if doc.can_create_in(&element.element_id) {
                parent_options
                    .push_str(&format!("<option value=\"{id}\">{name} · {class}</option>"));
            }
            if doc.is_instance_of(&element.element_id, ClassifierId::PartDefinition) {
                definitions.push_str(&format!("<option value=\"{id}\">{name}</option>"));
            }
            if !matches!(
                element.class.as_str(),
                "Package" | "Namespace" | "PartDefinition" | "PartUsage"
            ) {
                continue;
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
            // Clip labels to the padded node interior, including after rotation.
            // Preserve full strings in the title and the separate model list.
            let clip_width = (width - 24.).max(0.);
            let clip_height = (height - 16.).max(0.);
            diagram.push_str(&format!("<g><title>{class}: {name}</title><rect x=\"24\" y=\"{y}\" width=\"{width}\" height=\"{height}\" fill=\"white\" stroke=\"{stroke}\" stroke-width=\"2\"/><defs><clipPath id=\"preview-label-{index}\"><rect x=\"36\" y=\"{}\" width=\"{clip_width}\" height=\"{clip_height}\"/></clipPath></defs><g clip-path=\"url(#preview-label-{index})\"><text x=\"36\" y=\"{}\" font-size=\"12\" fill=\"#465b6d\">{class}</text><text x=\"36\" y=\"{}\" font-size=\"14\">{name}</text></g></g>",y+8.,y+22.,y+45.));
            y += height + 24.;
        }
        if tree.is_empty() {
            tree.push_str("<li><p>No model elements yet.</p></li>");
        }
        let selected = elements.iter().find(|e| e.element_id == self.selected);
        let selected_name = escape_textarea(selected.and_then(|e| e.name.as_deref()).unwrap_or(""));
        let scalar_inspector = if selected.is_some() {
            render_scalar_inspector(doc, &self.selected)
        } else {
            String::new()
        };
        let inspector = if let Some(element) = selected {
            let rename_form = if can_rename(doc, &self.selected) {
                format!(
                    "<form data-action=\"rename\"><label for=\"rename\">Element name</label><textarea id=\"rename\" name=\"name\" rows=\"2\" required>{selected_name}</textarea><div class=\"form-actions\"><button type=\"submit\">Stage rename</button></div></form>"
                )
            } else {
                String::new()
            };
            let rotate_control = if matches!(
                element.class.as_str(),
                "Package" | "Namespace" | "PartDefinition" | "PartUsage"
            ) {
                "<p><button type=\"button\" data-action=\"rotate\">Rotate preview</button></p>"
            } else {
                ""
            };
            format!(
                "<h3>{}</h3><p><small>Stable element ID<br>{}</small></p>{rename_form}{rotate_control}{scalar_inspector}",
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
/// Textarea APIs normalize line endings. Preserve the saved spelling when the
/// submitted value is otherwise unchanged, rather than silently rewriting it.
fn preserve_unchanged_text(original: Option<&str>, input: &str) -> String {
    if let Some(original) = original
        && original
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .replace('\0', "\u{fffd}")
            == input
    {
        return original.into();
    }
    input.into()
}
/// HTML discards the first LF in textarea markup. Add a sentinel LF so an
/// original leading LF survives. Encode CR to avoid parser preprocessing;
/// display HTML's NUL replacement explicitly, retaining original saved bytes
/// when the unchanged displayed value is submitted.
fn escape_textarea(input: &str) -> String {
    format!(
        "\n{}",
        escape(input)
            .replace('\r', "&#13;")
            .replace('\0', "\u{fffd}")
    )
}
fn can_rename(document: &Document, element: &str) -> bool {
    document
        .inspect_node(element)
        .is_ok_and(|info| info.writable_attributes().any(|a| a.name == "declaredName"))
}
/// Convert form strings to the model's exact scalar JSON types in Rust.
fn parse_scalar(attribute: &AttributeDescriptor, mode: &str, input: &str) -> Result<Value, String> {
    if !attribute.writable() {
        return Err("This property is read only.".into());
    }
    if mode == "null" {
        return if attribute.accepts_null() {
            Ok(Value::Null)
        } else {
            Err("This property cannot be cleared to null.".into())
        };
    }
    if mode != "value" {
        return Err("Choose whether to set or clear this value.".into());
    }
    match attribute.value_type.as_ref() {
        Some(ScalarType::String) => Ok(Value::String(preserve_unchanged_text(
            attribute.value.as_ref().and_then(Value::as_str),
            input,
        ))),
        Some(ScalarType::Boolean) => match input {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err("Choose true or false.".into()),
        },
        Some(ScalarType::Integer32) => input
            .trim()
            .parse::<i32>()
            .map(Value::from)
            .map_err(|_| "Enter a whole number from -2147483648 to 2147483647.".into()),
        Some(ScalarType::Double) => input
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .ok_or_else(|| "Enter a finite number.".into()),
        Some(ScalarType::Enumeration { literals, .. }) => {
            if literals.contains(&input) {
                Ok(Value::String(input.into()))
            } else {
                Err("Choose one of the listed values.".into())
            }
        }
        None => Err("This property type cannot be edited here.".into()),
    }
}
fn property_label(name: &str) -> String {
    let mut label = String::new();
    for (index, ch) in name.chars().enumerate() {
        if index == 0 {
            label.extend(ch.to_uppercase());
        } else {
            if ch.is_uppercase() {
                label.push(' ');
            }
            label.extend(ch.to_lowercase());
        }
    }
    label
}
fn stored_state(attribute: &AttributeDescriptor) -> String {
    match &attribute.value {
        None => "Not set (no saved value).".into(),
        Some(Value::Null) => "Explicitly empty (null).".into(),
        Some(value) => format!("Saved value: {}", escape(&value.to_string())),
    }
}
fn render_scalar_field(attribute: &AttributeDescriptor) -> String {
    let name = escape(attribute.name);
    let label = escape(&property_label(attribute.name));
    let input_id = format!("scalar-{name}");
    let description = format!("scalar-state-{name}");
    let text = escape_textarea(
        attribute
            .value
            .as_ref()
            .and_then(Value::as_str)
            .unwrap_or(""),
    );
    let control = match attribute.value_type.as_ref() {
        Some(ScalarType::String) => format!(
            "<textarea id=\"{input_id}\" name=\"value\" rows=\"3\" aria-describedby=\"{description}\">{text}</textarea>"
        ),
        Some(ScalarType::Boolean) => {
            let current = attribute.value.as_ref().and_then(Value::as_bool);
            format!(
                "<select id=\"{input_id}\" name=\"value\" aria-describedby=\"{description}\" required><option value=\"\" disabled {}>Choose true or false</option><option value=\"true\" {}>True</option><option value=\"false\" {}>False</option></select>",
                if current.is_none() { "selected" } else { "" },
                if current == Some(true) {
                    "selected"
                } else {
                    ""
                },
                if current == Some(false) {
                    "selected"
                } else {
                    ""
                }
            )
        }
        Some(ScalarType::Integer32) | Some(ScalarType::Double) => {
            let value = attribute
                .value
                .as_ref()
                .filter(|v| v.is_number())
                .map(Value::to_string)
                .unwrap_or_default();
            let bounds = if attribute.value_type == Some(ScalarType::Integer32) {
                "step=\"1\" min=\"-2147483648\" max=\"2147483647\""
            } else {
                "step=\"any\""
            };
            format!(
                "<input type=\"number\" id=\"{input_id}\" name=\"value\" value=\"{}\" {bounds} aria-describedby=\"{description}\" required>",
                escape(&value)
            )
        }
        Some(ScalarType::Enumeration { literals, .. }) => {
            let current = attribute.value.as_ref().and_then(Value::as_str);
            let recognized = current.is_some_and(|value| literals.contains(&value));
            let mut options = format!(
                "<option value=\"\" disabled {}>Choose a value</option>",
                if recognized { "" } else { "selected" }
            );
            for literal in literals {
                options.push_str(&format!(
                    "<option value=\"{}\" {}>{}</option>",
                    escape(literal),
                    if current == Some(*literal) {
                        "selected"
                    } else {
                        ""
                    },
                    escape(literal)
                ));
            }
            format!(
                "<select id=\"{input_id}\" name=\"value\" aria-describedby=\"{description}\" required>{options}</select>"
            )
        }
        None => return String::new(),
    };
    let null_option = if attribute.accepts_null() {
        "<option value=\"null\">Clear value (null)</option>"
    } else {
        ""
    };
    format!(
        "<form class=\"scalar-property\" data-action=\"attribute\" data-attribute=\"{name}\"><input type=\"hidden\" name=\"attribute\" value=\"{name}\"><label for=\"{input_id}\">{label}</label><p id=\"{description}\" class=\"property-state\">{}</p>{control}<label for=\"scalar-mode-{name}\">Change for {label}</label><select id=\"scalar-mode-{name}\" name=\"mode\"><option value=\"value\">Set a value</option>{null_option}</select><div class=\"form-actions\"><button type=\"submit\">Stage {label}</button></div></form>",
        stored_state(attribute)
    )
}
fn blocked_label(reason: &AttributeBlock) -> &'static str {
    match reason {
        AttributeBlock::Derived => "Calculated value; read only.",
        AttributeBlock::NotChangeable => "Read only.",
        AttributeBlock::ManyValued => "List editing is unavailable.",
        AttributeBlock::IdentityManaged => "Identifier; read only.",
        AttributeBlock::UnsupportedType(_) => "This value type cannot be edited here.",
    }
}
fn render_scalar_inspector(document: &Document, selected: &str) -> String {
    let Ok(inspection) = document.inspect_node(selected) else {
        return "<p>Properties are unavailable for this element type.</p>".into();
    };
    let mut editable = String::new();
    let mut unavailable = String::new();
    let mut count = 0;
    for attribute in &inspection.attributes {
        if let Some(reason) = &attribute.blocked {
            unavailable.push_str(&format!(
                "<dt>{}</dt><dd>{} {}</dd>",
                escape(&property_label(attribute.name)),
                blocked_label(reason),
                stored_state(attribute)
            ));
            count += 1;
        } else {
            editable.push_str(&render_scalar_field(attribute));
        }
    }
    for diagnostic in &inspection.diagnostics {
        let reason = match diagnostic.issue {
            AttributeError::ReferenceEditUnsupported {
                containment: true, ..
            } => "Contains other elements; editing is unavailable.",
            AttributeError::ReferenceEditUnsupported {
                containment: false, ..
            } => "Links to other elements; editing is unavailable.",
            _ => "This property is unavailable for editing.",
        };
        unavailable.push_str(&format!(
            "<dt>{}</dt><dd>{reason}</dd>",
            escape(&property_label(&diagnostic.name))
        ));
        count += 1;
    }
    let inherited_note = if inspection.unresolved_supertypes.is_empty() {
        ""
    } else {
        "<p>Some inherited properties are unavailable.</p>"
    };
    format!(
        "<section aria-label=\"Element properties\"><h3>Properties</h3><p>Not set and null are shown separately. Removing a saved value is unavailable.</p>{editable}<details><summary>Read-only and unavailable properties ({count})</summary><dl>{unavailable}</dl></details>{inherited_note}</section>"
    )
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
    fn rotated_narrow_usage_labels_are_clipped_with_full_text_preserved() {
        let mut app = App::default();
        app.handle(&create(0, "Vehicle")).unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        let package = app.editor.document().elements()[0].element_id.clone();
        let mut usage = create(1, "frontWheel");
        usage["fields"]["kind"] = json!("part_usage");
        usage["fields"]["parent"] = json!(package);
        app.handle(&usage).unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        app.selected = app
            .editor
            .document()
            .elements()
            .into_iter()
            .find(|e| e.class == "PartUsage")
            .unwrap()
            .element_id;
        app.handle(&json!({"action":"rotate"})).unwrap();
        let html = app.render();
        assert!(html.contains("width=\"70\" height=\"180\""));
        assert!(html.contains("<clipPath id=\"preview-label-2\"><rect x=\"36\" y=\"126\" width=\"46\" height=\"164\"/>"));
        assert!(html.contains("<g clip-path=\"url(#preview-label-2)\">"));
        assert!(html.contains("<title>PartUsage: frontWheel</title>"));
        assert!(
            html.contains("aria-pressed=\"true\">frontWheel<span class=\"kind\">PartUsage</span>")
        );
    }
    fn scalar_app(class: &str, data: Value) -> App {
        let document=Document::from_json(&json!({"json":{"version":"1.0","encoding":"utf-8"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[{"id":"resource","eClass":format!("sysml:{class}"),"data":data}]}).to_string()).unwrap();
        App {
            editor: Editor::new(document),
            selected: "element".into(),
            ..Default::default()
        }
    }
    fn attribute_request(name: &str, mode: &str, value: &str) -> Value {
        json!({"action":"attribute","fields":{"attribute":name,"mode":mode,"value":value}})
    }
    #[test]
    fn boolean_edit_stages_cancels_commits_and_uses_shared_history() {
        let mut app = scalar_app(
            "PartUsage",
            json!({"elementId":"element","declaredName":"part"}),
        );
        let original = app.editor.document().clone();
        app.handle(&attribute_request("isComposite", "value", "true"))
            .unwrap();
        assert_eq!(*app.editor.document(), original);
        assert_eq!(
            app.pending
                .as_ref()
                .unwrap()
                .document()
                .element("element")
                .unwrap()["data"]["isComposite"],
            true
        );
        assert!(
            app.handle(&attribute_request("isComposite", "value", "false"))
                .is_err()
        );
        app.handle(&json!({"action":"cancel"})).unwrap();
        assert_eq!(*app.editor.document(), original);
        app.handle(&attribute_request("isComposite", "value", "true"))
            .unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        assert_eq!(
            app.editor.document().element("element").unwrap()["data"]["isComposite"],
            true
        );
        app.handle(&json!({"action":"undo"})).unwrap();
        assert_eq!(*app.editor.document(), original);
        app.handle(&json!({"action":"redo"})).unwrap();
        assert_eq!(
            app.editor.document().element("element").unwrap()["data"]["isComposite"],
            true
        );
    }
    #[test]
    fn absent_null_and_empty_string_remain_distinct_without_default_materialization() {
        let mut app = scalar_app(
            "PartUsage",
            json!({"elementId":"element","declaredName":"part"}),
        );
        let before = app.editor.document().clone();
        let inspection = app.editor.document().inspect_node("element").unwrap();
        let composite = inspection
            .attributes
            .iter()
            .find(|a| a.name == "isComposite")
            .unwrap();
        let html = render_scalar_field(composite);
        assert!(html.contains("Not set (no saved value)."));
        assert!(html.contains("value=\"\" disabled selected>Choose true or false"));
        assert!(!html.contains("value=\"false\" selected"));
        assert_eq!(*app.editor.document(), before);
        app.handle(&attribute_request("declaredShortName", "null", ""))
            .unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        let short = app
            .editor
            .document()
            .inspect_node("element")
            .unwrap()
            .attributes
            .into_iter()
            .find(|a| a.name == "declaredShortName")
            .unwrap();
        assert!(render_scalar_field(&short).contains("Explicitly empty (null)."));
        app.handle(&attribute_request("declaredShortName", "value", ""))
            .unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        assert_eq!(
            app.editor.document().element("element").unwrap()["data"]["declaredShortName"],
            ""
        );
        assert!(
            app.handle(&attribute_request("declaredShortName", "unset", ""))
                .is_err()
        );
        assert!(app.pending.is_none());
    }
    #[test]
    fn integer_and_double_forms_parse_to_actual_json_numbers_and_reject_invalid_values() {
        let mut integer = scalar_app("LiteralInteger", json!({"elementId":"element"}));
        assert!(
            integer
                .render()
                .contains("class=\"kind\">LiteralInteger</span>")
        );
        assert!(
            integer
                .render()
                .contains("min=\"-2147483648\" max=\"2147483647\"")
        );
        let original = integer.editor.document().clone();
        for invalid in ["", "1.5", "2147483648", "-2147483649", "NaN"] {
            assert!(
                integer
                    .handle(&attribute_request("value", "value", invalid))
                    .is_err()
            );
            assert!(integer.pending.is_none());
            assert_eq!(*integer.editor.document(), original);
        }
        integer
            .handle(&attribute_request("value", "value", "-2147483648"))
            .unwrap();
        integer.handle(&json!({"action":"commit"})).unwrap();
        assert_eq!(
            integer.editor.document().element("element").unwrap()["data"]["value"],
            i32::MIN
        );
        let mut double = scalar_app("LiteralRational", json!({"elementId":"element"}));
        assert!(double.render().contains("step=\"any\""));
        for invalid in ["", "NaN", "inf", "1e9999"] {
            assert!(
                double
                    .handle(&attribute_request("value", "value", invalid))
                    .is_err()
            );
            assert!(double.pending.is_none());
        }
        double
            .handle(&attribute_request("value", "value", "1.25e2"))
            .unwrap();
        double.handle(&json!({"action":"commit"})).unwrap();
        assert_eq!(
            double.editor.document().element("element").unwrap()["data"]["value"],
            125.0
        );
    }
    #[test]
    fn enum_controls_use_exact_literals_and_reject_null_or_unlisted_values() {
        let mut app = scalar_app(
            "PartUsage",
            json!({"elementId":"element","declaredName":"part"}),
        );
        let descriptor = app
            .editor
            .document()
            .inspect_node("element")
            .unwrap()
            .attributes
            .into_iter()
            .find(|a| a.name == "direction")
            .unwrap();
        let field = render_scalar_field(&descriptor);
        for literal in ["in", "inout", "out"] {
            assert!(field.contains(&format!("value=\"{literal}\"")));
        }
        assert!(!field.contains("value=\"null\""));
        for (mode, value) in [("null", ""), ("value", "IN"), ("value", ""), ("value", "1")] {
            assert!(
                app.handle(&attribute_request("direction", mode, value))
                    .is_err()
            );
            assert!(app.pending.is_none());
        }
        app.handle(&attribute_request("direction", "value", "inout"))
            .unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        assert_eq!(
            app.editor.document().element("element").unwrap()["data"]["direction"],
            "inout"
        );
        assert!(
            app.handle(&attribute_request("isComposite", "value", "1"))
                .is_err()
        );
        assert!(
            app.handle(&attribute_request("isComposite", "null", ""))
                .is_err()
        );
    }
    #[test]
    fn blocked_properties_have_explanations_and_cannot_stage_mutations() {
        let mut app = scalar_app(
            "PartUsage",
            json!({"elementId":"element","declaredName":"part"}),
        );
        let original = app.editor.document().clone();
        for property in [
            "elementId",
            "owner",
            "ownedRelationship",
            "name",
            "aliasIds",
            "notAProperty",
        ] {
            assert!(
                app.handle(&attribute_request(property, "value", "changed"))
                    .is_err()
            );
            assert!(app.pending.is_none());
        }
        let html = app.render();
        assert!(html.contains("Identifier; read only."));
        assert!(html.contains("Calculated value; read only."));
        assert!(html.contains("List editing is unavailable."));
        assert!(html.contains("editing is unavailable."));
        assert!(!html.contains("data-attribute=\"elementId\""));
        assert_eq!(*app.editor.document(), original);
    }
    #[test]
    fn scalar_string_rendering_escapes_saved_and_staged_text() {
        let mut app = scalar_app(
            "PartUsage",
            json!({"elementId":"element","declaredName":"part"}),
        );
        app.handle(&attribute_request(
            "declaredShortName",
            "value",
            "<img onerror=\"boom\">&",
        ))
        .unwrap();
        let html = app.render();
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img onerror=&quot;boom&quot;&gt;&amp;"));
        app.handle(&json!({"action":"commit"})).unwrap();
        assert_eq!(
            app.editor.document().element("element").unwrap()["data"]["declaredShortName"],
            "<img onerror=\"boom\">&"
        );
    }
    #[test]
    fn unknown_imported_class_remains_selectable_without_advertising_edit_support() {
        let mut app = scalar_app(
            "UnknownClass",
            json!({"elementId":"element","declaredName":"unresolved"}),
        );
        let html = app.render();
        assert!(html.contains("Properties are unavailable for this element type."));
        assert!(!html.contains("data-action=\"rename\""));
        assert!(!html.contains("data-action=\"rotate\""));
        assert!(
            app.handle(&json!({"action":"rename","fields":{"name":"changed"}}))
                .is_err()
        );
        assert!(
            app.handle(&attribute_request("declaredName", "value", "changed"))
                .is_err()
        );
        assert!(app.pending.is_none());
    }
    #[test]
    fn generic_rename_form_stages_and_commits_imported_literal_and_action_names() {
        for class in ["LiteralInteger", "ActionUsage"] {
            let mut app = scalar_app(
                class,
                json!({"elementId":"element","declaredName":"original"}),
            );
            let original = app.editor.document().clone();
            assert!(app.render().contains("data-action=\"rename\""));
            assert!(
                app.handle(&json!({"action":"rename","fields":{"name":""}}))
                    .is_err()
            );
            assert!(app.pending.is_none());
            let request = json!({"action":"rename","fields":{"name":"imported <renamed>"}});
            app.handle(&request).unwrap();
            assert_eq!(*app.editor.document(), original);
            assert_eq!(
                app.pending
                    .as_ref()
                    .unwrap()
                    .document()
                    .element("element")
                    .unwrap()["data"]["declaredName"],
                "imported <renamed>"
            );
            app.handle(&json!({"action":"cancel"})).unwrap();
            assert_eq!(*app.editor.document(), original);
            app.handle(&request).unwrap();
            app.handle(&json!({"action":"commit"})).unwrap();
            assert_eq!(
                app.editor.document().element("element").unwrap()["data"]["declaredName"],
                "imported <renamed>"
            );
            app.handle(&json!({"action":"undo"})).unwrap();
            assert_eq!(*app.editor.document(), original);
            app.handle(&json!({"action":"redo"})).unwrap();
            assert_eq!(
                app.editor.document().element("element").unwrap()["data"]["declaredName"],
                "imported <renamed>"
            );
        }
    }
    #[test]
    fn shared_capabilities_offer_imported_parents_and_commit_created_typed_children() {
        for class in ["ActionUsage", "AttributeDefinition", "ConnectionDefinition"] {
            let mut app = scalar_app(
                class,
                json!({"elementId":"element","declaredName":"imported"}),
            );
            let original = app.editor.document().clone();
            let html = app.render();
            let parents = html
                .split("<select id=\"parent\"")
                .nth(1)
                .unwrap()
                .split("</select>")
                .next()
                .unwrap();
            let definitions = html
                .split("<select id=\"definition\"")
                .nth(1)
                .unwrap()
                .split("</select>")
                .next()
                .unwrap();
            assert!(parents.contains("value=\"element\""));
            assert_eq!(
                definitions.contains("value=\"element\""),
                class == "ConnectionDefinition"
            );
            let mut request = create(0, "child");
            request["fields"]["parent"] = json!("element");
            request["fields"]["kind"] = json!("part_usage");
            if class == "ConnectionDefinition" {
                request["fields"]["definition"] = json!("element");
            }
            app.handle(&request).unwrap();
            assert_eq!(*app.editor.document(), original);
            app.handle(&json!({"action":"commit"})).unwrap();
            let children = app.editor.document().elements();
            assert!(
                children
                    .iter()
                    .any(|e| e.class == "PartUsage" && e.name.as_deref() == Some("child"))
            );
            if class == "ActionUsage" {
                assert_eq!(
                    app.editor.document().element("element").unwrap()["data"]["ownedRelationship"]
                        [0]["eClass"],
                    "sysml:FeatureMembership"
                );
            }
            if class == "ConnectionDefinition" {
                let typing = children
                    .iter()
                    .find(|e| e.class == "FeatureTyping")
                    .unwrap();
                assert_eq!(
                    app.editor.document().element(&typing.element_id).unwrap()["data"]["type"],
                    "element"
                );
            }
        }
    }
    #[test]
    fn shared_capabilities_exclude_unknown_foreign_and_enum_imports_from_creation_and_typing() {
        for class in [
            "sysml:UnknownClass",
            "foreign:ActionUsage",
            "sysml:VisibilityKind",
        ] {
            let mut app = scalar_app(
                "PartUsage",
                json!({"elementId":"element","declaredName":"unsupported"}),
            );
            let mut value = app.editor.document().value().clone();
            value["content"][0]["eClass"] = json!(class);
            app.editor = Editor::new(Document::from_json(&value.to_string()).unwrap());
            let original = app.editor.document().clone();
            let html = app.render();
            for selector in ["parent", "definition"] {
                assert!(
                    !html
                        .split(&format!("<select id=\"{selector}\""))
                        .nth(1)
                        .unwrap()
                        .split("</select>")
                        .next()
                        .unwrap()
                        .contains("value=\"element\"")
                );
            }
            let mut request = create(0, "child");
            request["fields"]["parent"] = json!("element");
            assert!(app.handle(&request).is_err());
            assert!(app.pending.is_none());
            assert_eq!(*app.editor.document(), original);
            request["fields"]["parent"] = json!("");
            request["fields"]["kind"] = json!("part_usage");
            request["fields"]["definition"] = json!("element");
            assert!(app.handle(&request).is_err());
            assert!(app.pending.is_none());
            assert_eq!(*app.editor.document(), original);
        }
    }
    #[test]
    fn multiline_string_controls_preserve_leading_newline_edits_and_unchanged_crlf() {
        let mut comment = scalar_app(
            "Comment",
            json!({"elementId":"element","body":"\nfirst\nsecond\n"}),
        );
        let html = comment.render();
        assert!(html.contains("<textarea id=\"scalar-body\" name=\"value\" rows=\"3\" aria-describedby=\"scalar-state-body\">\n\nfirst\nsecond\n</textarea>"));
        comment
            .handle(&attribute_request("body", "value", "\nfirst\nchanged\n"))
            .unwrap();
        assert_eq!(
            comment
                .pending
                .as_ref()
                .unwrap()
                .document()
                .element("element")
                .unwrap()["data"]["body"],
            "\nfirst\nchanged\n"
        );
        comment.handle(&json!({"action":"cancel"})).unwrap();
        assert_eq!(
            comment.editor.document().element("element").unwrap()["data"]["body"],
            "\nfirst\nsecond\n"
        );
        let original = "\r\nfirst\r\nsecond\r";
        let mut literal = scalar_app(
            "LiteralString",
            json!({"elementId":"element","value":original}),
        );
        assert!(
            literal
                .render()
                .contains("&#13;\nfirst&#13;\nsecond&#13;</textarea>")
        );
        literal
            .handle(&attribute_request("value", "value", "\nfirst\nsecond\n"))
            .unwrap();
        literal.handle(&json!({"action":"commit"})).unwrap();
        assert_eq!(
            literal.editor.document().element("element").unwrap()["data"]["value"],
            original
        );
        let json = literal.editor.document().to_json().unwrap();
        assert_eq!(
            Document::from_json(&json).unwrap(),
            *literal.editor.document()
        );
        let mut named = scalar_app(
            "ActionUsage",
            json!({"elementId":"element","declaredName":"\nfirst\r\nsecond"}),
        );
        assert!(named.render().contains("<textarea id=\"rename\" name=\"name\" rows=\"2\" required>\n\nfirst&#13;\nsecond</textarea>"));
        named
            .handle(&json!({"action":"rename","fields":{"name":"\nfirst\nsecond"}}))
            .unwrap();
        named.handle(&json!({"action":"commit"})).unwrap();
        assert_eq!(
            named.editor.document().element("element").unwrap()["data"]["declaredName"],
            "\nfirst\r\nsecond"
        );
    }
    #[test]
    fn unchanged_textarea_submission_preserves_saved_nul_and_line_endings() {
        let original = "first\0second\r\nlast";
        let mut app = scalar_app(
            "LiteralString",
            json!({"elementId":"element","value":original}),
        );
        let html = app.render();
        assert!(!html.contains('\0'));
        assert!(html.contains("first\u{fffd}second&#13;\nlast</textarea>"));
        app.handle(&attribute_request(
            "value",
            "value",
            "first\u{fffd}second\nlast",
        ))
        .unwrap();
        assert_eq!(
            app.pending
                .as_ref()
                .unwrap()
                .document()
                .element("element")
                .unwrap()["data"]["value"],
            original
        );
        app.handle(&json!({"action":"commit"})).unwrap();
        assert_eq!(
            app.editor.document().element("element").unwrap()["data"]["value"],
            original
        );
        let exported = app.editor.document().to_json().unwrap();
        assert_eq!(
            Document::from_json(&exported).unwrap(),
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

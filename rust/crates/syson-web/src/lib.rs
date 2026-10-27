// SPDX-License-Identifier: EPL-2.0
//! Bounded browser authoring UI. Model semantics remain in syson-model.
#![deny(unsafe_op_in_unsafe_fn)]
mod bounded_render;
mod inspector_accessibility;
use bounded_render::Result as RenderResult;
pub mod layout_controller;
pub mod layout_geometry;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use syson_metamodel::ClassifierId;
use syson_model::{
    AttributeBlock, AttributeDescriptor, Document, Editor, ElementSummaryRef, NodeIds, ScalarType,
    Transaction,
};

const LIMIT: usize = 1_048_576;
const MODEL_BYTES: usize = 4 * LIMIT;
const PREVIEW_BYTES: usize = 16 * LIMIT;
const IMPORTED: &str = "Imported a new document. Previous document history was cleared.";
#[cfg(test)]
mod initialized_creation_tests;
const CREATION_ENTROPY_BYTES: usize = if cfg!(feature = "raw-factory-candidate") {
    320
} else {
    96
};

/// Browser platform glue supplies randomness; allocation and validation stay in Rust.
#[unsafe(no_mangle)]
pub extern "C" fn creation_entropy_bytes() -> u32 {
    CREATION_ENTROPY_BYTES as u32
}

#[cfg(feature = "raw-factory-candidate")]
fn initialized_kind(kind: &str) -> Option<ClassifierId> {
    match kind {
        "connection_definition" => Some(ClassifierId::ConnectionDefinition),
        "interface_definition" => Some(ClassifierId::InterfaceDefinition),
        "accept_action_usage" => Some(ClassifierId::AcceptActionUsage),
        "port_definition" => Some(ClassifierId::PortDefinition),
        _ => None,
    }
}

fn creation_kinds() -> String {
    let options = String::from(
        "<option value=\"package\">Package</option><option value=\"part_definition\">Part definition</option><option value=\"part_usage\">Part usage</option>",
    );
    #[cfg(feature = "raw-factory-candidate")]
    let options = options
        + "<option value=\"connection_definition\">Connection definition</option><option value=\"interface_definition\">Interface definition</option><option value=\"accept_action_usage\">Accept action usage</option><option value=\"port_definition\">Port definition</option>";
    options
}

fn has_diagram_preview(class: &str) -> bool {
    matches!(
        class,
        "Package" | "Namespace" | "PartDefinition" | "PartUsage"
    ) || (cfg!(feature = "raw-factory-candidate")
        && matches!(
            class,
            "ConnectionDefinition" | "InterfaceDefinition" | "AcceptActionUsage" | "PortDefinition"
        ))
}
struct App {
    editor: Editor,
    pending: Option<Transaction>,
    selected: String,
    active_property: String,
    message: String,
    error: bool,
    canvas: layout_controller::Canvas,
    accessibility: inspector_accessibility::InspectorAccessibility,
    discard_confirm: bool,
    recovery_focus: bool,
}
impl Default for App {
    fn default() -> Self {
        Self {
            editor: Editor::new(Document::empty()),
            pending: None,
            selected: String::new(),
            active_property: String::new(),
            message: "Create a package to begin. Changes are held in this browser tab.".into(),
            error: false,
            canvas: layout_controller::Canvas::default(),
            accessibility: inspector_accessibility::InspectorAccessibility::default(),
            discard_confirm: false,
            recovery_focus: false,
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
    if raw.len() != 96 && raw.len() != CREATION_ENTROPY_BYTES {
        return Err(format!(
            "Expected {CREATION_ENTROPY_BYTES} browser random bytes"
        ));
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
    (0..bytes.len() / 32)
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
    fn imported_candidate(&self, doc: Document) -> Result<Self, String> {
        let mut candidate = Self {
            editor: Editor::new(doc),
            canvas: self.canvas.preview_after_model_change()?,
            message: IMPORTED.into(),
            ..Self::default()
        };
        candidate.accessibility.finish("import", true, "", false);
        Ok(candidate)
    }
    fn preview_fits(&self, limit: usize) -> bool {
        self.try_render().is_ok_and(|html| html.len() <= limit)
    }
    fn handle(&mut self, request: &Value) -> Result<String, String> {
        self.accessibility.begin();
        self.recovery_focus = false;
        if !matches!(
            request["action"].as_str(),
            Some("discard_document" | "confirm_discard" | "cancel_discard")
        ) {
            self.discard_confirm = false;
        }
        if request["action"].as_str() == Some("attribute") && self.pending.is_none() {
            self.active_property = field(request, "attribute").into();
        }
        let result = self.handle_action(request);
        if result.is_ok() {
            let action = request["action"].as_str().unwrap_or("");
            if matches!(action, "select" | "import") {
                self.active_property.clear();
            }
            // Closed dispatch: these are the only actions that publish a new
            // committed document; staged/canvas/selection actions do not.
            if matches!(action, "commit" | "import" | "confirm_discard")
                || (matches!(action, "undo" | "redo")
                    && result.as_deref() == Ok("History applied."))
            {
                self.canvas.model_changed()?;
            } else if matches!(
                action,
                "select" | "stage" | "attribute" | "rename" | "cancel"
            ) {
                self.canvas.cancel()?;
            }
        }
        let doc = self
            .pending
            .as_ref()
            .map(Transaction::document)
            .unwrap_or_else(|| self.editor.document());
        self.accessibility.finish(
            request["action"].as_str().unwrap_or(""),
            result.is_ok(),
            field(request, "attribute"),
            can_rename(doc, &self.selected),
        );
        result
    }
    fn handle_action(&mut self, request: &Value) -> Result<String, String> {
        let action = request["action"].as_str().ok_or("Action missing")?;
        match action {
            "select" => {
                if self.pending.is_some() && field(request, "value") != self.selected {
                    return Err(
                        "Commit or cancel the staged change before selecting another element."
                            .into(),
                    );
                }
                self.selected = field(request, "value").into();
                Ok("Selection updated.".into())
            }
            "stage" => {
                if self.pending.is_some() {
                    return Err("Commit or cancel the current staged change first.".into());
                }
                #[cfg(feature = "raw-factory-candidate")]
                if let Some(classifier) = initialized_kind(field(request, "kind")) {
                    let identities = ids(request)?;
                    let name = field(request, "name");
                    let (tx, _) = syson_workflow::stage_initialized_creation(
                        self.editor.begin(), classifier, field(request, "parent"),
                        Some(name), identities, || false,
                    ).map_err(|_| "This element could not be created with the supplied parent and identities. No change was staged.".to_string())?;
                    self.pending = Some(tx);
                    return Ok("Change staged with its initialized children. Review the preview, then commit or cancel.".into());
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
                let candidate = self.imported_candidate(doc)?;
                // RR18a lifecycle admission only. This still renders fully;
                // RR18b must bound normal HTML allocation before construction.
                if !candidate.preview_fits(PREVIEW_BYTES) {
                    return Err("Import preview exceeds 16 MiB. Current document, history and canvas were retained.".into());
                }
                self.editor = candidate.editor;
                self.selected.clear();
                Ok(IMPORTED.into())
            }
            "recover_preview" => {
                self.canvas.cancel()?;
                if let Some(tx) = self.pending.take() {
                    tx.cancel();
                }
                self.selected.clear();
                self.active_property.clear();
                self.recovery_focus = true;
                Ok("Committed view recovered. Staged change was discarded; model history remains available.".into())
            }
            "discard_document" => {
                self.discard_confirm = true;
                Ok("Confirm discard in the recovery controls; this cannot be undone.".into())
            }
            "cancel_discard" => {
                self.discard_confirm = false;
                Ok("Discard cancelled. Committed data remains unchanged.".into())
            }
            "confirm_discard" => {
                if !self.discard_confirm {
                    return Err("Choose Discard current document first.".into());
                }
                self.canvas.preview_after_model_change()?;
                if let Some(tx) = self.pending.take() {
                    tx.cancel();
                }
                self.editor = Editor::new(Document::empty());
                self.selected.clear();
                self.active_property.clear();
                self.discard_confirm = false;
                self.recovery_focus = true;
                Ok("Document and history discarded. Create or import a new document.".into())
            }
            "layout" | "rotate" => {
                if self.pending.is_some() {
                    return Err(
                        "Commit or cancel the staged change before changing canvas geometry."
                            .into(),
                    );
                }
                let selected = if action == "rotate" {
                    vec![self.selected.clone()]
                } else {
                    vec![]
                };
                self.canvas.request(
                    self.editor.document(),
                    field(request, "labels"),
                    field(request, "revision"),
                    selected,
                    action == "rotate",
                )?;
                Ok("Canvas request prepared. Computation is queued.".into())
            }
            "canvas_compute" => {
                self.canvas.compute(field(request, "ticket"))?;
                Ok("Canvas computed. Publication is queued.".into())
            }
            "canvas_apply" => {
                let changed = self
                    .canvas
                    .apply(field(request, "ticket"), self.editor.document())?;
                Ok(if changed { "Canvas applied. After rotation, use Layout canvas to resize package containers." } else { "Stale canvas request discarded." }.into())
            }
            "canvas_cancel" => {
                self.canvas.cancel()?;
                Ok("Queued canvas work cancelled.".into())
            }
            "canvas_undo" | "canvas_redo" => {
                if self.pending.is_some() {
                    return Err("Commit or cancel the staged change first.".into());
                }
                let changed = self
                    .canvas
                    .history(action == "canvas_redo", self.editor.document())?;
                Ok(if changed {
                    "Canvas history applied."
                } else {
                    "No canvas history available."
                }
                .into())
            }
            "geometry_import" => {
                if self.pending.is_some() {
                    return Err("Commit or cancel the staged change first.".into());
                }
                self.canvas
                    .import(self.editor.document(), field(request, "json"))?;
                Ok("Canvas geometry imported for this committed document.".into())
            }
            _ => Err("Unsupported action.".into()),
        }
    }
    #[cfg(test)]
    fn render(&self) -> String {
        self.try_render().expect("finite renderer test fixture")
    }
    fn try_render(&self) -> RenderResult<String> {
        let doc = self
            .pending
            .as_ref()
            .map(Transaction::document)
            .unwrap_or_else(|| self.editor.document());
        // Every imported element remains selectable, including literal values and
        // relationship objects. Only the existing bounded kinds get SVG previews.
        let elements = doc.element_summaries();
        let tree = render_model_navigation(&elements, &self.selected)?;
        let mut parent_options =
            String::from("<option value=\"\">Document root (package only)</option>");
        let mut definitions = String::from("<option value=\"\">No explicit definition</option>");
        let mut diagram = String::new();
        let mut routes = String::new();
        let scene = if self.pending.is_none() {
            self.canvas.scene(self.editor.document())
        } else {
            None
        };
        let (mut left, mut top, mut right, mut bottom) = (0_f64, 0_f64, 420_f64, 250_f64);
        if let Some(scene) = scene {
            for edge in &scene.edges {
                let points = edge
                    .data
                    .as_ref()
                    .and_then(|d| d.bending_points.as_ref())
                    .unwrap();
                for p in points {
                    left = left.min(p.x - 24.);
                    top = top.min(p.y - 24.);
                    right = right.max(p.x + 24.);
                    bottom = bottom.max(p.y + 24.);
                }
                let route = points
                    .iter()
                    .map(|p| format!("{},{}", p.x, p.y))
                    .collect::<Vec<_>>()
                    .join(" ");
                routes.push_str(&format!("<polyline data-canvas-edge=\"{}\" points=\"{route}\" fill=\"none\" stroke=\"#465b6d\" stroke-width=\"2\"/>",escape(&edge.id)?));
            }
        }
        let mut y = 24.;
        for (index, element) in elements.iter().enumerate() {
            let id = escape(element.element_id)?;
            let name = escape(element.name.unwrap_or("(unnamed)"))?;
            let class = escape(element.class)?;
            if doc.can_create_in(element.element_id) {
                parent_options
                    .push_str(&format!("<option value=\"{id}\">{name} · {class}</option>"));
            }
            if doc.is_instance_of(element.element_id, ClassifierId::PartDefinition) {
                definitions.push_str(&format!("<option value=\"{id}\">{name}</option>"));
            }
            if !has_diagram_preview(element.class) {
                continue;
            }
            let node = scene.and_then(|s| s.nodes.iter().find(|n| n.id == element.element_id));
            let (x, node_y, width, height) = node
                .map(|n| {
                    (
                        n.position.x,
                        n.position.y,
                        n.width.unwrap(),
                        n.height.unwrap(),
                    )
                })
                .unwrap_or((24., y, 180., 70.));
            left = left.min(x - 24.);
            top = top.min(node_y - 24.);
            right = right.max(x + width + 24.);
            bottom = bottom.max(node_y + height + 24.);
            let stroke = if self.selected == element.element_id {
                "#1768b5"
            } else {
                "#8193a4"
            };
            // Clip labels to the padded node interior, including after rotation.
            // Preserve full strings in the title and the separate model list.
            let clip_width = (width - 24.).max(0.);
            let clip_height = (height - 16.).max(0.);
            let label_x = x + 12.;
            diagram.push_str(&format!("<g><title>{class}: {name}</title><rect data-canvas-node=\"{id}\" x=\"{x}\" y=\"{node_y}\" width=\"{width}\" height=\"{height}\" fill=\"white\" stroke=\"{stroke}\" stroke-width=\"2\"/><defs><clipPath id=\"preview-label-{index}\"><rect x=\"{label_x}\" y=\"{}\" width=\"{clip_width}\" height=\"{clip_height}\"/></clipPath></defs><g clip-path=\"url(#preview-label-{index})\"><text x=\"{label_x}\" y=\"{}\" font-size=\"12\" fill=\"#465b6d\">{class}</text><foreignObject x=\"{label_x}\" y=\"{}\" width=\"{clip_width}\" height=\"{clip_height}\"><div xmlns=\"http://www.w3.org/1999/xhtml\"><span data-canvas-label=\"{id}\" style=\"font:14px system-ui;line-height:24px;white-space:pre;display:inline-block\">{name}</span></div></foreignObject></g></g>",node_y+8.,node_y+16.,node_y+20.));
            y += height + 24.;
        }
        diagram.push_str(&routes);
        let selected = elements.iter().find(|e| e.element_id == self.selected);
        let selected_name = escape_textarea(selected.and_then(|e| e.name).unwrap_or(""))?;
        let scalar_inspector = if selected.is_some() {
            bounded_render::scalar_inspector(
                doc,
                &self.selected,
                self.accessibility
                    .focus()
                    .strip_prefix("scalar-")
                    .unwrap_or(&self.active_property),
                PREVIEW_BYTES,
            )?
        } else {
            String::new()
        };
        let inspector = if let Some(element) = selected {
            let rename_form = if can_rename(doc, &self.selected) {
                format!(
                    "<form data-action=\"rename\" aria-label=\"Rename selected element\"><label for=\"rename\">Element name</label><textarea id=\"rename\" name=\"name\" rows=\"2\" required>{selected_name}</textarea><div class=\"form-actions\"><button id=\"stage-rename\" type=\"submit\">Stage rename</button></div></form>"
                )
            } else {
                String::new()
            };
            let rotate_control = if has_diagram_preview(element.class) {
                "<p><button type=\"button\" data-action=\"rotate\">Rotate preview</button></p>"
            } else {
                ""
            };
            format!(
                "<h3>{}</h3><details id=\"element-identity\" class=\"identity-details\"><summary>Element identifier</summary><p class=\"element-id\">{}</p></details>{rename_form}{rotate_control}{scalar_inspector}",
                escape(element.class)?,
                escape(element.element_id)?
            )
        } else {
            "<p>Select an element in the model list.</p>".into()
        };
        let pending = if self.pending.is_some() {
            "<div class=\"form-actions\"><button id=\"commit-staged\" class=\"primary\" type=\"button\" data-action=\"commit\">Commit staged change</button><button id=\"cancel-staged\" type=\"button\" data-action=\"cancel\">Cancel staged change</button></div>"
        } else {
            ""
        };
        let json = escape(
            &self
                .editor
                .document()
                .to_json()
                .unwrap_or_else(|e| e.to_string()),
        )?;
        let kinds = creation_kinds();
        if scene.is_none() {
            bottom = bottom.max(y);
        }
        let view = format!("{left} {top} {} {}", right - left, bottom - top);
        let phase = self
            .canvas
            .phase()
            .map(|(ticket, phase)| {
                format!("data-canvas-ticket=\"{ticket}\" data-canvas-phase=\"{phase}\"")
            })
            .unwrap_or_default();
        let revision = self.canvas.revision();
        let geometry = escape_textarea(&self.canvas.export(self.editor.document()))?;
        let focus = escape(if self.recovery_focus {
            "name"
        } else {
            self.accessibility.focus()
        })?;
        Ok(format!(
            "<header><h1>SysON model editor</h1><nav class=\"toolbar\" aria-label=\"Model history\"><button id=\"model-undo\" type=\"button\" data-action=\"undo\">Undo</button><button id=\"model-redo\" type=\"button\" data-action=\"redo\">Redo</button></nav></header><p id=\"editor-status\" data-editor-status class=\"status {}\" role=\"{}\" aria-live=\"{}\" aria-atomic=\"true\">{}</p><div class=\"layout\" data-inspector-focus=\"{focus}\"><section class=\"panel\" aria-label=\"Model elements\"><h2>Model</h2>{tree}</section><section class=\"canvas\" data-canvas-revision=\"{revision}\" {phase}><h2>Diagram preview</h2><p><small>Arrange packages and parts with Layout canvas. Select an element in the model to edit its properties.</small></p><p><button type=\"button\" data-action=\"layout\">Layout canvas</button> <button type=\"button\" data-action=\"canvas_cancel\">Cancel canvas work</button></p><p><button type=\"button\" data-action=\"canvas_undo\">Canvas undo</button> <button type=\"button\" data-action=\"canvas_redo\">Canvas redo</button></p><div class=\"canvas-view-controls\" role=\"group\" aria-label=\"Diagram view\"><button id=\"canvas-actual-size\" type=\"button\" data-canvas-view=\"actual\" aria-pressed=\"true\">Actual size</button><button id=\"canvas-fit\" type=\"button\" data-canvas-view=\"fit\" aria-pressed=\"false\">Fit overview</button></div><p id=\"canvas-view-help\" class=\"canvas-view-help\">Actual size keeps labels readable. Focus the diagram and use arrow keys or scroll to move around. Fit overview shows the whole diagram.</p><div id=\"canvas-viewport\" class=\"canvas-viewport\" data-canvas-viewport data-canvas-view=\"actual\" tabindex=\"0\" role=\"region\" aria-label=\"Diagram viewport\" aria-describedby=\"canvas-view-help\"><svg viewBox=\"{view}\" preserveAspectRatio=\"xMidYMid meet\" aria-hidden=\"true\">{diagram}</svg></div></section><section class=\"panel inspector\" aria-label=\"Element inspector\"><h2 id=\"create-heading\">Create element</h2><form data-action=\"stage\" aria-labelledby=\"create-heading\"><label for=\"kind\">Element kind</label><select id=\"kind\" name=\"kind\">{kinds}</select><label for=\"parent\">Parent</label><select id=\"parent\" name=\"parent\">{parent_options}</select><label for=\"name\">Name</label><input id=\"name\" name=\"name\" required><label for=\"definition\">Definition for part usage</label><select id=\"definition\" name=\"definition\">{definitions}</select><div class=\"form-actions\"><button id=\"stage-create\" class=\"primary\" type=\"submit\">Stage creation</button></div></form>{pending}<hr><h2 id=\"selection-heading\" tabindex=\"-1\">Selection</h2>{inspector}</section></div><section class=\"file-panel\"><h2>Document JSON</h2><p>Copy the committed EMF JSON below to export. Import opens a new document and clears the current history.</p><label for=\"export-json\">Committed document export</label><textarea id=\"export-json\" readonly spellcheck=\"false\">{json}</textarea><details><summary>Import another document</summary><form data-action=\"import\"><label for=\"import-json\">EMF JSON document</label><textarea id=\"import-json\" name=\"json\" required spellcheck=\"false\"></textarea><div class=\"form-actions\"><button type=\"submit\">Import as new document and clear history</button></div></form></details><label for=\"export-geometry\">Canvas geometry export (separate from model)</label><textarea id=\"export-geometry\" readonly spellcheck=\"false\">{geometry}</textarea><details id=\"geometry-details\"><summary>Import canvas geometry</summary><form data-action=\"geometry_import\"><label for=\"import-geometry\">Canvas companion JSON for this exact document</label><textarea id=\"import-geometry\" name=\"json\" required spellcheck=\"false\"></textarea><button type=\"submit\">Import canvas geometry</button></form></details></section>",
            if self.error { "error" } else { "" },
            if self.error { "alert" } else { "status" },
            if self.error { "assertive" } else { "polite" },
            escape(&self.message)?
        ))
    }
}
fn navigation_button(element: &ElementSummaryRef<'_>, selected: &str) -> RenderResult<String> {
    let id = escape(element.element_id)?;
    let name = escape(element.name.unwrap_or("(unnamed)"))?;
    let class = escape(element.class)?;
    Ok(format!(
        "<button id=\"select-{id}\" type=\"button\" data-action=\"select\" data-value=\"{id}\" aria-pressed=\"{}\">{name}<span class=\"kind\">{class}</span></button>",
        selected == element.element_id
    ))
}
/// Nested native lists expose semantic ownership without pretending to implement
/// ARIA tree arrow-key behavior. Relationship objects remain available separately.
fn render_model_navigation(
    elements: &[ElementSummaryRef<'_>],
    selected: &str,
) -> RenderResult<String> {
    let by_id: BTreeMap<_, _> = elements.iter().map(|e| (e.element_id, e)).collect();
    let relationships: BTreeSet<_> = elements
        .iter()
        .filter(|e| {
            syson_metamodel::lookup(syson_metamodel::Package::Domain, e.class)
                .is_some_and(|c| syson_metamodel::is_subtype(c, ClassifierId::Relationship))
        })
        .map(|e| e.element_id)
        .collect();
    let mut children: BTreeMap<Option<&str>, Vec<&ElementSummaryRef<'_>>> = BTreeMap::new();
    for element in elements
        .iter()
        .filter(|e| !relationships.contains(e.element_id))
    {
        let mut owner = element.owner;
        let mut seen = BTreeSet::new();
        while let Some(id) = owner.filter(|id| relationships.contains(id)) {
            if !seen.insert(id) {
                owner = None;
                break;
            }
            owner = by_id.get(id).and_then(|e| e.owner);
        }
        children
            .entry(owner.filter(|id| by_id.contains_key(id)))
            .or_default()
            .push(element);
    }
    let mut html = String::from("<ul class=\"tree\" aria-label=\"Model hierarchy\">");
    let mut stack: Vec<_> = children
        .get(&None)
        .into_iter()
        .flatten()
        .rev()
        .map(|element| (Some(*element), 0))
        .collect();
    if stack.is_empty() {
        html.push_str(if relationships.is_empty() {
            "<li><p>No model elements yet.</p></li>"
        } else {
            "<li><p>This document contains relationship objects only.</p></li>"
        });
    }
    while let Some((element, depth)) = stack.pop() {
        let Some(element) = element else {
            html.push_str("</ul></li>");
            continue;
        };
        html.push_str(&format!(
            "<li style=\"--model-depth:{depth}\">{}",
            navigation_button(element, selected)?
        ));
        if let Some(nested) = children.get(&Some(element.element_id)) {
            html.push_str("<ul class=\"tree-children\">");
            stack.push((None, 0));
            stack.extend(nested.iter().rev().map(|e| (Some(*e), depth + 1)));
        } else {
            html.push_str("</li>");
        }
    }
    html.push_str("</ul>");
    if !relationships.is_empty() {
        let open = if relationships.contains(selected) {
            " open"
        } else {
            ""
        };
        html.push_str(&format!("<details id=\"model-relationships\"{open}><summary>Relationships ({})</summary><p class=\"navigation-help\">Inspect membership and typing links here.</p><ul class=\"tree\" aria-label=\"Relationship objects\">", relationships.len()));
        for element in elements
            .iter()
            .filter(|e| relationships.contains(e.element_id))
        {
            html.push_str(&format!(
                "<li>{}</li>",
                navigation_button(element, selected)?
            ));
        }
        html.push_str("</ul></details>");
    }
    Ok(html)
}
fn common_property(name: &str) -> bool {
    matches!(
        name,
        "declaredShortName" | "isAbstract" | "isComposite" | "direction" | "body" | "value"
    )
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
fn escape_textarea(input: &str) -> RenderResult<String> {
    bounded_render::escaped(input, true, PREVIEW_BYTES)
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
fn blocked_label(reason: &AttributeBlock) -> &'static str {
    match reason {
        AttributeBlock::Derived => "Calculated value; read only.",
        AttributeBlock::NotChangeable => "Read only.",
        AttributeBlock::ManyValued => "List editing is unavailable.",
        AttributeBlock::IdentityManaged => "Identifier; read only.",
        AttributeBlock::UnsupportedType(_) => "This value type cannot be edited here.",
    }
}
fn escape(input: &str) -> RenderResult<String> {
    bounded_render::escaped(input, false, PREVIEW_BYTES)
}
#[cfg(test)]
fn render_scalar_field(attribute: &AttributeDescriptor) -> String {
    bounded_render::scalar_field(attribute, PREVIEW_BYTES).expect("finite scalar test fixture")
}

thread_local! {static STATE:RefCell<App>=RefCell::new(App::default());static INPUT:RefCell<Vec<u8>>=const{RefCell::new(Vec::new())};static OUTPUT:RefCell<String>=const{RefCell::new(String::new())};}
// A native download link needs no loader/model substitute. Count canonical model
// bytes first, then percent-encode directly into one exactly reserved HTML buffer.
fn recovery_controls(app: &App) -> String {
    let confirm = if app.discard_confirm {
        "<p>Discard this document and all history? This cannot be undone.</p><button id=\"confirm-discard\" type=\"button\" data-action=\"confirm_discard\">Confirm discard document</button><button type=\"button\" data-action=\"cancel_discard\">Keep document</button>"
    } else {
        "<button type=\"button\" data-action=\"discard_document\">Discard current document</button>"
    };
    let focus = if app.discard_confirm {
        "confirm-discard"
    } else {
        "recover-preview"
    };
    format!(
        "<p>Export excludes staged changes. Recovering discards the staged change and selection.</p><section data-inspector-focus=\"{focus}\"><button id=\"recover-preview\" type=\"button\" data-action=\"recover_preview\">Recover committed view (discard staged change)</button><button type=\"button\" data-action=\"undo\">Undo last committed edit</button>{confirm}</section>"
    )
}
fn recovery_unavailable(app: &App) -> String {
    format!(
        "<h1>SysON model editor</h1><p id=\"editor-status\" data-editor-status class=\"error\" role=\"alert\" aria-live=\"assertive\" aria-atomic=\"true\">Preview/export unavailable. Recover or explicitly discard this document.</p>{}",
        recovery_controls(app)
    )
}
fn recovery_html(app: &App) -> Result<String, String> {
    use std::io::Write;
    let prefix = "<h1>SysON model editor</h1><p id=\"editor-status\" data-editor-status class=\"error\" role=\"alert\" aria-live=\"assertive\" aria-atomic=\"true\">Preview exceeds the 16 MiB limit. Committed model data remains in this tab.</p><a id=\"recovery-export\" style=\"display:inline-block;min-height:44px;padding:12px;box-sizing:border-box\" download=\"model.json\" href=\"data:application/json;charset=utf-8,";
    let suffix = format!(
        "\">Download committed model JSON</a>{}",
        recovery_controls(app)
    );
    let doc = app.editor.document();
    let size = doc
        .json_size(MODEL_BYTES, || false)
        .map_err(|e| e.to_string())?;
    let capacity = size
        .checked_mul(3)
        .and_then(|n| n.checked_add(prefix.len() + suffix.len()))
        .filter(|n| *n <= PREVIEW_BYTES)
        .ok_or("Recovery export exceeds preview limit")?;
    let mut output = String::new();
    output
        .try_reserve_exact(capacity)
        .map_err(|_| "Recovery export allocation refused")?;
    output.push_str(prefix);
    struct Encoded<'a> {
        output: &'a mut String,
        remaining: usize,
    }
    impl Write for Encoded<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.remaining {
                return Err(std::io::Error::other("Recovery export size changed"));
            }
            self.remaining -= bytes.len();
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            for byte in bytes {
                self.output.push('%');
                self.output.push(HEX[(byte >> 4) as usize] as char);
                self.output.push(HEX[(byte & 15) as usize] as char);
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    doc.write_json(
        &mut Encoded {
            output: &mut output,
            remaining: size,
        },
        MODEL_BYTES,
        || false,
    )
    .map_err(|e| e.to_string())?;
    output.push_str(&suffix);
    Ok(output)
}
fn paint() {
    STATE.with(|s| {
        OUTPUT.with(|o| {
            let app = s.borrow();
            *o.borrow_mut() = match app.try_render() {
                Ok(html) if html.len() <= PREVIEW_BYTES => html,
                Ok(html) => {
                    drop(html);
                    recovery_html(&app).unwrap_or_else(|_| recovery_unavailable(&app))
                }
                Err(_) => recovery_html(&app).unwrap_or_else(|_| recovery_unavailable(&app)),
            };
        })
    });
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
    fn closed_action_dispatch_keeps_canvas_for_staging_and_invalidates_only_publication() {
        fn layout(app: &mut App) {
            let labels = Value::Array(
                app.editor
                    .document()
                    .elements()
                    .iter()
                    .filter(|e| e.class == "Package")
                    .map(|e| json!({"id":e.element_id,"width":80.,"height":24.}))
                    .collect(),
            )
            .to_string();
            let ticket = app
                .canvas
                .request(
                    app.editor.document(),
                    &labels,
                    &app.canvas.revision(),
                    vec![],
                    false,
                )
                .unwrap();
            app.canvas.compute(&ticket).unwrap();
            app.canvas.apply(&ticket, app.editor.document()).unwrap();
        }
        let mut app = App::default();
        app.handle(&create(0, "Vehicle")).unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        layout(&mut app);
        let id = app.editor.document().elements()[0].element_id.clone();
        let committed = app.editor.document().to_json().unwrap();
        let geometry = app.canvas.export(app.editor.document());
        for action in [
            json!({"action":"select","fields":{"value":id}}),
            json!({"action":"rename","fields":{"name":"Draft"}}),
            json!({"action":"cancel"}),
            json!({"action":"undo-invalid"}),
            json!({"action":"import","fields":{"json":"invalid"}}),
        ] {
            let _ = app.handle(&action);
            assert_eq!(app.editor.document().to_json().unwrap(), committed);
            assert_eq!(app.canvas.export(app.editor.document()), geometry);
        }
        app.handle(&json!({"action":"rename","fields":{"name":"Changed"}}))
            .unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        assert!(app.canvas.export(app.editor.document()).is_empty());
        layout(&mut app);
        app.handle(&json!({"action":"undo"})).unwrap();
        assert_eq!(app.editor.document().to_json().unwrap(), committed);
        assert!(app.canvas.scene(app.editor.document()).is_none());
        layout(&mut app);
        app.handle(&json!({"action":"redo"})).unwrap();
        assert!(app.canvas.scene(app.editor.document()).is_none());
        layout(&mut app);
        let geometry = app.canvas.export(app.editor.document());
        assert_eq!(
            app.handle(&json!({"action":"redo"})).unwrap(),
            "No history available."
        );
        assert_eq!(app.canvas.export(app.editor.document()), geometry);
        let input = app.editor.document().to_json().unwrap();
        app.handle(&json!({"action":"import","fields":{"json":input}}))
            .unwrap();
        assert!(app.canvas.scene(app.editor.document()).is_none());
        layout(&mut app);
        app.handle(&json!({"action":"discard_document"})).unwrap();
        app.handle(&json!({"action":"confirm_discard"})).unwrap();
        assert!(app.canvas.scene(app.editor.document()).is_none());
        assert!(app.editor.document().elements().is_empty());
    }
    #[test]
    fn recovery_uri_retains_model_bytes_unicode_cr_lf_and_literal_numbers() {
        let empty = Document::empty().to_json().unwrap();
        let raw = r#"{"opaque":{"n":-0,"text":"雪\n\r&<","number":{"$serde_json::private::Number":"123"}},"#.to_owned() + empty.strip_prefix('{').unwrap();
        let app = App {
            editor: Editor::new(Document::from_json(&raw).unwrap()),
            ..App::default()
        };
        let expected = app.editor.document().to_json().unwrap();
        let html = recovery_html(&app).unwrap();
        assert!(html.len() <= PREVIEW_BYTES);
        let uri = html
            .split("charset=utf-8,")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert_eq!(uri.len() % 3, 0);
        let bytes: Vec<u8> = uri
            .as_bytes()
            .as_chunks::<3>()
            .0
            .iter()
            .map(|chunk| {
                assert_eq!(chunk[0], b'%');
                u8::from_str_radix(std::str::from_utf8(&chunk[1..]).unwrap(), 16).unwrap()
            })
            .collect();
        assert_eq!(bytes, expected.as_bytes());
        assert!(expected.contains("\"n\": -0"));
        assert!(expected.contains("\\n\\r"));
    }
    #[test]
    fn import_candidate_matches_final_presentation_and_inclusive_admission() {
        let mut app = App::default();
        app.canvas.cancel().unwrap();
        let candidate = app.imported_candidate(Document::empty()).unwrap();
        let expected = candidate.render();
        assert!(candidate.preview_fits(expected.len()));
        assert!(!candidate.preview_fits(expected.len() - 1));
        app.message = app
            .handle(
                &json!({"action":"import","fields":{"json":Document::empty().to_json().unwrap()}}),
            )
            .unwrap();
        assert_eq!(app.render(), expected);
    }
    #[test]
    fn exhausted_canvas_refuses_import_and_discard_before_editor_or_history_changes() {
        let mut app = App::default();
        app.handle(&create(0, "Before")).unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        app.selected = app.editor.document().elements()[0].element_id.clone();
        let selected = app.selected.clone();
        let before = app.editor.document().clone();
        app.canvas = layout_controller::Canvas::with_epoch_for_test(u64::MAX);
        assert!(
            app.handle(
                &json!({"action":"import","fields":{"json":Document::empty().to_json().unwrap()}})
            )
            .is_err()
        );
        assert_eq!(app.editor.document(), &before);
        assert_eq!(app.selected, selected);
        assert_eq!(app.canvas.revision(), u64::MAX.to_string());
        app.discard_confirm = true;
        assert!(app.handle(&json!({"action":"confirm_discard"})).is_err());
        assert_eq!(app.editor.document(), &before);
        assert!(app.editor.undo());
        assert_ne!(app.editor.document(), &before);
    }
    #[test]
    fn export_error_recovery_keeps_discard_confirmation_and_cancel_reachable() {
        let mut app = App {
            discard_confirm: true,
            ..App::default()
        };
        let html = recovery_unavailable(&app);
        assert!(html.contains("data-action=\"confirm_discard\""));
        assert!(html.contains("data-action=\"cancel_discard\""));
        app.handle(&json!({"action":"cancel_discard"})).unwrap();
        assert!(!recovery_unavailable(&app).contains("data-action=\"confirm_discard\""));
        assert!(app.handle(&json!({"action":"confirm_discard"})).is_err());
    }
    #[test]
    fn inspector_focus_follows_real_staging_without_changing_committed_data() {
        let mut app = App::default();
        app.handle(&create(0, "Vehicle")).unwrap();
        assert!(
            app.render()
                .contains("data-inspector-focus=\"commit-staged\"")
        );
        assert_eq!(app.editor.document().elements().len(), 0);
        app.handle(&json!({"action":"commit"})).unwrap();
        assert!(app.render().contains("data-inspector-focus=\"name\""));
        let id = app.editor.document().elements()[0].element_id.clone();
        app.handle(&json!({"action":"select","fields":{"value":id}}))
            .unwrap();
        assert!(app.render().contains("data-inspector-focus=\"rename\""));
        let before = app.editor.document().to_json().unwrap();
        app.handle(&json!({"action":"rename","fields":{"name":"draft"}}))
            .unwrap();
        assert!(app.render().contains("id=\"commit-staged\""));
        assert_eq!(app.editor.document().to_json().unwrap(), before);
        app.handle(&json!({"action":"cancel"})).unwrap();
        let html = app.render();
        assert!(html.contains("data-inspector-focus=\"rename\""));
        assert!(html.contains("aria-atomic=\"true\""));
        assert!(html.contains("aria-live=\"polite\""));
        assert_eq!(app.editor.document().to_json().unwrap(), before);
        app.handle(&json!({"action":"canvas_cancel"})).unwrap();
        assert!(app.render().contains("data-inspector-focus=\"\""));
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
        assert_eq!(escape("<&\"'>").unwrap(), "&lt;&amp;&quot;&#39;&gt;");
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
        let labels = Value::Array(
            app.editor
                .document()
                .elements()
                .iter()
                .filter(|e| matches!(e.class.as_str(), "Package" | "PartDefinition" | "PartUsage"))
                .map(|e| json!({"id":e.element_id,"width":80.,"height":24.}))
                .collect(),
        )
        .to_string();
        app.handle(
            &json!({"action":"layout","fields":{"labels":labels,"revision":app.canvas.revision()}}),
        )
        .unwrap();
        for action in ["canvas_compute", "canvas_apply"] {
            let ticket = app.canvas.phase().unwrap().0.to_string();
            app.handle(&json!({"action":action,"fields":{"ticket":ticket}}))
                .unwrap();
        }
        app.handle(&json!({"action":"rotate","fields":{"revision":app.canvas.revision()}}))
            .unwrap();
        for action in ["canvas_compute", "canvas_apply"] {
            let ticket = app.canvas.phase().unwrap().0.to_string();
            app.handle(&json!({"action":action,"fields":{"ticket":ticket}}))
                .unwrap();
        }
        let html = app.render();
        assert!(html.contains("width=\"70\" height=\"180\""));
        assert!(html.contains("width=\"46\" height=\"164\"/>"));
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
    #[test]
    fn model_navigation_nests_semantic_children_and_keeps_relationships_selectable() {
        let mut app = App::default();
        app.handle(&create(0, "Vehicle")).unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        let package = app.editor.document().elements()[0].element_id.clone();
        let mut usage = create(1, "wheel");
        usage["fields"]["kind"] = json!("part_usage");
        usage["fields"]["parent"] = json!(package);
        app.handle(&usage).unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        let elements = app.editor.document().element_summaries();
        let relationship = elements
            .iter()
            .find(|e| e.class == "OwningMembership")
            .unwrap();
        let child = elements.iter().find(|e| e.class == "PartUsage").unwrap();
        let html = render_model_navigation(&elements, "").unwrap();
        let (hierarchy, advanced) = html
            .split_once("<details id=\"model-relationships\"")
            .unwrap();
        assert!(hierarchy.contains("<ul class=\"tree-children\">"));
        assert!(hierarchy.contains(&format!("data-value=\"{}\"", child.element_id)));
        assert!(!hierarchy.contains(&format!("data-value=\"{}\"", relationship.element_id)));
        assert!(advanced.starts_with("><summary>Relationships (1)</summary>"));
        for element in &elements {
            assert_eq!(
                html.matches(&format!("id=\"select-{}\"", element.element_id))
                    .count(),
                1
            );
        }
        app.handle(&json!({"action":"select","fields":{"value":relationship.element_id}}))
            .unwrap();
        assert!(app.render().contains("id=\"model-relationships\" open"));
        let before = app.editor.document().clone();
        app.handle(&json!({"action":"canvas_cancel"})).unwrap();
        assert!(app.render().contains("id=\"model-relationships\" open"));
        assert_eq!(app.editor.document(), &before);
    }
    #[test]
    fn advanced_property_disclosure_stays_open_through_stage_commit_and_cancel() {
        let mut app = scalar_app(
            "PartUsage",
            json!({"elementId":"element","declaredName":"wheel","isConstant":false}),
        );
        let original = app.editor.document().clone();
        let html = app.render();
        let advanced = html.find("id=\"advanced-properties\"").unwrap();
        assert!(!html.contains("id=\"advanced-properties\" open"));
        for name in [
            "declaredShortName",
            "isAbstract",
            "isComposite",
            "direction",
        ] {
            assert!(html.find(&format!("id=\"scalar-{name}\"")).unwrap() < advanced);
        }
        assert!(html.find("id=\"scalar-isConstant\"").unwrap() > advanced);
        app.handle(&attribute_request("isConstant", "value", "true"))
            .unwrap();
        assert!(app.render().contains("id=\"advanced-properties\" open"));
        assert_eq!(app.editor.document(), &original);
        assert!(
            app.handle(&attribute_request("isAbstract", "value", "true"))
                .is_err()
        );
        assert!(app.render().contains("id=\"advanced-properties\" open"));
        // A blocked action must not hide the origin needed for cancel focus.
        assert!(
            app.handle(&json!({"action":"import","fields":{"json":"{}"}}))
                .is_err()
        );
        app.handle(&json!({"action":"cancel"})).unwrap();
        let html = app.render();
        assert!(html.contains("id=\"advanced-properties\" open"));
        assert!(html.contains("data-inspector-focus=\"scalar-isConstant\""));
        assert_eq!(app.editor.document(), &original);
        app.handle(&attribute_request("isConstant", "value", "true"))
            .unwrap();
        app.handle(&json!({"action":"select","fields":{"value":"element"}}))
            .unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        assert!(app.render().contains("id=\"advanced-properties\" open"));
        assert_eq!(
            app.editor.document().element("element").unwrap()["data"]["isConstant"],
            true
        );
        app.handle(&json!({"action":"select","fields":{"value":"element"}}))
            .unwrap();
        assert!(!app.render().contains("id=\"advanced-properties\" open"));
    }
    #[test]
    fn pending_property_edit_keeps_selection_and_commit_focus_on_its_original_element() {
        let doc = Document::from_json(&json!({
            "json":{"version":"1.0","encoding":"utf-8"},
            "ns":{"sysml":"http://www.eclipse.org/syson/sysml"},
            "content":[
                {"id":"wire-a","eClass":"sysml:PartUsage","data":{"elementId":"a","declaredName":"A","isAbstract":false}},
                {"id":"wire-b","eClass":"sysml:PartUsage","data":{"elementId":"b","declaredName":"B","isAbstract":false}}
            ]
        }).to_string()).unwrap();
        let mut app = App {
            editor: Editor::new(doc),
            selected: "a".into(),
            ..Default::default()
        };
        let original = app.editor.document().clone();
        app.handle(&attribute_request("isAbstract", "value", "true"))
            .unwrap();
        assert_eq!(
            app.handle(&json!({"action":"select","fields":{"value":"b"}})),
            Err("Commit or cancel the staged change before selecting another element.".into())
        );
        assert_eq!(app.selected, "a");
        assert_eq!(app.editor.document(), &original);
        assert_eq!(
            app.pending
                .as_ref()
                .unwrap()
                .document()
                .element("b")
                .unwrap()["data"]["isAbstract"],
            false
        );
        app.handle(&json!({"action":"commit"})).unwrap();
        assert_eq!(
            app.editor.document().element("a").unwrap()["data"]["isAbstract"],
            true
        );
        assert_eq!(
            app.editor.document().element("b").unwrap()["data"]["isAbstract"],
            false
        );
        assert!(
            app.render()
                .contains("data-inspector-focus=\"scalar-isAbstract\"")
        );
        app.handle(&json!({"action":"select","fields":{"value":"b"}}))
            .unwrap();
        assert_eq!(app.selected, "b");
    }
}

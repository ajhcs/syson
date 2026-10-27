// SPDX-License-Identifier: EPL-2.0
//! Cooperative ticket/history/geometry publication; no worker thread.
use super::layout_geometry::{Payload, Scene, computed, fail, identity, measured, number};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use syson_editor::{Edge, EdgeData, Node};
use syson_editor::{Point, finish_rotation, prepare_rotation};
use syson_model::Document;
const BYTE_LIMIT: usize = 1_048_576;
#[derive(Clone)]
struct Frame {
    model: String,
    scene: Scene,
}
#[derive(Clone)]
struct Pending {
    ticket: String,
    frame: Frame,
    selected: Vec<String>,
    result: Option<Scene>,
}
#[derive(Default)]
pub struct Canvas {
    epoch: u64,
    current: Option<Frame>,
    pending: Option<Pending>,
    undo: Vec<Option<Frame>>,
    redo: Vec<Option<Frame>>,
}
impl Canvas {
    fn bump(&mut self) -> Result<(), String> {
        self.epoch = self
            .epoch
            .checked_add(1)
            .ok_or("Canvas request identity exhausted.")?;
        Ok(())
    }
    pub fn revision(&self) -> String {
        self.epoch.to_string()
    }
    pub fn phase(&self) -> Option<(&str, &str)> {
        self.pending.as_ref().map(|p| {
            (
                p.ticket.as_str(),
                if p.result.is_some() {
                    "apply"
                } else {
                    "compute"
                },
            )
        })
    }
    pub fn scene(&self, doc: &Document) -> Option<&Scene> {
        let model = doc.to_json().ok()?;
        self.current
            .as_ref()
            .filter(|f| f.model == model)
            .map(|f| &f.scene)
    }
    pub fn cancel(&mut self) -> Result<(), String> {
        self.bump()?;
        self.pending = None;
        Ok(())
    }
    pub fn model_changed(&mut self) -> Result<(), String> {
        self.cancel()?;
        self.current = None;
        self.undo.clear();
        self.redo.clear();
        Ok(())
    }
    pub fn request(
        &mut self,
        doc: &Document,
        labels: &str,
        revision: &str,
        selected: Vec<String>,
        rotation: bool,
    ) -> Result<String, String> {
        if selected.len() > 32 {
            return Err(fail());
        }
        if revision != self.revision() {
            return Err("The document or selection changed before label measurement completed. Request layout again.".into());
        }
        let model = doc.to_json().map_err(|e| e.to_string())?;
        if model.len() > BYTE_LIMIT {
            return Err("This canvas request exceeds its document or selection limit.".into());
        }
        let scene = if rotation {
            let frame = self
                .current
                .as_ref()
                .filter(|f| f.model == model)
                .ok_or("Layout the canvas before rotating.")?;
            if selected.is_empty()
                || selected
                    .iter()
                    .any(|id| !frame.scene.nodes.iter().any(|n| &n.id == id))
            {
                return Err("Select a supported canvas node.".into());
            }
            if selected.iter().any(|id| {
                frame
                    .scene
                    .nodes
                    .iter()
                    .any(|n| n.payload.parent.as_deref() == Some(id))
            }) {
                return Err(
                    "Rotate canvas leaves only; use Layout canvas to resize package containers."
                        .into(),
                );
            }
            prepare_rotation(&frame.scene, &selected)
        } else {
            measured(
                doc,
                labels,
                self.epoch,
                self.current
                    .as_ref()
                    .filter(|f| f.model == model)
                    .map(|f| &f.scene),
            )?
        };
        self.bump()?;
        let ticket = self.revision();
        self.pending = Some(Pending {
            ticket: ticket.clone(),
            frame: Frame { model, scene },
            selected: if rotation { selected } else { vec![] },
            result: None,
        });
        Ok(ticket)
    }
    pub fn compute(&mut self, ticket: &str) -> Result<bool, String> {
        let Some(p) = self
            .pending
            .as_ref()
            .filter(|p| p.ticket == ticket && p.result.is_none())
            .cloned()
        else {
            return Ok(false);
        };
        let result = match computed(&p.frame.scene, &p.selected) {
            Ok(result) => result,
            Err(e) => {
                self.pending = None;
                return Err(e);
            }
        };
        self.pending.as_mut().unwrap().result = Some(result);
        Ok(true)
    }
    fn publish(&mut self, frame: Frame) {
        self.undo.push(self.current.take());
        if self.undo.len() > 32 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.current = Some(frame);
    }
    pub fn apply(&mut self, ticket: &str, doc: &Document) -> Result<bool, String> {
        let Some(p) = self
            .pending
            .as_ref()
            .filter(|p| p.ticket == ticket)
            .cloned()
        else {
            return Ok(false);
        };
        if p.frame.model != doc.to_json().map_err(|e| e.to_string())? {
            self.cancel()?;
            return Ok(false);
        }
        let Some(result) = p.result else {
            return Err("Canvas computation has not completed.".into());
        };
        let scene = if p.selected.is_empty() {
            result
        } else {
            finish_rotation(&p.frame.scene, &result, &p.selected)
        };
        let frame = Frame {
            model: p.frame.model,
            scene,
        };
        if let Err(error) = admit(&frame) {
            self.cancel()?;
            return Err(error);
        }
        self.cancel()?;
        self.publish(frame);
        Ok(true)
    }
    pub fn history(&mut self, redo: bool, doc: &Document) -> Result<bool, String> {
        let model = doc.to_json().map_err(|e| e.to_string())?;
        let next = if redo {
            self.redo.last()
        } else {
            self.undo.last()
        };
        if self.current.as_ref().is_some_and(|f| f.model != model)
            || next
                .and_then(|f| f.as_ref())
                .is_some_and(|f| f.model != model)
        {
            return Err("Canvas history belongs to a different committed document.".into());
        }
        self.cancel()?;
        let (source, target) = if redo {
            (&mut self.redo, &mut self.undo)
        } else {
            (&mut self.undo, &mut self.redo)
        };
        let Some(frame) = source.pop() else {
            return Ok(false);
        };
        target.push(self.current.take());
        self.current = frame;
        Ok(true)
    }
    pub fn export(&self, doc: &Document) -> String {
        let Ok(model) = doc.to_json() else {
            return String::new();
        };
        self.current
            .as_ref()
            .filter(|f| f.model == model)
            .map(encode)
            .unwrap_or_default()
    }
    pub fn import(&mut self, doc: &Document, input: &str) -> Result<(), String> {
        if input.len() > BYTE_LIMIT {
            return Err(fail());
        }
        let raw: Value = serde_json::from_str(input).map_err(|_| "Canvas JSON is invalid.")?;
        fields(&raw, &["version", "model", "nodes", "edges"])?;
        let model = doc.to_json().map_err(|e| e.to_string())?;
        if raw["version"].as_u64() != Some(1) || raw["model"].as_str() != Some(&model) {
            return Err("Canvas JSON belongs to a different committed document or version.".into());
        }
        let elements = doc.elements();
        if elements.iter().any(|e| {
            !matches!(
                e.class.as_str(),
                "Package"
                    | "PartDefinition"
                    | "PartUsage"
                    | "OwningMembership"
                    | "FeatureMembership"
                    | "FeatureTyping"
            )
        }) {
            return Err(fail());
        }
        let expected: Vec<_> = elements
            .iter()
            .filter(|e| matches!(e.class.as_str(), "Package" | "PartDefinition" | "PartUsage"))
            .collect();
        let rows = raw["nodes"]
            .as_array()
            .filter(|r| r.len() <= 32 && r.len() == expected.len())
            .ok_or_else(fail)?;
        let mut nodes = Vec::new();
        let mut seen = BTreeSet::new();
        for row in rows {
            fields(
                row,
                &[
                    "id", "parent", "header", "x", "y", "width", "height", "resized",
                ],
            )?;
            let id = identity(&row["id"])?;
            if !seen.insert(id) {
                return Err(fail());
            }
            let semantic = expected
                .iter()
                .find(|e| e.element_id == id)
                .ok_or_else(fail)?;
            if semantic.owner.as_ref().is_some_and(|p| {
                !expected
                    .iter()
                    .any(|e| e.element_id == *p && e.class == "Package")
            }) {
                return Err(fail());
            }
            if row["parent"].as_str() != semantic.owner.as_deref()
                || (semantic.owner.is_none() && !row["parent"].is_null())
            {
                return Err(fail());
            }
            let (x, y, w, h, header) = (
                number(&row["x"])?,
                number(&row["y"])?,
                number(&row["width"])?,
                number(&row["height"])?,
                number(&row["header"])?,
            );
            if [x, y, w, h, header]
                .iter()
                .any(|v| *v < 0. || v.fract() != 0.)
                || w <= 0.
                || h <= 0.
                || header <= 20.
                || x + w > 1_000_000.
                || y + h > 1_000_000.
            {
                return Err(fail());
            }
            nodes.push(Node {
                id: id.into(),
                position: Point { x, y },
                width: Some(w),
                height: Some(h),
                resized_by_user: row["resized"].as_bool().ok_or_else(fail)?,
                connection_handles: vec![],
                payload: Payload {
                    parent: semantic.owner.clone(),
                    header,
                },
            });
        }
        let expected_edges: Vec<_> = elements
            .iter()
            .filter(|e| e.class == "FeatureTyping")
            .collect();
        let rows = raw["edges"]
            .as_array()
            .filter(|r| r.len() <= 64 && r.len() == expected_edges.len())
            .ok_or_else(fail)?;
        let mut edges = Vec::new();
        seen.clear();
        for row in rows {
            fields(row, &["id", "source", "target", "points"])?;
            let id = identity(&row["id"])?;
            if !seen.insert(id) || !expected_edges.iter().any(|e| e.element_id == id) {
                return Err(fail());
            }
            let data = &doc.element(id).ok_or_else(fail)?["data"];
            let (source, target) = (identity(&data["typedFeature"])?, identity(&data["type"])?);
            if row["source"].as_str() != Some(source) || row["target"].as_str() != Some(target) {
                return Err(fail());
            }
            let a = nodes.iter().find(|n| n.id == source).ok_or_else(fail)?;
            let b = nodes.iter().find(|n| n.id == target).ok_or_else(fail)?;
            if !expected
                .iter()
                .any(|e| e.element_id == source && e.class == "PartUsage")
                || !expected
                    .iter()
                    .any(|e| e.element_id == target && e.class == "PartDefinition")
                || a.payload.parent != b.payload.parent
            {
                return Err(fail());
            }
            let points = row["points"]
                .as_array()
                .filter(|p| p.len() >= 2 && p.len() <= 4096)
                .ok_or_else(fail)?;
            let mut route = Vec::new();
            for p in points {
                let p = p.as_array().filter(|p| p.len() == 2).ok_or_else(fail)?;
                let (x, y) = (number(&p[0])?, number(&p[1])?);
                if (x * 2.).fract() != 0. || (y * 2.).fract() != 0. {
                    return Err(fail());
                }
                route.push(Point { x, y });
            }
            for pair in route.windows(2) {
                if pair[0] == pair[1] || (pair[0].x != pair[1].x && pair[0].y != pair[1].y) {
                    return Err(fail());
                }
            }
            let first = Point {
                x: a.position.x + a.width.unwrap(),
                y: a.position.y + a.height.unwrap() / 2.,
            };
            let last = Point {
                x: b.position.x,
                y: b.position.y + b.height.unwrap() / 2.,
            };
            if first == last || route.first() != Some(&first) || route.last() != Some(&last) {
                return Err(fail());
            }
            edges.push(Edge {
                id: id.into(),
                source: source.into(),
                target: target.into(),
                data: Some(EdgeData {
                    bending_points: Some(route),
                    payload: (),
                }),
            });
        }
        let frame = Frame {
            model,
            scene: Scene { nodes, edges },
        };
        admit(&frame)?;
        self.cancel()?;
        self.publish(frame);
        Ok(())
    }
}
fn encode(f: &Frame) -> String {
    json!({"version":1,"model":f.model,
            "nodes":f.scene.nodes.iter().map(|n| json!({"id":n.id,"parent":n.payload.parent,"header":n.payload.header,"x":n.position.x,"y":n.position.y,"width":n.width,"height":n.height,"resized":n.resized_by_user})).collect::<Vec<_>>(),
            "edges":f.scene.edges.iter().map(|e| json!({"id":e.id,"source":e.source,"target":e.target,"points":e.data.as_ref().and_then(|d| d.bending_points.as_ref()).map(|p|p.iter().map(|p|json!([p.x,p.y])).collect::<Vec<_>>())})).collect::<Vec<_>>() }).to_string()
}
fn admit(frame: &Frame) -> Result<(), String> {
    let encoded = encode(frame);
    // The loader submits this file as a JSON string inside the default 1MiB ABI.
    if encoded.len() > BYTE_LIMIT
        || serde_json::to_string(&encoded).map_err(|_| fail())?.len() + 2048 > BYTE_LIMIT
    {
        return Err("Canvas JSON exceeds the default file/ABI byte budget.".into());
    }
    Ok(())
}
fn fields(value: &Value, expected: &[&str]) -> Result<(), String> {
    let object = value.as_object().ok_or_else(fail)?;
    if object.len() != expected.len() || expected.iter().any(|k| !object.contains_key(*k)) {
        return Err(fail());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use syson_model::{Editor, NodeIds};
    fn ids(n: u32) -> NodeIds {
        NodeIds::new(
            &format!("00000000-0000-4000-8000-{n:012x}"),
            &format!("10000000-0000-4000-8000-{n:012x}"),
        )
        .unwrap()
    }
    fn document() -> Document {
        let mut editor = Editor::new(Document::empty());
        let mut tx = editor.begin();
        tx.create_package(None, ids(1), None, "Vehicle").unwrap();
        tx.create_part_definition(&ids(1).element, ids(2), ids(3), "Wheel")
            .unwrap();
        tx.create_part_usage(&ids(1).element, ids(4), ids(5), "wheel")
            .unwrap();
        tx.type_part_usage(&ids(4).element, &ids(2).element, ids(6))
            .unwrap();
        editor.commit(tx).unwrap();
        editor.document().clone()
    }
    fn labels(doc: &Document) -> String {
        Value::Array(
            doc.elements()
                .iter()
                .filter(|e| matches!(e.class.as_str(), "Package" | "PartDefinition" | "PartUsage"))
                .map(|e| json!({"id":e.element_id,"width":80.25,"height":24.25}))
                .collect(),
        )
        .to_string()
    }
    fn request(c: &mut Canvas, d: &Document, selected: Vec<String>, rotate: bool) -> String {
        let revision = c.revision();
        c.request(d, &labels(d), &revision, selected, rotate)
            .unwrap()
    }
    fn layout(c: &mut Canvas, d: &Document) {
        let ticket = request(c, d, vec![], false);
        assert_eq!(c.phase(), Some((ticket.as_str(), "compute")));
        assert!(c.compute(&ticket).unwrap());
        assert_eq!(c.phase(), Some((ticket.as_str(), "apply")));
        assert!(c.apply(&ticket, d).unwrap());
        assert!(c.phase().is_none());
    }
    #[test]
    fn publication_is_separate_from_real_computation_and_preserves_model_bytes() {
        let doc = document();
        let before = doc.to_json().unwrap();
        let mut c = Canvas::default();
        let ticket = request(&mut c, &doc, vec![], false);
        assert!(c.scene(&doc).is_none());
        assert!(c.apply(&ticket, &doc).is_err());
        assert!(c.compute(&ticket).unwrap());
        assert!(c.scene(&doc).is_none());
        assert!(!c.compute(&ticket).unwrap());
        assert!(c.apply(&ticket, &doc).unwrap());
        assert_eq!(c.scene(&doc).unwrap().edges.len(), 1);
        assert!(!c.apply(&ticket, &doc).unwrap());
        assert_eq!(doc.to_json().unwrap(), before);
    }
    #[test]
    fn superseded_results_and_cancelled_compute_or_apply_never_publish() {
        let doc = document();
        let mut c = Canvas::default();
        let old = request(&mut c, &doc, vec![], false);
        let new = request(&mut c, &doc, vec![], false);
        assert_ne!(old, new);
        assert!(!c.compute(&old).unwrap());
        assert!(!c.apply(&old, &doc).unwrap());
        c.cancel().unwrap();
        assert!(!c.compute(&new).unwrap());
        assert!(c.scene(&doc).is_none());
        let computed = request(&mut c, &doc, vec![], false);
        assert!(c.compute(&computed).unwrap());
        c.cancel().unwrap();
        assert!(!c.apply(&computed, &doc).unwrap());
        assert!(c.scene(&doc).is_none());
    }
    #[test]
    fn selection_and_model_invalidation_reject_stale_observations_and_results() {
        let doc = document();
        let mut c = Canvas::default();
        layout(&mut c, &doc);
        let saved = c.export(&doc);
        let revision = c.revision();
        let old = request(&mut c, &doc, vec![], false);
        c.cancel().unwrap();
        assert!(
            c.request(&doc, &labels(&doc), &revision, vec![], false)
                .is_err()
        );
        assert!(!c.compute(&old).unwrap());
        assert_eq!(c.export(&doc), saved);
        let old = request(&mut c, &doc, vec![], false);
        c.compute(&old).unwrap();
        c.model_changed().unwrap();
        assert!(!c.apply(&old, &doc).unwrap());
        assert!(c.export(&doc).is_empty());
        assert!(!c.history(false, &doc).unwrap());
    }
    #[test]
    fn leaf_rotation_keeps_unselected_nodes_and_first_selection_semantics() {
        let doc = document();
        let mut c = Canvas::default();
        layout(&mut c, &doc);
        let original = c.scene(&doc).unwrap().clone();
        let selected = vec![ids(4).element, ids(2).element];
        let ticket = request(&mut c, &doc, selected.clone(), true);
        c.compute(&ticket).unwrap();
        c.apply(&ticket, &doc).unwrap();
        let result = c.scene(&doc).unwrap();
        let usage = result.nodes.iter().find(|n| n.id == selected[0]).unwrap();
        let definition = result.nodes.iter().find(|n| n.id == selected[1]).unwrap();
        assert_eq!((usage.width, usage.height), (Some(70.), Some(180.)));
        assert_eq!(
            (definition.width, definition.height),
            (Some(180.), Some(70.))
        );
        assert_eq!(
            result.nodes.iter().find(|n| n.id == ids(1).element),
            original.nodes.iter().find(|n| n.id == ids(1).element)
        );
        assert_eq!(result.edges[0].id, original.edges[0].id);
        assert_ne!(result.edges[0].data, original.edges[0].data);
        assert!(c.history(false, &doc).unwrap());
        assert_eq!(c.scene(&doc), Some(&original));
        assert!(c.history(true, &doc).unwrap());
        assert_eq!(
            c.scene(&doc)
                .unwrap()
                .nodes
                .iter()
                .find(|n| n.id == selected[0])
                .unwrap()
                .height,
            Some(180.)
        );
    }
    #[test]
    fn canvas_history_and_geometry_json_roundtrip_are_model_bound() {
        let doc = document();
        let mut c = Canvas::default();
        layout(&mut c, &doc);
        let saved = c.export(&doc);
        assert!(c.history(false, &doc).unwrap());
        assert!(c.scene(&doc).is_none());
        assert!(c.history(true, &doc).unwrap());
        assert_eq!(c.export(&doc), saved);
        let mut fresh = Canvas::default();
        fresh.import(&doc, &saved).unwrap();
        assert_eq!(fresh.export(&doc), saved);
        assert!(fresh.history(false, &doc).unwrap());
        assert!(fresh.scene(&doc).is_none());
        assert!(fresh.history(true, &doc).unwrap());
        assert_eq!(fresh.export(&doc), saved);
    }
    #[test]
    fn foreign_document_cannot_read_restore_or_publish_another_model_geometry() {
        let doc = document();
        let mut c = Canvas::default();
        layout(&mut c, &doc);
        let saved = c.export(&doc);
        let old = request(&mut c, &doc, vec![], false);
        c.compute(&old).unwrap();
        let mut editor = Editor::new(doc.clone());
        let mut tx = editor.begin();
        tx.rename(&ids(1).element, "Changed").unwrap();
        editor.commit(tx).unwrap();
        let changed = editor.document();
        assert!(!c.apply(&old, changed).unwrap());
        assert!(c.scene(changed).is_none());
        assert!(c.export(changed).is_empty());
        assert!(c.import(changed, &saved).is_err());
        assert!(c.history(false, changed).is_err());
        assert_eq!(c.export(&doc), saved);
    }
    #[test]
    fn invalid_requests_preserve_existing_scene_and_pending_ticket() {
        let doc = document();
        let mut c = Canvas::default();
        layout(&mut c, &doc);
        let saved = c.export(&doc);
        let pending = request(&mut c, &doc, vec![], false);
        let revision = c.revision();
        assert!(c.request(&doc, "[]", &revision, vec![], false).is_err());
        assert!(
            c.request(&doc, &labels(&doc), &revision, vec![ids(1).element], true)
                .is_err()
        );
        assert!(
            c.request(&doc, &labels(&doc), &revision, vec!["missing".into()], true)
                .is_err()
        );
        assert!(
            c.request(
                &doc,
                &labels(&doc),
                &revision,
                vec![ids(4).element; 33],
                true
            )
            .is_err()
        );
        assert_eq!(c.phase(), Some((pending.as_str(), "compute")));
        assert_eq!(c.export(&doc), saved);
    }
    #[test]
    fn malformed_geometry_import_is_atomic_for_scene_history_and_pending() {
        let doc = document();
        let mut c = Canvas::default();
        layout(&mut c, &doc);
        let saved = c.export(&doc);
        let original: Value = serde_json::from_str(&saved).unwrap();
        let pending = request(&mut c, &doc, vec![], false);
        let history = c.undo.len();
        let mut variants = Vec::new();
        let mut v = original.clone();
        v["version"] = json!(2);
        variants.push(v);
        let mut v = original.clone();
        v["extra"] = json!(true);
        variants.push(v);
        let mut v = original.clone();
        v["nodes"][0]["parent"] = json!(false);
        variants.push(v);
        let mut v = original.clone();
        v["nodes"][1] = v["nodes"][0].clone();
        variants.push(v);
        let mut v = original.clone();
        v["nodes"][0]["width"] = json!(0);
        variants.push(v);
        let mut v = original.clone();
        v["nodes"][0]["header"] = json!(20.5);
        variants.push(v);
        let mut v = original.clone();
        v["nodes"][0]["x"] = json!(0.25);
        variants.push(v);
        let mut v = original.clone();
        v["edges"][0]["target"] = json!("missing");
        variants.push(v);
        let mut v = original.clone();
        v["edges"][0]["points"][0][0] = json!(0.25);
        variants.push(v);
        let mut v = original.clone();
        v["edges"][0]["points"] = json!([]);
        variants.push(v);
        for v in variants {
            assert!(c.import(&doc, &v.to_string()).is_err());
            assert_eq!(c.export(&doc), saved);
            assert_eq!(c.undo.len(), history);
            assert_eq!(c.phase(), Some((pending.as_str(), "compute")));
        }
        for input in ["{bad", "null"] {
            assert!(c.import(&doc, input).is_err());
        }
        assert!(c.import(&doc, &"x".repeat(1_048_577)).is_err());
    }
    #[test]
    fn escaped_model_cannot_publish_an_unimportable_companion_file() {
        let small = document();
        let mut editor = Editor::new(small.clone());
        let mut tx = editor.begin();
        tx.rename(&ids(1).element, &"\\".repeat(150_000)).unwrap();
        editor.commit(tx).unwrap();
        let large = editor.document();
        assert!(large.to_json().unwrap().len() < BYTE_LIMIT);
        let mut c = Canvas::default();
        layout(&mut c, &small);
        let saved = c.export(&small);
        assert!(serde_json::to_string(&saved).unwrap().len() + 2048 < BYTE_LIMIT);
        let history = c.undo.len();
        let ticket = request(&mut c, large, vec![], false);
        assert!(c.compute(&ticket).unwrap());
        let pending = c.pending.as_ref().unwrap();
        let candidate = Frame {
            model: pending.frame.model.clone(),
            scene: pending.result.clone().unwrap(),
        };
        let file = encode(&candidate);
        assert!(file.len() < BYTE_LIMIT);
        assert!(serde_json::to_string(&file).unwrap().len() + 2048 > BYTE_LIMIT);
        assert!(c.apply(&ticket, large).is_err());
        assert!(c.phase().is_none());
        assert_eq!(c.export(&small), saved);
        assert_eq!(c.undo.len(), history);
        let ticket = request(&mut c, &small, vec![], false);
        assert!(c.import(large, &file).is_err());
        assert_eq!(c.phase(), Some((ticket.as_str(), "compute")));
        assert_eq!(c.export(&small), saved);
        assert_eq!(c.undo.len(), history);
        let mut imported = Canvas::default();
        imported.import(&small, &saved).unwrap();
        assert_eq!(imported.export(&small), saved);
    }
    #[test]
    fn bounded_history_does_not_reuse_request_identities() {
        let doc = document();
        let mut c = Canvas::default();
        let mut tickets = BTreeSet::new();
        for _ in 0..40 {
            let ticket = request(&mut c, &doc, vec![], false);
            assert!(tickets.insert(ticket.clone()));
            c.compute(&ticket).unwrap();
            c.apply(&ticket, &doc).unwrap();
        }
        assert_eq!(c.undo.len(), 32);
        for _ in 0..32 {
            assert!(c.history(false, &doc).unwrap());
        }
        assert!(!c.history(false, &doc).unwrap());
        for _ in 0..32 {
            assert!(c.history(true, &doc).unwrap());
        }
        assert!(!c.history(true, &doc).unwrap());
    }
    #[test]
    fn exhausted_identity_counter_fails_without_mutating_existing_state() {
        let doc = document();
        let mut c = Canvas::default();
        layout(&mut c, &doc);
        let saved = c.export(&doc);
        c.epoch = u64::MAX;
        let revision = c.revision();
        assert!(
            c.request(&doc, &labels(&doc), &revision, vec![], false)
                .is_err()
        );
        assert!(c.cancel().is_err());
        assert!(c.history(false, &doc).is_err());
        assert_eq!(c.export(&doc), saved);
    }
}

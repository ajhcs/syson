// SPDX-License-Identifier: EPL-2.0
use super::*;
use serde_json::json;

fn request(kind: &str, parent: &str, name: &str, seed: u8) -> Value {
    json!({"action":"stage", "fields":{"kind":kind,"parent":parent,"name":name},
        "entropy": (0..CREATION_ENTROPY_BYTES).map(|i| ((i % 32) as u8).wrapping_add(seed).wrapping_add((i / 32) as u8)).collect::<Vec<_>>()})
}

fn app_with_package() -> (App, String) {
    let mut app = App::default();
    app.handle(&request("package", "", "System", 0)).unwrap();
    app.handle(&json!({"action":"commit"})).unwrap();
    let parent = app.editor.document().elements()[0].element_id.clone();
    (app, parent)
}

#[test]
fn creation_options_and_entropy_match_the_compiled_capability() {
    let options = creation_kinds();
    assert!(options.contains("part_usage"));
    assert_eq!(
        options.contains("connection_definition"),
        cfg!(feature = "raw-factory-candidate")
    );
    assert_eq!(
        creation_entropy_bytes(),
        if cfg!(feature = "raw-factory-candidate") {
            320
        } else {
            96
        }
    );
}

#[cfg(not(feature = "raw-factory-candidate"))]
#[test]
fn candidate_creation_is_refused_when_feature_is_disabled() {
    let (mut app, parent) = app_with_package();
    let before = app.editor.document().clone();
    assert!(
        app.handle(&request("port_definition", &parent, "IO", 40))
            .is_err()
    );
    assert!(app.pending.is_none());
    assert_eq!(app.editor.document(), &before);
}

#[cfg(feature = "raw-factory-candidate")]
#[test]
fn all_four_browser_actions_stage_complete_graphs_cancel_commit_history_and_reload() {
    for (kind, class, children) in [
        ("connection_definition", "ConnectionDefinition", 2),
        ("interface_definition", "InterfaceDefinition", 4),
        ("accept_action_usage", "AcceptActionUsage", 2),
        ("port_definition", "PortDefinition", 1),
    ] {
        let (mut app, parent) = app_with_package();
        let before = app.editor.document().clone();
        let creation = request(kind, &parent, "New element", 40);
        app.handle(&creation).unwrap();
        assert_eq!(app.editor.document(), &before);
        let staged = app.pending.as_ref().unwrap().document();
        let root = staged
            .elements()
            .into_iter()
            .find(|e| e.class == class)
            .unwrap();
        assert_eq!(root.name.as_deref(), Some("New element"));
        assert_eq!(
            staged.element(&root.element_id).unwrap()["data"]["ownedRelationship"]
                .as_array()
                .unwrap()
                .len(),
            children
        );
        assert!(app.render().contains("New element"));
        app.handle(&json!({"action":"cancel"})).unwrap();
        assert_eq!(app.editor.document(), &before);
        app.handle(&creation).unwrap();
        app.handle(&json!({"action":"commit"})).unwrap();
        let after = app.editor.document().clone();
        app.handle(&json!({"action":"undo"})).unwrap();
        assert_eq!(app.editor.document(), &before);
        app.handle(&json!({"action":"redo"})).unwrap();
        assert_eq!(app.editor.document(), &after);
        let exported = after.to_json().unwrap();
        app.handle(&json!({"action":"import","fields":{"json":exported}}))
            .unwrap();
        assert_eq!(app.editor.document(), &after);
        assert!(!app.editor.undo());
        assert!(app.render().contains(&format!("{class}</span>")));
        assert!(has_diagram_preview(class));
    }
}

#[cfg(feature = "raw-factory-candidate")]
#[test]
fn failed_creation_and_staging_guards_preserve_document_and_redo() {
    let (mut app, parent) = app_with_package();
    app.handle(&request("package", "", "Other", 1)).unwrap();
    app.handle(&json!({"action":"commit"})).unwrap();
    app.handle(&json!({"action":"undo"})).unwrap();
    let before = app.editor.document().clone();
    for (parent, name, entropy) in [
        ("", "Missing parent", vec![0_u8; 320]),
        (parent.as_str(), "", vec![0_u8; 320]),
        (parent.as_str(), "Duplicate identities", vec![0_u8; 320]),
        (
            parent.as_str(),
            "Exhausted identities",
            (0..96).collect::<Vec<u8>>(),
        ),
    ] {
        let mut creation = request("connection_definition", parent, name, 40);
        creation["entropy"] = json!(entropy);
        assert!(app.handle(&creation).is_err());
        assert!(app.pending.is_none());
        assert_eq!(app.editor.document(), &before);
    }
    app.handle(&json!({"action":"redo"})).unwrap();
    assert!(
        app.editor
            .document()
            .elements()
            .iter()
            .any(|e| e.name.as_deref() == Some("Other"))
    );
    app.handle(&request("port_definition", &parent, "IO", 40))
        .unwrap();
    let staged = app.pending.as_ref().unwrap().document().clone();
    assert!(
        app.handle(&request("interface_definition", &parent, "Blocked", 50))
            .is_err()
    );
    assert!(
        app.handle(&json!({"action":"import","fields":{"json":before.to_json().unwrap()}}))
            .is_err()
    );
    assert!(app.handle(&json!({"action":"undo"})).is_err());
    assert_eq!(app.pending.as_ref().unwrap().document(), &staged);
}

// SPDX-License-Identifier: EPL-2.0
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use syson_domain::{identity_context::*, uuid::Uuid};
use syson_model::{Document, Editor, NodeIds};
use syson_text_import::{Error, Importer};
struct Fresh(u32);
impl FreshIdProvider for Fresh {
    type Error = ();
    fn next_v4(&mut self) -> Result<Uuid, ()> {
        self.0 += 1;
        let mut bytes = [0; 16];
        bytes[6] = 0x40;
        bytes[8] = 0x80;
        bytes[12..].copy_from_slice(&self.0.to_be_bytes());
        Ok(Uuid(bytes))
    }
}
#[test]
fn generated_roles_survive_real_import_history_and_refused_publication() {
    let ordinary = ElementContext {
        class: ElementClass::Other,
        resource: ResourceObservation::Unresolved,
        library_namespace: LibraryNamespaceObservation::Absent,
        declared_name: Observed::Unresolved,
        qualified_name: Observed::Unresolved,
    };
    let mut provider = Fresh(0);
    let mut ids = Vec::new();
    for n in 1..=6 {
        let generated = generate_semantic_id(Some(ordinary), &mut provider).unwrap();
        let resource = format!("10000000-0000-4000-8000-{n:012x}");
        ids.push(NodeIds::new(&resource, &generated.id.0.to_string()).unwrap());
    }
    // This reservation is caller-private planning, not an Editor-owned allocator.
    let reserved = SelectedIdentityRegistry::restore(ids.iter().map(|pair| {
        SelectedIdentity::from_fields(Some(&pair.resource), Some(&pair.element)).unwrap()
    }))
    .unwrap();
    assert_eq!((provider.0, reserved.len()), (6, 6));
    let importer = Importer::pinned().unwrap();
    let source = "package P { part def T; part u : T; }";
    let cancel = Arc::new(AtomicBool::new(false));
    let mut editor = Editor::new(Document::empty());
    let stage =
        |editor: &Editor, ids| importer.stage(editor.begin(), source, None, ids, cancel.clone());
    let staged = stage(&editor, ids.clone()).unwrap();
    assert_eq!(editor.document(), &Document::empty());
    assert!(staged.commit(&mut editor).unwrap());
    let committed = editor.document().clone();
    for pair in &ids {
        assert_ne!(pair.resource, pair.element);
        assert_eq!(
            committed.element(&pair.element).unwrap()["id"],
            pair.resource
        );
    }
    let usage = "00000000-0000-4000-8000-000000000004";
    let target = "00000000-0000-4000-8000-000000000002";
    let typing = &committed.element(usage).unwrap()["data"]["ownedRelationship"][0]["data"];
    assert_eq!(typing["type"], target);
    let reopened = Document::from_json(&committed.to_json().unwrap()).unwrap();
    assert_eq!(reopened, committed);
    assert!(editor.undo());
    let history = editor.history_state();
    let failed = stage(&editor, vec![]);
    assert!(matches!(failed, Err(Error::IdentitiesExhausted)));
    let staged = stage(&editor, ids).unwrap();
    cancel.store(true, Ordering::Relaxed);
    assert_eq!(staged.commit(&mut editor), Err(Error::Cancelled));
    assert_eq!(editor.document(), &Document::empty());
    assert_eq!(editor.history_state(), history);
    assert_eq!(reserved.len(), 6);
    assert!(editor.redo());
    assert_eq!(editor.document(), &committed);
}

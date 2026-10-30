// SPDX-License-Identifier: EPL-2.0
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use syson_domain::{identity_context::*, uuid::Uuid};
use syson_model::{Document, Editor, NodeIds};
use syson_workflow::SnapshotStore;

fn fresh(n: u8) -> Uuid {
    let mut bytes = [0; 16];
    bytes[6] = 0x40;
    bytes[8] = 0x80;
    bytes[15] = n;
    Uuid(bytes)
}
struct Provider(u8);
impl FreshIdProvider for Provider {
    type Error = &'static str;
    fn next_v4(&mut self) -> Result<Uuid, Self::Error> {
        self.0 += 1;
        Ok(fresh(self.0))
    }
}
fn ordinary() -> ElementContext<'static> {
    ElementContext {
        class: ElementClass::Other,
        resource: ResourceObservation::Unresolved,
        library_namespace: LibraryNamespaceObservation::Absent,
        declared_name: Observed::Unresolved,
        qualified_name: Observed::Unresolved,
    }
}
fn output() -> PathBuf {
    let root = PathBuf::from(
        std::env::var_os("SYSON_TEST_DIR")
            .expect("SYSON_TEST_DIR must name an existing workspace directory"),
    );
    assert!(root.is_absolute() && root.is_dir());
    let root = root.canonicalize().unwrap();
    assert!(root.starts_with("/workspace"));
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let own = root.join(format!("f45-identities-{}-{nonce}", std::process::id()));
    fs::create_dir(&own).unwrap();
    own
}
fn fields(document: &Document) -> Vec<(String, String)> {
    document
        .elements()
        .into_iter()
        .map(|e| (e.resource_id, e.element_id))
        .collect()
}
fn selected(fields: &[(String, String)]) -> Vec<SelectedIdentity<'_>> {
    fields
        .iter()
        .map(|(resource, semantic)| {
            SelectedIdentity::from_fields(Some(resource), Some(semantic)).unwrap()
        })
        .collect()
}
#[test]
fn actual_selected_import_cancel_collisions_and_file_reload_preserve_identity_roles() {
    let fixture = include_str!(
        "../../../../../rust/crates/syson-model/tests/fixtures/simple-project.emf.json"
    );
    let imported = Document::from_json(fixture).unwrap();
    let initial_fields = fields(&imported);
    assert_eq!(initial_fields.len(), 11);
    assert_eq!(
        initial_fields[0],
        (
            "686dee81-b94c-4393-b15b-65dd73dfdd24".into(),
            "bea38691-2169-4b70-87ac-ffa289824fde".into()
        )
    );
    let own = output();
    let store = SnapshotStore::open(&own).unwrap();
    let original_path = store.save_new("imported", &imported).unwrap();
    let original_bytes = fs::read(&original_path).unwrap();
    let mut editor = Editor::new(imported);
    let mut provider = Provider(0);
    let generated = generate_semantic_id(Some(ordinary()), &mut provider).unwrap();
    let resource = fresh(77).to_string();
    let semantic = generated.id.0.to_string();
    assert_ne!(resource, semantic);
    let mut tx = editor.begin();
    tx.create_package(
        None,
        NodeIds::new(&resource, &semantic).unwrap(),
        None,
        "Generated",
    )
    .unwrap();
    editor.commit(tx).unwrap();
    let committed = editor.document().clone();
    let expected = fields(&committed);
    assert_eq!(expected.len(), 12);
    assert!(expected.contains(&(resource.clone(), semantic.clone())));
    let mut registry = SelectedIdentityRegistry::restore(selected(&expected)).unwrap();
    let saved = store.save_new("committed", &committed).unwrap();
    let saved_bytes = fs::read(&saved).unwrap();
    let cancelled = generate_semantic_id(Some(ordinary()), &mut provider).unwrap();
    let mut tx = editor.begin();
    tx.create_package(
        None,
        NodeIds::new(&fresh(78).to_string(), &cancelled.id.0.to_string()).unwrap(),
        None,
        "Cancelled",
    )
    .unwrap();
    tx.cancel();
    assert_eq!(editor.document(), &committed);
    let mut semantic_collision = editor.begin();
    assert!(
        semantic_collision
            .create_package(
                None,
                NodeIds::new(&fresh(79).to_string(), &semantic).unwrap(),
                None,
                "DuplicateSemantic"
            )
            .is_err()
    );
    assert_eq!(editor.document(), &committed);
    let other = generate_semantic_id(Some(ordinary()), &mut provider).unwrap();
    let mut resource_collision = editor.begin();
    assert!(
        resource_collision
            .create_package(
                None,
                NodeIds::new(&resource, &other.id.0.to_string()).unwrap(),
                None,
                "DuplicateResource"
            )
            .is_err()
    );
    assert_eq!(editor.document(), &committed);
    assert_eq!(provider.0, 3);
    let extra_resource = fresh(79).to_string();
    let rejected = SelectedIdentity::from_fields(Some(&extra_resource), Some(&semantic)).unwrap();
    assert_eq!(
        registry.admit(rejected),
        Err(Collision::Semantic(generated.id))
    );
    assert!(!registry.contains_resource(ResourceId(fresh(79))));
    assert_eq!(fs::read(&saved).unwrap(), saved_bytes);
    assert_eq!(fs::read(&original_path).unwrap(), original_bytes);
    let reloaded = SnapshotStore::open(&own)
        .unwrap()
        .load("committed")
        .unwrap();
    assert_eq!(reloaded, committed);
    assert_eq!(fields(&reloaded), expected);
    let recovered_fields = fields(&reloaded);
    let recovered = SelectedIdentityRegistry::restore(selected(&recovered_fields)).unwrap();
    assert_eq!(recovered.len(), 12);
    assert!(recovered.contains_resource(ResourceId(fresh(77))));
    assert!(recovered.contains_semantic(generated.id));
    let original = SnapshotStore::open(&own).unwrap().load("imported").unwrap();
    assert_eq!(fields(&original), initial_fields);
}

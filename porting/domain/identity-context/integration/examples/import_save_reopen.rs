// SPDX-License-Identifier: EPL-2.0
//! Finite text import, typed edit and immutable snapshot process boundary.
use std::{fs::File, path::PathBuf, sync::{Arc, atomic::AtomicBool}, time::Instant};
use syson_identity_context_integration::{Result, capped, require, workspace_root};
use syson_domain::{identity_context::*, uuid::Uuid};
use syson_model::{Document, Editor, NodeIds};
use syson_text_import::Importer;
use syson_workflow::SnapshotStore;

const SOURCE: &str = "package Imported { part def A; part def B; part a : A; part b : B; }";
const LIMIT: u64 = 4 * 1024 * 1024;

fn pair(n: u8) -> NodeIds {
    NodeIds::new(
        &format!("10000000-0000-4000-8000-{n:012x}"),
        &format!("20000000-0000-4000-8000-{n:012x}"),
    )
    .expect("fixed role identifiers")
}
struct Fresh(u8);
impl FreshIdProvider for Fresh {
    type Error = ();
    fn next_v4(&mut self) -> std::result::Result<Uuid, ()> {
        self.0 += 1;
        let mut bytes = [0; 16];
        bytes[0] = 0x20;
        bytes[6] = 0x40;
        bytes[8] = 0x80;
        bytes[15] = self.0;
        Ok(Uuid(bytes))
    }
}
fn verify(document: &Document) -> Result<usize> {
    let elements = document.elements();
    require(elements.len() == 12, "expected twelve identities")?;
    let selected = elements.iter().map(|e| {
        SelectedIdentity::from_fields(Some(&e.resource_id), Some(&e.element_id))
    }).collect::<std::result::Result<Vec<_>, _>>().map_err(|_| "invalid selected fields")?;
    let registry = SelectedIdentityRegistry::restore(selected).map_err(|_| "registry collision")?;
    require(registry.len() == 12, "registry cardinality drift")?;
    let names = ["Imported", "A", "", "B", "", "EditedA", "", "b", "", "", "", "Generated"];
    for (index, name) in names.iter().enumerate() {
        let ids = pair((index + 1) as u8);
        let node = document.element(&ids.element).ok_or("missing semantic role")?;
        require(node["id"] == ids.resource, "resource role drift")?;
        if !name.is_empty() {
            require(node["data"]["declaredName"] == *name, "edited name drift")?;
        }
        let identity = SelectedIdentity::from_fields(Some(&ids.resource), Some(&ids.element))
            .map_err(|_| "fixed selected identity")?;
        require(registry.contains_resource(identity.resource()), "resource reservation missing")?;
        require(registry.contains_semantic(identity.semantic()), "semantic reservation missing")?;
    }
    for (typing, usage, target) in [(10, 6, 2), (11, 8, 4)] {
        let node = document.element(&pair(typing).element).ok_or("missing actual typing")?;
        require(node["eClass"] == "sysml:FeatureTyping", "typing class drift")?;
        require(node["data"]["type"] == pair(target).element, "linked target drift")?;
        require(node["data"]["typedFeature"] == pair(usage).element, "typed usage drift")?;
    }
    Ok(registry.len())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    require(
        args.len() == 4,
        "usage: import_save_reopen write|read ROOT KEY",
    )?;
    let (mode, root, key) = (&args[1], PathBuf::from(&args[2]), &args[3]);
    require(matches!(mode.as_str(), "write" | "read"), "unknown mode")?;
    workspace_root(&root)?;
    require(
        !key.is_empty()
            && key.len() <= 64
            && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'),
        "bounded snapshot key required",
    )?;
    let store = SnapshotStore::open(&root)?;
    let (mut stage, mut edit, mut save, mut load) = (0, 0, 0, 0);
    let document = if mode == "write" {
        let importer = Importer::pinned().map_err(|_| "pinned importer unavailable")?;
        let mut editor = Editor::new(Document::empty());
        let clock = Instant::now();
        let staged = importer
            .stage(
                editor.begin(),
                SOURCE,
                None,
                (1..=11).map(pair),
                Arc::new(AtomicBool::new(false)),
            )
            .map_err(|_| "finite staging refused")?;
        require(staged.allocations() == 11, "unexpected import allocations")?;
        staged
            .commit(&mut editor)
            .map_err(|_| "finite commit refused")?;
        stage = clock.elapsed().as_nanos();
        store.save_new(&format!("{key}-baseline"), editor.document())?;
        let clock = Instant::now();
        let mut provider = Fresh(11);
        let context = ElementContext {
            class: ElementClass::Other,
            resource: ResourceObservation::Unresolved,
            library_namespace: LibraryNamespaceObservation::Absent,
            declared_name: Observed::Unresolved,
            qualified_name: Observed::Unresolved,
        };
        let generated = generate_semantic_id(Some(context), &mut provider)
            .map_err(|_| "identity generation refused")?;
        require(
            generated.id.0.to_string() == pair(12).element && provider.0 == 12,
            "generated identity drift",
        )?;
        let mut transaction = editor.begin();
        transaction.create_package(None, pair(12), None, "Generated")?;
        transaction.rename(&pair(6).element, "EditedA")?;
        editor.commit(transaction)?;
        edit = clock.elapsed().as_nanos();
        let clock = Instant::now();
        store.save_new(key, editor.document())?;
        save = clock.elapsed().as_nanos();
        editor.document().clone()
    } else {
        let clock = Instant::now();
        let document = store.load(key)?;
        load = clock.elapsed().as_nanos();
        document
    };
    let registry = verify(&document)?;
    let canonical = document.to_json()?;
    let persisted = capped(File::open(root.join(format!("{key}.emf.json")))?, LIMIT)?;
    require(persisted.len() as u64 <= LIMIT, "snapshot byte bound")?;
    require(
        persisted == canonical.as_bytes(),
        "persisted canonical bytes drift",
    )?;
    drop(document);
    drop(store);
    println!(
        "M06A2/1\t{mode}\t{}\t{stage}\t{edit}\t{save}\t{load}\t{}\t12\t{registry}",
        std::process::id(),
        persisted.len()
    );
    Ok(())
}

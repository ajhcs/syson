// SPDX-License-Identifier: EPL-2.0
//! Public-API baseline: full candidate setup precedes the retention assertion.
use syson_model::{Document, Editor};
fn fixture(nodes: usize) -> Document {
    let mut content = Vec::with_capacity(nodes);
    for n in 0..nodes {
        content.push(serde_json::json!({"id":format!("10000000-0000-4000-8000-{n:012x}"),"eClass":"sysml:Package","data":{"elementId":format!("20000000-0000-4000-8000-{n:012x}"),"declaredName":format!("Package{n}")}}));
    }
    let value = serde_json::json!({"json":{"version":"1.0","encoding":"utf-8"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":content});
    Document::from_json(&value.to_string()).unwrap()
}
#[test]
fn repeated_commits_do_not_retain_more_than_sixty_four_undo_snapshots() {
    let mut editor = Editor::new(fixture(1000));
    for n in 1..=256 {
        let mut transaction = editor.begin();
        transaction.rename("20000000-0000-4000-8000-000000000000", &format!("Revision{n}")).unwrap();
        assert!(editor.commit(transaction).unwrap());
    }
    let mut undos = 0;
    while editor.undo() { undos += 1; }
    eprintln!("RR20 legacy public undo entries retained after256commits={undos}");
    assert!(undos <= 64, "unbounded history retained {undos} undo snapshots");
}

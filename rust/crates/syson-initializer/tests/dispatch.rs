// SPDX-License-Identifier: EPL-2.0
// Prepared source-contract assertions; NOT_RUN.
use syson_initializer::{DISPATCH, SOURCE_METHODS, dispatch_sequence};
use syson_metamodel::ClassifierId;
#[test]
fn complete_declaration_schedule() {
    assert_eq!(DISPATCH.len(), 175);
    assert_eq!(DISPATCH.iter().filter(|r| r.raw_constructible).count(), 167);
    assert_eq!(SOURCE_METHODS.len(), 36);
    let mut names = std::collections::BTreeSet::new();
    for row in DISPATCH {
        assert!(names.insert(row.classifier.metadata().name()));
        assert!(!row.sequence.is_empty());
    }
}
#[test]
fn interface_preserves_connection_fallthrough_schedule() {
    let s = dispatch_sequence(ClassifierId::InterfaceDefinition).unwrap();
    let position = |name| s.iter().position(|n| *n == name).unwrap();
    assert!(position("InterfaceDefinition") < position("ConnectionDefinition"));
    assert!(position("ConnectionDefinition") < position("Definition"));
}

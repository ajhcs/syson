// SPDX-License-Identifier: EPL-2.0
use serde_json::json;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};
use syson_reference_binding::{
    TARGET,
    inspector::{JSON_BYTES, contained, exact_inventory, inspect, read_json},
};

fn area(name: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let base = std::env::var_os("REFERENCE_BINDING_TEST_ROOT")
        .expect("root must allocate REFERENCE_BINDING_TEST_ROOT under /workspace");
    let base = PathBuf::from(base);
    assert!(base.is_absolute() && base.starts_with("/workspace"));
    let path = base.join(format!(
        "{name}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).unwrap();
    fs::canonicalize(path).unwrap()
}
fn file(root: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();
    path
}
fn set(paths: &[PathBuf]) -> BTreeSet<PathBuf> {
    paths.iter().cloned().collect()
}
fn attempt(root: &Path, request: &Path) -> Result<serde_json::Value, String> {
    inspect(
        root,
        root,
        request,
        Path::new("/usr/bin/git"),
        Path::new("/usr/bin/sha256sum"),
    )
}

#[test]
fn json_parsing_returns_the_exact_bytes_without_line_ending_or_text_rewriting() {
    let root = area("json-bytes");
    let bytes = b"{\"name\":\"// preserved\",\"ok\":true}\r\n";
    let path = file(&root, "receipt with spaces.json", bytes);
    let (json, actual) = read_json(&root, path.to_str().unwrap()).unwrap();
    assert_eq!(actual, bytes);
    assert_eq!(json["name"], "// preserved");
    assert_eq!(contained(&root, "receipt with spaces.json").unwrap(), path);
}
#[test]
fn path_escape_control_and_nonregular_inputs_refuse() {
    let root = area("paths");
    let receipt = file(&root, "receipt.json", b"{}");
    assert!(contained(&root, "../outside.json").is_err());
    assert!(contained(&root, "sub/../receipt.json").is_err());
    assert!(contained(&root, "receipt.json\n").is_err());
    assert!(contained(&root, "/etc/passwd").is_err());
    assert!(contained(&root, root.to_str().unwrap()).is_err());
    assert!(contained(&root, "missing.json").is_err());
    assert_eq!(
        contained(&root, receipt.to_str().unwrap()).unwrap(),
        receipt
    );
}
#[test]
fn receipt_and_source_byte_limits_refuse_before_reaching_git() {
    let root = area("json-limits");
    let large = file(&root, "large.json", &vec![b' '; JSON_BYTES as usize + 1]);
    assert!(
        read_json(&root, large.to_str().unwrap())
            .unwrap_err()
            .contains("byte limit")
    );
    assert!(attempt(&root, &large).unwrap_err().contains("byte limit"));
    let bad = file(&root, "bad.json", b"{\"field\":");
    assert!(read_json(&root, bad.to_str().unwrap()).is_err());
    // This branch hashes only the tiny owned artifact before refusing the
    // oversized source; it does not invoke Git or read the source body.
    let artifact = file(&root, "tiny.jar", b"owned fixture");
    let source = root.join("oversized.java");
    fs::File::create(&source)
        .unwrap()
        .set_len(1_048_577)
        .unwrap();
    let request = file(
        &root,
        "large-source-request.json",
        json!({
            "target_revision":TARGET,"claim":"selected-source-runtime",
            "files":{"artifact":{"path":artifact,"sha256":"0".repeat(64)},
            "harness":{"path":source,"sha256":"0".repeat(64)}}
        })
        .to_string()
        .as_bytes(),
    );
    assert!(
        attempt(&root, &request)
            .unwrap_err()
            .contains("file byte limit")
    );
}
#[test]
fn exact_direct_jar_inventory_refuses_extra_missing_and_nested_entries() {
    let root = area("jars");
    let dir = root.join("lib");
    fs::create_dir(&dir).unwrap();
    let a = file(&dir, "a.jar", b"a");
    let b = file(&dir, "b.jar", b"b");
    let expected = set(&[a.clone(), b.clone()]);
    exact_inventory(&root, &dir, &expected, false).unwrap();
    assert!(exact_inventory(&root, &dir, &set(&[a]), false).is_err());
    let mut missing = expected.clone();
    missing.insert(dir.join("missing.jar"));
    assert!(exact_inventory(&root, &dir, &missing, false).is_err());
    file(
        &dir,
        "injected.jar",
        b"undeclared executable classpath input",
    );
    assert!(exact_inventory(&root, &dir, &expected, false).is_err());
    let other = area("nested-jar");
    let nested = file(&other, "lib/sub/hidden.jar", b"hidden");
    assert!(exact_inventory(&other, &other.join("lib"), &set(&[nested]), false).is_err());
}
#[test]
fn exact_recursive_class_inventory_refuses_extra_classes_and_resources() {
    let root = area("classes");
    let main = file(&root, "classes/Oracle.class", b"main");
    let helper = file(&root, "classes/org/example/Helper.class", b"helper");
    let expected = set(&[main.clone(), helper]);
    let dir = root.join("classes");
    exact_inventory(&root, &dir, &expected, true).unwrap();
    assert!(exact_inventory(&root, &dir, &set(&[main]), true).is_err());
    file(&root, "classes/org/example/Injected.class", b"extra");
    assert!(exact_inventory(&root, &dir, &expected, true).is_err());
    let other = area("class-resource");
    let class = file(&other, "classes/Oracle.class", b"main");
    file(
        &other,
        "classes/META-INF/services/provider",
        b"undeclared resource",
    );
    assert!(exact_inventory(&other, &other.join("classes"), &set(&[class]), true).is_err());
}
#[cfg(unix)]
#[test]
fn file_symlink_containment_and_stricter_classpath_symlink_rules_are_distinct() {
    use std::os::unix::fs::symlink;
    let root = area("symlinks");
    let outside = area("outside");
    let target = file(&root, "actual.json", b"{}");
    symlink(&target, root.join("inside.json")).unwrap();
    assert_eq!(contained(&root, "inside.json").unwrap(), target);
    let external = file(&outside, "outside.json", b"{}");
    symlink(external, root.join("outside.json")).unwrap();
    assert!(contained(&root, "outside.json").is_err());
    let jar = file(&root, "lib/actual.jar", b"jar");
    symlink(&jar, root.join("lib/alias.jar")).unwrap();
    assert!(exact_inventory(&root, &root.join("lib"), &set(&[jar]), false).is_err());
    let class = file(&root, "classes/Oracle.class", b"class");
    symlink(root.join("classes"), root.join("linked-classes")).unwrap();
    assert!(
        exact_inventory(
            &root,
            &root.join("linked-classes"),
            &set(&[class.clone()]),
            true
        )
        .is_err()
    );
    fs::create_dir(root.join("link-parent")).unwrap();
    symlink(root.join("classes"), root.join("link-parent/ancestor")).unwrap();
    assert!(
        exact_inventory(
            &root,
            &root.join("link-parent/ancestor"),
            &set(&[class]),
            true
        )
        .is_err()
    );
    let nested = file(&root, "classes/nested/Leaf.class", b"leaf");
    assert!(
        exact_inventory(
            &root,
            &root.join("link-parent/ancestor/nested"),
            &set(&[nested]),
            true
        )
        .is_err()
    );
}
#[test]
fn classpath_entry_and_directory_depth_budgets_refuse_unbounded_tree_shapes() {
    let root = area("entry-limit");
    let mut expected = BTreeSet::new();
    for n in 0..257 {
        expected.insert(file(&root, &format!("lib/{n}.jar"), b""));
    }
    assert!(
        exact_inventory(&root, &root.join("lib"), &expected, false)
            .unwrap_err()
            .contains("entry limit")
    );
    let other = area("tree-depth");
    let name = format!("classes/{}Leaf.class", "d/".repeat(34));
    let leaf = file(&other, &name, b"leaf");
    assert!(
        exact_inventory(&other, &other.join("classes"), &set(&[leaf]), true)
            .unwrap_err()
            .contains("depth limit")
    );
}
#[test]
fn invalid_pin_claim_root_and_escaped_input_fail_before_git_lookup() {
    let root = area("request-guards");
    let bad = file(
        &root,
        "bad-pin.json",
        json!({"target_revision":"0".repeat(40),"claim":"selected-source-runtime"})
            .to_string()
            .as_bytes(),
    );
    assert!(attempt(&root, &bad).unwrap_err().contains("invalid pin"));
    let bad = file(
        &root,
        "escalation.json",
        json!({"target_revision":TARGET,"claim":"whole-artifact-equivalence"})
            .to_string()
            .as_bytes(),
    );
    assert!(
        attempt(&root, &bad)
            .unwrap_err()
            .contains("claim escalation")
    );
    let outside = area("other-root");
    assert!(
        inspect(
            &root,
            &outside,
            &bad,
            Path::new("/usr/bin/git"),
            Path::new("/usr/bin/sha256sum")
        )
        .unwrap_err()
        .contains("repository escaped")
    );
    let bad=file(&root,"escaped-artifact.json",json!({"target_revision":TARGET,"claim":"selected-source-runtime","files":{"artifact":{"path":"../outside.jar","sha256":"0".repeat(64)}}}).to_string().as_bytes());
    assert!(attempt(&root, &bad).unwrap_err().contains("path traversal"));
}

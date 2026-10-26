// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
use syson_verify_composition::verify;
static NEXT: AtomicUsize = AtomicUsize::new(0);
fn git(repo: &Path, args: &[&str]) -> String {
    let config = repo.join("empty-config");
    let hooks = repo.join("no-hooks");
    let output = Command::new("git")
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "user.name=C03 Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
        ])
        .arg(format!("core.hooksPath={}", hooks.display()))
        .arg("-c")
        .arg(format!("core.attributesFile={}", config.display()))
        .args(args)
        .current_dir(repo)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_ATTR_NOSYSTEM", "1")
        .env_remove("GIT_CONFIG_COUNT")
        .env("GIT_CONFIG_GLOBAL", config)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn write(repo: &Path, path: &str, bytes: impl AsRef<[u8]>) {
    let file = repo.join(path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, bytes).unwrap();
}
fn commit(repo: &Path, files: &[&str]) -> String {
    let mut args = vec!["add", "--"];
    args.extend_from_slice(files);
    git(repo, &args);
    git(
        repo,
        &["commit", "--no-gpg-sign", "-m", "synthetic composition"],
    );
    git(repo, &["rev-parse", "HEAD"])
}
fn suffix(module: &str) -> String {
    format!("\n#[path = \"{module}.rs\"]\npub mod {module};\n")
}
struct Fixture {
    repo: PathBuf,
    manifest: Value,
}
impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from(
            std::env::var_os("SYSON_COMPOSITION_TEST_ROOT")
                .expect("coordinator must provide owned absolute fixture root"),
        );
        assert!(root.is_absolute() && root.is_dir());
        let repo = root.join(format!(
            "composition-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&repo).unwrap();
        fs::create_dir(repo.join("no-hooks")).unwrap();
        fs::write(repo.join("empty-config"), b"").unwrap();
        git(
            &repo,
            &["init", "--template=", "--initial-branch=base", "."],
        );
        write(&repo, "lib.rs", "base\n");
        write(&repo, "inherited.txt", "baseline untouched\n");
        let base = commit(&repo, &["lib.rs", "inherited.txt"]);
        let base_blob = git(&repo, &["rev-parse", &format!("{base}:lib.rs")]);
        let mut sources = Vec::new();
        let mut exports = Vec::new();
        for unit in ["a", "b"] {
            git(&repo, &["switch", "-c", &format!("source-{unit}"), "base"]);
            let code = format!("{unit}.rs");
            let fixture = format!("fixtures/{unit}.json");
            write(&repo, &code, format!("source {unit}\n"));
            write(&repo, &fixture, format!("{{\"unit\":\"{unit}\"}}\n"));
            write(&repo, "lib.rs", format!("base\n{}", suffix(unit)));
            let head = commit(&repo, &[&code, &fixture, "lib.rs"]);
            sources.push(json!({"unit":unit,"head_ref":format!("refs/heads/source-{unit}"),"commit":head,"base":base,
                "allowed_paths":[code,fixture],"files":[{"path":code,"blob":git(&repo,&["rev-parse",&format!("{head}:{code}")])},
                {"path":fixture,"blob":git(&repo,&["rev-parse",&format!("{head}:{fixture}")])}]}));
            exports.push(json!({"unit":unit,"module":unit,"path":format!("{unit}.rs")}));
        }
        git(&repo, &["switch", "-c", "candidate", "base"]);
        for unit in ["a", "b"] {
            write(&repo, &format!("{unit}.rs"), format!("source {unit}\n"));
            write(
                &repo,
                &format!("fixtures/{unit}.json"),
                format!("{{\"unit\":\"{unit}\"}}\n"),
            );
        }
        write(
            &repo,
            "lib.rs",
            format!("base\n{}{}", suffix("a"), suffix("b")),
        );
        let candidate = commit(
            &repo,
            &[
                "a.rs",
                "b.rs",
                "fixtures/a.json",
                "fixtures/b.json",
                "lib.rs",
            ],
        );
        Self {
            repo,
            manifest: json!({"schema":1,"base":{"head_ref":"refs/heads/base","commit":base},
            "candidate":{"head_ref":"refs/heads/candidate","commit":candidate},"sources":sources,
            "generated":[{"path":"lib.rs","base_blob":base_blob,"exports":exports}]}),
        }
    }
    fn check(&self) -> Result<Value, String> {
        verify(&self.repo, &serde_json::to_vec(&self.manifest).unwrap())
    }
}
#[test]
fn exact_real_git_composition_passes_without_mutating_index_refs_or_config() {
    let f = Fixture::new();
    let index = fs::read(f.repo.join(".git/index")).unwrap();
    let config = fs::read(f.repo.join(".git/config")).unwrap();
    let refs = git(&f.repo, &["show-ref"]);
    let result = f.check().unwrap();
    assert_eq!(result["selected_files"], 4);
    assert_eq!(result["sources"], 2);
    assert_eq!(result["generated_files"], 1);
    assert_eq!(result["whole_program_acceptance"], false);
    assert_eq!(fs::read(f.repo.join(".git/index")).unwrap(), index);
    assert_eq!(fs::read(f.repo.join(".git/config")).unwrap(), config);
    assert_eq!(git(&f.repo, &["show-ref"]), refs);
}
#[test]
fn stale_source_base_and_candidate_heads_are_refused() {
    let f = Fixture::new();
    for pointer in ["/base/commit", "/candidate/commit", "/sources/0/commit"] {
        let mut m = f.manifest.clone();
        *m.pointer_mut(pointer).unwrap() = json!("0".repeat(40));
        assert!(
            verify(&f.repo, &serde_json::to_vec(&m).unwrap())
                .unwrap_err()
                .contains("stale")
        );
    }
}
#[test]
fn missing_fixture_conflicting_owner_and_wrong_blob_are_refused() {
    let mut f = Fixture::new();
    f.manifest["sources"][0]["files"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(f.check().unwrap_err().contains("delta"));
    let mut f = Fixture::new();
    git(&f.repo, &["switch", "source-b"]);
    write(&f.repo, "a.rs", "source a\n");
    let head = commit(&f.repo, &["a.rs"]);
    git(&f.repo, &["switch", "candidate"]);
    f.manifest["sources"][1]["commit"] = json!(head);
    f.manifest["sources"][1]["allowed_paths"]
        .as_array_mut()
        .unwrap()
        .push(json!("a.rs"));
    let repeated = f.manifest["sources"][0]["files"][0].clone();
    f.manifest["sources"][1]["files"]
        .as_array_mut()
        .unwrap()
        .push(repeated);
    assert!(f.check().unwrap_err().contains("source path conflict"));
    let mut f = Fixture::new();
    f.manifest["sources"][0]["files"][0]["blob"] = json!("0".repeat(40));
    assert!(f.check().unwrap_err().contains("blob"));
    let f = Fixture::new();
    fs::remove_file(f.repo.join("fixtures/a.json")).unwrap();
    assert!(f.check().unwrap_err().contains("candidate path"));
}
#[test]
fn changed_working_candidate_and_undeclared_committed_file_are_refused() {
    let mut f = Fixture::new();
    write(&f.repo, "a.rs", "tampered\n");
    assert!(f.check().unwrap_err().contains("candidate file bytes"));
    write(&f.repo, "a.rs", "source a\n");
    write(&f.repo, "unexpected.txt", "undeclared\n");
    let next = commit(&f.repo, &["unexpected.txt"]);
    f.manifest["candidate"]["commit"] = json!(next);
    assert!(f.check().unwrap_err().contains("candidate contains"));
}
#[test]
fn unknown_exports_wrong_order_and_modified_common_source_are_refused() {
    let mut f = Fixture::new();
    let original = f.manifest.clone();
    f.manifest["generated"][0]["exports"][0]["unit"] = json!("unknown");
    assert!(f.check().unwrap_err().contains("unknown generated unit"));
    f.manifest = original.clone();
    f.manifest["generated"][0]["exports"]
        .as_array_mut()
        .unwrap()
        .reverse();
    assert!(f.check().unwrap_err().contains("export/order"));
    f.manifest = original;
    git(&f.repo, &["switch", "source-a"]);
    write(
        &f.repo,
        "lib.rs",
        format!("base\n{}undeclared common change\n", suffix("a")),
    );
    let head = commit(&f.repo, &["lib.rs"]);
    git(&f.repo, &["switch", "candidate"]);
    f.manifest["sources"][0]["commit"] = json!(head);
    assert!(f.check().unwrap_err().contains("common-export"));
}
#[test]
fn path_escape_invalid_oid_unknown_fields_and_manifest_budget_are_refused() {
    let f = Fixture::new();
    for bad in [
        "../outside",
        "/tmp/outside",
        "a/../outside",
        "a\\outside",
        ".git/config",
        "a:object",
        "a\0x",
    ] {
        let mut m = f.manifest.clone();
        m["sources"][0]["files"][0]["path"] = json!(bad);
        assert!(
            verify(&f.repo, &serde_json::to_vec(&m).unwrap()).is_err(),
            "{bad:?}"
        );
    }
    let mut m = f.manifest.clone();
    m["generated"][0]["exports"][0]["path"] = json!("../../outside.rs");
    assert!(verify(&f.repo, &serde_json::to_vec(&m).unwrap()).is_err());
    let mut m = f.manifest.clone();
    m["base"]["commit"] = json!("not-an-oid");
    assert!(verify(&f.repo, &serde_json::to_vec(&m).unwrap()).is_err());
    let mut m = f.manifest.clone();
    m["unknown"] = json!(true);
    assert!(verify(&f.repo, &serde_json::to_vec(&m).unwrap()).is_err());
    assert!(verify(&f.repo, &vec![b' '; 2 * 1024 * 1024 + 1]).is_err());
}
#[cfg(unix)]
#[test]
fn symlink_and_changed_executable_mode_are_refused() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let f = Fixture::new();
    fs::set_permissions(f.repo.join("a.rs"), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(f.check().unwrap_err().contains("mode"));
    fs::set_permissions(f.repo.join("a.rs"), fs::Permissions::from_mode(0o644)).unwrap();
    write(&f.repo, "target.txt", "source a\n");
    fs::remove_file(f.repo.join("a.rs")).unwrap();
    symlink("target.txt", f.repo.join("a.rs")).unwrap();
    assert!(f.check().unwrap_err().contains("symlink"));
}

#[cfg(unix)]
#[test]
fn executable_mode_uses_git_owner_bit_not_group_or_other_bits() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    fs::set_permissions(f.repo.join("a.rs"), fs::Permissions::from_mode(0o654)).unwrap();
    assert!(f.check().is_ok());
    let mut f = Fixture::new();
    git(&f.repo, &["switch", "source-a"]);
    fs::set_permissions(f.repo.join("a.rs"), fs::Permissions::from_mode(0o755)).unwrap();
    let source = commit(&f.repo, &["a.rs"]);
    git(&f.repo, &["switch", "candidate"]);
    fs::set_permissions(f.repo.join("a.rs"), fs::Permissions::from_mode(0o755)).unwrap();
    let candidate = commit(&f.repo, &["a.rs"]);
    f.manifest["sources"][0]["commit"] = json!(source);
    f.manifest["candidate"]["commit"] = json!(candidate);
    assert!(f.check().is_ok());
    // Git treats 0645 as nonexecutable even though its other-execute bit is set.
    fs::set_permissions(f.repo.join("a.rs"), fs::Permissions::from_mode(0o645)).unwrap();
    assert!(f.check().unwrap_err().contains("mode"));
}

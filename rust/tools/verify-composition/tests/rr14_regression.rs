// SPDX-License-Identifier: EPL-2.0
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
fn directory() -> PathBuf {
    let root = PathBuf::from(
        std::env::var_os("SYSON_COMPOSITION_TEST_ROOT")
            .expect("owned composition fixture root required"),
    );
    assert!(root.is_absolute() && root.starts_with("/workspace") && root.is_dir());
    assert!(
        !root
            .components()
            .any(|p| p == std::path::Component::ParentDir)
    );
    let dir = root.join(format!(
        "rr14-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    dir
}
fn fifo(path: &Path) {
    assert!(path.is_absolute() && path.starts_with("/workspace"));
    let output = Command::new("/usr/bin/mkfifo")
        .arg("--")
        .arg(path)
        .env_clear()
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn cli(repo: &Path, manifest: &Path) -> (Output, bool) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_syson-verify-composition"))
        .arg(repo)
        .arg(manifest)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let start = Instant::now();
    loop {
        if child.try_wait().unwrap().is_some() {
            return (child.wait_with_output().unwrap(), false);
        }
        if start.elapsed() >= Duration::from_secs(2) {
            let _ = child.kill();
            return (child.wait_with_output().unwrap(), true);
        }
        thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn fifo_manifest_refuses_before_open_can_block() {
    let dir = directory();
    let manifest = dir.join("manifest.fifo");
    fifo(&manifest);
    let (output, timed_out) = cli(&dir, &manifest);
    assert!(
        !timed_out,
        "manifest FIFO blocked before regular-file admission; child was killed and waited"
    );
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("composition refused"));
}

#[test]
fn directory_final_symlink_missing_and_device_inputs_refuse_promptly() {
    let dir = directory();
    let regular = dir.join("regular.json");
    fs::write(&regular, b"preserved regular bytes").unwrap();
    let linked = dir.join("linked.json");
    std::os::unix::fs::symlink("regular.json", &linked).unwrap();
    for path in [&dir, &linked, &dir.join("missing.json")] {
        let (output, timed_out) = cli(&dir, path);
        assert!(
            !timed_out,
            "special/missing input did not refuse before the deadline"
        );
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("composition refused"));
    }
    // This fixed device is metadata-only under the production admission gate.
    // No special file is created and no external device state is changed.
    assert!(
        syson_verify_composition::read_file(Path::new("/dev/null"))
            .unwrap_err()
            .contains("regular file required")
    );
    assert_eq!(fs::read(&regular).unwrap(), b"preserved regular bytes");
}
#[test]
fn exact_regular_limit_and_ancestor_symlink_alias_preserve_selected_bytes() {
    let dir = directory();
    let actual = dir.join("actual");
    fs::create_dir(&actual).unwrap();
    let path = actual.join("manifest.json");
    let exact = vec![b'x'; 2 * 1024 * 1024];
    fs::write(&path, &exact).unwrap();
    assert_eq!(syson_verify_composition::read_file(&path).unwrap(), exact);
    let alias = dir.join("alias");
    std::os::unix::fs::symlink("actual", &alias).unwrap();
    assert_eq!(
        syson_verify_composition::read_file(&alias.join("manifest.json")).unwrap(),
        exact
    );
    let before = fs::read(&path).unwrap();
    assert_eq!(before.len(), 2 * 1024 * 1024);
    fs::write(&path, vec![b'x'; before.len() + 1]).unwrap();
    assert!(
        syson_verify_composition::read_file(&path)
            .unwrap_err()
            .contains("file byte limit")
    );
    assert_eq!(
        fs::metadata(&path).unwrap().len(),
        (before.len() + 1) as u64
    );
}
fn git(repo: &Path, args: &[&str]) -> String {
    let empty = repo.parent().unwrap().join("empty-config");
    let hooks = repo.parent().unwrap().join("no-hooks");
    let output = Command::new("/usr/bin/git")
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "user.name=RR14 Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
        ])
        .arg(format!("core.hooksPath={}", hooks.display()))
        .arg("-c")
        .arg(format!("core.attributesFile={}", empty.display()))
        .arg("-C")
        .arg(repo)
        .args(args)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_ATTR_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", &empty)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
#[test]
fn regular_manifest_passes_actual_git_composition_and_retains_blob_mode_delta_guards() {
    use serde_json::{Value, json};
    let dir = directory();
    fs::write(dir.join("empty-config"), b"").unwrap();
    fs::create_dir(dir.join("no-hooks")).unwrap();
    let repo = dir.join("repo");
    fs::create_dir(&repo).unwrap();
    git(
        &repo,
        &["init", "--template=", "--initial-branch=base", "."],
    );
    fs::write(repo.join("inherited.txt"), b"unchanged baseline\n").unwrap();
    git(&repo, &["add", "--", "inherited.txt"]);
    git(&repo, &["commit", "--no-gpg-sign", "-m", "owned base"]);
    let base = git(&repo, &["rev-parse", "HEAD"]);
    git(&repo, &["switch", "-c", "source"]);
    fs::write(repo.join("unit.rs"), b"source bytes only; never compiled\n").unwrap();
    git(&repo, &["add", "--", "unit.rs"]);
    git(&repo, &["commit", "--no-gpg-sign", "-m", "owned source"]);
    let source = git(&repo, &["rev-parse", "HEAD"]);
    let blob = git(&repo, &["rev-parse", "HEAD:unit.rs"]);
    git(&repo, &["switch", "-c", "candidate"]);
    let mut manifest = json!({"schema":1,
        "base":{"head_ref":"refs/heads/base","commit":base},
        "candidate":{"head_ref":"refs/heads/candidate","commit":source},
        "sources":[{"unit":"one","head_ref":"refs/heads/source","commit":source,"base":base,
            "allowed_paths":["unit.rs"],"files":[{"path":"unit.rs","blob":blob}]}],
        "generated":[]});
    let path = dir.join("manifest.json");
    let bytes = serde_json::to_vec(&manifest).unwrap();
    fs::write(&path, &bytes).unwrap();
    let index = fs::read(repo.join(".git/index")).unwrap();
    let config = fs::read(repo.join(".git/config")).unwrap();
    let refs = git(&repo, &["show-ref"]);
    let (output, timed_out) = cli(&repo, &path);
    assert!(!timed_out);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let accepted: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(accepted["status"], "VERIFIED_EXPLICIT_SOURCE_COMPOSITION");
    assert_eq!(accepted["selected_files"], 1);
    assert_eq!(accepted["whole_program_acceptance"], false);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(fs::read(repo.join(".git/index")).unwrap(), index);
    assert_eq!(fs::read(repo.join(".git/config")).unwrap(), config);
    assert_eq!(git(&repo, &["show-ref"]), refs);

    manifest["sources"][0]["files"][0]["blob"] = json!("0".repeat(40));
    assert!(
        syson_verify_composition::verify(&repo, &serde_json::to_vec(&manifest).unwrap())
            .unwrap_err()
            .contains("blob")
    );
    fs::write(repo.join("unit.rs"), b"uncommitted wrong bytes").unwrap();
    assert!(
        syson_verify_composition::verify(&repo, &bytes)
            .unwrap_err()
            .contains("file bytes")
    );
    fs::write(repo.join("unit.rs"), b"source bytes only; never compiled\n").unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(repo.join("unit.rs"), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(
        syson_verify_composition::verify(&repo, &bytes)
            .unwrap_err()
            .contains("mode")
    );
    fs::set_permissions(repo.join("unit.rs"), fs::Permissions::from_mode(0o644)).unwrap();
    fs::write(repo.join("unexpected.txt"), b"undeclared bytes").unwrap();
    git(&repo, &["add", "--", "unexpected.txt"]);
    git(
        &repo,
        &["commit", "--no-gpg-sign", "-m", "owned undeclared delta"],
    );
    manifest["sources"][0]["files"][0]["blob"] = json!(blob);
    manifest["candidate"]["commit"] = json!(git(&repo, &["rev-parse", "HEAD"]));
    assert!(
        syson_verify_composition::verify(&repo, &serde_json::to_vec(&manifest).unwrap())
            .unwrap_err()
            .contains("candidate contains")
    );
}

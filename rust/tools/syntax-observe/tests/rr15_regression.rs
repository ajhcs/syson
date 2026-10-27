// SPDX-License-Identifier: EPL-2.0
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
fn directory() -> PathBuf {
    let base =
        PathBuf::from(std::env::var_os("SYSON_TEST_DIR").expect("owned workspace root required"))
            .canonicalize()
            .unwrap();
    assert!(base.starts_with("/workspace"));
    let dir = base.join(format!(
        "rr15-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    dir
}
fn run(path: &Path) -> Output {
    assert!(path.is_absolute() && path.starts_with("/workspace"));
    let child = Command::new(env!("CARGO_BIN_EXE_syson-syntax-observe"))
        .arg(path)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    wait(child)
}
fn wait(mut child: Child) -> Output {
    let start = Instant::now();
    let mut timed_out = false;
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if start.elapsed() >= Duration::from_secs(3) {
            timed_out = true;
            let _ = child.kill();
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    if timed_out {
        assert!(
            output.stdout.is_empty(),
            "timed-out CLI printed an observation"
        );
        panic!(
            "syntax CLI exceeded the 3-second owned child deadline; killed and waited: {}",
            output.status
        );
    }
    output
}
#[test]
fn fifo_fixture_refuses_without_blocking_or_printing_an_observation() {
    let file = directory().join("input.sysml");
    let child = Command::new("/usr/bin/mkfifo")
        .arg("--")
        .arg(&file)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    assert!(wait(child).status.success());
    let output = run(&file);
    assert!(!output.status.success(), "FIFO admitted");
    assert!(output.stdout.is_empty(), "refusal printed an observation");
}

fn refuses(path: &Path) -> Output {
    let output = run(path);
    assert!(
        !output.status.success(),
        "unexpected success for {}",
        path.display()
    );
    assert!(output.stdout.is_empty(), "refusal printed an observation");
    assert!(!output.stderr.is_empty(), "refusal lost its diagnostic");
    output
}

#[test]
fn valid_and_lexically_invalid_observations_match_original_bytes() {
    let fixtures: &[(&str, &[u8], &[u8])] = &[
        (
            "names.sysml",
            include_bytes!(
                "../../../../porting/validation/parser-corpus/fixtures/raw-names-strings.sysml"
            ),
            include_bytes!(
                "../../../../porting/runs/parser-corpus-rust-01/fixtures/pc01.stdout.txt"
            ),
        ),
        (
            "types.kerml",
            include_bytes!(
                "../../../../porting/validation/parser-corpus/fixtures/datatype-reference.kerml"
            ),
            include_bytes!(
                "../../../../porting/runs/parser-corpus-rust-01/fixtures/pc08.stdout.txt"
            ),
        ),
        (
            "malformed.sysml",
            include_bytes!(
                "../../../../porting/validation/parser-corpus/fixtures/unterminated-string.sysml"
            ),
            include_bytes!(
                "../../../../porting/runs/parser-corpus-rust-01/fixtures/pc12.stdout.txt"
            ),
        ),
    ];
    let dir = directory();
    for (name, input, original) in fixtures {
        let path = dir.join(name);
        fs::write(&path, input).unwrap();
        let output = run(&path);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        assert_eq!(
            output.stdout.as_slice(),
            *original,
            "historical output changed for {name}"
        );
        assert_eq!(fs::read(path).unwrap(), *input);
    }
}

#[test]
fn exact_byte_limit_including_unicode_is_admitted_and_next_byte_refused() {
    let path = directory().join("boundary.sysml");
    let mut input = include_bytes!(
        "../../../../porting/validation/parser-corpus/fixtures/raw-names-strings.sysml"
    )
    .to_vec();
    input.resize(1024, b' ');
    fs::write(&path, &input).unwrap();
    let output = run(&path);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let observed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(observed["input_bytes"], 1024);
    assert_eq!(observed["syntax_recognized"], true);
    assert!(observed["input_utf16_units"].as_u64().unwrap() < 1024);
    input.push(b' ');
    fs::write(&path, &input).unwrap();
    refuses(&path);
    assert_eq!(fs::read(path).unwrap(), input);
}

#[test]
fn invalid_utf8_suffix_and_directory_never_print_observations() {
    let dir = directory();
    let invalid = dir.join("invalid.sysml");
    fs::write(&invalid, [0xff]).unwrap();
    refuses(&invalid);
    for name in ["fixture.txt", "fixture.SYSML", "fixture.sysml.json"] {
        let path = dir.join(name);
        fs::write(&path, b"package P;").unwrap();
        let output = refuses(&path);
        assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported fixture suffix"));
    }
    let path = dir.join("directory.kerml");
    fs::create_dir(&path).unwrap();
    refuses(&path);
}

#[test]
fn final_symlinks_refuse_but_stable_ancestor_alias_preserves_output() {
    use std::os::unix::fs::symlink;
    let dir = directory();
    let source = include_bytes!(
        "../../../../porting/validation/parser-corpus/fixtures/datatype-reference.kerml"
    );
    let target = dir.join("target.kerml");
    fs::write(&target, source).unwrap();
    let link = dir.join("link.kerml");
    symlink(&target, &link).unwrap();
    refuses(&link);
    let dangling = dir.join("dangling.sysml");
    symlink(dir.join("absent.sysml"), &dangling).unwrap();
    refuses(&dangling);
    let alias = dir.join("alias");
    symlink(&dir, &alias).unwrap();
    let output = run(&alias.join("target.kerml"));
    assert!(output.status.success());
    assert_eq!(
        output.stdout.as_slice(),
        include_bytes!("../../../../porting/runs/parser-corpus-rust-01/fixtures/pc08.stdout.txt")
            .as_slice()
    );
    assert_eq!(fs::read(target).unwrap(), source);
}

// SPDX-License-Identifier: EPL-2.0
//! Native syntax observation only. This tool never executes the reference parser.
use serde_json::json;
use std::io;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::io::Read;
use std::path::Path;
use syson_syntax::{Grammar, Language, Options};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        return Err("expected exactly one reviewed fixture path".into());
    }
    let path = Path::new(&args[1]);
    let language = match path.extension().and_then(|s| s.to_str()) {
        Some("sysml") => Language::SysML,
        Some("kerml") => Language::KerML,
        _ => return Err("unsupported fixture suffix".into()),
    };
    let input = read_input(path)?;
    let grammar = Grammar::load(language).map_err(|e| format!("grammar load failed: {e:?}"))?;
    let mut output = json!({"language":format!("{language:?}"),"input_bytes":input.len(),"input_utf16_units":input.encode_utf16().count(),"ast_status":"NotPerformed","semantic_status":"NotPerformed"});
    match grammar.recognize(&input, Options::default()) {
        Ok(r) => {
            output["syntax_recognized"] = json!(true);
            output["tokens"] = json!(r.tokens.len());
            output["visible_tokens"] = json!(r.tokens.iter().filter(|t| !t.hidden).count());
            output["cst_nodes"] = json!(r.cst.len());
            output["root_byte_start"] = json!(r.cst[r.root].span.start.byte);
            output["root_byte_end"] = json!(r.cst[r.root].span.end.byte);
            output["state_history_merges"] = json!(r.ambiguous_state_merges);
            output["semantic_status"] = json!(format!("{:?}", r.semantics));
        }
        Err(e) => {
            output["syntax_recognized"] = json!(false);
            output["error_debug"] = json!(format!("{e:?}"));
        }
    }
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
const MAX_INPUT: u64 = 1024;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn read_input(path: &Path) -> io::Result<String> {
    read_regular(path, || {}, || {})
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
fn read_input(_: &Path) -> io::Result<String> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "regular fixture admission requires Linux x86_64",
    ))
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn same_file(a: &std::fs::Metadata, b: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    a.is_file()
        && b.is_file()
        && a.dev() == b.dev()
        && a.ino() == b.ino()
        && a.len() == b.len()
        && a.mtime() == b.mtime()
        && a.mtime_nsec() == b.mtime_nsec()
        && a.ctime() == b.ctime()
        && a.ctime_nsec() == b.ctime_nsec()
}

// Stable ancestor aliases are permitted; the final component must not be a symlink.
// These checks do not provide a hostile-writer snapshot, openat parent pinning,
// arbitrary device-substitution protection, or a deadline for regular disk I/O.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn read_regular(
    path: &Path,
    before_open: impl FnOnce(),
    after_read: impl FnOnce(),
) -> io::Result<String> {
    use std::os::unix::fs::OpenOptionsExt;
    let refused = || {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "fixture must remain the same regular file of at most 1024 bytes",
        )
    };
    let admitted = std::fs::symlink_metadata(path)?;
    if !admitted.is_file() || admitted.len() > MAX_INPUT {
        return Err(refused());
    }
    before_open();
    // Linux x86_64 O_NONBLOCK | O_NOFOLLOW, fixed on this supported target only.
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(0x800 | 0x20000)
        .open(path)?;
    let opened = file.metadata()?;
    if !same_file(&admitted, &opened) {
        return Err(refused());
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(admitted.len() as usize)
        .map_err(|_| io::Error::other("fixture allocation refused"))?;
    let mut reader = (&file).take(admitted.len() + 1);
    let mut chunk = [0u8; 256];
    loop {
        let read = reader.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        if bytes.len() + read > admitted.len() as usize {
            return Err(refused());
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    after_read();
    if bytes.len() as u64 != admitted.len()
        || !same_file(&opened, &file.metadata()?)
        || !same_file(&opened, &std::fs::symlink_metadata(path)?)
    {
        return Err(refused());
    }
    String::from_utf8(bytes).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        )
    })
}

#[cfg(all(test, target_os = "linux", target_arch = "x86_64"))]
mod rr15_unit {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        process::{Child, Command, Output, Stdio},
        thread,
        time::{Duration, Instant},
    };
    fn wait(mut child: Child) -> Output {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let output = child.wait_with_output().unwrap();
                panic!(
                    "owned RR15 race child exceeded deadline; killed and waited: {}",
                    output.status
                );
            }
            thread::sleep(Duration::from_millis(10));
        }
        child.wait_with_output().unwrap()
    }
    fn directory() -> PathBuf {
        let base = PathBuf::from(
            std::env::var_os("SYSON_TEST_DIR").expect("owned workspace root required"),
        )
        .canonicalize()
        .unwrap();
        assert!(base.starts_with("/workspace"));
        let path = base.join(format!("rr15-races-{}", std::process::id()));
        fs::create_dir(&path).unwrap();
        path
    }
    #[test]
    fn deterministic_replacement_races_refuse_under_child_deadline() {
        if std::env::var_os("SYSON_RR15_RACE_CHILD").is_none() {
            let root = std::env::var_os("SYSON_TEST_DIR").expect("owned workspace root required");
            let child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "rr15_unit::deterministic_replacement_races_refuse_under_child_deadline",
                    "--nocapture",
                ])
                .env_clear()
                .env("SYSON_TEST_DIR", root)
                .env("SYSON_RR15_RACE_CHILD", "1")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let output = wait(child);
            assert!(
                output.status.success(),
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(
                stdout.contains("running 1 test") && stdout.contains("1 passed"),
                "wrong exact filter: {stdout}"
            );
            return;
        }
        use std::os::unix::fs::symlink;
        let dir = directory();
        let path = dir.join("input.sysml");
        let other = dir.join("other.sysml");
        fs::write(&other, b"package Q;").unwrap();
        for phase in 0..5 {
            fs::write(&path, b"package P;").unwrap();
            let replace = || {
                fs::rename(&path, dir.join(format!("retired-{phase}"))).unwrap();
                match phase {
                    0 => {
                        let child = Command::new("/usr/bin/mkfifo")
                            .arg("--")
                            .arg(&path)
                            .env_clear()
                            .stdin(Stdio::null())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .unwrap();
                        assert!(wait(child).status.success());
                    }
                    1 => symlink(&other, &path).unwrap(),
                    _ => {
                        fs::copy(&other, &path).unwrap();
                    }
                }
            };
            let result = if phase < 3 {
                read_regular(&path, replace, || {})
            } else if phase == 3 {
                read_regular(&path, || {}, replace)
            } else {
                read_regular(
                    &path,
                    || {},
                    || {
                        fs::write(&path, b"longer replacement").unwrap();
                    },
                )
            };
            assert!(result.is_err(), "replacement phase {phase} admitted");
            fs::remove_file(&path).unwrap();
        }
        assert_eq!(fs::read(&other).unwrap(), b"package Q;");
        // Alias retargeting is detected by actual opened identity, not alias text.
        let first = dir.join("first");
        let second = dir.join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        fs::write(first.join("input.sysml"), b"package P;").unwrap();
        fs::write(second.join("input.sysml"), b"package Q;").unwrap();
        let alias = dir.join("alias");
        symlink(&first, &alias).unwrap();
        assert!(
            read_regular(
                &alias.join("input.sysml"),
                || {
                    fs::remove_file(&alias).unwrap();
                    symlink(&second, &alias).unwrap();
                },
                || {}
            )
            .is_err()
        );
    }
}

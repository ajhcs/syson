// SPDX-License-Identifier: EPL-2.0
//! Read-only verification of explicit same-base Git blob compositions.
#![forbid(unsafe_code)]
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_ITEMS: usize = 4096;
type Result<T> = std::result::Result<T, String>;
fn need(ok: bool, message: impl Into<String>) -> Result<()> {
    if ok { Ok(()) } else { Err(message.into()) }
}
/// Read a bounded regular file without a blocking FIFO open. Final symlinks
/// refuse; ancestor symlinks may name an admitted regular target. The opened
/// descriptor must match pre-admission identity and stable post-read metadata.
/// This is not a hostile-writer snapshot or a regular-disk-I/O deadline.
/// Fixed Linux x86_64 flags are required; other targets refuse explicitly.
pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        read_regular(path, || {})
    }
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    {
        let _ = path;
        Err("nonblocking regular-file admission requires Linux x86_64".into())
    }
}
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn read_regular(path: &Path, before_open: impl FnOnce()) -> Result<Vec<u8>> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    // Linux x86_64 bits/fcntl-linux.h: O_NONBLOCK=04000, O_NOFOLLOW=0400000.
    // Never substitute platform-specific numeric flags on an unadmitted target.
    const NONBLOCK_NOFOLLOW: i32 = 0x800 | 0x20000;
    let regular = |meta: &fs::Metadata| -> Result<()> {
        need(
            meta.is_file(),
            "regular file required; symlinks and special files refuse",
        )?;
        need(meta.len() <= MAX_BYTES as u64, "file byte limit")
    };
    let same = |a: &fs::Metadata, b: &fs::Metadata| {
        a.dev() == b.dev()
            && a.ino() == b.ino()
            && a.len() == b.len()
            && a.mtime() == b.mtime()
            && a.mtime_nsec() == b.mtime_nsec()
            && a.ctime() == b.ctime()
            && a.ctime_nsec() == b.ctime_nsec()
    };
    let admitted = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    regular(&admitted)?;
    // Private seam for deterministic unit races. Production always supplies a
    // no-op and the fixed opener below; no public callback or runtime flag exists.
    before_open();
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(NONBLOCK_NOFOLLOW)
        .open(path)
        .map_err(|e| e.to_string())?;
    let opened = file.metadata().map_err(|e| e.to_string())?;
    regular(&opened)?;
    need(
        same(&admitted, &opened),
        "file identity or metadata changed during admission",
    )?;
    let mut bytes = Vec::new();
    (&file)
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    need(bytes.len() <= MAX_BYTES, "file byte limit")?;
    let descriptor_after = file.metadata().map_err(|e| e.to_string())?;
    let path_after = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    regular(&descriptor_after)?;
    regular(&path_after)?;
    need(
        same(&opened, &descriptor_after) && same(&descriptor_after, &path_after),
        "file identity or metadata changed during read",
    )?;
    Ok(bytes)
}
fn object<'a>(v: &'a Value, keys: &[&str]) -> Result<&'a serde_json::Map<String, Value>> {
    let o = v.as_object().ok_or("object required")?;
    need(
        o.len() == keys.len() && keys.iter().all(|k| o.contains_key(*k)),
        "unknown or missing manifest field",
    )?;
    Ok(o)
}
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or_else(|| "string required".into())
}
fn array(v: &Value) -> Result<&Vec<Value>> {
    let a = v.as_array().ok_or("array required")?;
    need(a.len() <= MAX_ITEMS, "manifest array limit")?;
    Ok(a)
}
fn oid(s: &str) -> Result<()> {
    need(
        s.len() == 40
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "lowercase 40-hex object ID required",
    )
}
fn path(s: &str) -> Result<()> {
    need(
        !s.is_empty()
            && s.len() <= 1024
            && !s.contains(['\\', ':', '*', '?', '[', ']'])
            && !s.chars().any(char::is_control)
            && s.split('/')
                .all(|c| !c.is_empty() && c != "." && c != ".." && c != ".git"),
        "unsafe repository-relative path",
    )
}
fn scope(s: &str) -> Result<()> {
    path(s.strip_suffix('/').unwrap_or(s))
}
fn within(file: &str, scope: &str) -> bool {
    if let Some(dir) = scope.strip_suffix('/') {
        file.starts_with(&format!("{dir}/"))
    } else {
        file == scope
    }
}
fn branch(s: &str) -> Result<()> {
    need(
        s.starts_with("refs/heads/")
            && s.len() <= 240
            && !s.contains("..")
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
            && s.split('/')
                .all(|c| !c.is_empty() && !c.ends_with('.') && !c.ends_with(".lock")),
        "unsafe head ref",
    )
}
struct Git {
    root: PathBuf,
}
impl Git {
    fn run(&self, args: &[&str]) -> Result<Vec<u8>> {
        let mut c = Command::new("git");
        c.args([
            "--no-pager",
            "--no-replace-objects",
            "-c",
            "core.fsmonitor=false",
            "-C",
        ])
        .arg(&self.root)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env_remove("GIT_CONFIG_COUNT")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_OBJECT_DIRECTORY")
        .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
        let mut child = c.spawn().map_err(|e| format!("git spawn: {e}"))?;
        let mut bytes = Vec::new();
        let read = child
            .stdout
            .take()
            .unwrap()
            .take((8 * MAX_BYTES + 1) as u64)
            .read_to_end(&mut bytes);
        if read.is_err() || bytes.len() > 8 * MAX_BYTES {
            let _ = child.kill();
            let _ = child.wait();
            return Err("git output limit/read failure".into());
        }
        let status = child.wait().map_err(|e| format!("git wait: {e}"))?;
        need(
            status.success(),
            format!("read-only git command failed: {}", args[0]),
        )?;
        Ok(bytes)
    }
    fn line(&self, args: &[&str]) -> Result<String> {
        String::from_utf8(self.run(args)?)
            .map(|s| s.trim_end_matches('\n').to_owned())
            .map_err(|_| "non-UTF8 Git metadata".into())
    }
    fn head(&self, v: &Value) -> Result<String> {
        object(v, &["head_ref", "commit"])?;
        let name = text(&v["head_ref"])?;
        let expected = text(&v["commit"])?;
        branch(name)?;
        oid(expected)?;
        need(
            self.line(&[
                "rev-parse",
                "--verify",
                "--end-of-options",
                &format!("{name}^{{commit}}"),
            ])? == expected,
            "stale head ref",
        )?;
        Ok(expected.into())
    }
    fn entry(&self, commit: &str, file: &str) -> Result<(String, String)> {
        oid(commit)?;
        path(file)?;
        let raw = self.run(&["ls-tree", "-z", commit, "--", file])?;
        let value = std::str::from_utf8(&raw).map_err(|_| "non-UTF8 tree entry")?;
        let rows: Vec<_> = value.split('\0').filter(|s| !s.is_empty()).collect();
        need(rows.len() == 1, "missing or ambiguous selected file")?;
        let (info, actual) = rows[0].split_once('\t').ok_or("invalid tree entry")?;
        let fields: Vec<_> = info.split(' ').collect();
        need(
            actual == file
                && fields.len() == 3
                && fields[1] == "blob"
                && matches!(fields[0], "100644" | "100755"),
            "selected entry must be an exact regular file",
        )?;
        oid(fields[2])?;
        Ok((fields[0].into(), fields[2].into()))
    }
    fn blob(&self, blob: &str) -> Result<Vec<u8>> {
        oid(blob)?;
        let size = self
            .line(&["cat-file", "-s", blob])?
            .parse::<usize>()
            .map_err(|_| "invalid blob size")?;
        need(size <= MAX_BYTES, "selected blob byte limit")?;
        let bytes = self.run(&["cat-file", "blob", blob])?;
        need(bytes.len() == size, "blob size mismatch")?;
        Ok(bytes)
    }
    fn changes(&self, base: &str, head: &str) -> Result<BTreeSet<String>> {
        oid(base)?;
        oid(head)?;
        let bytes = self.run(&[
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            "-r",
            "-z",
            "--no-renames",
            "--no-ext-diff",
            "--no-textconv",
            base,
            head,
        ])?;
        let raw = std::str::from_utf8(&bytes).map_err(|_| "non-UTF8 delta path")?;
        let mut result = BTreeSet::new();
        for p in raw.split('\0').filter(|s| !s.is_empty()) {
            path(p)?;
            result.insert(p.into());
        }
        need(result.len() <= MAX_ITEMS, "delta path limit")?;
        Ok(result)
    }
    fn ancestor(&self, base: &str, head: &str) -> Result<()> {
        self.run(&["merge-base", "--is-ancestor", base, head])
            .map(|_| ())
    }
    fn candidate_file(&self, file: &str, mode: &str, bytes: &[u8]) -> Result<()> {
        let mut current = self.root.clone();
        for part in file.split('/') {
            current.push(part);
            let meta =
                fs::symlink_metadata(&current).map_err(|e| format!("candidate path: {e}"))?;
            need(
                !meta.file_type().is_symlink(),
                "candidate path contains symlink",
            )?;
        }
        let meta = fs::metadata(&current).map_err(|e| e.to_string())?;
        need(
            meta.is_file() && meta.len() <= MAX_BYTES as u64,
            "candidate file size/type",
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            need(
                (meta.permissions().mode() & 0o100 != 0) == (mode == "100755"),
                "candidate executable mode differs",
            )?;
        }
        #[cfg(not(unix))]
        let _ = mode;
        need(
            read_file(&current)? == bytes,
            "changed candidate file bytes",
        )
    }
}
fn export_target(file: &str, relative: &str) -> Result<String> {
    need(
        !relative.is_empty() && !relative.contains(['\\', ':', '"', '\n', '\r']),
        "unsafe export path",
    )?;
    let mut parts: Vec<&str> = file.split('/').collect();
    parts.pop();
    for part in relative.split('/') {
        if part == ".." {
            need(parts.pop().is_some(), "export path escape")?;
        } else {
            need(!part.is_empty() && part != ".", "unsafe export component")?;
            parts.push(part);
        }
    }
    let resolved = parts.join("/");
    path(&resolved)?;
    Ok(resolved)
}
fn export_line(export: &Value) -> Result<Vec<u8>> {
    object(export, &["unit", "module", "path"])?;
    let module = text(&export["module"])?;
    let relative = text(&export["path"])?;
    need(
        !module.is_empty()
            && module.bytes().enumerate().all(|(i, b)| {
                b == b'_' || b.is_ascii_alphabetic() || (i > 0 && b.is_ascii_digit())
            }),
        "invalid Rust module name",
    )?;
    Ok(format!("\n#[path = \"{relative}\"]\npub mod {module};\n").into_bytes())
}
/// Validate explicit same-base source branches and a committed candidate. The
/// candidate's selected/generated regular files are also compared to disk.
/// The coordinator separately freezes/builds the full dependency closure.
/// Git commands never mutate refs/index/config, invoke filters or fetch objects.
pub fn verify(repo: &Path, bytes: &[u8]) -> Result<Value> {
    need(bytes.len() <= MAX_BYTES, "manifest byte limit")?;
    let m: Value = serde_json::from_slice(bytes).map_err(|e| format!("manifest JSON: {e}"))?;
    object(&m, &["schema", "base", "candidate", "sources", "generated"])?;
    need(m["schema"] == 1, "unsupported manifest schema")?;
    let git = Git {
        root: repo
            .canonicalize()
            .map_err(|e| format!("repository path: {e}"))?,
    };
    need(
        Path::new(&git.line(&["rev-parse", "--show-toplevel"])?)
            .canonicalize()
            .map_err(|e| e.to_string())?
            == git.root,
        "repository must be worktree root",
    )?;
    let base = git.head(&m["base"])?;
    let candidate = git.head(&m["candidate"])?;
    need(
        git.line(&["rev-parse", "--verify", "HEAD^{commit}"])? == candidate,
        "candidate HEAD differs",
    )?;
    git.ancestor(&base, &candidate)?;
    let sources = array(&m["sources"])?;
    let generated = array(&m["generated"])?;
    need(
        !sources.is_empty() && sources.len() <= 64 && generated.len() <= 8,
        "source/generated count",
    )?;
    let mut units = BTreeMap::new();
    let mut files: BTreeMap<String, (String, String, String)> = BTreeMap::new();
    for source in sources {
        object(
            source,
            &[
                "unit",
                "head_ref",
                "commit",
                "base",
                "allowed_paths",
                "files",
            ],
        )?;
        let unit = text(&source["unit"])?;
        need(!unit.is_empty() && unit.len() <= 64, "invalid unit")?;
        need(
            units.insert(unit.to_owned(), source).is_none(),
            "duplicate source unit",
        )?;
        let head = git
            .head(&serde_json::json!({"head_ref":source["head_ref"],"commit":source["commit"]}))?;
        need(text(&source["base"])? == base, "source base differs")?;
        git.ancestor(&base, &head)?;
        for allowed in array(&source["allowed_paths"])? {
            scope(text(allowed)?)?;
        }
        need(
            !array(&source["files"])?.is_empty(),
            "source selection empty",
        )?;
        for file in array(&source["files"])? {
            object(file, &["path", "blob"])?;
            let p = text(&file["path"])?;
            let blob = text(&file["blob"])?;
            path(p)?;
            oid(blob)?;
            need(
                array(&source["allowed_paths"])?
                    .iter()
                    .any(|s| within(p, s.as_str().unwrap_or(""))),
                "source file outside ownership",
            )?;
            let (mode, actual) = git.entry(&head, p)?;
            need(actual == blob, "source blob mismatch")?;
            need(
                files
                    .insert(p.into(), (unit.into(), mode, blob.into()))
                    .is_none(),
                "undeclared source path conflict",
            )?;
        }
    }
    need(files.len() <= MAX_ITEMS, "global selected file limit")?;
    let mut generated_names = BTreeSet::new();
    let mut module_names = BTreeSet::new();
    let mut by_unit: BTreeMap<String, BTreeMap<String, Vec<u8>>> = BTreeMap::new();
    for g in generated {
        object(g, &["path", "base_blob", "exports"])?;
        let p = text(&g["path"])?;
        let declared = text(&g["base_blob"])?;
        path(p)?;
        oid(declared)?;
        need(
            !files.contains_key(p) && generated_names.insert(p.to_owned()),
            "generated path conflict",
        )?;
        let (mode, actual) = git.entry(&base, p)?;
        need(actual == declared, "generated base blob mismatch")?;
        let base_bytes = git.blob(declared)?;
        need(
            base_bytes.last() == Some(&b'\n'),
            "generated base must end newline",
        )?;
        let mut result = base_bytes.clone();
        need(!array(&g["exports"])?.is_empty(), "generated exports empty")?;
        for export in array(&g["exports"])? {
            let append = export_line(export)?;
            let unit = text(&export["unit"])?;
            need(units.contains_key(unit), "unknown generated unit")?;
            let target = export_target(p, text(&export["path"])?)?;
            need(
                files
                    .get(&target)
                    .is_some_and(|(owner, _, _)| owner == unit),
                "export not backed by own source file",
            )?;
            need(
                module_names.insert(text(&export["module"])?.to_owned()),
                "duplicate generated module",
            )?;
            result.extend_from_slice(&append);
            by_unit
                .entry(unit.into())
                .or_default()
                .entry(p.into())
                .or_default()
                .extend_from_slice(&append);
        }
        need(result.len() <= MAX_BYTES, "generated byte limit")?;
        for (unit, source) in &units {
            let appended = by_unit
                .get(unit)
                .and_then(|rows| rows.get(p))
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let (source_mode, blob) = git.entry(text(&source["commit"])?, p)?;
            let source_bytes = git.blob(&blob)?;
            need(
                source_mode == mode
                    && source_bytes.starts_with(&base_bytes)
                    && source_bytes[base_bytes.len()..] == *appended,
                "undeclared source common-export change",
            )?;
        }
        let (candidate_mode, blob) = git.entry(&candidate, p)?;
        need(
            candidate_mode == mode && git.blob(&blob)? == result,
            "candidate generated export/order mismatch",
        )?;
        git.candidate_file(p, &mode, &result)?;
    }
    for (unit, source) in &units {
        let selected: BTreeSet<_> = files
            .iter()
            .filter(|(_, v)| v.0 == *unit)
            .map(|(p, _)| p.clone())
            .collect();
        let delta = git.changes(&base, text(&source["commit"])?)?;
        let allowed: BTreeSet<_> = selected.union(&generated_names).cloned().collect();
        need(
            delta.is_subset(&allowed) && selected.is_subset(&delta),
            "omitted or undeclared source delta file",
        )?;
    }
    let mut allowed = generated_names.clone();
    allowed.extend(files.keys().cloned());
    need(
        git.changes(&base, &candidate)? == allowed,
        "candidate contains missing/undeclared changes",
    )?;
    for (p, (_, mode, blob)) in &files {
        let (candidate_mode, candidate_blob) = git.entry(&candidate, p)?;
        need(
            candidate_mode == *mode && candidate_blob == *blob,
            "candidate source file identity/mode mismatch",
        )?;
        git.candidate_file(p, mode, &git.blob(blob)?)?;
    }
    Ok(
        serde_json::json!({"status":"VERIFIED_EXPLICIT_SOURCE_COMPOSITION","base":base,"candidate":candidate,"sources":units.len(),"selected_files":files.len(),"generated_files":generated_names.len(),"whole_program_acceptance":false}),
    )
}

#[cfg(all(test, target_os = "linux", target_arch = "x86_64"))]
mod rr14_open_races {
    use super::read_regular;
    use std::{
        fs,
        path::PathBuf,
        process::{Command, Stdio},
        thread,
        time::{Duration, Instant},
    };
    fn fixture() -> PathBuf {
        let root = PathBuf::from(
            std::env::var_os("SYSON_COMPOSITION_TEST_ROOT").expect("owned fixture root required"),
        );
        assert!(root.is_absolute() && root.starts_with("/workspace") && root.is_dir());
        assert!(
            !root
                .components()
                .any(|p| p == std::path::Component::ParentDir)
        );
        let dir = root.join(format!("rr14-race-{}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        dir
    }
    fn run(name: &str, case: impl FnOnce()) {
        if std::env::var("RR14_PRIVATE_RACE_CHILD").ok().as_deref() == Some(name) {
            case();
            return;
        }
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", name, "--nocapture", "--test-threads=1"])
            .env("RR14_PRIVATE_RACE_CHILD", name)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let start = Instant::now();
        let mut timed_out = false;
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if start.elapsed() >= Duration::from_secs(2) {
                timed_out = true;
                let _ = child.kill();
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            !timed_out,
            "private admission race blocked; child killed and waited"
        );
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("running 1 test") && stdout.contains("1 passed"),
            "race child did not execute its exact case: {stdout}"
        );
    }
    fn replace(kind: &str) {
        let dir = fixture();
        let target = dir.join("manifest.json");
        let replacement = dir.join("replacement");
        fs::write(&target, b"original regular bytes").unwrap();
        match kind {
            "fifo" => {
                let output = Command::new("/usr/bin/mkfifo")
                    .arg("--")
                    .arg(&replacement)
                    .env_clear()
                    .output()
                    .unwrap();
                assert!(output.status.success());
            }
            "symlink" => {
                fs::write(dir.join("referent.json"), b"replacement bytes").unwrap();
                std::os::unix::fs::symlink("referent.json", &replacement).unwrap();
            }
            "regular" => {
                fs::write(&replacement, b"replacement bytes").unwrap();
            }
            _ => unreachable!(),
        }
        let result = read_regular(&target, || {
            fs::rename(&target, dir.join("retained-original.json")).unwrap();
            fs::rename(&replacement, &target).unwrap();
        });
        assert!(result.is_err(), "changed admitted target was accepted");
        assert_eq!(
            fs::read(dir.join("retained-original.json")).unwrap(),
            b"original regular bytes"
        );
    }
    #[test]
    fn regular_to_fifo_after_admission_refuses_without_blocking() {
        run(
            "rr14_open_races::regular_to_fifo_after_admission_refuses_without_blocking",
            || replace("fifo"),
        );
    }
    #[test]
    fn regular_to_symlink_after_admission_refuses() {
        run(
            "rr14_open_races::regular_to_symlink_after_admission_refuses",
            || replace("symlink"),
        );
    }
    #[test]
    fn regular_to_another_inode_after_admission_refuses() {
        run(
            "rr14_open_races::regular_to_another_inode_after_admission_refuses",
            || replace("regular"),
        );
    }
}

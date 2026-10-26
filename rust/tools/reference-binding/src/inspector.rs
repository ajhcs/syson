// SPDX-License-Identifier: EPL-2.0
//! Current local inspection only; no historical-state or publisher attestation.
use crate::{
    BASE, NAMES, ROLES, Result, TARGET, UnverifiedFacts, admit, array, checked, claim, hex, need,
    object, text,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};
pub const JSON_BYTES: u64 = 262_144;
const SOURCE_BYTES: u64 = 1_048_576;
const FILE_BYTES: u64 = 1_073_741_824;

/// Existing root-contained regular file. Internal symlinks resolve to their
/// actual target; classpath directories separately reject every symlink entry.
pub fn contained(root: &Path, raw: &str) -> Result<PathBuf> {
    need(
        raw.len() <= 8192 && !raw.chars().any(char::is_control),
        "invalid file path",
    )?;
    let path = Path::new(raw);
    need(
        path.components()
            .all(|c| matches!(c, Component::RootDir | Component::Normal(_))),
        "path traversal/prefix refused",
    )?;
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let path = fs::canonicalize(path).map_err(|e| format!("missing input {raw}: {e}"))?;
    need(
        path.starts_with(root) && path.is_file(),
        "file/symlink escaped root or is not regular",
    )?;
    Ok(path)
}
fn run(tool: &Path, args: &[String], input: Option<&[u8]>, max: u64) -> Result<Vec<u8>> {
    let mut command = Command::new(tool);
    command
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0");
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_CONFIG_COUNT",
        "GIT_CONFIG_PARAMETERS",
        "GIT_NAMESPACE",
        "GIT_SHALLOW_FILE",
        "GIT_REPLACE_REF_BASE",
    ] {
        command.env_remove(key);
    }
    let mut child = command
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let result: Result<Vec<u8>> = (|| {
        if let Some(bytes) = input {
            child
                .stdin
                .take()
                .ok_or("stdin unavailable")?
                .write_all(bytes)
                .map_err(|e| e.to_string())?;
        }
        let mut out = Vec::new();
        child
            .stdout
            .take()
            .ok_or("stdout unavailable")?
            .take(max + 1)
            .read_to_end(&mut out)
            .map_err(|e| e.to_string())?;
        need(out.len() as u64 <= max, "metadata primitive output limit")?;
        Ok(out)
    })();
    if result.is_err() {
        let _ = child.kill();
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    let out = result?;
    need(status.success(), "Git/SHA256 primitive failed")?;
    Ok(out)
}
fn digest(tool: &Path, path: Option<&Path>, bytes: Option<&[u8]>) -> Result<String> {
    let args = match path {
        Some(p) => vec!["--".into(), p.to_str().ok_or("non-UTF8 path")?.into()],
        None => vec!["-".into()],
    };
    let out = run(tool, &args, bytes, 16384)?;
    let hash = std::str::from_utf8(&out)
        .map_err(|e| e.to_string())?
        .split_whitespace()
        .next()
        .ok_or("empty digest")?;
    need(hex(hash, 64), "invalid SHA256 output")?;
    Ok(hash.into())
}
fn record(f: &mut UnverifiedFacts, raw: &str, hash: String) -> Result<()> {
    need(
        f.files.len() < 256 || f.files.contains_key(raw),
        "file count limit",
    )?;
    f.files.insert(raw.into(), hash);
    Ok(())
}
fn add(root: &Path, raw: &str, sha: &Path, f: &mut UnverifiedFacts, max: u64) -> Result<()> {
    let path = contained(root, raw)?;
    need(
        fs::metadata(&path).map_err(|e| e.to_string())?.len() <= max,
        "file byte limit",
    )?;
    if !f.files.contains_key(raw) {
        record(f, raw, digest(sha, Some(&path), None)?)?;
    }
    Ok(())
}
pub fn read_json(root: &Path, raw: &str) -> Result<(Value, Vec<u8>)> {
    let path = contained(root, raw)?;
    need(
        fs::metadata(&path).map_err(|e| e.to_string())?.len() <= JSON_BYTES,
        "JSON receipt byte limit",
    )?;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(JSON_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    need(bytes.len() as u64 <= JSON_BYTES, "JSON receipt byte limit")?;
    let json = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    Ok((json, bytes))
}
/// Exact complete regular-file inventory, not merely a subset of named hashes.
/// Direct JAR inventory disallows subdirectories; class inventory is recursive.
pub fn exact_inventory(
    root: &Path,
    directory: &Path,
    expected: &BTreeSet<PathBuf>,
    recursive: bool,
) -> Result<()> {
    need(
        !fs::symlink_metadata(directory)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink(),
        "classpath directory symlink refused",
    )?;
    let canonical = fs::canonicalize(directory).map_err(|e| e.to_string())?;
    need(
        canonical.starts_with(root) && canonical == directory && canonical.is_dir(),
        "classpath directory escaped root or has symlink ancestors",
    )?;
    let mut pending = vec![(directory.to_path_buf(), 0)];
    let mut actual = BTreeSet::new();
    let mut count = 0;
    while let Some((dir, depth)) = pending.pop() {
        need(depth <= 32, "classpath directory depth limit")?;
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            count += 1;
            need(count <= 256, "classpath entry limit")?;
            let entry = entry.map_err(|e| e.to_string())?;
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            let path = entry.path();
            need(!kind.is_symlink(), "classpath symlink entry refused")?;
            if kind.is_dir() {
                need(
                    recursive && expected.iter().any(|p| p.starts_with(&path)),
                    "unexpected classpath directory",
                )?;
                pending.push((path, depth + 1));
            } else {
                need(kind.is_file(), "nonregular classpath entry")?;
                actual.insert(path);
            }
        }
    }
    need(
        &actual == expected,
        "extra or missing effective classpath file",
    )
}
fn classpath(root: &Path, review: &Value, execution: &Value, parent: &Path) -> Result<()> {
    let jars: BTreeSet<_> = object(&review["dependency_jar_sha256"])?
        .keys()
        .map(PathBuf::from)
        .collect();
    need(jars.len() == 9, "partial JAR inventory")?;
    let dir = jars
        .first()
        .and_then(|p| p.parent())
        .ok_or("missing JAR directory")?;
    need(
        jars.iter().all(|p| p.parent() == Some(dir)),
        "multiple JAR directories",
    )?;
    exact_inventory(root, dir, &jars, false)?;
    let classes = object(&execution["class_sha256"])?
        .keys()
        .map(|p| parent.join(p))
        .collect();
    exact_inventory(root, &parent.join("classes"), &classes, true)
}
/// Only operator-selected Git/SHA256 primitives execute. No manifest can choose
/// an executable. No shell, compiler/JVM/server/network/semantic fallback runs.
/// Files are sampled individually, not locked into an atomic filesystem snapshot.
pub fn inspect(root: &Path, repo: &Path, request: &Path, git: &Path, sha: &Path) -> Result<Value> {
    for tool in [git, sha] {
        need(
            tool.is_absolute() && tool.is_file(),
            "operator must supply absolute regular metadata primitive paths",
        )?;
    }
    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let repo = fs::canonicalize(repo).map_err(|e| e.to_string())?;
    need(
        repo.starts_with(&root) && repo.is_dir(),
        "repository escaped inspection root",
    )?;
    let (r, request_bytes) = read_json(&root, request.to_str().ok_or("non-UTF8 request")?)?;
    need(
        r["target_revision"] == TARGET && r["claim"] == "selected-source-runtime",
        "invalid pin or claim escalation",
    )?;
    let mut f = UnverifiedFacts::default();
    for role in ROLES {
        let (path, _) = claim(&r, role)?;
        if ["artifact", "harness"].contains(&role) {
            add(
                &root,
                path,
                sha,
                &mut f,
                if role == "harness" {
                    SOURCE_BYTES
                } else {
                    FILE_BYTES
                },
            )?;
        } else {
            let (value, bytes) = read_json(&root, path)?;
            record(&mut f, path, digest(sha, None, Some(&bytes))?)?;
            f.documents.insert(role.into(), value);
        }
    }
    let review = f.documents["review"].clone();
    let execution = f.documents["execution"].clone();
    let parent = Path::new(claim(&r, "execution")?.0)
        .parent()
        .ok_or("missing execution directory")?;
    classpath(&root, &review, &execution, parent)?;
    for key in ["source_sha256", "dependency_jar_sha256", "tools"] {
        for p in object(&review[key])?.keys() {
            add(
                &root,
                p,
                sha,
                &mut f,
                if key == "source_sha256" {
                    SOURCE_BYTES
                } else {
                    FILE_BYTES
                },
            )?;
        }
    }
    for p in object(&execution["source_before"])?.keys() {
        add(&root, p, sha, &mut f, FILE_BYTES)?;
    }
    for p in object(&execution["class_sha256"])?.keys() {
        add(
            &root,
            parent.join(p).to_str().ok_or("non-UTF8 class path")?,
            sha,
            &mut f,
            SOURCE_BYTES,
        )?;
    }
    let mut args = vec![
        "--no-pager".into(),
        "--no-replace-objects".into(),
        "-c".into(),
        "core.hooksPath=/dev/null".into(),
        "-c".into(),
        "core.fsmonitor=false".into(),
        "-c".into(),
        format!("core.worktree={}", repo.display()),
        "-C".into(),
        repo.to_str().ok_or("non-UTF8 repository")?.into(),
    ];
    for source in array(&r["sources"])? {
        let path = text(&source["path"])?;
        let repo_path = text(&source["repo_path"])?;
        need(
            NAMES
                .iter()
                .any(|n| repo_path == format!("{BASE}/{n}.java")),
            "unadmitted target Git source path",
        )?;
        add(&root, path, sha, &mut f, SOURCE_BYTES)?;
        let mut a = args.clone();
        a.extend(["show".into(), format!("{TARGET}:{repo_path}")]);
        let bytes = run(git, &a, None, SOURCE_BYTES)?;
        f.git_sources
            .insert(repo_path.into(), digest(sha, None, Some(&bytes))?);
    }
    let label = text(
        &f.documents["image_config"]["config"]["Labels"]["org.opencontainers.image.revision"],
    )?
    .to_owned();
    need(hex(&label, 40), "invalid artifact label revision")?;
    args.extend([
        "diff".into(),
        "--no-ext-diff".into(),
        "--no-textconv".into(),
        "--name-only".into(),
        "-z".into(),
        TARGET.into(),
        label.clone(),
    ]);
    let out = run(git, &args, None, SOURCE_BYTES)?;
    f.delta = out
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| String::from_utf8(p.to_vec()).map_err(|e| e.to_string()))
        .collect::<Result<_>>()?;
    f.git_revision = TARGET.into();
    f.delta_from = TARGET.into();
    f.delta_to = label;
    let mut output = admit(&r, &f)?;
    need(
        review["status"] == "SCOPED_FINAL_COMPOSITION_INPUT_REVIEW",
        "require new scoped final review; historical receipt cannot be relabeled",
    )?;
    let old = &review["historical_review"];
    let previous = text(&old["path"])?;
    add(&root, previous, sha, &mut f, JSON_BYTES)?;
    checked(&f, previous, text(&old["sha256"])?)?;
    let execution_link = &review["execution_receipt"];
    need(
        text(&execution_link["path"])? == claim(&r, "execution")?.0
            && text(&execution_link["sha256"])? == claim(&r, "execution")?.1,
        "final review execution provenance mismatch",
    )?;
    output["admission"] = json!("CURRENT_LOCAL_INPUTS_INSPECTED_WITH_OPEN_PARENT_C01");
    output["file_inspection_performed"] = json!(true);
    output["inspected_file_count"] = json!(f.files.len());
    output["request_sha256"] = json!(digest(sha, None, Some(&request_bytes))?);
    output["current_classpath_inventory"] =
        json!({"jars":9,"classes":5,"extras":"REFUSED","symlinks":"REFUSED"});
    output["inspection_boundary"] = json!(
        "Current retained input consistency only; no atomic snapshot, historical dependency immutability, retroactive approval, receipt authenticity or whole-artifact equivalence."
    );
    Ok(output)
}

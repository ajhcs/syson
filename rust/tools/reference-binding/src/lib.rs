// SPDX-License-Identifier: EPL-2.0
//! Local receipt admission, not publisher authentication or whole-artifact equivalence.
#![forbid(unsafe_code)]
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
pub const TARGET: &str = "6ba2ac64e6b16ee2b05752ad55395eebb87d4910";
pub const NAMES: [&str; 4] = [
    "AnnotationAnnotatingElementMigrationParticipant",
    "ConnectorAsUsageSourceAndTargetMigrationParticipant",
    "MigrationParticipantService",
    "ViewUsageExposeElementMigrationParticipant",
];
const BASE: &str = "backend/application/syson-application-configuration/src/main/java/org/eclipse/syson/application/migration";
const ROLES: [&str; 7] = [
    "artifact",
    "harness",
    "reference",
    "image_config",
    "image_manifest",
    "review",
    "execution",
];
type Result<T> = std::result::Result<T, String>;
#[derive(Default, Clone)]
pub struct UnverifiedFacts {
    pub files: BTreeMap<String, String>,
    pub documents: BTreeMap<String, Value>,
    pub git_sources: BTreeMap<String, String>,
    pub git_revision: String,
    pub delta_from: String,
    pub delta_to: String,
    pub delta: Vec<String>,
}
fn need(ok: bool, message: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(message.into()) }
}
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or_else(|| "required string missing".into())
}
fn object(v: &Value) -> Result<&serde_json::Map<String, Value>> {
    v.as_object()
        .ok_or_else(|| "required object missing".into())
}
fn array(v: &Value) -> Result<&Vec<Value>> {
    v.as_array().ok_or_else(|| "required array missing".into())
}
fn hex(s: &str, n: usize) -> bool {
    s.len() == n
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn checked(f: &UnverifiedFacts, path: &str, sha: &str) -> Result<()> {
    need(
        hex(sha, 64) && f.files.get(path).is_some_and(|h| h == sha),
        &format!("missing or changed input: {path}"),
    )
}
fn claim<'a>(r: &'a Value, role: &str) -> Result<(&'a str, &'a str)> {
    Ok((
        text(&r["files"][role]["path"])?,
        text(&r["files"][role]["sha256"])?,
    ))
}
fn argv(v: &Value) -> Result<Vec<&str>> {
    array(&v["argv"])?.iter().map(text).collect()
}
/// Pure validation of supplied unverified facts. This core performs no I/O and does not establish
/// that any file or Git object was inspected, or that a receipt is authentic.
pub fn admit(r: &Value, f: &UnverifiedFacts) -> Result<Value> {
    let keys: BTreeSet<_> = object(r)?.keys().map(String::as_str).collect();
    need(
        keys == BTreeSet::from(["schema", "target_revision", "claim", "files", "sources"]),
        "unexpected/missing request fields",
    )?;
    need(
        r["schema"] == "syson-reference-binding/1" && r["claim"] == "selected-source-runtime",
        "only selected-source-runtime claim is supported; C01 remains OPEN",
    )?;
    need(
        hex(text(&r["target_revision"])?, 40) && r["target_revision"] == TARGET,
        "stale or invalid target source pin",
    )?;
    need(
        object(&r["files"])?
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            == BTreeSet::from(ROLES),
        "missing or extra evidence roles",
    )?;
    for role in ROLES {
        let (p, h) = claim(r, role)?;
        checked(f, p, h)?;
    }
    let doc = |role: &str| {
        f.documents
            .get(role)
            .ok_or_else(|| format!("missing parsed receipt: {role}"))
    };
    let reference = doc("reference")?;
    let config = doc("image_config")?;
    let manifest = doc("image_manifest")?;
    let review = doc("review")?;
    let execution = doc("execution")?;
    let label = text(&config["config"]["Labels"]["org.opencontainers.image.revision"])?;
    need(
        hex(label, 40)
            && config["config"]["Labels"] == reference["labels"]
            && reference["source_pin"] == TARGET,
        "artifact/source label or target observation mismatch",
    )?;
    need(
        reference["jar_sha256"] == claim(r, "artifact")?.1
            && reference["manifest_sha256"] == claim(r, "image_manifest")?.1,
        "official artifact/manifest hash relationship mismatch",
    )?;
    let digest = format!("sha256:{}", claim(r, "image_config")?.1);
    need(
        reference["config_digest"] == digest && manifest["config"]["digest"] == digest,
        "effective image config digest mismatch",
    )?;
    need(
        f.git_revision == TARGET && f.delta_from == TARGET && f.delta_to == label,
        "stale Git source or revision-pair observations",
    )?;
    let expected_delta: BTreeSet<_> = array(&reference["source_delta_paths"])?
        .iter()
        .map(text)
        .collect::<Result<_>>()?;
    need(
        f.delta.iter().map(String::as_str).collect::<BTreeSet<_>>() == expected_delta
            && f.delta.len() == expected_delta.len(),
        "Git source-label delta disagrees with reference receipt",
    )?;
    need(
        review["source_pin"] == TARGET
            && review["annotation_processing"] == "DISABLED -proc:none"
            && review["compile_target"] == "--release21",
        "missing/mismatched selected effective compile configuration",
    )?;
    let deps = object(&review["dependency_jar_sha256"])?;
    need(
        deps.len() == 9,
        "partial dependency proof: require all nine reviewed JARs",
    )?;
    for (p, h) in deps {
        checked(f, p, text(h)?)?;
    }
    let reviewed_sources = object(&review["source_sha256"])?;
    need(
        reviewed_sources.len() == 4 && array(&r["sources"])?.len() == 4,
        "partial selected source proof: require all four classes",
    )?;
    let before = object(&execution["source_before"])?;
    need(
        execution["source_unchanged"] == true
            && execution["status"] == "PASS_FOUR_MATCHED_VERSION_ORDERED_COMPOSITIONS",
        "execution did not record unchanged successful selected inputs",
    )?;
    for (p, h) in before {
        checked(f, p, text(h)?)?;
    }
    let mut seen = BTreeSet::new();
    for s in array(&r["sources"])? {
        let repo = text(&s["repo_path"])?;
        let path = text(&s["path"])?;
        let sha = text(&s["sha256"])?;
        let name = NAMES
            .iter()
            .find(|n| repo == format!("{BASE}/{n}.java"))
            .ok_or("unadmitted selected source path")?;
        need(seen.insert(*name), "duplicate selected source coverage")?;
        checked(f, path, sha)?;
        need(
            f.git_sources.get(repo).is_some_and(|h| h == sha)
                && before.get(path).is_some_and(|h| h == sha),
            "compiled source differs from target Git object or captured input",
        )?;
        let reviewed = reviewed_sources
            .iter()
            .find(|(p, _)| {
                Path::new(p)
                    .file_name()
                    .is_some_and(|p| p == format!("{name}.java").as_str())
            })
            .ok_or("selected source absent from review")?;
        checked(f, reviewed.0, text(reviewed.1)?)?;
        need(
            reviewed.1 == sha,
            "reviewed and compiled selected sources differ",
        )?;
    }
    let commands = array(&execution["commands"])?;
    need(
        commands.len() == 4 && commands.iter().all(|c| c["exit_code"] == 0),
        "partial/failed recorded execution",
    )?;
    let compile = argv(&commands[1])?;
    let runtime = argv(&commands[3])?;
    let tools = object(&review["tools"])?;
    for (p, h) in tools {
        checked(f, p, text(h)?)?;
    }
    need(
        compile.len() == 15 && runtime.len() == 9 && tools.len() == 2,
        "selected compile/runtime argv missing",
    )?;
    need(
        tools.contains_key(compile[0])
            && compile[0].ends_with("/javac")
            && tools.contains_key(runtime[0])
            && runtime[0].ends_with("/java"),
        "unbound compiler/runtime executable",
    )?;
    let exec_path = Path::new(claim(r, "execution")?.0);
    let parent = exec_path
        .parent()
        .ok_or("execution receipt has no parent")?;
    let class_dir = parent.join("classes");
    let class_dir = class_dir.to_str().ok_or("non-UTF8 class directory")?;
    let dep_dir = Path::new(deps.keys().next().ok_or("no dependencies")?)
        .parent()
        .ok_or("no JAR directory")?;
    need(
        deps.keys().all(|p| Path::new(p).parent() == Some(dep_dir)),
        "dependency directory mismatch",
    )?;
    let cp = format!("{}/*", dep_dir.display());
    need(
        compile[1..10]
            == [
                "-J-XX:ActiveProcessorCount=2",
                "-J-Xmx256m",
                "-proc:none",
                "--release",
                "21",
                "-cp",
                cp.as_str(),
                "-d",
                class_dir,
            ],
        "selected effective compiler argv mismatch",
    )?;
    let harness = claim(r, "harness")?;
    need(
        before.get(harness.0).is_some_and(|h| h == harness.1),
        "unbound oracle harness",
    )?;
    need(
        review["harness_sha256"] == harness.1,
        "stale review harness: require a new scoped final review matching compiled oracle",
    )?;
    let mut source_args: BTreeSet<_> = array(&r["sources"])?
        .iter()
        .map(|s| text(&s["path"]))
        .collect::<Result<_>>()?;
    source_args.insert(harness.0);
    need(
        compile[10..].iter().copied().collect::<BTreeSet<_>>() == source_args,
        "compile source list mismatch",
    )?;
    let class_cp = format!("{class_dir}:{cp}");
    need(
        runtime[1] == "-Xmx256m"
            && runtime[2] == "-XX:ActiveProcessorCount=2"
            && runtime[3].starts_with("-Djava.io.tmpdir=")
            && runtime[4..8] == ["-cp", class_cp.as_str(), "MigrationOracle", "sequence"],
        "selected effective runtime argv mismatch",
    )?;
    need(
        before.contains_key(runtime[8]),
        "runtime fixture input not captured",
    )?;
    let classes = object(&execution["class_sha256"])?;
    let expected_classes: BTreeSet<_> = NAMES
        .iter()
        .map(|n| format!("classes/org/eclipse/syson/application/migration/{n}.class"))
        .chain(["classes/MigrationOracle.class".into()])
        .collect();
    need(
        classes.keys().cloned().collect::<BTreeSet<_>>() == expected_classes,
        "partial compiled class coverage",
    )?;
    for (p, h) in classes {
        checked(
            f,
            parent.join(p).to_str().ok_or("non-UTF8 class path")?,
            text(h)?,
        )?;
    }
    Ok(
        json!({"admission":"POLICY_CONSISTENT_WITH_UNVERIFIED_FACTS","file_inspection_performed":false,"C01":"OPEN","whole_artifact_equivalence":false,"target_revision":TARGET,"artifact_label_revision":label,"label_matches_target":label==TARGET,"selected_source_classes":4,"identified_dependency_jars":9,"source_label_delta_paths":f.delta,"remaining_requirements":["Provide a reviewed Maven toolchain, Java21, effective POM/settings/profiles and complete dependency/source checksums, including SNAPSHOT dependencies.","Reproduce frontend npm ci/build and copied static assets using pinned lockfiles/toolchains; capture backend clean verify inputs and outputs at target revision.","Bind rebuilt application/image to target revision or explicitly resolve every source-label delta; matching version strings do not establish equivalence.","Close inherited Sirius/EMF/Syside, standard libraries, effective schema/routes/UI extensions and DB changelogs under parent C01."],"boundary":"Local file/Git/receipt consistency only; not receipt authenticity, complete artifact rebuild, native deserialization, SQL or UI acceptance."}),
    )
}

pub mod inspector;

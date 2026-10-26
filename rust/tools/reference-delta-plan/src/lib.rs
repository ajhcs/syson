// SPDX-License-Identifier: EPL-2.0
//! Source-aware recipe planning over the coordinator's actual Git/ZIP capture.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::Path;
pub const MAX_BYTES: usize = 8 * 1024 * 1024;
pub const LABEL: &str = "b380dada415ce05155150f23cb3b74ae21cc45e4";
pub const TARGET: &str = "6ba2ac64e6b16ee2b05752ad55395eebb87d4910";
pub const ARTIFACT: &str = "85c031352695d6ac8f059f0215ca493c58a16805a111233998dd443d6990c9e1";
const MODULES: [&str; 3] = [
    "backend/metamodel/syson-sysml-metamodel-edit",
    "backend/views/syson-diagram-common-view",
    "backend/views/syson-standard-diagrams-view",
];
const CLASSES: [(usize, &str); 7] = [
    (0, "org/eclipse/syson/sysml/provider/ItemUsageItemProvider"),
    (
        0,
        "org/eclipse/syson/sysml/provider/ReferenceUsageItemProvider",
    ),
    (
        1,
        "org/eclipse/syson/diagram/common/view/nodes/AbstractReferenceUsageBorderNodeDescriptionProvider",
    ),
    (
        1,
        "org/eclipse/syson/diagram/common/view/nodes/ForkActionNodeDescriptionProvider",
    ),
    (
        1,
        "org/eclipse/syson/diagram/common/view/nodes/JoinActionNodeDescriptionProvider",
    ),
    (
        1,
        "org/eclipse/syson/diagram/common/view/tools/ParameterCompartmentNodeToolProvider",
    ),
    (
        2,
        "org/eclipse/syson/standard/diagrams/view/SDVDescriptionProvider",
    ),
];
const DOCS: [&str; 4] = [
    "CHANGELOG.adoc",
    "doc/content/modules/user-manual/pages/release-notes/2026.11.0.adoc",
    "doc/content/modules/user-manual/assets/images/release-notes-diagram-sysml-icons-hamonization.png",
    "doc/content/modules/user-manual/assets/images/release-notes-explorer-sysml-icons-harmonization.png",
];
type Result<T> = std::result::Result<T, String>;
fn need(ok: bool, why: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(why.into()) }
}
fn fields(v: &Value, keys: &str) -> Result<()> {
    let m = v.as_object().ok_or("object required")?;
    let mut keys = keys.split_ascii_whitespace();
    need(
        m.len() == keys.clone().count() && keys.all(|k| m.contains_key(k)),
        "unknown/missing packet fields",
    )
}
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or("string required".into())
}
fn number(v: &Value) -> Result<u64> {
    v.as_u64().ok_or("unsigned integer required".into())
}
fn array(v: &Value, max: usize) -> Result<&Vec<Value>> {
    let a = v.as_array().ok_or("array required")?;
    need(a.len() <= max, "packet item limit")?;
    Ok(a)
}
fn digest(v: &Value, n: usize) -> Result<()> {
    let s = text(v)?;
    need(
        s.len() == n
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "invalid lowercase digest",
    )
}
fn path(v: &Value) -> Result<&str> {
    let s = text(v)?;
    let plain = s.strip_suffix('/').unwrap_or(s);
    need(
        s.len() <= 1024
            && !s.contains(['\\', ':'])
            && !s.chars().any(char::is_control)
            && plain
                .split('/')
                .all(|p| !p.is_empty() && p != "." && p != ".."),
        "invalid archive/source path",
    )?;
    Ok(s)
}
fn unsigned(v: &Value) -> Result<()> {
    for key in ["signatures", "manifest_digests"] {
        need(
            array(&v[key], 128)?.is_empty(),
            "signed/digest-bearing changed archive refused",
        )?;
    }
    Ok(())
}
fn source(v: &Value, target: bool) -> Result<()> {
    if target {
        fields(v, "blob mode sha256 bytes text")?;
    } else {
        fields(v, "blob mode sha256 bytes")?;
    }
    digest(&v["blob"], 40)?;
    digest(&v["sha256"], 64)?;
    need(
        v["mode"] == "100644" && number(&v["bytes"])? <= 16 * 1024 * 1024,
        "unsupported source mode/size",
    )
}
fn member(v: &Value) -> Result<()> {
    path(&v["entry"])?;
    digest(&v["sha256"], 64)?;
    digest(&v["metadata_sha256"], 64)?;
    number(&v["bytes"])?;
    Ok(())
}
fn family(entry: &str, root: &str) -> bool {
    entry.strip_prefix(root).is_some_and(|s| {
        s == ".class" || (s.starts_with('$') && s.ends_with(".class") && !s.contains('/'))
    })
}
fn jar(module: usize) -> String {
    format!(
        "BOOT-INF/lib/{}-2026.9.3.jar",
        MODULES[module].rsplit('/').next().unwrap_or("")
    )
}
fn set(v: &Value, max: usize) -> Result<BTreeSet<&str>> {
    let a = array(v, max)?;
    let s = a.iter().map(path).collect::<Result<BTreeSet<_>>>()?;
    need(s.len() == a.len(), "duplicate scan identity")?;
    Ok(s)
}
/// No ZIP/compiler execution: provenance and full repack byte/metadata checks are
/// mandatory coordinator gates. This planner derives a recipe, not attestation.
pub fn plan(bytes: &[u8]) -> Result<Value> {
    need(bytes.len() <= MAX_BYTES, "packet byte limit")?;
    let v: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    fields(&v, "schema artifact delta containers scan hits compiled")?;
    need(v["schema"] == 1, "unsupported packet schema")?;
    let artifact = &v["artifact"];
    fields(
        artifact,
        "sha256 source_revision target_revision signatures manifest_digests non_signature_markers",
    )?;
    need(
        artifact["sha256"] == ARTIFACT
            && artifact["source_revision"] == LABEL
            && artifact["target_revision"] == TARGET,
        "wrong artifact/source pins",
    )?;
    unsigned(artifact)?;
    let mut marker = json!({"entry":"META-INF/BOOT.SF","bytes":0,"sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"});
    need(
        artifact["non_signature_markers"] == json!([marker]),
        "only exact empty BOOT.SF marker admitted",
    )?;
    marker["disposition"] = json!("PRESERVE_UNCHANGED_EMPTY_BOOT_MARKER");
    let jars: BTreeSet<_> = (0..3).map(jar).collect();
    let mut containers = BTreeMap::new();
    let mut indexes = BTreeMap::new();
    let mut member_total = 0;
    for c in array(&v["containers"], 312)? {
        fields(
            c,
            "entry sha256 members signatures manifest_digests compression",
        )?;
        let name = path(&c["entry"])?;
        digest(&c["sha256"], 64)?;
        need(
            name.starts_with("BOOT-INF/lib/")
                && name.ends_with(".jar")
                && number(&c["compression"])? == 0,
            "nested JAR must remain STORED",
        )?;
        need(containers.insert(name, c).is_none(), "duplicate container")?;
        if jars.contains(name) {
            unsigned(c)?;
            let mut index = BTreeMap::new();
            for m in array(&c["members"], 20000)? {
                fields(m, "entry sha256 bytes metadata_sha256")?;
                member(m)?;
                member_total += 1;
                need(
                    member_total <= 20000 && index.insert(path(&m["entry"])?, m).is_none(),
                    "duplicate/oversized member index",
                )?;
            }
            indexes.insert(name, index);
        } else {
            need(c["members"].is_null(), "unexpected full member index")?;
        }
    }
    need(
        containers.len() == 312 && indexes.len() == 3,
        "incomplete container/index inventory",
    )?;
    let mut scan_containers: BTreeSet<_> = containers.keys().copied().collect();
    scan_containers.insert("BOOT-INF/classes");
    fields(&v["scan"], "containers queries")?;
    need(
        set(&v["scan"]["containers"], 313)? == scan_containers,
        "incomplete global membership scan",
    )?;
    let mut delta = BTreeMap::new();
    let mut kinds = BTreeMap::new();
    let mut queries = BTreeSet::new();
    let mut added = 0;
    for row in array(&v["delta"], 125)? {
        fields(row, "path status label target")?;
        let p = path(&row["path"])?;
        need(delta.insert(p, row).is_none(), "duplicate source delta")?;
        source(&row["target"], true)?;
        if row["status"] == "A" {
            need(
                row["label"].is_null(),
                "added source must have no label blob",
            )?;
            added += 1;
        } else {
            need(row["status"] == "M", "unsupported delta direction/status")?;
            source(&row["label"], false)?;
        }
        if DOCS.contains(&p) {
            need(row["target"]["text"].is_null(), "unexpected document text")?;
            continue;
        }
        let (module, relative) = MODULES
            .iter()
            .enumerate()
            .find_map(|(n, prefix)| {
                p.strip_prefix(&format!("{prefix}/src/main/"))
                    .map(|r| (n, r))
            })
            .ok_or("unknown source module")?;
        let kind = if let Some(class) = relative
            .strip_prefix("java/")
            .and_then(|s| s.strip_suffix(".java"))
        {
            need(
                CLASSES.contains(&(module, class)) && row["status"] == "M",
                "unexpected Java family",
            )?;
            let code = text(&row["target"]["text"])?;
            need(
                code.len() as u64 == number(&row["target"]["bytes"])? && code.len() <= 1024 * 1024,
                "Java source length mismatch",
            )?;
            let package = format!(
                "package {};",
                class
                    .rsplit_once('/')
                    .ok_or("missing Java package")?
                    .0
                    .replace('/', ".")
            );
            let packages: Vec<_> = code
                .lines()
                .map(str::trim)
                .filter(|l| l.starts_with("package "))
                .collect();
            need(
                packages == vec![package.as_str()],
                "source package/binary path mismatch",
            )?;
            ("family", class)
        } else {
            let entry = relative
                .strip_prefix("resources/")
                .filter(|s| s.ends_with(".svg"))
                .ok_or("unexpected runtime resource")?;
            need(
                module < 2 && row["target"]["text"].is_null(),
                "unexpected resource source",
            )?;
            ("resource", entry)
        };
        queries.insert(p);
        kinds.insert(p, (module, kind.0, kind.1));
    }
    need(
        delta.len() == 125
            && queries.len() == 121
            && added == 9
            && DOCS.iter().all(|p| delta.contains_key(p)),
        "incomplete selected source delta",
    )?;
    need(
        set(&v["scan"]["queries"], 121)? == queries,
        "incomplete selected member queries",
    )?;
    let mut hits: BTreeMap<&str, Vec<&Value>> = BTreeMap::new();
    let mut hit_ids = BTreeSet::new();
    for h in array(&v["hits"], 1024)? {
        fields(
            h,
            "source_path container entry sha256 bytes metadata_sha256",
        )?;
        member(h)?;
        let p = path(&h["source_path"])?;
        let (module, _, _) = kinds.get(p).ok_or("unknown membership query")?;
        let container = path(&h["container"])?;
        need(
            container == jar(*module),
            "ambiguous/colliding member container",
        )?;
        need(
            hit_ids.insert((container, path(&h["entry"])?)),
            "duplicate member hit",
        )?;
        let old = indexes[container]
            .get(path(&h["entry"])?)
            .ok_or("stale member hit")?;
        need(
            ["sha256", "bytes", "metadata_sha256"]
                .iter()
                .all(|k| old[*k] == h[*k]),
            "stale member hit metadata",
        )?;
        hits.entry(p).or_default().push(h);
    }
    let mut compiled: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut outputs = BTreeSet::new();
    for c in array(&v["compiled"], 128)? {
        fields(c, "source_path entry sha256 bytes major")?;
        digest(&c["sha256"], 64)?;
        let p = path(&c["source_path"])?;
        let (_, kind, root) = kinds.get(p).ok_or("unexpected compiled source")?;
        let entry = path(&c["entry"])?;
        need(
            *kind == "family"
                && family(entry, root)
                && number(&c["major"])? == 65
                && number(&c["bytes"])? <= 16 * 1024 * 1024,
            "unexpected compiled output/class version",
        )?;
        need(outputs.insert(entry), "colliding compiled output")?;
        compiled.entry(p).or_default().insert(entry);
    }
    let mut resources = Vec::new();
    let mut families = Vec::new();
    let mut resource_counts = [0; 2];
    let mut additions = 0;
    for (p, (module, kind, entry)) in kinds {
        let container = jar(module);
        let found = hits.remove(p).unwrap_or_default();
        let index = &indexes[container.as_str()];
        if kind == "resource" {
            need(
                !index.keys().any(|n| {
                    n.strip_prefix("META-INF/versions/")
                        .and_then(|s| s.split_once('/'))
                        .is_some_and(|(_, r)| r == entry)
                }),
                "multi-release SVG shadow refused",
            )?;
            let row = delta[p];
            let old = if row["status"] == "A" {
                additions += 1;
                need(
                    found.is_empty() && !index.contains_key(entry),
                    "added SVG already exists",
                )?;
                Value::Null
            } else {
                need(
                    found.len() == 1
                        && found[0]["entry"] == entry
                        && found[0]["sha256"] == row["label"]["sha256"]
                        && found[0]["bytes"] == row["label"]["bytes"],
                    "missing/mismatched label SVG",
                )?;
                row["label"]["sha256"].clone()
            };
            resource_counts[module] += 1;
            resources.push(json!({"source_path":p,"container":container,"entry":entry,"old_sha256":old,"new_sha256":row["target"]["sha256"]}));
        } else {
            let old: BTreeSet<_> = found
                .iter()
                .map(|h| text(&h["entry"]))
                .collect::<Result<_>>()?;
            let full: BTreeSet<_> = index.keys().copied().filter(|n| family(n, entry)).collect();
            need(
                old == full && old.contains(format!("{entry}.class").as_str()),
                "incomplete old class family",
            )?;
            need(
                !index.keys().any(|n| {
                    n.strip_prefix("META-INF/versions/")
                        .and_then(|s| s.split_once('/'))
                        .is_some_and(|(_, r)| family(r, entry))
                }),
                "multi-release family refused",
            )?;
            need(
                old.iter().all(|n| family(n, entry)),
                "unexpected/versioned family hit",
            )?;
            let new = compiled.remove(p).ok_or("missing compiled family")?;
            need(
                new.contains(format!("{entry}.class").as_str()),
                "compiled top class missing",
            )?;
            families.push(json!({"source_path":p,"container":container,"root":entry,"old_members":old,"new_members":new}));
        }
    }
    need(
        resources.len() == 114
            && families.len() == 7
            && additions == 7
            && resource_counts == [102, 12]
            && compiled.is_empty(),
        "unexpected source-operation coverage",
    )?;
    let docs: Vec<_> = DOCS
        .iter()
        .map(|p| json!({"source_path":p,"disposition":"DOCUMENTATION_ONLY_NOT_OVERLAID"}))
        .collect();
    let unchanged: Vec<_> = containers
        .iter()
        .filter(|(n, _)| !jars.contains(**n))
        .map(|(n, c)| json!({"entry":n,"sha256":c["sha256"]}))
        .collect();
    Ok(
        json!({"schema":1,"artifact_sha256":ARTIFACT,"source_revision":LABEL,"target_revision":TARGET,"resources":resources,"families":families,"docs":docs,"unchanged_containers":unchanged,"claim":"SOURCE_DELTA_DERIVED_REFERENCE_ONLY","C01":"OPEN","whole_build_source_binding":"OPEN","independent_build":false,"non_signature_markers":[marker]}),
    )
}
pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    need(
        std::fs::symlink_metadata(path)
            .map_err(|e| e.to_string())?
            .is_file(),
        "regular nonsymlink packet required",
    )?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    need(bytes.len() <= MAX_BYTES, "packet byte limit")?;
    Ok(bytes)
}

// SPDX-License-Identifier: EPL-2.0
use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};
use syson_native_library_catalog::{Catalog, LibrarySource, MAX_FILE, MAX_TOTAL, manifest};
fn checked_path(root: &Path, name: &str) -> Result<PathBuf, String> {
    let relative = Path::new(name);
    if relative
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
        || relative.as_os_str().is_empty()
    {
        return Err("relative normal file path required".into());
    }
    let target = root.join(relative);
    for ancestor in target.ancestors() {
        if fs::symlink_metadata(ancestor)
            .map_err(|e| e.to_string())?
            .is_symlink()
        {
            return Err("symlink refused".into());
        }
    }
    Ok(target)
}
fn read(root: &Path, name: &str, limit: usize) -> Result<Vec<u8>, String> {
    let target = checked_path(root, name)?;
    let metadata = fs::symlink_metadata(&target).map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > limit as u64 {
        return Err("regular bounded file required".into());
    }
    let mut raw = Vec::new();
    fs::File::open(target)
        .map_err(|e| e.to_string())?
        .take(limit as u64 + 1)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    if raw.len() > limit {
        return Err("file byte limit".into());
    }
    Ok(raw)
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: syson-native-library-catalog ROOT RELATIVE_MANIFEST".into());
    }
    let root = Path::new(&args[0]);
    if !root.is_absolute() || !root.is_dir() {
        return Err("absolute owned root required".into());
    }
    let name = args[1].to_str().ok_or("manifest UTF8 path required")?;
    let raw = read(root, name, 256 * 1024)?;
    let entries = manifest(std::str::from_utf8(&raw).map_err(|e| e.to_string())?)?;
    let mut total = 0u64;
    for e in &entries {
        let m = fs::symlink_metadata(checked_path(root, &e.file)?).map_err(|e| e.to_string())?;
        if !m.is_file() || m.len() > MAX_FILE as u64 {
            return Err("bounded regular payload required".into());
        }
        total += m.len();
        if total > MAX_TOTAL as u64 {
            return Err("aggregate byte limit".into());
        }
    }
    let mut payloads = Vec::new();
    total = 0;
    for e in &entries {
        let raw = read(root, &e.file, MAX_FILE)?;
        total += raw.len() as u64;
        if total > MAX_TOTAL as u64 {
            return Err("aggregate byte limit".into());
        }
        payloads.push(String::from_utf8(raw).map_err(|e| e.to_string())?);
    }
    let inputs: Vec<_> = entries
        .iter()
        .zip(&payloads)
        .map(|(e, json)| LibrarySource {
            classpath_path: &e.classpath_path,
            resource_uri: &e.resource_uri,
            json,
        })
        .collect();
    let catalog = Catalog::from_resources(&inputs)?;
    let mut out = std::io::stdout().lock();
    out.write_all(b"{\"claim\":\"PHYSICAL_LIBRARY_CATALOGUE_ONLY\",\"resources\":[")
        .map_err(|e| e.to_string())?;
    for (i, r) in catalog.resources().iter().enumerate() {
        if i != 0 {
            out.write_all(b",").map_err(|e| e.to_string())?;
        }
        let nodes: Vec<_> = r.nodes().iter().map(|n| serde_json::json!({"node_id":n.node_id,"semantic_id":n.semantic_id,"e_class":n.e_class,"declared_name":n.declared_name,"pointer":n.pointer})).collect();
        serde_json::to_writer(&mut out, &serde_json::json!({"classpath_path":r.classpath_path,"resource_uri":r.resource_uri,"nodes":nodes})).map_err(|e| e.to_string())?;
    }
    out.write_all(b"]}\n").map_err(|e| e.to_string())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("catalogue refused: {e}");
        std::process::exit(1);
    }
}

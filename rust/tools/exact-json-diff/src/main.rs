// SPDX-License-Identifier: EPL-2.0
#![forbid(unsafe_code)]
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use syson_exact_json_diff::{MAX_INPUT, compare};
fn read(path: &Path) -> Result<Vec<u8>, String> {
    let m = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !m.is_file() || m.len() > MAX_INPUT as u64 {
        return Err("bounded regular nonsymlink file required".into());
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_INPUT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    Ok(bytes)
}
fn run() -> Result<i32, String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: syson-exact-json-diff LEFT.json RIGHT.json".into());
    }
    let report = compare(&read(Path::new(&args[0]))?, &read(Path::new(&args[1]))?)?;
    let code = if report.is_match() { 0 } else { 1 };
    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer(&mut stdout, &report.into_json()).map_err(|e| e.to_string())?;
    stdout.write_all(b"\n").map_err(|e| e.to_string())?;
    Ok(code)
}
fn main() {
    let code = match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("comparison refused: {e}");
            2
        }
    };
    std::process::exit(code);
}

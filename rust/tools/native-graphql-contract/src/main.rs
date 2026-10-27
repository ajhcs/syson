// SPDX-License-Identifier: EPL-2.0
#![forbid(unsafe_code)]
use std::io::{Read, Write};
use syson_native_graphql_contract::{CAPABILITIES, MAX_BYTES, QUERY, inspect};
fn execute() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let arg = args
        .next()
        .ok_or("usage: syson-native-graphql-contract RESPONSE_JSON | --query | --capabilities")?;
    if args.next().is_some() {
        return Err("expected one argument".into());
    }
    if arg == "--query" {
        print!("{QUERY}");
        return Ok(());
    }
    if arg == "--capabilities" {
        print!("{CAPABILITIES}");
        return Ok(());
    }
    if !std::fs::symlink_metadata(&arg)
        .map_err(|e| e.to_string())?
        .is_file()
    {
        return Err("input must be a regular nonsymlink file".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(arg)
        .map_err(|e| e.to_string())?
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_BYTES {
        return Err("response byte budget exceeded".into());
    }
    let output = inspect(std::str::from_utf8(&bytes).map_err(|e| e.to_string())?)?;
    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer(&mut stdout, &output).map_err(|e| e.to_string())?;
    stdout.write_all(b"\n").map_err(|e| e.to_string())?;
    Ok(())
}
fn main() {
    if let Err(error) = execute() {
        eprintln!("schema contract refused: {error}");
        std::process::exit(1);
    }
}

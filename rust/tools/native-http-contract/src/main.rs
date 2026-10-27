#![forbid(unsafe_code)]
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use syson_native_http_contract::{MAX_INPUT, PROBES, inspect_mappings, inspect_probes};
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 1 && args[0] == "--probes" {
        return std::io::stdout()
            .lock()
            .write_all(PROBES.as_bytes())
            .map_err(|e| e.to_string());
    }
    if args.len() != 2 || (args[0] != "mappings" && args[0] != "probes") {
        return Err(
            "usage: syson-native-http-contract mappings|probes FILE; --probes emits fixed manifest"
                .into(),
        );
    }
    let path = Path::new(&args[1]);
    let metadata = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_INPUT as u64 {
        return Err("bounded regular nonsymlink input required".into());
    }
    let mut input = String::new();
    File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_INPUT as u64 + 1)
        .read_to_string(&mut input)
        .map_err(|e| e.to_string())?;
    let output = if args[0] == "mappings" {
        inspect_mappings(&input)?
    } else {
        inspect_probes(&input)?
    };
    let mut stdout = std::io::stdout().lock();
    serde_json::to_writer(&mut stdout, &output).map_err(|e| e.to_string())?;
    stdout.write_all(b"\n").map_err(|e| e.to_string())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

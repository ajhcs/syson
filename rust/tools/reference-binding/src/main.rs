// SPDX-License-Identifier: EPL-2.0
#![forbid(unsafe_code)]
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 5 {
        eprintln!("usage: syson-reference-binding ROOT REPOSITORY REQUEST_JSON GIT SHA256SUM");
        std::process::exit(2);
    }
    let paths: Vec<_> = args.iter().map(std::path::Path::new).collect();
    match syson_reference_binding::inspector::inspect(
        paths[0], paths[1], paths[2], paths[3], paths[4],
    ) {
        Ok(output) => println!(
            "{}",
            serde_json::to_string_pretty(&output).expect("JSON value serializes")
        ),
        Err(error) => {
            eprintln!("reference binding refused: {error}");
            std::process::exit(1);
        }
    }
}

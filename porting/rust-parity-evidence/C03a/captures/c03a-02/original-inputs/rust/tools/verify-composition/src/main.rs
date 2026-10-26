// SPDX-License-Identifier: EPL-2.0
use std::{env, path::Path};
fn main() {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let result = if args.len() == 2 {
        syson_verify_composition::read_file(Path::new(&args[1]))
            .and_then(|bytes| syson_verify_composition::verify(Path::new(&args[0]), &bytes))
    } else {
        Err("usage: syson-verify-composition <candidate-worktree> <manifest.json>".into())
    };
    match result {
        Ok(value) => println!("{value}"),
        Err(message) => {
            eprintln!("composition refused: {message}");
            std::process::exit(1);
        }
    }
}

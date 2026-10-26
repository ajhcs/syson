// SPDX-License-Identifier: EPL-2.0
use syson_native_db_contract::{project, read_file};
fn main() {
    let result = (|| -> Result<_, String> {
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        if args.len() != 1 {
            return Err("usage: syson-native-db-contract <native-catalog.json>".into());
        }
        project(&read_file(std::path::Path::new(&args[0]))?)
    })();
    match result {
        Ok(value) => println!("{value}"),
        Err(error) => {
            eprintln!("catalog projection refused: {error}");
            std::process::exit(1);
        }
    }
}

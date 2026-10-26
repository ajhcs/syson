// SPDX-License-Identifier: EPL-2.0
fn main() {
    let result = (|| -> Result<_, String> {
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        if args.len() != 1 {
            return Err("usage: syson-reference-delta-plan <actual-packet.json>".into());
        }
        syson_reference_delta_plan::plan(&syson_reference_delta_plan::read_file(
            std::path::Path::new(&args[0]),
        )?)
    })();
    match result {
        Ok(v) => println!("{v}"),
        Err(e) => {
            eprintln!("delta planning refused: {e}");
            std::process::exit(1);
        }
    }
}

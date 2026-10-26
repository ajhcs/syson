// SPDX-License-Identifier: EPL-2.0
use syson_native_project_templates::{Catalogue, read_file};
fn main() {
    let result = (|| -> Result<_, String> {
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        if args.len() != 1 && args.len() != 3 {
            return Err(
                "usage: syson-native-project-templates <capture.json> [observation-id template-id]"
                    .into(),
            );
        }
        let catalogue = Catalogue::from_capture(&read_file(std::path::Path::new(&args[0]))?)?;
        let selection = if args.len() == 3 {
            Some((
                args[1].to_str().ok_or("UTF-8 observation ID required")?,
                args[2].to_str().ok_or("UTF-8 template ID required")?,
            ))
        } else {
            None
        };
        catalogue.projection(selection)
    })();
    match result {
        Ok(v) => println!("{v}"),
        Err(e) => {
            eprintln!("template capture refused: {e}");
            std::process::exit(1);
        }
    }
}

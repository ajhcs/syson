// SPDX-License-Identifier: EPL-2.0
//! Report syntax admission with the library's default options.
use serde_json::json;
use std::io::{self, Read};
use syson_syntax::{Error, Grammar, Language, Options};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let language = match std::env::args().nth(1).as_deref() {
        Some("sysml") => Language::SysML,
        Some("kerml") => Language::KerML,
        _ => return Err("usage: library_admission sysml|kerml < input".into()),
    };
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let grammar =
        Grammar::load(language).map_err(|error| format!("cannot load grammar: {error:?}"))?;
    let result = match grammar.recognize(&input, Options::default()) {
        Ok(_) => json!({"status": "accepted"}),
        Err(Error::BudgetExceeded { resource, limit }) => {
            json!({"status": "budget", "resource": resource, "limit": limit})
        }
        Err(error) => json!({"status": "refused", "diagnostic": format!("{error:?}")}),
    };
    println!("{result}");
    Ok(())
}

// Developer consumer: each result comes from the actual public F15 API.
fn main() {
    let path = std::env::args().nth(1).expect("explicit corpus path");
    let corpus: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let rows: Vec<serde_json::Value> = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| serde_json::from_str(&measurement_bridge::run(&case.to_string())).unwrap())
        .collect();
    println!("{}", serde_json::json!({"rows":rows}));
}

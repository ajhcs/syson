use serde_json::{Value, json};
use syson_native_http_contract::{MAX_INPUT, inspect_mappings, inspect_probes};

fn mappings() -> Value {
    serde_json::from_str(include_str!("fixtures/native-mappings.json")).unwrap()
}
fn probes() -> Value {
    serde_json::from_str(include_str!("fixtures/native-probes.json")).unwrap()
}
fn reject_mapping(value: &Value) {
    assert!(inspect_mappings(&value.to_string()).is_err());
}
fn reject_probe(value: &Value) {
    assert!(inspect_probes(&value.to_string()).is_err());
}
fn mvc(value: &mut Value) -> &mut Value {
    for context in value["contexts"].as_object_mut().unwrap().values_mut() {
        if let Some(servlets) = context["mappings"]["dispatcherServlets"].as_object_mut() {
            for entries in servlets.values_mut() {
                for entry in entries.as_array_mut().unwrap() {
                    if entry["details"]["requestMappingConditions"].is_object() {
                        return entry;
                    }
                }
            }
        }
    }
    panic!("actual capture contains no MVC conditions")
}
fn response(value: &mut Value) -> &mut Value {
    &mut value["observations"][0]["response"]
}
#[test]
fn actual_native_mapping_tree_is_preserved_with_unknown_conditions() {
    let mut input = mappings();
    mvc(&mut input)["details"]["requestMappingConditions"]["futureVersion"] =
        json!({"negated":true,"range":"[1,2)","opaque":[null,{"x":"é"}]});
    let output = inspect_mappings(&input.to_string()).unwrap();
    assert_eq!(output["mappings"], input);
    assert_eq!(output["C01"], "OPEN");
    assert_eq!(output["summary"]["contexts"], 1);
    assert_eq!(output["summary"]["handlers"], 54);
    assert_eq!(output["summary"]["mvc_handlers"], 33);
    assert_eq!(
        input["contexts"]["application"]["mappings"]["servletFilters"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
}
#[test]
fn actual_twenty_two_observations_survive_without_error_body_normalization() {
    let input = probes();
    let output = inspect_probes(&input.to_string()).unwrap();
    assert_eq!(output["capture"], input);
    assert_eq!(output["summary"]["probes"], 22);
    for observation in input["observations"].as_array().unwrap() {
        assert_eq!(
            output["summary"]["statuses"][observation["id"].as_str().unwrap()],
            observation["response"]["status"]
        );
    }
    assert_eq!(output["C01"], "OPEN");
    let observations = input["observations"].as_array().unwrap();
    let selected = |id: &str| observations.iter().find(|o| o["id"] == id).unwrap();
    let shell = &selected("graphql-get")["response"]["body_utf8"];
    assert_eq!(selected("graphql-get")["response"]["status"], 200);
    assert_eq!(selected("graphql-get")["response"]["body_bytes"], 504);
    assert_eq!(
        observations
            .iter()
            .filter(|o| o["response"].get("body_utf8") == Some(shell))
            .count(),
        9
    );
    assert_eq!(selected("graphql-head")["response"]["body_bytes"], 0);
    let syntax: Value = serde_json::from_str(
        selected("invalid-syntax-json")["response"]["body_utf8"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        syntax["errors"][0]["extensions"]["classification"],
        "InvalidSyntax"
    );
    assert_eq!(selected("cors-denied")["response"]["status"], 403);
}
#[test]
fn all_six_mvc_conditions_are_required_and_array_typed() {
    for key in [
        "methods", "patterns", "consumes", "produces", "headers", "params",
    ] {
        let mut input = mappings();
        mvc(&mut input)["details"]["requestMappingConditions"]
            .as_object_mut()
            .unwrap()
            .remove(key);
        reject_mapping(&input);
        let mut input = mappings();
        mvc(&mut input)["details"]["requestMappingConditions"][key] = json!({});
        reject_mapping(&input);
    }
}
#[test]
fn actual_mvc_expression_shapes_refuse_malformed_media_header_and_param_conditions() {
    let valid = json!({"methods":["GET","HEAD"],"patterns":["/x"],
        "consumes":[{"mediaType":"application/json","negated":true}],
        "produces":[{"mediaType":"text/plain","negated":false}],
        "headers":[{"name":"X-Test","value":null,"negated":true}],
        "params":[{"name":"p","value":"é","negated":false}]});
    let mut input = mappings();
    mvc(&mut input)["details"]["requestMappingConditions"] = valid;
    assert!(inspect_mappings(&input.to_string()).is_ok());
    for (key, member, bad) in [
        ("consumes", "mediaType", json!(false)),
        ("produces", "negated", json!("false")),
        ("headers", "value", json!([])),
        ("params", "name", json!(null)),
    ] {
        let mut corrupt = input.clone();
        mvc(&mut corrupt)["details"]["requestMappingConditions"][key][0][member] = bad;
        reject_mapping(&corrupt);
    }
    mvc(&mut input)["details"]["requestMappingConditions"]["methods"] = json!([null]);
    reject_mapping(&input);
}
#[test]
fn partial_contexts_categories_and_duplicate_mapping_identities_refuse() {
    reject_mapping(&json!({"contexts":{}}));
    reject_mapping(&json!({"contexts":{"app":{"mappings":{"dispatcherServlets":[]}}}}));
    let mut input = mappings();
    mvc(&mut input)["handler"] = json!(null);
    reject_mapping(&input);
    let mut input = mappings();
    'done: for context in input["contexts"].as_object_mut().unwrap().values_mut() {
        if let Some(servlets) = context["mappings"]["dispatcherServlets"].as_object_mut() {
            for entries in servlets.values_mut() {
                if let Some(entries) = entries.as_array_mut()
                    && !entries.is_empty()
                {
                    entries.push(entries[0].clone());
                    break 'done;
                }
            }
        }
    }
    reject_mapping(&input);
}
#[test]
fn missing_duplicate_unknown_probe_ids_and_diagnostic_contexts_refuse() {
    let mut input = probes();
    input["observations"].as_array_mut().unwrap().pop();
    reject_probe(&input);
    let mut input = probes();
    input["observations"][1]["id"] = input["observations"][0]["id"].clone();
    reject_probe(&input);
    let mut input = probes();
    input["observations"][0]["id"] = json!("unknown");
    reject_probe(&input);
    let mut input = probes();
    input["context"]["diagnostic_mappings"] = json!(false);
    reject_probe(&input);
    let mut input = probes();
    input["context"]["anonymous"] = json!(false);
    reject_probe(&input);
    let mut input = probes();
    input["context"]
        .as_object_mut()
        .unwrap()
        .remove("anonymous");
    reject_probe(&input);
    let mut input = probes();
    input["context"]["application_origin"] = json!("http://example.com:80");
    reject_probe(&input);
    for key in [
        "artifact_sha256",
        "source_label_revision",
        "target_revision",
        "configuration_sha256",
    ] {
        let mut input = probes();
        input["context"].as_object_mut().unwrap().remove(key);
        reject_probe(&input);
        let mut input = probes();
        input["context"][key] = json!("not-an-identity");
        reject_probe(&input);
    }
}
#[test]
fn exact_request_contract_refuses_mutation_redirect_auth_cookie_and_header_drift() {
    for (key, bad) in [
        ("method", "DELETE"),
        ("path", "https://example.com/"),
        ("body_utf8", "mutation { deleteProject }"),
    ] {
        let mut input = probes();
        input["observations"][0]["request"][key] = json!(bad);
        reject_probe(&input);
    }
    for pair in [
        ["Authorization", "Bearer x"],
        ["Cookie", "session=x"],
        ["Host", "example.com"],
        ["Accept-Encoding", "gzip"],
    ] {
        let mut input = probes();
        input["observations"][0]["request"]["headers"]
            .as_array_mut()
            .unwrap()
            .push(json!(pair));
        reject_probe(&input);
    }
    let mut input = probes();
    input["observations"][0]["request"]["body_hex"] = json!("");
    reject_probe(&input);
    let mut input = probes();
    input["observations"][0]["request"]["headers"]
        .as_array_mut()
        .unwrap()
        .pop();
    reject_probe(&input);
}
#[test]
fn response_status_encoding_byte_count_headers_and_elapsed_errors_refuse() {
    for bad in [json!(99), json!(600), json!("200")] {
        let mut input = probes();
        response(&mut input)["status"] = bad;
        reject_probe(&input);
    }
    let mut input = probes();
    response(&mut input)["body_bytes"] = json!(u64::MAX);
    reject_probe(&input);
    let mut input = probes();
    response(&mut input)["headers"] = json!([["X-Bad", "a\r\nb"]]);
    reject_probe(&input);
    let mut input = probes();
    response(&mut input)["headers"] = json!([["Bad Name", "x"]]);
    reject_probe(&input);
    let mut input = probes();
    input["observations"][0]["elapsed_ms"] = json!(-1);
    reject_probe(&input);
    let mut input = probes();
    let r = response(&mut input);
    r.as_object_mut().unwrap().remove("body_utf8");
    r["body_hex"] = json!("a");
    reject_probe(&input);
}
#[test]
fn binary_body_and_repeated_response_headers_survive_exactly() {
    let mut input = probes();
    let r = response(&mut input);
    r.as_object_mut().unwrap().remove("body_utf8");
    r.as_object_mut().unwrap().remove("body_hex");
    r["body_hex"] = json!("00ffC3a9");
    r["body_bytes"] = json!(4);
    r["headers"] = json!([
        ["Vary", "Accept"],
        ["Vary", "Origin"],
        ["Set-Cookie", "a=1"],
        ["Set-Cookie", "b=2"]
    ]);
    assert_eq!(
        inspect_probes(&input.to_string()).unwrap()["capture"],
        input
    );
    response(&mut input)["body_utf8"] = json!("also present");
    reject_probe(&input);
}
#[test]
fn bounded_inputs_and_opaque_values_keep_compact_output_proportional() {
    let oversized = " ".repeat(MAX_INPUT + 1);
    assert!(inspect_mappings(&oversized).is_err());
    assert!(inspect_probes(&oversized).is_err());
    let mut input = mappings();
    input["opaque"] = json!([null,"é",[1,2],{"future":true}]);
    let raw = input.to_string();
    let output = inspect_mappings(&raw).unwrap();
    assert_eq!(output["mappings"], input);
    assert!(output.to_string().len() < raw.len() + 1024);
    input["opaque"] = json!([vec![0; 200_001]]);
    reject_mapping(&input);
}

#[test]
fn raw_large_integer_and_actual_elapsed_decimal_survive_without_float_rounding() {
    let large = "900719925474099312345";
    let add_opaque = |source: &str| {
        format!(
            "{},\"opaqueLarge\":{large},\"opaqueDecimal\":1.5184049843810499}}",
            source.trim_end().strip_suffix('}').unwrap()
        )
    };
    let mapping = add_opaque(include_str!("fixtures/native-mappings.json"));
    let output = inspect_mappings(&mapping).unwrap();
    assert_eq!(output["mappings"]["opaqueLarge"].to_string(), large);
    assert_eq!(
        output["mappings"]["opaqueDecimal"].to_string(),
        "1.5184049843810499"
    );
    let probes = add_opaque(include_str!("fixtures/native-probes.json"));
    let output = inspect_probes(&probes).unwrap();
    assert_eq!(output["capture"]["opaqueLarge"].to_string(), large);
    assert_eq!(
        output["capture"]["opaqueDecimal"].to_string(),
        "1.5184049843810499"
    );
    let actual = output["capture"]["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "absent-element")
        .unwrap();
    assert_eq!(actual["elapsed_ms"].to_string(), "1.5184049843810499");
}

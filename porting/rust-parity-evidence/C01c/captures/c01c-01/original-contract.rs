// SPDX-License-Identifier: EPL-2.0
use serde_json::{Value, json};
use syson_native_graphql_contract::{MAX_BYTES, MAX_MEMBERS, MAX_TYPES, MAX_WRAPPERS, inspect};
const NATIVE: &str = include_str!("fixtures/native-official-label-introspection.json");
const BAD_FAITH: &str = include_str!("fixtures/native-badfaith-introspection.json");
fn fixture() -> Value {
    serde_json::from_str(NATIVE).unwrap()
}
fn schema(v: &mut Value) -> &mut Value {
    &mut v["data"]["__schema"]
}
fn ty<'a>(v: &'a mut Value, name: &str) -> &'a mut Value {
    schema(v)["types"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|t| t["name"] == name)
        .unwrap()
}
fn first_field(v: &mut Value) -> &mut Value {
    &mut ty(v, "Query")["fields"][0]
}
fn argument(v: &mut Value) -> &mut Value {
    &mut ty(v, "Query")["fields"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|f| !f["args"].as_array().unwrap().is_empty())
        .unwrap()["args"][0]
}
fn refuses(v: &Value) {
    assert!(inspect(&v.to_string()).is_err());
}
fn named(kind: &str, name: &str) -> Value {
    json!({"kind":kind,"name":name,"ofType":null})
}
fn wrapped(kind: &str, inner: Value) -> Value {
    json!({"kind":kind,"name":null,"ofType":inner})
}

#[test]
fn actual_official_runtime_response_preserves_the_entire_selected_schema() {
    let input = fixture();
    let before = input.clone();
    let output = inspect(NATIVE).unwrap();
    assert_eq!(output["schema"], input["data"]["__schema"]);
    assert_eq!(input, before);
    assert_eq!(output["counts"]["types"], 671);
    assert_eq!(output["counts"]["fields"], 1418);
    assert_eq!(output["counts"]["input_fields"], 809);
    assert_eq!(output["counts"]["directives"], 7);
    assert_eq!(output["C01"], "OPEN");
    assert!(
        output["scope"]
            .as_str()
            .unwrap()
            .contains("Observed standard")
    );
}
#[test]
fn raw_default_literal_descriptions_deprecations_and_member_order_are_not_reparsed() {
    // Synthetic edits test opaque-value fidelity, not native schema acceptance.
    let mut input = fixture();
    let raw = "{color: RED, text: \"🚗\\n\", list: [1, 2]}";
    let arg = argument(&mut input);
    arg["defaultValue"] = json!(raw);
    arg["description"] = json!("quoted // text 🚗");
    arg["isDeprecated"] = json!(true);
    arg["deprecationReason"] = json!("legacy input");
    ty(&mut input, "Query")["fields"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let output = inspect(&input.to_string()).unwrap();
    assert_eq!(output["schema"], input["data"]["__schema"]);
    assert!(output["schema"].to_string().contains("legacy input"));
}
#[test]
fn actual_native_bad_faith_refusal_and_partial_graphql_errors_are_never_schema_success() {
    assert!(inspect(BAD_FAITH).unwrap_err().contains("GraphQL errors"));
    let mut partial = fixture();
    partial["errors"] = serde_json::from_str::<Value>(BAD_FAITH).unwrap()["errors"].clone();
    refuses(&partial);
    for bad in [
        "{",
        "null",
        "[]",
        "{}",
        "{\"data\":null}",
        "{\"data\":{\"__schema\":null}}",
        "{\"errors\":null}",
    ] {
        assert!(inspect(bad).is_err());
    }
    let mut empty = fixture();
    empty["errors"] = json!([]);
    assert!(inspect(&empty.to_string()).is_ok());
}
#[test]
fn missing_full_query_selections_fail_instead_of_becoming_empty_or_null() {
    for key in [
        "description",
        "queryType",
        "mutationType",
        "subscriptionType",
        "types",
        "directives",
    ] {
        let mut input = fixture();
        schema(&mut input).as_object_mut().unwrap().remove(key);
        refuses(&input);
    }
    for key in [
        "fields",
        "inputFields",
        "interfaces",
        "enumValues",
        "possibleTypes",
        "specifiedByURL",
        "isOneOf",
        "description",
    ] {
        let mut input = fixture();
        ty(&mut input, "Query").as_object_mut().unwrap().remove(key);
        refuses(&input);
    }
    for key in [
        "defaultValue",
        "type",
        "description",
        "isDeprecated",
        "deprecationReason",
    ] {
        let mut input = fixture();
        argument(&mut input).as_object_mut().unwrap().remove(key);
        refuses(&input);
    }
    let mut input = fixture();
    let index = schema(&mut input)["types"]
        .as_array()
        .unwrap()
        .iter()
        .position(|t| t["name"] == "__Schema")
        .unwrap();
    schema(&mut input)["types"]
        .as_array_mut()
        .unwrap()
        .remove(index);
    refuses(&input);
}
#[test]
fn duplicate_type_member_argument_enum_directive_and_edge_identities_refuse() {
    let mut input = fixture();
    let duplicate = schema(&mut input)["types"][0].clone();
    schema(&mut input)["types"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    refuses(&input);
    input = fixture();
    let duplicate = first_field(&mut input).clone();
    ty(&mut input, "Query")["fields"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    refuses(&input);
    input = fixture();
    let duplicate = argument(&mut input).clone();
    let fields = ty(&mut input, "Query")["fields"].as_array_mut().unwrap();
    fields
        .iter_mut()
        .find(|f| !f["args"].as_array().unwrap().is_empty())
        .unwrap()["args"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    refuses(&input);
    input = fixture();
    let duplicate = ty(&mut input, "__TypeKind")["enumValues"][0].clone();
    ty(&mut input, "__TypeKind")["enumValues"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    refuses(&input);
    input = fixture();
    let duplicate = schema(&mut input)["directives"][0].clone();
    schema(&mut input)["directives"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    refuses(&input);
    input = fixture();
    let union = schema(&mut input)["types"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|t| t["kind"] == "UNION")
        .unwrap();
    let duplicate = union["possibleTypes"][0].clone();
    union["possibleTypes"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    refuses(&input);
}
#[test]
fn dangling_wrong_kind_wrong_position_and_invalid_graphql_names_refuse() {
    for bad in [
        named("SCALAR", "NotADeclaredType"),
        named("OBJECT", "String"),
        named("INPUT_OBJECT", "AddGroupInput"),
    ] {
        let mut input = fixture();
        first_field(&mut input)["type"] = bad;
        refuses(&input);
    }
    let mut input = fixture();
    argument(&mut input)["type"] = named("OBJECT", "Query");
    refuses(&input);
    input = fixture();
    first_field(&mut input)["name"] = json!("not-a-name");
    refuses(&input);
    input = fixture();
    schema(&mut input)["queryType"] = named("SCALAR", "String");
    refuses(&input);
    input = fixture();
    ty(&mut input, "Query")["interfaces"] = json!([named("OBJECT", "Query")]);
    refuses(&input);
    input = fixture();
    schema(&mut input)["directives"][0]["locations"] = json!(["NOT_A_DECLARED_LOCATION"]);
    refuses(&input);
}
#[test]
fn wrappers_must_terminate_at_named_types_and_respect_nested_nullability() {
    for bad in [
        json!({"kind":"LIST","name":null}),
        json!({"kind":"LIST","name":null,"ofType":null}),
        wrapped("NON_NULL", wrapped("NON_NULL", named("SCALAR", "String"))),
        json!({"kind":"LIST","name":"bad","ofType":named("SCALAR","String")}),
    ] {
        let mut input = fixture();
        first_field(&mut input)["type"] = bad;
        refuses(&input);
    }
    let mut input = fixture();
    let mut value = named("SCALAR", "String");
    for n in 0..MAX_WRAPPERS {
        value = wrapped(
            if n.is_multiple_of(2) {
                "LIST"
            } else {
                "NON_NULL"
            },
            value,
        );
    }
    first_field(&mut input)["type"] = value.clone();
    assert!(inspect(&input.to_string()).is_ok());
    first_field(&mut input)["type"] = wrapped("LIST", value);
    assert!(
        inspect(&input.to_string())
            .unwrap_err()
            .contains("wrapper budget")
    );
}
#[test]
fn typed_optional_metadata_is_required_and_defaults_are_graphql_literal_strings() {
    let mut input = fixture();
    argument(&mut input)["defaultValue"] = json!({"not":"a literal string"});
    refuses(&input);
    input = fixture();
    argument(&mut input)["isDeprecated"] = json!("false");
    refuses(&input);
    input = fixture();
    schema(&mut input)["directives"][0]
        .as_object_mut()
        .unwrap()
        .remove("isRepeatable");
    refuses(&input);
    input = fixture();
    schema(&mut input)["directives"][0]["locations"] = json!([]);
    refuses(&input);
    input = fixture();
    ty(&mut input, "AddGroupInput")["isOneOf"] = json!("false");
    refuses(&input);
    input = fixture();
    ty(&mut input, "Query")["specifiedByURL"] = json!("https://example.invalid/not-a-scalar");
    refuses(&input);
}
#[test]
fn byte_type_and_member_budgets_refuse_before_expensive_identity_processing() {
    assert!(
        inspect(&" ".repeat(MAX_BYTES + 1))
            .unwrap_err()
            .contains("byte budget")
    );
    let mut input = fixture();
    schema(&mut input)["types"] = json!(vec![Value::Null; MAX_TYPES + 1]);
    assert!(
        inspect(&input.to_string())
            .unwrap_err()
            .contains("type budget")
    );
    input = fixture();
    ty(&mut input, "__TypeKind")["enumValues"] = json!(vec![Value::Null; MAX_MEMBERS + 1]);
    assert!(
        inspect(&input.to_string())
            .unwrap_err()
            .contains("member budget")
    );
}

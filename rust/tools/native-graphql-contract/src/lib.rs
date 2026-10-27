// SPDX-License-Identifier: EPL-2.0
//! Validate an observed full introspection contract; no SDL parser or network.
#![forbid(unsafe_code)]
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_BYTES: usize = 32 * 1024 * 1024;
/// Physical source values/keys, including overwritten duplicate occurrences.
pub const MAX_LITERAL_VALUES: usize = 2_000_000;
/// Whole-value depth (root is one), including opaque extensions.
pub const MAX_LITERAL_DEPTH: usize = 128;
pub const MAX_LITERAL_SCALAR_BYTES: usize = 2 * MAX_BYTES;
pub const MAX_TYPES: usize = 20_000;
pub const MAX_MEMBERS: usize = 200_000;
pub const MAX_WRAPPERS: usize = 32;
pub const QUERY: &str =
    include_str!("../../../../porting/rust-parity-evidence/C01c/introspection.graphql");
pub const CAPABILITIES: &str =
    include_str!("../../../../porting/rust-parity-evidence/C01c/capabilities-single.graphql");
const NAMED: [&str; 6] = [
    "SCALAR",
    "OBJECT",
    "INTERFACE",
    "UNION",
    "ENUM",
    "INPUT_OBJECT",
];
const OUTPUT: [&str; 5] = ["SCALAR", "OBJECT", "INTERFACE", "UNION", "ENUM"];
const INPUT: [&str; 3] = ["SCALAR", "ENUM", "INPUT_OBJECT"];
type Result<T> = std::result::Result<T, String>;
fn need(ok: bool, why: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(why.into()) }
}
fn field<'a>(v: &'a Value, key: &str) -> Result<&'a Value> {
    v.get(key)
        .ok_or_else(|| format!("incomplete selection: missing {key}"))
}
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or_else(|| "expected string".into())
}
fn name(v: &Value) -> Result<&str> {
    let n = text(v)?;
    let mut chars = n.bytes();
    need(
        chars
            .next()
            .is_some_and(|b| b == b'_' || b.is_ascii_alphabetic())
            && chars.all(|b| b == b'_' || b.is_ascii_alphanumeric()),
        "invalid GraphQL name",
    )?;
    Ok(n)
}
fn nullable(v: &Value, key: &str) -> Result<()> {
    let v = field(v, key)?;
    need(
        v.is_null() || v.is_string(),
        &format!("invalid nullable string {key}"),
    )
}
fn boolean(v: &Value, key: &str) -> Result<()> {
    need(
        field(v, key)?.is_boolean(),
        &format!("invalid Boolean {key}"),
    )
}
fn deprecation(v: &Value) -> Result<()> {
    boolean(v, "isDeprecated")?;
    nullable(v, "deprecationReason")
}
fn unique(rows: &[Value]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for row in rows {
        need(
            seen.insert(name(field(row, "name")?)?),
            "duplicate member identity",
        )?;
    }
    Ok(())
}
struct Count {
    members: usize,
    fields: usize,
    args: usize,
    inputs: usize,
    enums: usize,
    interfaces: usize,
    possible: usize,
}
impl Count {
    fn rows<'a>(&mut self, v: &'a Value, key: &str, active: bool) -> Result<&'a [Value]> {
        let value = field(v, key)?;
        if !active {
            need(value.is_null(), &format!("unexpected {key} for type kind"))?;
            return Ok(&[]);
        }
        let rows = value
            .as_array()
            .ok_or_else(|| format!("incomplete array {key}"))?;
        self.members = self
            .members
            .checked_add(rows.len())
            .filter(|n| *n <= MAX_MEMBERS)
            .ok_or("member budget exceeded")?;
        Ok(rows)
    }
}
fn reference<'a>(
    mut v: &'a Value,
    types: &BTreeMap<&str, &str>,
    allowed: &[&str],
) -> Result<&'a str> {
    let mut non_null = false;
    for depth in 0..=MAX_WRAPPERS {
        let kind = text(field(v, "kind")?)?;
        if matches!(kind, "LIST" | "NON_NULL") {
            need(depth < MAX_WRAPPERS, "wrapper budget exceeded")?;
            need(field(v, "name")?.is_null(), "wrapper must have null name")?;
            need(
                !(non_null && kind == "NON_NULL"),
                "NON_NULL cannot wrap NON_NULL",
            )?;
            non_null = kind == "NON_NULL";
            v = field(v, "ofType")?;
            need(
                v.is_object(),
                "truncated wrapper: capture deeper ofType selection",
            )?;
        } else {
            let n = name(field(v, "name")?)?;
            need(
                types.get(n).copied() == Some(kind) && allowed.contains(&kind),
                "dangling or incompatible type reference",
            )?;
            need(
                v.get("ofType").is_none_or(Value::is_null),
                "named type has ofType child",
            )?;
            return Ok(n);
        }
    }
    Err("unterminated wrapper".into())
}
fn input_values(rows: &[Value], types: &BTreeMap<&str, &str>) -> Result<()> {
    unique(rows)?;
    for row in rows {
        nullable(row, "description")?;
        nullable(row, "defaultValue")?;
        deprecation(row)?;
        reference(field(row, "type")?, types, &INPUT)?;
    }
    Ok(())
}
/// Preserve all selected values/member order, including descriptions and raw
/// GraphQL-literal defaultValue strings. No default-literal or SDL parsing occurs.
/// The caller binds the response to the exact full query and runtime receipt;
/// this function cannot prove the server omitted no inaccessible/unrequested data.
pub fn inspect(input: &str) -> Result<Value> {
    need(input.len() <= MAX_BYTES, "response byte budget exceeded")?;
    let admitted = syson_json_value::parse_with_duplicate_keys(
        input,
        syson_json_value::Limits {
            bytes: MAX_BYTES,
            depth: MAX_LITERAL_DEPTH,
            values: MAX_LITERAL_VALUES,
            keys: MAX_LITERAL_VALUES,
            scalar_bytes: MAX_LITERAL_SCALAR_BYTES,
        },
        syson_json_value::DuplicateKeys::LastWins,
        || false,
    )
    .map_err(|e| format!("literal JSON refused: {e}"))?;
    let (response, _) = admitted.into_value(|| false).map_err(|e| e.to_string())?;
    if let Some(errors) = response.get("errors") {
        need(
            errors.as_array().is_some_and(Vec::is_empty),
            "GraphQL errors or malformed errors; no partial schema admission",
        )?;
    }
    let schema = field(field(&response, "data")?, "__schema")?;
    need(schema.is_object(), "missing effective __schema object")?;
    nullable(schema, "description")?;
    let rows = field(schema, "types")?
        .as_array()
        .ok_or("incomplete types array")?;
    need(
        !rows.is_empty() && rows.len() <= MAX_TYPES,
        "type budget or empty schema",
    )?;
    unique(rows)?;
    let mut types = BTreeMap::new();
    for t in rows {
        let n = name(field(t, "name")?)?;
        let kind = text(field(t, "kind")?)?;
        need(
            NAMED.contains(&kind),
            "unknown or wrapped top-level type kind",
        )?;
        types.insert(n, kind);
    }
    for (n, k) in [
        ("__Schema", "OBJECT"),
        ("__Type", "OBJECT"),
        ("__Field", "OBJECT"),
        ("__InputValue", "OBJECT"),
        ("__EnumValue", "OBJECT"),
        ("__Directive", "OBJECT"),
        ("__TypeKind", "ENUM"),
        ("__DirectiveLocation", "ENUM"),
    ] {
        need(
            types.get(n).copied() == Some(k),
            "missing standard introspection type",
        )?;
    }
    for (key, required) in [
        ("queryType", true),
        ("mutationType", false),
        ("subscriptionType", false),
    ] {
        let root = field(schema, key)?;
        if root.is_null() {
            need(!required, "query root is required")?;
        } else {
            need(
                root["kind"] == "OBJECT",
                "schema roots must directly name OBJECT types",
            )?;
            reference(root, &types, &["OBJECT"])?;
        }
    }
    let mut count = Count {
        members: 0,
        fields: 0,
        args: 0,
        inputs: 0,
        enums: 0,
        interfaces: 0,
        possible: 0,
    };
    for t in rows {
        let kind = text(field(t, "kind")?)?;
        nullable(t, "description")?;
        nullable(t, "specifiedByURL")?;
        if kind != "SCALAR" {
            need(
                field(t, "specifiedByURL")?.is_null(),
                "specifiedByURL only applies to scalars",
            )?;
        }
        if kind == "INPUT_OBJECT" {
            boolean(t, "isOneOf")?;
        } else {
            need(
                field(t, "isOneOf")?.is_null() || field(t, "isOneOf")? == false,
                "isOneOf only applies to input objects",
            )?;
        }
        let fields = count.rows(t, "fields", matches!(kind, "OBJECT" | "INTERFACE"))?;
        unique(fields)?;
        count.fields += fields.len();
        for f in fields {
            nullable(f, "description")?;
            deprecation(f)?;
            reference(field(f, "type")?, &types, &OUTPUT)?;
            let args = count.rows(f, "args", true)?;
            count.args += args.len();
            input_values(args, &types)?;
        }
        let inputs = count.rows(t, "inputFields", kind == "INPUT_OBJECT")?;
        count.inputs += inputs.len();
        input_values(inputs, &types)?;
        let enums = count.rows(t, "enumValues", kind == "ENUM")?;
        unique(enums)?;
        count.enums += enums.len();
        for e in enums {
            nullable(e, "description")?;
            deprecation(e)?;
        }
        for (key, active, allowed) in [
            (
                "interfaces",
                matches!(kind, "OBJECT" | "INTERFACE"),
                "INTERFACE",
            ),
            (
                "possibleTypes",
                matches!(kind, "UNION" | "INTERFACE"),
                "OBJECT",
            ),
        ] {
            let refs = count.rows(t, key, active)?;
            let mut seen = BTreeSet::new();
            for r in refs {
                need(
                    r["kind"] == allowed,
                    "interface/possibleTypes edges must directly name their target kind",
                )?;
                need(
                    seen.insert(reference(r, &types, &[allowed])?),
                    "duplicate interface/possible type",
                )?;
            }
            if key == "interfaces" {
                count.interfaces += refs.len();
            } else {
                count.possible += refs.len();
            }
        }
    }
    let location_type = rows
        .iter()
        .find(|t| t["name"] == "__DirectiveLocation")
        .ok_or("missing directive locations enum")?;
    let allowed_locations: BTreeSet<_> = field(location_type, "enumValues")?
        .as_array()
        .ok_or("invalid directive locations enum")?
        .iter()
        .map(|e| name(field(e, "name")?))
        .collect::<Result<_>>()?;
    let directives = count.rows(schema, "directives", true)?;
    unique(directives)?;
    for d in directives {
        nullable(d, "description")?;
        boolean(d, "isRepeatable")?;
        let args = count.rows(d, "args", true)?;
        count.args += args.len();
        input_values(args, &types)?;
        let locations = count.rows(d, "locations", true)?;
        need(!locations.is_empty(), "empty directive locations")?;
        let mut seen = BTreeSet::new();
        for location in locations {
            let location = name(location)?;
            need(
                allowed_locations.contains(location),
                "unknown directive location",
            )?;
            need(seen.insert(location), "duplicate directive location")?;
        }
    }
    let mut report = json!({"C01":"OPEN","scope":"Observed standard full-introspection contract only; not source/artifact, resolver/auth, custom applied-directive usage or implicit __typename behavior parity.","wrapper_capture":"ALL_CHAINS_TERMINATE_AT_DECLARED_NAMED_TYPES","expected_operation":"C01cEffectiveSchema","counts":{"types":types.len(),"fields":count.fields,"arguments_including_directives":count.args,"input_fields":count.inputs,"enum_values":count.enums,"interface_edges":count.interfaces,"possible_type_edges":count.possible,"directives":directives.len(),"members":count.members}});
    // Keep literal numbers out of json!'s generic to_value serialization.
    report["schema"] = schema.clone();
    Ok(report)
}

#![forbid(unsafe_code)]
//! Inspect captured native observations; no HTTP client or route implementation.
use serde_json::{Map, Value, json};

pub const MAX_INPUT: usize = 32 * 1024 * 1024;
pub const PROBES: &str = include_str!("../../../../porting/rust-parity-evidence/C01f/probes.json");
type Result<T> = std::result::Result<T, String>;
fn need(ok: bool, message: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(message.into()) }
}
fn object(value: &Value) -> Result<&Map<String, Value>> {
    value.as_object().ok_or_else(|| "expected object".into())
}
fn array(value: &Value) -> Result<&Vec<Value>> {
    value.as_array().ok_or_else(|| "expected array".into())
}
fn text(value: &Value) -> Result<&str> {
    value.as_str().ok_or_else(|| "expected string".into())
}
fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value> {
    object(value)?
        .get(key)
        .ok_or_else(|| format!("missing {key}"))
}
fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    text(field(value, key)?)
}
fn parse(input: &str) -> Result<Value> {
    need(input.len() <= MAX_INPUT, "input byte limit")?;
    let value: Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    let mut pending = vec![&value];
    let (mut nodes, mut bytes) = (0usize, 0usize);
    while let Some(item) = pending.pop() {
        nodes += 1;
        match item {
            Value::Object(map) => {
                need(nodes + pending.len() + map.len() <= 200_000, "value budget")?;
                for (key, child) in map {
                    bytes += key.len();
                    need(key.len() <= 8192, "object key limit")?;
                    pending.push(child);
                }
            }
            Value::Array(items) => {
                need(
                    nodes + pending.len() + items.len() <= 200_000,
                    "value budget",
                )?;
                pending.extend(items);
            }
            Value::String(s) => {
                need(s.len() <= 4 * 1024 * 1024, "string byte limit")?;
                bytes += s.len();
            }
            _ => {}
        }
        need(
            nodes + pending.len() <= 200_000 && bytes <= 24 * 1024 * 1024,
            "value budget",
        )?;
    }
    Ok(value)
}
fn strings(value: &Value) -> Result<()> {
    for item in array(value)? {
        text(item)?;
    }
    Ok(())
}
fn conditions(value: &Value) -> Result<()> {
    object(value)?;
    for key in ["methods", "patterns"] {
        strings(field(value, key)?)?;
    }
    for key in ["consumes", "produces"] {
        for item in array(field(value, key)?)? {
            string(item, "mediaType")?;
            need(
                field(item, "negated")?.is_boolean(),
                "media negation must be boolean",
            )?;
        }
    }
    for key in ["headers", "params"] {
        for item in array(field(value, key)?)? {
            string(item, "name")?;
            let v = field(item, "value")?;
            need(
                v.is_null() || v.is_string(),
                "condition value must be string or null",
            )?;
            need(
                field(item, "negated")?.is_boolean(),
                "condition negation must be boolean",
            )?;
        }
    }
    Ok(())
}
/// Preserve every supplied mapping category and condition, including unknown keys.
/// Admission describes this captured diagnostic context, not runtime authenticity.
pub fn inspect_mappings(input: &str) -> Result<Value> {
    let raw = parse(input)?;
    let contexts = object(field(&raw, "contexts")?)?;
    need(
        !contexts.is_empty() && contexts.len() <= 64,
        "context count",
    )?;
    let (mut handlers, mut mvc) = (0usize, 0usize);
    for context in contexts.values() {
        let mappings = object(field(context, "mappings")?)?;
        for (category, entries) in mappings {
            match category.as_str() {
                "dispatcherServlets" => {
                    for entries in object(entries)?.values() {
                        let mut identities = std::collections::BTreeSet::new();
                        for entry in array(entries)? {
                            handlers += 1;
                            let handler = string(entry, "handler")?;
                            let predicate = string(entry, "predicate")?;
                            need(
                                identities.insert((handler, predicate)),
                                "duplicate handler identity",
                            )?;
                            if let Some(details) = object(entry)?.get("details")
                                && let Some(c) = object(details)?.get("requestMappingConditions")
                            {
                                conditions(c)?;
                                mvc += 1;
                            }
                            need(handlers <= 20_000, "handler count")?;
                        }
                    }
                }
                "servlets" | "servletFilters" => {
                    for entry in array(entries)? {
                        object(entry)?;
                    }
                }
                _ => {
                    need(
                        entries.is_object() || entries.is_array(),
                        "mapping category container",
                    )?;
                }
            }
        }
    }
    need(handlers > 0 && mvc > 0, "no structured MVC handlers")?;
    Ok(
        json!({"C01":"OPEN", "scope":"observed diagnostic handler mappings", "summary":{"contexts":contexts.len(),"handlers":handlers,"mvc_handlers":mvc},"mappings":raw}),
    )
}
fn origin(value: &str) -> Result<&str> {
    let host = value
        .strip_prefix("http://127.0.0.1:")
        .ok_or("origin must be loopback HTTP")?;
    let port = host.parse::<u16>().map_err(|_| "invalid origin port")?;
    need(port != 0 && host == port.to_string(), "invalid origin port")?;
    Ok(value.strip_prefix("http://").unwrap())
}
fn headers(value: &Value) -> Result<Vec<(&str, &str)>> {
    let items = array(value)?;
    need(items.len() <= 256, "header count")?;
    items
        .iter()
        .map(|pair| {
            let pair = array(pair)?;
            need(pair.len() == 2, "header pair length")?;
            let (name, value) = (text(&pair[0])?, text(&pair[1])?);
            need(
                !name.is_empty()
                    && name
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b)),
                "invalid header name",
            )?;
            need(
                value.len() <= 16_384 && !value.contains(['\r', '\n', '\0']),
                "invalid header value",
            )?;
            Ok((name, value))
        })
        .collect()
}
fn body_bytes(value: &Value) -> Result<usize> {
    let map = object(value)?;
    match (map.get("body_utf8"), map.get("body_hex")) {
        (Some(body), None) => Ok(text(body)?.len()),
        (None, Some(body)) => {
            let s = text(body)?;
            need(
                s.len().is_multiple_of(2) && s.bytes().all(|b| b.is_ascii_hexdigit()),
                "invalid hex body",
            )?;
            Ok(s.len() / 2)
        }
        _ => Err("exactly one body encoding required".into()),
    }
}
fn request(actual: &Value, expected: &Value, app_origin: &str) -> Result<()> {
    need(
        !object(actual)?.contains_key("body_hex"),
        "fixed requests use UTF8 only",
    )?;
    for key in ["method", "path", "body_utf8"] {
        need(
            field(actual, key)? == field(expected, key)?,
            "request differs from fixed probe",
        )?;
    }
    let mut required: Vec<(String, String)> = headers(field(expected, "headers")?)?
        .into_iter()
        .map(|(k, v)| {
            (
                k.to_ascii_lowercase(),
                v.replace("{application_origin}", app_origin),
            )
        })
        .collect();
    required.extend([
        ("host".into(), origin(app_origin)?.into()),
        ("connection".into(), "close".into()),
        ("accept-encoding".into(), "identity".into()),
    ]);
    if string(actual, "method")? == "POST" {
        required.push((
            "content-length".into(),
            string(actual, "body_utf8")?.len().to_string(),
        ));
    }
    let got: Vec<(String, String)> = headers(field(actual, "headers")?)?
        .into_iter()
        .map(|(k, v)| (k.to_ascii_lowercase(), v.into()))
        .collect();
    need(
        got.len() == required.len(),
        "unexpected or missing request header",
    )?;
    for expected in required {
        need(
            got.iter().filter(|v| **v == expected).count() == 1,
            "request header mismatch",
        )?;
    }
    Ok(())
}
/// Inspect exactly the canonical nonmutating probes, preserving wire observations.
/// Status/error bodies are observations: no backend outcome is fabricated here.
pub fn inspect_probes(input: &str) -> Result<Value> {
    let raw = parse(input)?;
    let context = field(&raw, "context")?;
    need(
        field(context, "anonymous")? == true,
        "anonymous context required",
    )?;
    for (key, len) in [
        ("artifact_sha256", 64),
        ("source_label_revision", 40),
        ("target_revision", 40),
        ("configuration_sha256", 64),
    ] {
        let value = string(context, key)?;
        need(
            value.len() == len && value.bytes().all(|b| b.is_ascii_hexdigit()),
            "missing or malformed capture identity",
        )?;
    }
    let app = string(context, "application_origin")?;
    origin(app)?;
    let management = string(context, "management_origin")?;
    origin(management)?;
    need(
        app != management && field(context, "diagnostic_mappings")? == true,
        "diagnostic context required",
    )?;
    let manifest: Value = serde_json::from_str(PROBES).map_err(|e| e.to_string())?;
    let expected = array(field(&manifest, "cases")?)?;
    let observations = array(field(&raw, "observations")?)?;
    need(
        observations.len() == expected.len(),
        "probe coverage differs from fixed manifest",
    )?;
    let mut seen = std::collections::BTreeSet::new();
    let mut statuses = Map::new();
    for observation in observations {
        let id = string(observation, "id")?;
        need(seen.insert(id), "duplicate probe id")?;
        let case = expected
            .iter()
            .find(|case| case["id"] == id)
            .ok_or("unknown probe id")?;
        request(field(observation, "request")?, case, app)?;
        let response = field(observation, "response")?;
        let status = field(response, "status")?
            .as_u64()
            .ok_or("invalid status")?;
        need((100..=599).contains(&status), "invalid status")?;
        statuses.insert(id.into(), json!(status));
        headers(field(response, "headers")?)?;
        let bytes = body_bytes(response)?;
        need(
            bytes <= 2 * 1024 * 1024
                && field(response, "body_bytes")?.as_u64() == Some(bytes as u64),
            "response body byte count",
        )?;
        let elapsed = field(observation, "elapsed_ms")?
            .as_f64()
            .ok_or("invalid elapsed time")?;
        need(elapsed.is_finite() && elapsed >= 0., "invalid elapsed time")?;
    }
    Ok(
        json!({"C01":"OPEN","scope":"observed fixed anonymous HTTP probes; diagnostic mappings enabled","summary":{"probes":observations.len(),"statuses":statuses},"capture":raw}),
    )
}

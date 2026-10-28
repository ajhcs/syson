// SPDX-License-Identifier: EPL-2.0
//! Count complete HTML helper fragments before owning output. Model inspection
//! remains the existing owned authority; it is prepared once, not per pass.
use super::{blocked_label, common_property};
use std::{fmt, io};
use syson_model::{AttributeDescriptor, AttributeError, Document, NodeInspection, ScalarType};
pub(crate) const MAX_BYTES: usize = 16 * 1024 * 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RenderError {
    InvalidLimit,
    Limit,
    Allocation,
    Json,
    Model,
}
impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidLimit => "Invalid HTML helper byte limit.",
            Self::Limit => "HTML helper exceeds its preview byte limit.",
            Self::Allocation => "HTML helper allocation was refused.",
            Self::Json => "HTML helper JSON output failed.",
            Self::Model => "Model exceeds its admitted preview limits.",
        })
    }
}
impl std::error::Error for RenderError {}
pub(crate) type Result<T = ()> = std::result::Result<T, RenderError>;
pub(crate) struct Sink {
    bytes: Option<Vec<u8>>,
    written: usize,
    limit: usize,
    error: Option<RenderError>,
}
impl Sink {
    fn bytes(&mut self, bytes: &[u8]) -> Result {
        let Some(next) = self
            .written
            .checked_add(bytes.len())
            .filter(|n| *n <= self.limit)
        else {
            self.error = Some(RenderError::Limit);
            return Err(RenderError::Limit);
        };
        if let Some(output) = &mut self.bytes {
            output.extend_from_slice(bytes);
        }
        self.written = next;
        Ok(())
    }
    pub(crate) fn raw(&mut self, text: &str) -> Result {
        self.bytes(text.as_bytes())
    }
    pub(crate) fn formatted(&mut self, args: fmt::Arguments<'_>) -> Result {
        fmt::write(self, args).map_err(|_| self.error.unwrap_or(RenderError::Json))
    }
    pub(crate) fn escaped(&mut self, text: &str, textarea: bool) -> Result {
        if textarea {
            self.raw("\n")?;
        }
        escape_chunks(text.as_bytes(), textarea, |part| self.bytes(part))
    }
    fn json(&mut self, value: &serde_json::Value) -> Result {
        struct Escaping<'a>(&'a mut Sink);
        impl io::Write for Escaping<'_> {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                escape_chunks(bytes, false, |part| self.0.bytes(part))
                    .map_err(|_| io::Error::other("HTML limit"))?;
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        serde_json::to_writer(Escaping(self), value)
            .map_err(|_| self.error.unwrap_or(RenderError::Json))
    }
    fn state(&mut self, attribute: &AttributeDescriptor) -> Result {
        match &attribute.value {
            None => self.raw("Not set (no saved value)."),
            Some(serde_json::Value::Null) => self.raw("Explicitly empty (null)."),
            Some(value) if short_saved_value(value) => {
                self.raw("Saved value: ")?;
                self.json(value)
            }
            Some(_) => self.raw("Saved value is long. Expand Full saved value to read or copy it."),
        }
    }
    fn saved_disclosure(&mut self, attribute: &AttributeDescriptor) -> Result {
        let Some(value) = attribute
            .value
            .as_ref()
            .filter(|value| !short_saved_value(value))
        else {
            return Ok(());
        };
        let name = Text(attribute.name, false);
        let label = Label(attribute.name);
        self.formatted(format_args!("<details id=\"saved-value-{name}\"><summary>Full saved value for {label}</summary><label for=\"saved-json-{name}\">Saved JSON value for {label}</label><textarea id=\"saved-json-{name}\" readonly rows=\"6\" wrap=\"off\" style=\"white-space:pre;overflow-wrap:normal;word-break:normal;overflow:auto;max-height:12em\">\n"))?;
        self.json(value)?;
        self.raw("</textarea></details>")
    }
}
// Bound the compact-JSON counter and reject long strings by byte length before
// asking serde to scan their payload. No intermediate JSON String is owned.
const INLINE_JSON_BYTES: usize = 80;
fn short_saved_value(value: &serde_json::Value) -> bool {
    if value
        .as_str()
        .is_some_and(|text| text.len() > INLINE_JSON_BYTES - 2)
    {
        return false;
    }
    struct Count(usize);
    impl io::Write for Count {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .filter(|n| *n <= INLINE_JSON_BYTES)
                .ok_or_else(|| io::Error::other("long saved value"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count(0), value).is_ok()
}
impl fmt::Write for Sink {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.raw(text).map_err(|_| fmt::Error)
    }
}
pub(crate) fn collect(limit: usize, emit: impl Fn(&mut Sink) -> Result) -> Result<String> {
    if limit > MAX_BYTES {
        return Err(RenderError::InvalidLimit);
    }
    let mut count = Sink {
        bytes: None,
        written: 0,
        limit,
        error: None,
    };
    emit(&mut count)?;
    if let Some(error) = count.error {
        return Err(error);
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(count.written)
        .map_err(|_| RenderError::Allocation)?;
    let mut output = Sink {
        bytes: Some(bytes),
        written: 0,
        limit: count.written,
        error: None,
    };
    emit(&mut output)?;
    if let Some(error) = output.error {
        return Err(error);
    }
    if output.written != count.written {
        return Err(RenderError::Limit);
    }
    String::from_utf8(output.bytes.unwrap()).map_err(|_| RenderError::Json)
}
fn escape_chunks(bytes: &[u8], textarea: bool, mut write: impl FnMut(&[u8]) -> Result) -> Result {
    let mut start = 0;
    for (index, byte) in bytes.iter().enumerate() {
        let replacement: Option<&[u8]> = match byte {
            b'&' => Some(b"&amp;"),
            b'<' => Some(b"&lt;"),
            b'>' => Some(b"&gt;"),
            b'"' => Some(b"&quot;"),
            b'\'' => Some(b"&#39;"),
            b'\r' if textarea => Some(b"&#13;"),
            0 if textarea => Some("\u{fffd}".as_bytes()),
            _ => None,
        };
        if let Some(replacement) = replacement {
            write(&bytes[start..index])?;
            write(replacement)?;
            start = index + 1;
        }
    }
    write(&bytes[start..])
}
struct Text<'a>(&'a str, bool);
impl fmt::Display for Text<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.1 {
            f.write_str("\n")?;
        }
        escape_chunks(self.0.as_bytes(), self.1, |part| {
            f.write_str(std::str::from_utf8(part).map_err(|_| RenderError::Json)?)
                .map_err(|_| RenderError::Limit)
        })
        .map_err(|_| fmt::Error)
    }
}
struct Label<'a>(&'a str);
impl fmt::Display for Label<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = [0; 4];
        for (index, ch) in self.0.chars().enumerate() {
            if index == 0 {
                for upper in ch.to_uppercase() {
                    write!(f, "{}", Text(upper.encode_utf8(&mut buffer), false))?;
                }
            } else {
                if ch.is_uppercase() {
                    f.write_str(" ")?;
                }
                for lower in ch.to_lowercase() {
                    write!(f, "{}", Text(lower.encode_utf8(&mut buffer), false))?;
                }
            }
        }
        Ok(())
    }
}
pub(crate) fn escaped(text: &str, textarea: bool, limit: usize) -> Result<String> {
    collect(limit, |out| out.escaped(text, textarea))
}
// Descriptors are supplied by admitted-model inspection or finite trusted tests,
// not an arbitrary untrusted Value API. JSON chunks may split UTF8; byte sinks
// preserve them until the complete output's one UTF8 validation.
pub(crate) fn emit_field(out: &mut Sink, attribute: &AttributeDescriptor) -> Result {
    let Some(kind) = &attribute.value_type else {
        return Ok(());
    };
    let name = Text(attribute.name, false);
    let label = Label(attribute.name);
    out.formatted(format_args!("<form class=\"scalar-property\" data-action=\"attribute\" data-attribute=\"{name}\" aria-label=\"Edit {label}\"><input type=\"hidden\" name=\"attribute\" value=\"{name}\"><label for=\"scalar-{name}\">{label}</label><p id=\"scalar-state-{name}\" class=\"property-state\">"))?;
    out.state(attribute)?;
    out.raw("</p>")?;
    out.saved_disclosure(attribute)?;
    match kind {
        ScalarType::String => {
            let rows = if attribute.name == "declaredShortName" {
                1
            } else {
                3
            };
            let text = Text(
                attribute
                    .value
                    .as_ref()
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(""),
                true,
            );
            out.formatted(format_args!("<textarea id=\"scalar-{name}\" name=\"value\" rows=\"{rows}\" aria-describedby=\"scalar-state-{name}\">{text}</textarea>"))?;
        }
        ScalarType::Boolean => {
            let current = attribute
                .value
                .as_ref()
                .and_then(serde_json::Value::as_bool);
            out.formatted(format_args!("<select id=\"scalar-{name}\" name=\"value\" aria-describedby=\"scalar-state-{name}\" required><option value=\"\" disabled {}>Choose true or false</option><option value=\"true\" {}>True</option><option value=\"false\" {}>False</option></select>", if current.is_none() { "selected" } else { "" }, if current == Some(true) { "selected" } else { "" }, if current == Some(false) { "selected" } else { "" }))?;
        }
        ScalarType::Integer32 | ScalarType::Double => {
            let bounds = if *kind == ScalarType::Integer32 {
                "step=\"1\" min=\"-2147483648\" max=\"2147483647\""
            } else {
                "step=\"any\""
            };
            out.formatted(format_args!(
                "<input type=\"number\" id=\"scalar-{name}\" name=\"value\" value=\""
            ))?;
            if let Some(value) = attribute.value.as_ref().filter(|value| value.is_number()) {
                out.json(value)?;
            }
            out.formatted(format_args!(
                "\" {bounds} aria-describedby=\"scalar-state-{name}\" required>"
            ))?;
        }
        ScalarType::Enumeration { literals, .. } => {
            let current = attribute.value.as_ref().and_then(serde_json::Value::as_str);
            let recognized = current.is_some_and(|value| literals.contains(&value));
            out.formatted(format_args!("<select id=\"scalar-{name}\" name=\"value\" aria-describedby=\"scalar-state-{name}\" required><option value=\"\" disabled {}>Choose a value</option>", if recognized { "" } else { "selected" }))?;
            for literal in literals {
                out.formatted(format_args!(
                    "<option value=\"{}\" {}>{}</option>",
                    Text(literal, false),
                    if current == Some(*literal) {
                        "selected"
                    } else {
                        ""
                    },
                    Text(literal, false)
                ))?;
            }
            out.raw("</select>")?;
        }
    }
    let null_option = if attribute.accepts_null() {
        "<option value=\"null\">Clear value (null)</option>"
    } else {
        ""
    };
    out.formatted(format_args!("<label for=\"scalar-mode-{name}\">Change for {label}</label><select id=\"scalar-mode-{name}\" name=\"mode\"><option value=\"value\">Set a value</option>{null_option}</select><div class=\"form-actions\"><button id=\"stage-attribute-{name}\" type=\"submit\">Stage {label}</button></div></form>"))
}
#[cfg(test)]
pub(crate) fn scalar_field(attribute: &AttributeDescriptor, limit: usize) -> Result<String> {
    collect(limit, |out| emit_field(out, attribute))
}
pub(crate) fn emit_inspection(out: &mut Sink, inspection: &NodeInspection, active: &str) -> Result {
    out.raw("<section aria-label=\"Element properties\"><h3>Properties</h3><p>Stage a property change, then commit or cancel it above.</p>")?;
    for attribute in inspection
        .attributes
        .iter()
        .filter(|a| a.blocked.is_none() && common_property(a.name))
    {
        emit_field(out, attribute)?;
    }
    let advanced = inspection
        .attributes
        .iter()
        .filter(|a| a.blocked.is_none() && !common_property(a.name))
        .count();
    if advanced != 0 {
        let open = if !common_property(active)
            && inspection.writable_attributes().any(|a| a.name == active)
        {
            " open"
        } else {
            ""
        };
        out.formatted(format_args!("<details id=\"advanced-properties\"{open}><summary>More properties ({advanced})</summary>"))?;
        for attribute in inspection
            .attributes
            .iter()
            .filter(|a| a.blocked.is_none() && !common_property(a.name))
        {
            emit_field(out, attribute)?;
        }
        out.raw("</details>")?;
    }
    let count = inspection
        .attributes
        .iter()
        .filter(|a| a.blocked.is_some())
        .count()
        + inspection.diagnostics.len();
    out.formatted(format_args!("<details id=\"unavailable-properties\"><summary>Read-only and unavailable properties ({count})</summary><dl>"))?;
    for attribute in &inspection.attributes {
        if let Some(reason) = &attribute.blocked {
            out.formatted(format_args!(
                "<dt>{}</dt><dd>{} ",
                Label(attribute.name),
                blocked_label(reason)
            ))?;
            out.state(attribute)?;
            out.saved_disclosure(attribute)?;
            out.raw("</dd>")?;
        }
    }
    for diagnostic in &inspection.diagnostics {
        let reason = match diagnostic.issue {
            AttributeError::ReferenceEditUnsupported {
                containment: true, ..
            } => "Contains other elements; editing is unavailable.",
            AttributeError::ReferenceEditUnsupported {
                containment: false, ..
            } => "Links to other elements; editing is unavailable.",
            _ => "This property is unavailable for editing.",
        };
        out.formatted(format_args!(
            "<dt>{}</dt><dd>{reason}</dd>",
            Label(&diagnostic.name)
        ))?;
    }
    out.raw("</dl></details>")?;
    if !inspection.unresolved_supertypes.is_empty() {
        out.raw("<p>Some inherited properties are unavailable.</p>")?;
    }
    out.raw("</section>")
}
pub(crate) fn scalar_inspector(
    doc: &Document,
    selected: &str,
    active: &str,
    limit: usize,
) -> Result<String> {
    if limit > MAX_BYTES {
        return Err(RenderError::InvalidLimit);
    }
    doc.json_size(super::MODEL_BYTES, || false)
        .map_err(|_| RenderError::Model)?;
    let inspection = doc.inspect_node(selected).ok();
    collect(limit, |out| match &inspection {
        Some(inspection) => emit_inspection(out, inspection, active),
        None => out.raw("<p>Properties are unavailable for this element type.</p>"),
    })
}

#[cfg(test)]
#[path = "../tests/support/rr18b3c_alloc.rs"]
mod allocation;

#[cfg(test)]
mod tests {
    use super::allocation::observed;
    use super::*;
    fn packet() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../../../../porting/rust-parity-evidence/RR18b3c/fixtures/legacy-helpers.json"
        ))
        .unwrap()
    }
    fn descriptor(text: &str) -> AttributeDescriptor {
        let raw = serde_json::json!({"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},
            "content":[{"id":"resource","eClass":"sysml:Package","data":{"elementId":"element","declaredName":text}}]});
        Document::from_json(&raw.to_string())
            .unwrap()
            .inspect_node("element")
            .unwrap()
            .attributes
            .into_iter()
            .find(|a| a.name == "declaredName")
            .unwrap()
    }
    #[test]
    fn original_escape_bytes_match_all_frozen_native_helper_oracles() {
        for row in packet()["escapes"].as_array().unwrap() {
            assert_eq!(
                escaped(
                    row["input"].as_str().unwrap(),
                    row["textarea"].as_bool().unwrap(),
                    MAX_BYTES
                )
                .unwrap(),
                row["expected"].as_str().unwrap(),
                "{}",
                row["id"]
            );
        }
    }
    #[test]
    fn original_scalar_field_bytes_and_exact_wrapper_budgets_match() {
        for row in packet()["fields"].as_array().unwrap() {
            let doc = Document::from_json(row["model"].as_str().unwrap()).unwrap();
            let inspection = doc.inspect_node(row["selected"].as_str().unwrap()).unwrap();
            let attr = inspection
                .attributes
                .iter()
                .find(|a| a.name == row["attribute"].as_str().unwrap())
                .unwrap();
            let expected = row["expected"].as_str().unwrap();
            assert_eq!(
                scalar_field(attr, expected.len()).unwrap(),
                expected,
                "{}",
                row["id"]
            );
            if !expected.is_empty() {
                assert_eq!(
                    scalar_field(attr, expected.len() - 1),
                    Err(RenderError::Limit)
                );
            }
        }
    }
    #[test]
    fn original_complete_inspector_bytes_and_budgets_match() {
        for row in packet()["inspectors"].as_array().unwrap() {
            let doc = Document::from_json(row["model"].as_str().unwrap()).unwrap();
            let expected = row["expected"].as_str().unwrap();
            let selected = row["selected"].as_str().unwrap();
            let active = row["active"].as_str().unwrap();
            assert_eq!(
                scalar_inspector(&doc, selected, active, expected.len()).unwrap(),
                expected,
                "{}",
                row["id"]
            );
            assert_eq!(
                scalar_inspector(&doc, selected, active, expected.len() - 1),
                Err(RenderError::Limit)
            );
        }
    }
    #[test]
    fn manual_unicode_entities_textarea_sentinel_and_byte_chunk_boundaries() {
        let input = "\n\r\0雪🧭&<>'\"";
        let plain = "\n\r\0雪🧭&amp;&lt;&gt;&#39;&quot;";
        let textarea = "\n\n&#13;\u{fffd}雪🧭&amp;&lt;&gt;&#39;&quot;";
        assert_eq!((input.len(), plain.len(), textarea.len()), (15, 34, 41));
        assert_eq!(escaped(input, false, 34).unwrap(), plain);
        assert_eq!(escaped(input, true, 41).unwrap(), textarea);
        assert_eq!(escaped(input, true, 40), Err(RenderError::Limit));
        assert_eq!(
            escaped(&(input.to_owned() + "x"), true, 41),
            Err(RenderError::Limit)
        );
        assert_eq!(escaped("&quot;", false, 10).unwrap(), "&amp;quot;");
        assert_eq!(escaped("", false, 0).unwrap(), "");
        assert_eq!(escaped("", true, 0), Err(RenderError::Limit));
        assert_eq!(escaped("", true, 1).unwrap(), "\n");
        assert_eq!(
            escaped("", false, MAX_BYTES + 1),
            Err(RenderError::InvalidLimit)
        );
        for size in 1..=input.len() {
            let output = collect(41, |out| {
                out.raw("\n")?;
                for chunk in input.as_bytes().chunks(size) {
                    escape_chunks(chunk, true, |part| out.bytes(part))?;
                }
                Ok(())
            })
            .unwrap();
            assert_eq!(output, textarea);
        }
    }
    #[test]
    fn literal_number_string_marker_objects_and_json_utf8_chunks_stay_distinct() {
        let raw = r#"{"json":{"version":"1.0"},"ns":{"sysml":"http://www.eclipse.org/syson/sysml"},"content":[],"opaque":{"zero":-0,"exp":1e+003,"big":123456789012345678901234567890,"string":"7","number":{"$serde_json::private::Number":"7"},"raw":{"$serde_json::private::RawValue":"false"}}}"#;
        let doc = Document::from_json(raw).unwrap();
        let values = &doc.value()["opaque"];
        for (key, expected) in [
            ("zero", "-0"),
            ("exp", "1e+003"),
            ("big", "123456789012345678901234567890"),
            ("string", "&quot;7&quot;"),
            (
                "number",
                "{&quot;$serde_json::private::Number&quot;:&quot;7&quot;}",
            ),
            (
                "raw",
                "{&quot;$serde_json::private::RawValue&quot;:&quot;false&quot;}",
            ),
        ] {
            assert_eq!(
                collect(MAX_BYTES, |out| out.json(&values[key])).unwrap(),
                expected
            );
        }
        let encoded = "\"雪🧭\"";
        for size in 1..=encoded.len() {
            assert_eq!(
                collect(19, |out| {
                    for part in encoded.as_bytes().chunks(size) {
                        escape_chunks(part, false, |bytes| out.bytes(bytes))?;
                    }
                    Ok(())
                })
                .unwrap(),
                "&quot;雪🧭&quot;"
            );
        }
    }
    #[test]
    fn prepared_large_scalar_and_escaping_refuse_before_output_copy() {
        let attr = descriptor(&"'".repeat(1_700_000));
        let text = "&".repeat(4_000_000);
        for (result, stats) in [
            observed(|| scalar_field(&attr, MAX_BYTES)),
            observed(|| escaped(&text, false, MAX_BYTES)),
        ] {
            assert_eq!(result, Err(RenderError::Limit));
            eprintln!("RR18B3C helper refusal: {stats:?}");
            assert!(stats.requested <= 4096 && stats.largest <= 1024);
        }
    }
    #[test]
    fn whole_inspector_is_counted_before_any_individually_fitting_field_copy() {
        let mut attr = descriptor(&"&".repeat(2000));
        let mut second = attr.clone();
        second.name = "declaredShortName";
        let node = NodeInspection {
            classifier: syson_metamodel::ClassifierId::Package,
            attributes: vec![attr.clone(), second],
            diagnostics: vec![],
            unresolved_supertypes: vec![],
        };
        let all = collect(MAX_BYTES, |out| emit_inspection(out, &node, "")).unwrap();
        let limit = all.len() - 1;
        for attribute in &node.attributes {
            assert!(scalar_field(attribute, limit).is_ok());
        }
        let (result, stats) = observed(|| collect(limit, |out| emit_inspection(out, &node, "")));
        assert_eq!(result, Err(RenderError::Limit));
        assert!(stats.requested <= 4096 && stats.largest <= 1024);
        attr.value = None;
        assert!(
            scalar_field(&attr, MAX_BYTES)
                .unwrap()
                .contains("Not set (no saved value).")
        );
        attr.value = Some(serde_json::Value::Null);
        assert!(
            scalar_field(&attr, MAX_BYTES)
                .unwrap()
                .contains("Explicitly empty (null).")
        );
    }
    #[test]
    fn swallowed_count_error_still_refuses_before_reservation() {
        let (result, stats) = observed(|| {
            collect(0, |out| {
                let _ = out.raw("x");
                Ok(())
            })
        });
        assert_eq!(result, Err(RenderError::Limit));
        assert_eq!((stats.requested, stats.largest), (0, 0));
    }
    #[test]
    fn swallowed_emission_error_cannot_publish_an_equal_length_output() {
        let pass = std::cell::Cell::new(0);
        let result = collect(2, |out| {
            if pass.get() == 0 {
                out.raw("xy")?;
            } else {
                let _ = out.raw("xyz");
                out.raw("xy")?;
            }
            pass.set(pass.get() + 1);
            Ok(())
        });
        assert_eq!(pass.get(), 2);
        assert_eq!(result, Err(RenderError::Limit));
    }
}

// SPDX-License-Identifier: EPL-2.0
// Behavioral sources: Langium 1.2.1 (TypeFox), Syside (Sensmetry UAB), SysON (Obeo).
// Exact source mapping and retained notices: NOTICE.md and porting/follow-ons/F10.
//! Primitive conversion only: no parsing, CST-rule resolution, linking or semantic import.
#![forbid(unsafe_code)]

#[derive(Clone, Debug, PartialEq)]
pub enum Presence<T> {
    Undefined,
    Null,
    Value(T),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lexeme<'a> {
    pub text: &'a str,
    pub span: Span,
}
#[derive(Clone, Copy, Debug)]
pub struct Rule<'a> {
    pub name: &'a str,
    pub declared_type: Option<&'a str>,
}
#[derive(Clone, Copy, Debug)]
pub enum Dispatch<'a> {
    Raw,
    Rule(Rule<'a>),
    UnresolvedRuleCall,
}
/// UTF-16 units preserve JavaScript slicing, including malformed-input lone surrogates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsString(pub Vec<u16>);
impl JsString {
    pub fn to_utf8(&self) -> Result<String, std::string::FromUtf16Error> {
        String::from_utf16(&self.0)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    String(JsString),
    Number(f64),
    Boolean(bool),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Converted<'a> {
    pub lexeme: Lexeme<'a>,
    pub value: Primitive,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Cancelled,
    BudgetExceeded,
    InvalidSpan,
    NonAsciiDispatch,
    UnresolvedRule,
    UnsupportedBigInt,
    UnsupportedDate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConversionError {
    pub kind: ErrorKind,
    pub span: Option<Span>,
}
/// Cancellation is checked before conversion, while materializing UTF-16, and after conversion.
/// A bounded standard-library numeric parse itself is not preemptible.
pub struct Control<'a> {
    pub max_utf16_units: usize,
    pub is_cancelled: &'a dyn Fn() -> bool,
}
impl Control<'_> {
    fn check(&self) -> Result<(), ErrorKind> {
        if (self.is_cancelled)() {
            Err(ErrorKind::Cancelled)
        } else {
            Ok(())
        }
    }
    fn units(&self, text: &str) -> Result<Vec<u16>, ErrorKind> {
        self.check()?;
        let mut units = Vec::new();
        for unit in text.encode_utf16() {
            if units.len() == self.max_utf16_units {
                return Err(ErrorKind::BudgetExceeded);
            }
            if units.len() % 1024 == 0 {
                self.check()?;
            }
            units.push(unit);
        }
        Ok(units)
    }
}

pub fn convert<'a>(
    dispatch: Dispatch<'_>,
    input: Presence<Lexeme<'a>>,
    control: &Control<'_>,
) -> Result<Presence<Converted<'a>>, ConversionError> {
    let span = match &input {
        Presence::Value(v) => Some(v.span),
        _ => None,
    };
    let run = || -> Result<_, ErrorKind> {
        control.check()?;
        let lexeme = match input {
            Presence::Undefined => return Ok(Presence::Undefined),
            Presence::Null => return Ok(Presence::Null),
            Presence::Value(v) => v,
        };
        if lexeme.span.end.checked_sub(lexeme.span.start) != Some(lexeme.text.len()) {
            return Err(ErrorKind::InvalidSpan);
        }
        let units = control.units(lexeme.text)?;
        let text = lexeme.text;
        let value = match dispatch {
            Dispatch::UnresolvedRuleCall => return Err(ErrorKind::UnresolvedRule),
            Dispatch::Raw => Primitive::String(JsString(units)),
            Dispatch::Rule(rule) => {
                if !rule.name.is_ascii() || rule.declared_type.is_some_and(|v| !v.is_ascii()) {
                    return Err(ErrorKind::NonAsciiDispatch);
                }
                if rule.name.eq_ignore_ascii_case("INT") {
                    Primitive::Number(parse_int(text))
                } else if rule.name.eq_ignore_ascii_case("STRING") {
                    Primitive::String(convert_string(&units))
                } else if rule.name.eq_ignore_ascii_case("ID") {
                    Primitive::String(JsString(if units.first() == Some(&94) {
                        units[1..].to_vec()
                    } else {
                        units
                    }))
                } else if rule.name.eq_ignore_ascii_case("REGEXLITERAL") {
                    let a = 1.min(units.len());
                    let b = units.len().saturating_sub(1);
                    Primitive::String(JsString(units[a.min(b)..a.max(b)].to_vec()))
                } else {
                    match rule
                        .declared_type
                        .unwrap_or("")
                        .to_ascii_lowercase()
                        .as_str()
                    {
                        "number" => Primitive::Number(number(text)),
                        "boolean" => Primitive::Boolean(text.eq_ignore_ascii_case("true")),
                        "bigint" => return Err(ErrorKind::UnsupportedBigInt),
                        "date" => return Err(ErrorKind::UnsupportedDate),
                        _ => Primitive::String(JsString(units)),
                    }
                }
            }
        };
        control.check()?;
        Ok(Presence::Value(Converted { lexeme, value }))
    };
    run().map_err(|kind| ConversionError { kind, span })
}

fn escape(unit: u16) -> u16 {
    match unit {
        98 => 8,
        102 => 12,
        110 => 10,
        114 => 13,
        116 => 9,
        118 => 11,
        48 => 0,
        other => other,
    }
}
fn convert_string(units: &[u16]) -> JsString {
    let mut out = Vec::new();
    let mut i = 1;
    while i < units.len().saturating_sub(1) {
        let mut unit = units[i];
        if unit == 92 {
            i += 1;
            unit = escape(units.get(i).copied().unwrap_or(0));
        }
        out.push(unit);
        i += 1;
    }
    JsString(out)
}

/// Later Syside name processing, separate from STRING/STRING_VALUE conversion.
/// Undefined and the empty declared string sanitize to undefined; quoted empty remains present.
pub fn sanitize_name(
    raw: Option<&str>,
    control: &Control<'_>,
) -> Result<Option<JsString>, ErrorKind> {
    control.check()?;
    let Some(raw) = raw.filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let units = control.units(raw)?;
    if units.first() != Some(&39) {
        control.check()?;
        return Ok(Some(JsString(units)));
    }
    let body = &units[1.min(units.len())..units.len().saturating_sub(1).max(1.min(units.len()))];
    let mut out = Vec::new();
    let mut i = 0;
    while i < body.len() {
        if body[i] == 92
            && body
                .get(i + 1)
                .is_some_and(|u| [98, 116, 110, 102, 114, 34, 39, 92].contains(u))
        {
            i += 1;
            out.push(escape(body[i]));
        } else {
            out.push(body[i]);
        }
        i += 1;
    }
    control.check()?;
    Ok(Some(JsString(out)))
}

/// Exact ASCII SimpleNameRegex and name-escaping layer; keywords are a caller concern.
pub fn unsanitize_name(name: &JsString, control: &Control<'_>) -> Result<JsString, ErrorKind> {
    control.check()?;
    if name.0.len() > control.max_utf16_units {
        return Err(ErrorKind::BudgetExceeded);
    }
    let letter = |u: u16| u == 95 || (65..=90).contains(&u) || (97..=122).contains(&u);
    if name.0.first().is_some_and(|u| letter(*u))
        && name.0.iter().all(|u| letter(*u) || (48..=57).contains(u))
    {
        control.check()?;
        return Ok(name.clone());
    }
    if control.max_utf16_units < 2 {
        return Err(ErrorKind::BudgetExceeded);
    }
    let mut out = vec![39];
    for unit in &name.0 {
        if let Some(c) = match unit {
            8 => Some(98),
            9 => Some(116),
            10 => Some(110),
            12 => Some(102),
            13 => Some(114),
            39 => Some(39),
            92 => Some(92),
            _ => None,
        } {
            out.extend([92, c]);
        } else {
            out.push(*unit);
        }
        if out.len().saturating_add(1) > control.max_utf16_units {
            return Err(ErrorKind::BudgetExceeded);
        }
    }
    out.push(39);
    control.check()?;
    Ok(JsString(out))
}

fn js_space(c: char) -> bool {
    matches!(c,'\u{0009}'..='\u{000d}'|'\u{0020}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}')
}
fn decimal(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut i = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let start = i;
    while bytes.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    let mut digits = i - start;
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        digits += i - start;
    }
    if digits == 0 {
        return false;
    }
    if matches!(bytes.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(bytes.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == start {
            return false;
        }
    }
    i == bytes.len()
}
fn number(input: &str) -> f64 {
    let text = input.trim_matches(js_space);
    if text.is_empty() {
        return 0.0;
    }
    match text {
        "Infinity" | "+Infinity" => return f64::INFINITY,
        "-Infinity" => return f64::NEG_INFINITY,
        _ => {}
    }
    if let Some(digits) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        return radix(digits, 4);
    }
    if let Some(digits) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O")) {
        return radix(digits, 3);
    }
    if let Some(digits) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
        return radix(digits, 1);
    }
    if decimal(text) {
        text.parse().unwrap_or(f64::NAN)
    } else {
        f64::NAN
    }
}
fn parse_int(input: &str) -> f64 {
    let text = input.trim_start_matches(js_space);
    let negative = text.starts_with('-');
    let text = if text.starts_with(['+', '-']) {
        &text[1..]
    } else {
        text
    };
    let value = if let Some(digits) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        let end = digits
            .find(|c: char| !c.is_ascii_hexdigit())
            .unwrap_or(digits.len());
        radix(&digits[..end], 4)
    } else {
        let end = text
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(text.len());
        if end == 0 {
            f64::NAN
        } else {
            text[..end].parse().unwrap_or(f64::NAN)
        }
    };
    if negative { -value } else { value }
}
/// Correctly round power-of-two integer strings once, retaining guard/sticky bits.
fn radix(digits: &str, shift: u32) -> f64 {
    if digits.is_empty() {
        return f64::NAN;
    }
    let mut significant = 0u64;
    let mut count = 0usize;
    let mut guard = false;
    let mut sticky = false;
    for c in digits.chars() {
        let Some(digit) = c.to_digit(1 << shift).filter(|_| c.is_ascii()) else {
            return f64::NAN;
        };
        for bit in (0..shift).rev().map(|n| (digit >> n) & 1) {
            if count == 0 && bit == 0 {
                continue;
            }
            count += 1;
            if count <= 53 {
                significant = (significant << 1) | u64::from(bit);
            } else if count == 54 {
                guard = bit != 0;
            } else {
                sticky |= bit != 0;
            }
        }
    }
    if count <= 53 {
        return significant as f64;
    }
    if guard && (sticky || significant & 1 != 0) {
        significant += 1;
    }
    if significant == 1 << 53 {
        significant >>= 1;
        count += 1;
    }
    let exponent = count - 1 + 1023;
    if exponent >= 2047 {
        f64::INFINITY
    } else {
        f64::from_bits(((exponent as u64) << 52) | (significant & ((1 << 52) - 1)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SysOnLiteralKind {
    Integer,
    Rational,
}
/// The Java translator uses original CST spelling, not the rounded numeric value.
pub fn syson_literal_kind(raw: &str) -> SysOnLiteralKind {
    if raw.contains(['.', 'E', 'e']) {
        SysOnLiteralKind::Rational
    } else {
        SysOnLiteralKind::Integer
    }
}

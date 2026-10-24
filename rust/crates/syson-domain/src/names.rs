// Copyright (c) 2024, 2026 Obeo.
// This program and accompanying materials are made available under the terms
// of the Eclipse Public License v2.0, https://www.eclipse.org/legal/epl-2.0/.
// SPDX-License-Identifier: EPL-2.0
// Contributors: Obeo - initial API and implementation
// Rust translation and compatibility tests: SysON Rust port contributors.

//! Translations of NameHelper and NameConflictingFilter.
use std::collections::HashSet;

/// Exact word keyword set in upstream SysMLv2Keywords (operators excluded).
pub const KEYWORDS: &[&str] = &[
    "dependency",
    "from",
    "to",
    "comment",
    "about",
    "locale",
    "doc",
    "rep",
    "language",
    "metadata",
    "def",
    "abstract",
    "ref",
    "redefines",
    "standard",
    "library",
    "package",
    "filter",
    "alias",
    "for",
    "import",
    "all",
    "public",
    "private",
    "protected",
    "specializes",
    "ordered",
    "nonunique",
    "defined",
    "by",
    "subsets",
    "references",
    "crosses",
    "variation",
    "variant",
    "in",
    "out",
    "inout",
    "derived",
    "constant",
    "end",
    "default",
    "attribute",
    "enum",
    "occurrence",
    "individual",
    "snapshot",
    "timeslice",
    "event",
    "then",
    "item",
    "part",
    "port",
    "binding",
    "bind",
    "succession",
    "first",
    "connection",
    "connect",
    "interface",
    "allocation",
    "allocate",
    "flow",
    "message",
    "of",
    "action",
    "perform",
    "accept",
    "via",
    "at",
    "after",
    "when",
    "send",
    "assign",
    "if",
    "else",
    "while",
    "loop",
    "until",
    "terminate",
    "merge",
    "decide",
    "join",
    "fork",
    "state",
    "parallel",
    "entry",
    "do",
    "exit",
    "exhibit",
    "transition",
    "calc",
    "return",
    "constraint",
    "assert",
    "not",
    "requirement",
    "subject",
    "assume",
    "require",
    "frame",
    "actor",
    "stakeholder",
    "satisfy",
    "concern",
    "case",
    "objective",
    "analysis",
    "verification",
    "verify",
    "use",
    "include",
    "view",
    "render",
    "rendering",
    "expose",
    "viewpoint",
];

/// Java String.split("::") semantics, including discarded trailing empties.
/// This deliberately does not parse quoted sections (upstream does not either).
pub fn parse_qualified_name(name: &str) -> Vec<&str> {
    if name.is_empty() {
        return vec![""];
    }
    let mut parts: Vec<_> = name.split("::").collect();
    while parts.last() == Some(&"") {
        parts.pop();
    }
    parts
}

/// Only removes a pair of outer single quotes; does not interpret escapes.
pub fn unescape_string(name: Option<&str>) -> Option<&str> {
    name.map(|s| {
        if s.len() > 1 && s.starts_with('\'') && s.ends_with('\'') {
            &s[1..s.len() - 1]
        } else {
            s
        }
    })
}

/// Upstream printable identifier rules are ASCII and case sensitive.
pub fn printable_name(name: Option<&str>, escape_single_quotes: bool) -> String {
    let Some(name) = name.filter(|s| !s.is_empty()) else {
        return String::new();
    };
    let first = name.as_bytes()[0];
    if !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        || !(first.is_ascii_alphabetic() || first == b'_')
        || KEYWORDS.contains(&name)
    {
        let body = if escape_single_quotes {
            name.replace('\'', "\\'")
        } else {
            name.to_owned()
        };
        format!("'{body}'")
    } else {
        name.to_owned()
    }
}

/// Apache Commons Text escapeJava-compatible escaping over Java UTF-16 units.
/// The Java helper's null passthrough is represented by Option.
pub fn escape_string(value: Option<&str>) -> Option<String> {
    use std::fmt::Write;
    value.map(|value| {
        let mut out = String::new();
        for c in value.encode_utf16() {
            match c {
                8 => out.push_str("\\b"),
                9 => out.push_str("\\t"),
                10 => out.push_str("\\n"),
                12 => out.push_str("\\f"),
                13 => out.push_str("\\r"),
                34 => out.push_str("\\\""),
                92 => out.push_str("\\\\"),
                32..=127 => out.push(char::from_u32(u32::from(c)).unwrap()),
                _ => {
                    write!(out, "\\u{c:04X}").unwrap();
                }
            }
        }
        out
    })
}

/// Optional quotes and whitespace semantics from ElementUtil.nameMatches.
pub fn name_matches(candidate: Option<&str>, query: Option<&str>) -> bool {
    let (Some(candidate), Some(query)) = (candidate, query) else {
        return false;
    };
    candidate.trim_matches(java_whitespace) == query.trim_matches(java_whitespace)
        || (query.starts_with('\'')
            && query.ends_with('\'')
            && format!("'{candidate}'").trim_matches(java_whitespace)
                == query.trim_matches(java_whitespace))
}

/// Stateful, ordered filter. Failure on short name does not roll back the name.
#[derive(Debug, Default)]
pub struct NameConflictingFilter {
    used: HashSet<String>,
}
impl NameConflictingFilter {
    pub fn check_name(&mut self, name: Option<&str>) -> bool {
        name.is_none_or(|name| self.used.insert(name.to_owned()))
    }
    pub fn check_element(&mut self, name: Option<&str>, short_name: Option<&str>) -> bool {
        self.check_name(name) && self.check_name(short_name)
    }
    /// Alias identity is explicit: Java chooses the alias by reference identity,
    /// not String equality. Callers must preserve that distinction when adapting.
    pub fn check_membership(
        &mut self,
        alias: Option<&str>,
        alias_is_distinct: bool,
        member: Option<(Option<&str>, Option<&str>)>,
    ) -> bool {
        match (alias, alias_is_distinct, member) {
            (Some(alias), true, Some(_)) => self.check_name(Some(alias)),
            (_, _, Some((name, short))) => self.check_element(name, short),
            _ => true,
        }
    }
    pub fn fill_used_names<'a>(
        &mut self,
        elements: impl IntoIterator<Item = (Option<&'a str>, Option<&'a str>)>,
    ) {
        for (name, short) in elements {
            for value in [name, short].into_iter().flatten() {
                self.used.insert(value.to_owned());
            }
        }
    }
}

// Character.isWhitespace (Java strip), which excludes nonbreaking spaces.
fn java_whitespace(c: char) -> bool {
    matches!(c, '\u{9}'..='\u{d}' | '\u{1c}'..='\u{20}' | '\u{1680}' |
        '\u{2000}'..='\u{2006}' | '\u{2008}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{205f}' | '\u{3000}')
}

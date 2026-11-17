// SPDX-License-Identifier: EPL-2.0
//! Attribute the first finite importer refusal using a bounded syntax-only CST.
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs::File,
    io::{self, Read},
    path::Path,
};
use syson_syntax::{CstKind, Error as SyntaxError, Grammar, Language, Options, TokenKind};

const MAX_CASES: usize = 512;
const MAX_JSON_BYTES: u64 = 16 * 1024 * 1024;
const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;

// These rules identify concrete syntax, not broad grammar wrappers. A row
// counts only when the named snippet independently reaches the same rule
// and token with the same importer error family in this run.
const RULES: &[(&str, &str, &str)] = &[
    ("Abstract", "abstract", "abstract-def"),
    ("ActionDefinition", "action", "action-def"),
    ("AliasMember", "alias", "alias"),
    ("AllocationDefinition", "allocation", "allocation-def"),
    ("AttributeDefinition", "attribute", "attribute-def"),
    ("Comment", "REGULAR_COMMENT", "block-comment"),
    ("Documentation", "REGULAR_COMMENT", "doc-comment"),
    ("FeatureValue", "=", "feature-value"),
    ("FeatureValue", ":=", "default-value"),
    ("Import", "import", "import-member"),
    ("ImportedReference", "::*", "import-all"),
    ("ImportedReference", "::**", "import-recursive"),
    ("MultiplicityPart", "nonunique", "nonunique"),
    ("MultiplicityRange", "[", "multiplicity"),
    ("PortDefinition", "port", "port-def"),
    ("RequirementDefinition", "requirement", "requirement-def"),
    ("SubclassificationPart", ":>", "subclassification"),
    ("SubclassificationPart", "specializes", "specialization-kw"),
    ("Subsets", ":>", "subsetting"),
    ("UsageBody", ";", "untyped-usage"),
    ("UsageBody", "{", "usage-body"),
    ("Visibility", "private", "visibility-member"),
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Construct {
    rule: &'static str,
    token: &'static str,
    repro: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Disposition {
    NotRustRefusal,
    UnresolvedLink,
    OtherRefusal,
    InvalidOffset,
    Malformed,
    ImporterCapped,
    RecognizerCapped,
    SyntaxRejected,
    NoVerifiedRule,
    AmbiguousRule,
    ReproUnverified,
    Classified,
}

#[derive(Debug)]
struct Finding {
    disposition: Disposition,
    position: Option<(usize, usize, usize)>,
    construct: Option<Construct>,
}

fn read_json(path: &Path) -> Result<Value, Box<dyn Error>> {
    let mut raw = Vec::new();
    File::open(path)?
        .take(MAX_JSON_BYTES + 1)
        .read_to_end(&mut raw)?;
    if raw.len() as u64 > MAX_JSON_BYTES {
        return Err(invalid("oversized report input").into());
    }
    Ok(serde_json::from_slice(&raw)?)
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

// Match the recognizer's newline convention. Offsets count UTF-8 bytes;
// one-based displayed columns count UTF-16 code units.
fn location(source: &str, offset: usize) -> Option<(usize, usize, usize)> {
    let mut line = 1;
    let mut column = 1;
    let mut previous_cr = false;
    for character in source.get(..offset)?.chars() {
        match character {
            '\r' => {
                line += 1;
                column = 1;
            }
            '\n' => {
                line += usize::from(!previous_cr);
                column = 1;
            }
            _ => column += character.len_utf16(),
        }
        previous_cr = character == '\r';
    }
    Some((offset, line, column))
}

fn smallest_rule(
    candidates: impl Iterator<Item = (usize, Construct)>,
) -> Result<Option<Construct>, ()> {
    let mut shortest = usize::MAX;
    let mut matches = BTreeSet::new();
    for (width, construct) in candidates {
        if width < shortest {
            matches.clear();
            shortest = width;
        }
        if width == shortest {
            matches.insert(construct);
        }
    }
    if matches.len() > 1 {
        Err(())
    } else {
        Ok(matches.first().copied())
    }
}

fn identify(grammar: &Grammar, source: &str, error: &str, options: Options<'_>) -> Finding {
    use Disposition::*;
    let mut finding = Finding {
        disposition: OtherRefusal,
        position: None,
        construct: None,
    };
    if error.starts_with("Link(Unresolved(") {
        finding.disposition = UnresolvedLink;
        return finding;
    }
    if error == "Parse(Limit)" || error.starts_with("Parse(BudgetExceeded {") {
        finding.disposition = ImporterCapped;
        return finding;
    }
    let unsupported = error
        .strip_prefix("Parse(Unsupported { offset: ")
        .and_then(|tail| tail.strip_suffix(" })"));
    let malformed = error
        .strip_prefix("Parse(Malformed { offset: ")
        .and_then(|tail| tail.strip_suffix(" })"))
        .and_then(|tail| tail.split_once(", expected: "))
        .map(|(offset, _)| offset);
    let Some(offset) = unsupported.or(malformed) else {
        return finding;
    };
    let Ok(offset) = offset.parse::<usize>() else {
        finding.disposition = InvalidOffset;
        return finding;
    };
    finding.position = location(source, offset);
    if finding.position.is_none() {
        finding.disposition = InvalidOffset;
        return finding;
    }
    if malformed.is_some() {
        finding.disposition = Malformed;
        return finding;
    }

    let recognized = match grammar.recognize(source, options) {
        Ok(recognized) => recognized,
        Err(SyntaxError::BudgetExceeded { .. }) => {
            finding.disposition = RecognizerCapped;
            return finding;
        }
        Err(_) => {
            finding.disposition = SyntaxRejected;
            return finding;
        }
    };
    let token = recognized.tokens.iter().find(|token| {
        !token.hidden && token.span.start.byte <= offset && offset < token.span.end.byte
    });
    let Some(token) = token else {
        finding.disposition = NoVerifiedRule;
        return finding;
    };
    let (TokenKind::Keyword(token) | TokenKind::Terminal(token)) = &token.kind;
    // The finite importer refuses an untyped body before typing and a typed
    // body after typing. Only the typed case has a matching body repro here.
    let untyped_body = token == "{"
        && !recognized
            .cst
            .iter()
            .filter(|node| {
                matches!(&node.kind, CstKind::Rule(rule) if rule == "PartUsage")
                    && node.span.start.byte <= offset
                    && offset < node.span.end.byte
            })
            .min_by_key(|node| node.span.end.byte - node.span.start.byte)
            .is_some_and(|usage| {
                recognized.cst.iter().any(|node| {
                    matches!(&node.kind, CstKind::Rule(rule) if rule == "FeatureTyping")
                        && usage.span.start.byte <= node.span.start.byte
                        && node.span.end.byte <= offset
                })
            });
    let candidates = recognized.cst.iter().filter_map(|node| {
        let CstKind::Rule(rule) = &node.kind else {
            return None;
        };
        if !(node.span.start.byte <= offset && offset < node.span.end.byte) {
            return None;
        }
        if untyped_body && rule == "UsageBody" {
            return None;
        }
        let &(rule, token, repro) = RULES
            .iter()
            .find(|(name, anchor, _)| name == rule && anchor == token)?;
        Some((
            node.span.end.byte - node.span.start.byte,
            Construct { rule, token, repro },
        ))
    });
    match smallest_rule(candidates) {
        Ok(Some(construct)) => {
            finding.construct = Some(construct);
            finding.disposition = ReproUnverified;
        }
        Ok(None) => finding.disposition = NoVerifiedRule,
        Err(()) => finding.disposition = AmbiguousRule,
    }
    finding
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 120
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn collected_reports(
    dir: &Path,
    grammar: &Grammar,
) -> Result<BTreeMap<String, Finding>, Box<dyn Error>> {
    let summary = read_json(&dir.join("summary.json"))?;
    let Some(rows) = summary["cases"].as_array() else {
        return Err(invalid("missing report case list").into());
    };
    if rows.is_empty() || rows.len() > MAX_CASES {
        return Err(invalid("unsupported report case count").into());
    }
    let mut findings = BTreeMap::new();
    let mut source_bytes = 0;
    let mut verified = BTreeSet::new();
    for row in rows {
        let Some(id) = row["id"].as_str().filter(|id| valid_id(id)) else {
            return Err(invalid("invalid report case ID").into());
        };
        if findings.contains_key(id) {
            return Err(invalid("duplicate report case ID").into());
        }
        let case = read_json(&dir.join("cases").join(format!("{id}.json")))?;
        for key in [
            "id",
            "origin",
            "source_sha256",
            "verdict",
            "rust_status",
            "rust_error",
        ] {
            if case[key] != row[key] {
                return Err(
                    invalid(&format!("case {id} disagrees with summary field {key}")).into(),
                );
            }
        }
        let source = case["source"]
            .as_str()
            .ok_or_else(|| invalid("missing case source"))?;
        source_bytes += source.len();
        if source_bytes > MAX_SOURCE_BYTES {
            return Err(invalid("oversized aggregate report source").into());
        }
        let is_refusal = matches!(
            row["verdict"].as_str(),
            Some("RUST_REFUSES" | "BOTH_REFUSE")
        ) && row["rust_status"] == "error";
        let finding = if is_refusal {
            let error = row["rust_error"]
                .as_str()
                .ok_or_else(|| invalid("missing Rust refusal message"))?;
            identify(grammar, source, error, Options::default())
        } else {
            Finding {
                disposition: Disposition::NotRustRefusal,
                position: None,
                construct: None,
            }
        };
        if row["origin"] == "inline"
            && let Some(construct) = finding.construct.filter(|construct| construct.repro == id)
        {
            verified.insert(construct);
        }
        findings.insert(id.to_owned(), finding);
    }
    for finding in findings.values_mut() {
        if finding
            .construct
            .is_some_and(|construct| verified.contains(&construct))
        {
            finding.disposition = Disposition::Classified;
        }
    }
    Ok(findings)
}

fn render(findings: &BTreeMap<String, Finding>) -> String {
    use std::fmt::Write;
    let mut out = String::from("# Import refusal analysis\n\n");
    let mut groups = BTreeMap::<Construct, Vec<&str>>::new();
    let mut counts = BTreeMap::new();
    for (id, finding) in findings {
        *counts
            .entry(format!("{:?}", finding.disposition))
            .or_insert(0) += 1;
        if finding.disposition == Disposition::Classified
            && let Some(construct) = finding.construct
        {
            groups.entry(construct).or_default().push(id);
        }
    }
    writeln!(out, "{} report cases.\n", findings.len()).unwrap();
    out.push_str("| Disposition | Cases |\n| --- | ---: |\n");
    for (name, count) in counts {
        writeln!(out, "| {name} | {count} |").unwrap();
    }
    out.push_str("\n## Blocked by construct\n\n| CST rule | Token | Repro | Cases |\n| --- | --- | --- | --- |\n");
    for (construct, cases) in groups {
        writeln!(
            out,
            "| `{}` | `{}` | [{}](cases/{}.json) | {} |",
            construct.rule,
            construct.token,
            construct.repro,
            construct.repro,
            cases.join(", ")
        )
        .unwrap();
    }
    out.push_str("\n## Rust refusals\n\n| Case | Disposition | Byte | Line | UTF-16 column | CST rule |\n| --- | --- | ---: | ---: | ---: | --- |\n");
    for (id, finding) in findings {
        if finding.disposition == Disposition::NotRustRefusal {
            continue;
        }
        let position = finding
            .position
            .map(|(byte, line, column)| format!("{byte} | {line} | {column}"))
            .unwrap_or_else(|| "— | — | —".into());
        let rule = if finding.disposition == Disposition::Classified {
            finding.construct.map_or("—", |construct| construct.rule)
        } else {
            "—"
        };
        writeln!(
            out,
            "| [{id}](cases/{id}.json) | {:?} | {position} | {rule} |",
            finding.disposition
        )
        .unwrap();
    }
    out
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let directory = arguments
        .next()
        .ok_or_else(|| invalid("usage: classify_import_refusals REPORT_DIRECTORY"))?;
    if arguments.next().is_some() {
        return Err(invalid("expected one report directory").into());
    }
    let grammar = Grammar::load(Language::SysML)
        .map_err(|error| invalid(&format!("load pinned grammar: {error:?}")))?;
    let findings = collected_reports(Path::new(&directory), &grammar)?;
    print!("{}", render(&findings));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_keep_utf8_offsets_and_utf16_columns() {
        let source = "☀🧭\r\n雪🧭alias";
        assert_eq!(location(source, 4), None);
        assert_eq!(location(source, 7), Some((7, 1, 4)));
        assert_eq!(location(source, 8), Some((8, 2, 1)));
        assert_eq!(location(source, 9), Some((9, 2, 1)));
        assert_eq!(location(source, 16), Some((16, 2, 4)));
        assert_eq!(location(source, source.len() + 1), None);
    }

    #[test]
    fn equally_small_distinct_concrete_rules_are_ambiguous() {
        let alias = Construct {
            rule: "AliasMember",
            token: "alias",
            repro: "alias",
        };
        let other = Construct {
            rule: "Other",
            ..alias
        };
        assert_eq!(smallest_rule([(6, alias), (6, other)].into_iter()), Err(()));
        assert_eq!(
            smallest_rule([(4, alias), (6, other)].into_iter()),
            Ok(Some(alias))
        );
        assert_eq!(
            smallest_rule([(4, alias), (4, alias)].into_iter()),
            Ok(Some(alias))
        );
    }

    #[test]
    fn actual_cst_and_refusal_offset_select_the_alias_construct() {
        let grammar = Grammar::load(Language::SysML).unwrap();
        let source = "package '雪🧭' { part def V; alias Car for V; }";
        let offset = source.find("alias").unwrap();
        let error = format!("Parse(Unsupported {{ offset: {offset} }})");
        let actual = identify(&grammar, source, &error, Options::default());
        assert_eq!(actual.disposition, Disposition::ReproUnverified);
        assert_eq!(actual.construct.unwrap().rule, "AliasMember");
        assert_eq!(actual.position, Some((offset, 1, offset - 3)));
    }

    #[test]
    fn usage_body_classification_requires_typing_before_the_body() {
        let grammar = Grammar::load(Language::SysML).unwrap();
        let classify = |source: &str| {
            let offset = source.rfind('{').unwrap();
            identify(
                &grammar,
                source,
                &format!("Parse(Unsupported {{ offset: {offset} }})"),
                Options::default(),
            )
        };
        let typed = classify("package P { part def T; part a : T { part w : T; } }");
        let untyped = classify("package P { part a { part w; } }");
        assert_eq!(typed.construct.unwrap().repro, "usage-body");
        assert_eq!(untyped.disposition, Disposition::NoVerifiedRule);
    }

    #[test]
    fn malformed_capped_unresolved_and_no_rule_stay_distinct() {
        let grammar = Grammar::load(Language::SysML).unwrap();
        let source = "package P {";
        let options = Options::default();
        assert_eq!(
            identify(&grammar, source, "Parse(Limit)", options).disposition,
            Disposition::ImporterCapped
        );
        assert_eq!(
            identify(
                &grammar,
                source,
                "Parse(Malformed { offset: 11, expected: \"}\" })",
                options
            )
            .disposition,
            Disposition::Malformed
        );
        assert_eq!(
            identify(
                &grammar,
                source,
                "Parse(Unsupported { offset: 10 })",
                options
            )
            .disposition,
            Disposition::SyntaxRejected
        );
        assert_eq!(
            identify(&grammar, source, "Link(Unresolved(\"Missing\"))", options).disposition,
            Disposition::UnresolvedLink
        );
        assert_eq!(
            identify(
                &grammar,
                source,
                "Parse(Unsupported { offset: 10 })",
                Options {
                    max_chart_states: 1,
                    ..options
                }
            )
            .disposition,
            Disposition::RecognizerCapped
        );
        assert_eq!(
            identify(
                &grammar,
                "package P {}",
                "Parse(Unsupported { offset: 8 })",
                options
            )
            .disposition,
            Disposition::NoVerifiedRule
        );
    }
}

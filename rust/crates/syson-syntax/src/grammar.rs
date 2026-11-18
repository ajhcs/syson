// Copyright (c) 2026 SysON Rust port contributors.
// SPDX-License-Identifier: EPL-2.0
use crate::TokenKind;
use crate::recognizer::CstKind;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    KerML,
    SysML,
}
impl Language {
    fn source(self) -> &'static str {
        match self {
            Self::KerML => include_str!("../data/kerml.json"),
            Self::SysML => include_str!("../data/sysml.json"),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::KerML => "KerML",
            Self::SysML => "SysML",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrammarError {
    pub location: String,
    pub unsupported: String,
}
fn bad(location: &str, reason: impl Into<String>) -> GrammarError {
    GrammarError {
        location: location.into(),
        unsupported: reason.into(),
    }
}
fn string<'a>(v: &'a Value, key: &str, loc: &str) -> Result<&'a str, GrammarError> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| bad(loc, format!("missing string {key}")))
}
fn keys(v: &Value, allowed: &[&str], loc: &str) -> Result<(), GrammarError> {
    let o = v.as_object().ok_or_else(|| bad(loc, "expected object"))?;
    if let Some(key) = o.keys().find(|k| !allowed.contains(&k.as_str())) {
        return Err(bad(loc, format!("unsupported attribute {key}")));
    }
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Symbol {
    Nonterminal(usize),
    Token(TokenKind),
}
#[derive(Clone, Debug)]
pub(crate) struct Production {
    pub lhs: usize,
    pub rhs: Vec<Symbol>,
}
#[derive(Clone, Debug)]
pub(crate) struct First<T = TokenKind> {
    pub(crate) tokens: HashSet<T>,
    pub(crate) nullable: bool,
    pub(crate) predicts_nullable: bool,
}
impl<T> Default for First<T> {
    fn default() -> Self {
        Self {
            tokens: HashSet::new(),
            nullable: false,
            predicts_nullable: false,
        }
    }
}

fn production_first<'a>(
    production: &'a Production,
    nonterminals: &[First<&'a TokenKind>],
) -> First<&'a TokenKind> {
    let mut first = First::default();
    if let Some(Symbol::Nonterminal(id)) = production.rhs.first() {
        first.predicts_nullable = nonterminals[*id].nullable || nonterminals[*id].predicts_nullable;
    }
    for symbol in &production.rhs {
        match symbol {
            Symbol::Token(token) => {
                first.tokens.insert(token);
                return first;
            }
            Symbol::Nonterminal(id) => {
                first
                    .tokens
                    .extend(nonterminals[*id].tokens.iter().copied());
                if !nonterminals[*id].nullable {
                    return first;
                }
            }
        }
    }
    first.nullable = true;
    first
}

fn first_sets(productions: &[Production], labels: usize) -> Vec<First> {
    let mut dependents = vec![Vec::new(); labels];
    for (index, production) in productions.iter().enumerate() {
        for symbol in &production.rhs {
            if let Symbol::Nonterminal(id) = symbol {
                dependents[*id].push(index);
            }
        }
    }
    // Start with no established facts. Every change can only add FIRST
    // tokens or prove nullability, and requeues all affected productions.
    let mut nonterminals = vec![First::default(); labels];
    let mut queue: VecDeque<_> = (0..productions.len()).collect();
    let mut pending = vec![true; productions.len()];
    while let Some(index) = queue.pop_front() {
        pending[index] = false;
        let production = &productions[index];
        let first = production_first(production, &nonterminals);
        let lhs = &mut nonterminals[production.lhs];
        let before = (lhs.tokens.len(), lhs.nullable, lhs.predicts_nullable);
        lhs.tokens.extend(first.tokens);
        lhs.nullable |= first.nullable;
        lhs.predicts_nullable |= first.predicts_nullable;
        if before != (lhs.tokens.len(), lhs.nullable, lhs.predicts_nullable) {
            for &dependent in &dependents[production.lhs] {
                if !pending[dependent] {
                    pending[dependent] = true;
                    queue.push_back(dependent);
                }
            }
        }
    }
    productions
        .iter()
        .map(|p| {
            let first = production_first(p, &nonterminals);
            First {
                tokens: first.tokens.into_iter().cloned().collect(),
                nullable: first.nullable,
                predicts_nullable: first.predicts_nullable,
            }
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct Grammar {
    pub language: Language,
    pub(crate) entry_name: String,
    pub(crate) parser_rules: BTreeMap<String, usize>,
    pub(crate) keywords: BTreeSet<String>,
    pub(crate) all_keywords: BTreeSet<String>,
    pub(crate) reachable: BTreeSet<usize>,
    pub(crate) punctuation: BTreeSet<String>,
    pub(crate) productions: Vec<Production>,
    pub(crate) first: Option<Vec<First>>,
    pub(crate) by_lhs: Vec<Vec<usize>>,
    pub(crate) labels: Vec<CstKind>,
}
impl Grammar {
    pub fn load(language: Language) -> Result<Self, GrammarError> {
        Self::from_json(language, language.source())
    }
    /// Unknown syntax constructs/terminal changes fail closed at compilation.
    /// Interface and return-type metadata remain data, not executable AST semantics.
    pub fn from_json(language: Language, source: &str) -> Result<Self, GrammarError> {
        let root: Value =
            serde_json::from_str(source).map_err(|e| bad("grammar", e.to_string()))?;
        if root.get("$type").and_then(Value::as_str) != Some("Grammar") {
            return Err(bad("grammar", "UnsupportedConstruct root type"));
        }
        keys(
            &root,
            &[
                "$type",
                "definesHiddenTokens",
                "hiddenTokens",
                "imports",
                "interfaces",
                "isDeclared",
                "name",
                "rules",
                "types",
                "usedGrammars",
            ],
            "grammar",
        )?;
        if root.get("definesHiddenTokens").and_then(Value::as_bool) != Some(false)
            || !root
                .get("hiddenTokens")
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty)
            || !root
                .get("usedGrammars")
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty)
        {
            return Err(bad(
                "grammar",
                "global hidden tokens or used grammars unsupported",
            ));
        }
        if root.get("name").and_then(Value::as_str) != Some(language.name()) {
            return Err(bad("grammar", "wrong language"));
        }
        if !root
            .get("imports")
            .and_then(Value::as_array)
            .is_some_and(Vec::is_empty)
        {
            return Err(bad("grammar", "unresolved imports"));
        }
        let rules = root
            .get("rules")
            .and_then(Value::as_array)
            .ok_or_else(|| bad("grammar", "missing rules"))?;
        let contracts: Value = serde_json::from_str(include_str!("../data/terminals.json"))
            .map_err(|e| bad("terminal contracts", e.to_string()))?;
        let mut terminal_names = BTreeSet::new();
        let mut parser_rules = BTreeMap::new();
        let mut entries = Vec::new();
        let mut labels = Vec::new();
        for (i, r) in rules.iter().enumerate() {
            let loc = format!("rules@{i}");
            let name = string(r, "name", &loc)?;
            let kind = string(r, "$type", &loc)?;
            match kind {
                "TerminalRule" => {
                    keys(
                        r,
                        &["$type", "definition", "fragment", "hidden", "name", "type"],
                        &loc,
                    )?;
                    if !terminal_names.insert(name.to_owned()) {
                        return Err(bad(&loc, "duplicate terminal"));
                    }
                    let signature = json!({"definition":normalize(&r["definition"],&root)?,"fragment":r["fragment"],"hidden":r["hidden"]});
                    if contracts.get(name) != Some(&signature) {
                        return Err(bad(&loc, format!("unsupported terminal definition {name}")));
                    }
                }
                "ParserRule" => {
                    keys(
                        r,
                        &[
                            "$type",
                            "name",
                            "entry",
                            "fragment",
                            "dataType",
                            "returnType",
                            "definition",
                            "definesHiddenTokens",
                            "hiddenTokens",
                            "parameters",
                            "wildcard",
                        ],
                        &loc,
                    )?;
                    if !r
                        .get("parameters")
                        .and_then(Value::as_array)
                        .is_some_and(Vec::is_empty)
                        || r.get("wildcard").and_then(Value::as_bool) != Some(false)
                        || r.get("definesHiddenTokens").and_then(Value::as_bool) != Some(false)
                        || !r
                            .get("hiddenTokens")
                            .and_then(Value::as_array)
                            .is_some_and(Vec::is_empty)
                    {
                        return Err(bad(
                            &loc,
                            "parameters/wildcard/local hidden tokens unsupported",
                        ));
                    }
                    if parser_rules.insert(name.into(), i).is_some() {
                        return Err(bad(&loc, "duplicate parser rule"));
                    }
                    if r.get("entry").and_then(Value::as_bool) == Some(true) {
                        entries.push(name.to_owned());
                    }
                }
                _ => return Err(bad(&loc, format!("unsupported rule kind {kind}"))),
            }
            labels.push(CstKind::Rule(name.into()));
        }
        let expected: BTreeSet<String> = contracts.as_object().unwrap().keys().cloned().collect();
        if terminal_names != expected {
            return Err(bad(
                "terminals",
                "expected exact twelve terminal definitions",
            ));
        }
        if entries.len() != 1 {
            return Err(bad("grammar", "expected exactly one entry"));
        }
        let mut b = Builder {
            root: &root,
            rules,
            productions: Vec::new(),
            by_lhs: vec![Vec::new(); rules.len()],
            labels,
            keywords: BTreeSet::new(),
        };
        for (i, r) in rules.iter().enumerate() {
            if r["$type"] == "ParserRule" {
                let node = b.lower(&r["definition"], &format!("rules@{i}/definition"))?;
                b.production(i, vec![Symbol::Nonterminal(node)]);
            }
        }
        let entry = parser_rules[&entries[0]];
        let mut reachable = BTreeSet::new();
        let mut pending = vec![entry];
        let mut keywords = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !reachable.insert(id) {
                continue;
            }
            for p in &b.by_lhs[id] {
                for symbol in &b.productions[*p].rhs {
                    match symbol {
                        Symbol::Nonterminal(n) => pending.push(*n),
                        Symbol::Token(TokenKind::Keyword(k)) => {
                            keywords.insert(k.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
        let punctuation = keywords
            .iter()
            .filter(|s| {
                !s.as_bytes()
                    .first()
                    .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
            })
            .cloned()
            .collect();
        Ok(Self {
            language,
            entry_name: entries.remove(0),
            parser_rules,
            keywords,
            all_keywords: b.keywords,
            reachable,
            punctuation,
            first: (source == language.source())
                .then(|| first_sets(&b.productions, b.labels.len())),
            productions: b.productions,
            by_lhs: b.by_lhs,
            labels: b.labels,
        })
    }
}
fn target<'a>(root: &'a Value, reference: &str) -> Result<&'a Value, GrammarError> {
    let Some(path) = reference.strip_prefix("#/") else {
        return Err(bad(reference, "nonlocal reference"));
    };
    let mut v = root;
    for part in path.split('/') {
        let (name, index) = part
            .split_once('@')
            .map_or((part, None), |(n, i)| (n, Some(i)));
        v = v
            .get(name)
            .ok_or_else(|| bad(reference, "missing reference target"))?;
        if let Some(index) = index {
            v = v
                .get(
                    index
                        .parse::<usize>()
                        .map_err(|_| bad(reference, "bad reference index"))?,
                )
                .ok_or_else(|| bad(reference, "reference index out of bounds"))?;
        }
    }
    Ok(v)
}
fn normalize(v: &Value, root: &Value) -> Result<Value, GrammarError> {
    match v {
        Value::Object(o) => {
            if let Some(reference) = o.get("$ref").and_then(Value::as_str) {
                return Ok(json!({"target":string(target(root,reference)?,"name",reference)?}));
            }
            let mut out = serde_json::Map::new();
            for (k, v) in o {
                out.insert(k.clone(), normalize(v, root)?);
            }
            Ok(Value::Object(out))
        }
        Value::Array(a) => a
            .iter()
            .map(|v| normalize(v, root))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        _ => Ok(v.clone()),
    }
}
struct Builder<'a> {
    root: &'a Value,
    rules: &'a [Value],
    productions: Vec<Production>,
    by_lhs: Vec<Vec<usize>>,
    labels: Vec<CstKind>,
    keywords: BTreeSet<String>,
}
impl Builder<'_> {
    fn allocate(&mut self, label: CstKind) -> usize {
        let id = self.labels.len();
        self.labels.push(label);
        self.by_lhs.push(Vec::new());
        id
    }
    fn production(&mut self, lhs: usize, rhs: Vec<Symbol>) {
        let id = self.productions.len();
        self.productions.push(Production { lhs, rhs });
        self.by_lhs[lhs].push(id);
    }
    fn lower(&mut self, node: &Value, loc: &str) -> Result<usize, GrammarError> {
        let kind = string(node, "$type", loc)?;
        let (label, rhs): (_, Vec<Vec<Symbol>>) = match kind {
            "Keyword" => {
                keys(node, &["$type", "value", "cardinality"], loc)?;
                let value = string(node, "value", loc)?;
                if value.is_empty() {
                    return Err(bad(loc, "empty keyword"));
                }
                self.keywords.insert(value.into());
                (
                    CstKind::Keyword(value.into()),
                    vec![vec![Symbol::Token(TokenKind::Keyword(value.into()))]],
                )
            }
            "RuleCall" => {
                keys(node, &["$type", "rule", "arguments", "cardinality"], loc)?;
                if !node
                    .get("arguments")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
                {
                    return Err(bad(loc, "rule arguments unsupported"));
                }
                let reference = string(&node["rule"], "$ref", loc)?;
                let r = target(self.root, reference)?;
                let name = string(r, "name", loc)?;
                let symbol = if r["$type"] == "TerminalRule" {
                    if r["fragment"] == true {
                        return Err(bad(loc, "terminal fragment called as a parser token"));
                    }
                    Symbol::Token(TokenKind::Terminal(name.into()))
                } else if r["$type"] == "ParserRule" {
                    let index = reference
                        .strip_prefix("#/rules@")
                        .and_then(|i| i.parse::<usize>().ok())
                        .ok_or_else(|| bad(loc, "parser reference shape"))?;
                    if self.rules.get(index).is_none() {
                        return Err(bad(loc, "rule index"));
                    }
                    Symbol::Nonterminal(index)
                } else {
                    return Err(bad(loc, "rule reference does not target a rule"));
                };
                (CstKind::RuleCall(name.into()), vec![vec![symbol]])
            }
            "Group" | "Alternatives" => {
                keys(node, &["$type", "elements", "cardinality"], loc)?;
                let elements = node
                    .get("elements")
                    .and_then(Value::as_array)
                    .ok_or_else(|| bad(loc, "missing elements"))?;
                let ids = elements
                    .iter()
                    .enumerate()
                    .map(|(i, n)| self.lower(n, &format!("{loc}/elements@{i}")))
                    .collect::<Result<Vec<_>, _>>()?;
                if kind == "Group" {
                    (
                        CstKind::Group,
                        vec![ids.into_iter().map(Symbol::Nonterminal).collect()],
                    )
                } else {
                    (
                        CstKind::Alternatives,
                        ids.into_iter()
                            .map(|i| vec![Symbol::Nonterminal(i)])
                            .collect(),
                    )
                }
            }
            "Assignment" => {
                keys(
                    node,
                    &["$type", "feature", "operator", "terminal", "cardinality"],
                    loc,
                )?;
                let feature = string(node, "feature", loc)?.to_owned();
                let operator = string(node, "operator", loc)?.to_owned();
                if !["=", "+=", "?="].contains(&operator.as_str()) {
                    return Err(bad(loc, "unknown assignment operator"));
                }
                let child = self.lower(&node["terminal"], &format!("{loc}/terminal"))?;
                (
                    CstKind::Assignment { feature, operator },
                    vec![vec![Symbol::Nonterminal(child)]],
                )
            }
            "Action" => {
                keys(
                    node,
                    &["$type", "type", "feature", "operator", "cardinality"],
                    loc,
                )?;
                let reference = string(&node["type"], "$ref", loc)?;
                let type_name = string(target(self.root, reference)?, "name", loc)?.to_owned();
                let optional = |key: &str| -> Result<Option<String>, GrammarError> {
                    match node.get(key) {
                        None => Ok(None),
                        Some(Value::String(value)) => Ok(Some(value.clone())),
                        Some(_) => Err(bad(loc, format!("nonstring action {key}"))),
                    }
                };
                let feature = optional("feature")?;
                let operator = optional("operator")?;
                if feature.is_some() != operator.is_some() {
                    return Err(bad(loc, "incomplete action feature/operator pair"));
                }
                if operator
                    .as_deref()
                    .is_some_and(|o| !matches!(o, "=" | "+="))
                {
                    return Err(bad(loc, "unknown action operator"));
                }
                (
                    CstKind::Action {
                        type_name,
                        feature,
                        operator,
                    },
                    vec![vec![]],
                )
            }
            "CrossReference" => {
                keys(
                    node,
                    &[
                        "$type",
                        "type",
                        "terminal",
                        "deprecatedSyntax",
                        "cardinality",
                    ],
                    loc,
                )?;
                if node.get("deprecatedSyntax").and_then(Value::as_bool) != Some(false) {
                    return Err(bad(loc, "deprecated crossreference syntax unsupported"));
                }
                let reference = string(&node["type"], "$ref", loc)?;
                let type_name = string(target(self.root, reference)?, "name", loc)?.to_owned();
                let child = self.lower(&node["terminal"], &format!("{loc}/terminal"))?;
                (
                    CstKind::CrossReference { type_name },
                    vec![vec![Symbol::Nonterminal(child)]],
                )
            }
            _ => return Err(bad(loc, format!("UnsupportedConstruct {kind}"))),
        };
        let base = self.allocate(label);
        for production in rhs {
            self.production(base, production);
        }
        let Some(cardinality) = node.get("cardinality") else {
            return Ok(base);
        };
        let cardinality = cardinality
            .as_str()
            .ok_or_else(|| bad(loc, "nonstring cardinality"))?;
        let wrapper = self.allocate(CstKind::Repetition(cardinality.into()));
        match cardinality {
            "?" => {
                self.production(wrapper, vec![]);
                self.production(wrapper, vec![Symbol::Nonterminal(base)]);
            }
            "*" => {
                self.production(wrapper, vec![]);
                self.production(
                    wrapper,
                    vec![Symbol::Nonterminal(base), Symbol::Nonterminal(wrapper)],
                );
            }
            "+" => {
                let rest = self.allocate(CstKind::Repetition("*".into()));
                self.production(rest, vec![]);
                self.production(
                    rest,
                    vec![Symbol::Nonterminal(base), Symbol::Nonterminal(rest)],
                );
                self.production(
                    wrapper,
                    vec![Symbol::Nonterminal(base), Symbol::Nonterminal(rest)],
                );
            }
            _ => {
                return Err(bad(
                    loc,
                    format!("UnsupportedConstruct cardinality {cardinality}"),
                ));
            }
        }
        Ok(wrapper)
    }
}

#[cfg(test)]
mod prediction_tests {
    use super::*;
    use crate::Options;

    #[test]
    fn dead_predictions_preserve_nullable_witness_order_and_expected_tokens() {
        let mut source: Value = serde_json::from_str(Language::KerML.source()).unwrap();
        let rules = source["rules"].as_array_mut().unwrap();
        for rule in rules.iter_mut().filter(|rule| rule["entry"] == true) {
            rule["entry"] = false.into();
        }
        let offset = rules.len();
        let reference = |n: usize| json!({"$type":"RuleCall","arguments":[],"rule":{"$ref":format!("#/rules@{}", offset + n)}});
        let keyword = |token: &str| json!({"$type":"Keyword","value":token});
        let witness = [
            vec![vec![reference(3), reference(2)]],
            vec![vec![keyword("b")], vec![]],
            vec![vec![keyword("b"), reference(4)], vec![]],
            vec![vec![reference(1)]],
            vec![vec![], vec![reference(2), keyword("a")]],
        ];
        for (index, branches) in witness.into_iter().enumerate() {
            let elements: Vec<_> = branches
                .into_iter()
                .map(|symbols| json!({"$type":"Group","elements":symbols}))
                .collect();
            rules.push(json!({
                "$type":"ParserRule","name":format!("R{index}"),"entry":index == 0,
                "wildcard":false,"definesHiddenTokens":false,"hiddenTokens":[],"parameters":[],
                "definition":{"$type":"Alternatives","elements":elements}
            }));
        }
        let mut custom = Grammar::from_json(Language::KerML, &source.to_string()).unwrap();
        assert!(custom.first.is_none());
        let ordered = custom.recognize("b", Options::default()).unwrap();
        let rejected = custom.recognize("a", Options::default()).unwrap_err();
        custom.first = Some(first_sets(&custom.productions, custom.labels.len()));
        assert_eq!(custom.recognize("b", Options::default()), Ok(ordered));
        assert_eq!(custom.recognize("a", Options::default()), Err(rejected));
    }
}

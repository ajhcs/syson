// Copyright (c) 2026 SysON Rust port contributors.
// SPDX-License-Identifier: EPL-2.0
//! Earley CFG membership and one CST derivation; upstream ALL(*) choice/recovery
//! is not emulated. Ambiguous state merges are reported, never AST semantics.
use crate::grammar::Symbol;
use crate::{Error, Grammar, Point, Span, Token, TokenKind, Work};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CstKind {
    Rule(String),
    RuleCall(String),
    Keyword(String),
    Token(TokenKind),
    Group,
    Alternatives,
    Repetition(String),
    Assignment {
        feature: String,
        operator: String,
    },
    Action {
        type_name: String,
        feature: Option<String>,
        operator: Option<String>,
    },
    CrossReference {
        type_name: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CstNode {
    pub kind: CstKind,
    pub span: Span,
    pub children: Vec<usize>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticStatus {
    NotPerformed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recognition {
    /// All lexical tokens including hidden notes/whitespace, with exact spans.
    pub tokens: Vec<Token>,
    /// Only nodes reachable from the chosen syntax derivation; no semantic AST.
    pub cst: Vec<CstNode>,
    pub root: usize,
    pub ambiguous_state_merges: usize,
    pub semantics: SemanticStatus,
}
#[derive(Clone, Debug)]
struct State {
    production: usize,
    dot: usize,
    start: usize,
    children: Vec<usize>,
}
#[derive(Default)]
struct Chart {
    states: Vec<State>,
    seen: BTreeMap<(usize, usize, usize), usize>,
    waiting: BTreeMap<usize, Vec<usize>>,
    completed: BTreeMap<(usize, usize), Vec<usize>>,
}
fn add(
    g: &Grammar,
    chart: &mut Chart,
    state: State,
    total: &mut usize,
    merges: &mut usize,
    work: &mut Work<'_>,
) -> Result<(), Error> {
    work.tick()?;
    let key = (state.production, state.dot, state.start);
    if let Some(index) = chart.seen.get(&key) {
        if chart.states[*index].children != state.children {
            *merges = merges.saturating_add(1);
        }
        return Ok(());
    }
    work.limit("chart states", *total + 1, work.options.max_chart_states)?;
    *total += 1;
    let index = chart.states.len();
    chart.seen.insert(key, index);
    let production = &g.productions[state.production];
    match production.rhs.get(state.dot) {
        Some(Symbol::Nonterminal(nt)) => chart.waiting.entry(*nt).or_default().push(index),
        None => chart
            .completed
            .entry((production.lhs, state.start))
            .or_default()
            .push(index),
        _ => {}
    }
    chart.states.push(state);
    Ok(())
}
fn node(nodes: &mut Vec<CstNode>, mut value: CstNode, work: &mut Work<'_>) -> Result<usize, Error> {
    work.tick()?;
    work.limit("CST nodes", nodes.len() + 1, work.options.max_cst_nodes)?;
    for child in &value.children {
        let span = nodes[*child].span;
        if span.start.byte < value.span.start.byte {
            value.span.start = span.start;
        }
        if span.end.byte > value.span.end.byte {
            value.span.end = span.end;
        }
    }
    let id = nodes.len();
    nodes.push(value);
    Ok(id)
}
fn span_at(visible: &[&Token], start: usize, end: usize, eof: Point) -> Span {
    let a = visible.get(start).map_or(eof, |t| t.span.start);
    let b = if end > start {
        visible.get(end - 1).map_or(eof, |t| t.span.end)
    } else {
        a
    };
    Span { start: a, end: b }
}
pub(crate) fn recognize(
    g: &Grammar,
    rule: &str,
    input: &str,
    tokens: Vec<Token>,
    eof: Point,
    work: &mut Work<'_>,
) -> Result<Recognition, Error> {
    let Some(&entry) = g.parser_rules.get(rule) else {
        return Err(Error::UnknownRule(rule.into()));
    };
    if !g.reachable.contains(&entry) {
        return Err(Error::UnsupportedEntry(rule.into()));
    }
    let visible: Vec<&Token> = tokens.iter().filter(|t| !t.hidden).collect();
    let mut charts: Vec<Chart> = (0..=visible.len()).map(|_| Chart::default()).collect();
    let mut nodes = Vec::new();
    let mut total = 0;
    let mut merges = 0;
    let mut root = None;
    for &production in &g.by_lhs[entry] {
        add(
            g,
            &mut charts[0],
            State {
                production,
                dot: 0,
                start: 0,
                children: vec![],
            },
            &mut total,
            &mut merges,
            work,
        )?;
    }
    for position in 0..charts.len() {
        let mut cursor = 0;
        while cursor < charts[position].states.len() {
            work.tick()?;
            let state = charts[position].states[cursor].clone();
            cursor += 1;
            let production = &g.productions[state.production];
            match production.rhs.get(state.dot) {
                Some(Symbol::Token(expected)) => {
                    if visible.get(position).is_some_and(|t| t.kind == *expected) {
                        let leaf = node(
                            &mut nodes,
                            CstNode {
                                kind: CstKind::Token(expected.clone()),
                                span: visible[position].span,
                                children: vec![],
                            },
                            work,
                        )?;
                        let mut next = state.clone();
                        next.dot += 1;
                        next.children.push(leaf);
                        add(
                            g,
                            &mut charts[position + 1],
                            next,
                            &mut total,
                            &mut merges,
                            work,
                        )?;
                    }
                }
                Some(Symbol::Nonterminal(next)) => {
                    for &p in &g.by_lhs[*next] {
                        add(
                            g,
                            &mut charts[position],
                            State {
                                production: p,
                                dot: 0,
                                start: position,
                                children: vec![],
                            },
                            &mut total,
                            &mut merges,
                            work,
                        )?;
                    }
                    // Nullable completions can precede this waiting state.
                    let completed = charts[position]
                        .completed
                        .get(&(*next, position))
                        .cloned()
                        .unwrap_or_default();
                    for index in completed {
                        work.tick()?;
                        let c = charts[position].states[index].clone();
                        let child = node(
                            &mut nodes,
                            CstNode {
                                kind: g.labels[*next].clone(),
                                span: span_at(&visible, position, position, eof),
                                children: c.children,
                            },
                            work,
                        )?;
                        let mut advanced = state.clone();
                        advanced.dot += 1;
                        advanced.children.push(child);
                        add(
                            g,
                            &mut charts[position],
                            advanced,
                            &mut total,
                            &mut merges,
                            work,
                        )?;
                    }
                }
                None => {
                    let complete = node(
                        &mut nodes,
                        CstNode {
                            kind: g.labels[production.lhs].clone(),
                            span: span_at(&visible, state.start, position, eof),
                            children: state.children.clone(),
                        },
                        work,
                    )?;
                    if production.lhs == entry && state.start == 0 && position == visible.len() {
                        root.get_or_insert(complete);
                    }
                    let waiting = charts[state.start]
                        .waiting
                        .get(&production.lhs)
                        .cloned()
                        .unwrap_or_default();
                    for index in waiting {
                        work.tick()?;
                        let mut previous = charts[state.start].states[index].clone();
                        previous.dot += 1;
                        previous.children.push(complete);
                        add(
                            g,
                            &mut charts[position],
                            previous,
                            &mut total,
                            &mut merges,
                            work,
                        )?;
                    }
                }
            }
        }
        if position < visible.len() && charts[position + 1].states.is_empty() {
            break;
        }
    }
    if let Some(root) = root {
        let mut reachable = BTreeSet::new();
        let mut todo = vec![root];
        while let Some(id) = todo.pop() {
            work.tick()?;
            if reachable.insert(id) {
                todo.extend(nodes[id].children.iter().copied());
            }
        }
        let mapping: BTreeMap<_, _> = reachable
            .iter()
            .enumerate()
            .map(|(new, old)| (*old, new))
            .collect();
        let root = mapping[&root];
        let cst = reachable
            .into_iter()
            .map(|id| {
                let mut n = nodes[id].clone();
                n.children = n.children.iter().map(|id| mapping[id]).collect();
                n
            })
            .collect();
        return Ok(Recognition {
            tokens,
            cst,
            root,
            ambiguous_state_merges: merges,
            semantics: SemanticStatus::NotPerformed,
        });
    }
    let farthest = charts
        .iter()
        .rposition(|c| !c.states.is_empty())
        .unwrap_or(0);
    let mut expected = BTreeSet::new();
    for state in &charts[farthest].states {
        let p = &g.productions[state.production];
        if let Some(Symbol::Token(t)) = p.rhs.get(state.dot) {
            expected.insert(match t {
                TokenKind::Keyword(s) => format!("keyword {s:?}"),
                TokenKind::Terminal(s) => s.clone(),
            });
        }
        if p.lhs == entry && state.start == 0 && state.dot == p.rhs.len() {
            expected.insert("end of input".into());
        }
    }
    let span = visible.get(farthest).map_or(
        Span {
            start: eof,
            end: eof,
        },
        |t| t.span,
    );
    let found = visible
        .get(farthest)
        .and_then(|t| t.text(input))
        .map(str::to_owned);
    Err(Error::Unexpected {
        span,
        found,
        expected: expected.into_iter().collect(),
    })
}

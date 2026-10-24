// Copyright (c) 2026 SysON Rust port contributors.
// SPDX-License-Identifier: EPL-2.0
//! Bounded syntax recognition over complete candidate Syside grammar DATA.
//! No AST semantics, name linking, validation, recovery or SysON import is supplied.
#![forbid(unsafe_code)]
mod grammar;
mod lexer;
mod recognizer;
pub use grammar::{Grammar, GrammarError, Language};
pub use lexer::{Point, Span, Token, TokenKind};
pub use recognizer::{CstKind, CstNode, Recognition, SemanticStatus};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug)]
pub struct Options<'a> {
    pub cancellation: Option<&'a AtomicBool>,
    pub max_input_bytes: usize,
    pub max_tokens: usize,
    pub max_steps: usize,
    pub max_chart_states: usize,
    pub max_cst_nodes: usize,
}
impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            cancellation: None,
            max_input_bytes: 1_048_576,
            max_tokens: 100_000,
            max_steps: 5_000_000,
            max_chart_states: 250_000,
            max_cst_nodes: 250_000,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Cancelled,
    BudgetExceeded {
        resource: &'static str,
        limit: usize,
    },
    Lexical {
        span: Span,
        message: String,
        expected: Vec<String>,
    },
    Unexpected {
        span: Span,
        found: Option<String>,
        expected: Vec<String>,
    },
    UnknownRule(String),
    UnsupportedEntry(String),
}
struct Work<'a> {
    options: Options<'a>,
    steps: usize,
}
impl Work<'_> {
    fn tick(&mut self) -> Result<(), Error> {
        if self
            .options
            .cancellation
            .is_some_and(|c| c.load(Ordering::Relaxed))
        {
            return Err(Error::Cancelled);
        }
        if self.steps >= self.options.max_steps {
            return Err(Error::BudgetExceeded {
                resource: "steps",
                limit: self.options.max_steps,
            });
        }
        self.steps += 1;
        Ok(())
    }
    fn limit(&self, resource: &'static str, count: usize, limit: usize) -> Result<(), Error> {
        if count > limit {
            Err(Error::BudgetExceeded { resource, limit })
        } else {
            Ok(())
        }
    }
}
impl Grammar {
    pub fn lex(&self, input: &str, options: Options<'_>) -> Result<Vec<Token>, Error> {
        let mut work = Work { options, steps: 0 };
        work.tick()?;
        work.limit("input bytes", input.len(), options.max_input_bytes)?;
        lexer::lex(self, input, &mut work).map(|(tokens, _)| tokens)
    }
    /// Generalized CFG syntax only. Actions/assignments/crossrefs are explicit
    /// CST markers, never executed as AST construction or semantic linking.
    pub fn recognize(&self, input: &str, options: Options<'_>) -> Result<Recognition, Error> {
        self.recognize_rule(&self.entry_name, input, options)
    }
    pub fn recognize_rule(
        &self,
        rule: &str,
        input: &str,
        options: Options<'_>,
    ) -> Result<Recognition, Error> {
        let mut work = Work { options, steps: 0 };
        work.tick()?;
        work.limit("input bytes", input.len(), options.max_input_bytes)?;
        let (tokens, eof) = lexer::lex(self, input, &mut work)?;
        recognizer::recognize(self, rule, input, tokens, eof, &mut work)
    }
    pub fn rule_names(&self) -> impl Iterator<Item = &str> {
        self.parser_rules.keys().map(String::as_str)
    }
    pub fn all_declared_keywords(&self) -> impl Iterator<Item = &str> {
        self.all_keywords.iter().map(String::as_str)
    }
    pub fn keywords(&self) -> impl Iterator<Item = &str> {
        self.keywords.iter().map(String::as_str)
    }
}

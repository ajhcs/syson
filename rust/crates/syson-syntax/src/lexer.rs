// Copyright (c) 2022-2025 Sensmetry UAB and others; 2026 SysON Rust port contributors.
// SPDX-License-Identifier: EPL-2.0
// Terminal translation source: candidate Syside KerML.expressions.langium.
// Original source/license declarations are retained in data/LICENSE-SYSIDE.
use crate::{Error, Grammar, Work};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub byte: usize,
    pub utf16: usize,
    pub line: usize,
    pub column_utf16: usize,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: Point,
    pub end: Point,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Keyword(String),
    Terminal(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub hidden: bool,
    pub span: Span,
}
impl Token {
    pub fn text<'a>(&self, input: &'a str) -> Option<&'a str> {
        input.get(self.span.start.byte..self.span.end.byte)
    }
}
fn whitespace(c: char) -> bool {
    matches!(c,'\u{9}'..='\u{d}'|'\u{20}'|'\u{a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}')
}
fn id_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}
fn id_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
fn advance(
    mut p: Point,
    text: &str,
    previous_cr: &mut bool,
    work: &mut Work<'_>,
) -> Result<Point, Error> {
    for c in text.chars() {
        work.tick()?;
        p.byte += c.len_utf8();
        p.utf16 += c.len_utf16();
        match c {
            '\r' => {
                p.line += 1;
                p.column_utf16 = 0;
                *previous_cr = true;
            }
            '\n' => {
                if !*previous_cr {
                    p.line += 1;
                }
                p.column_utf16 = 0;
                *previous_cr = false;
            }
            _ => {
                p.column_utf16 += c.len_utf16();
                *previous_cr = false;
            }
        }
    }
    Ok(p)
}
fn numeric_len(bytes: &[u8], work: &mut Work<'_>) -> Result<usize, Error> {
    let mut i = 0;
    while bytes.get(i).is_some_and(u8::is_ascii_digit) {
        work.tick()?;
        i += 1;
    }
    if bytes.get(i) == Some(&b'.') && bytes.get(i + 1).is_some_and(u8::is_ascii_digit) {
        i += 1;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            work.tick()?;
            i += 1;
        }
    }
    if i == 0 {
        return Ok(0);
    }
    if matches!(bytes.get(i), Some(b'e' | b'E')) {
        let mut e = i + 1;
        while matches!(bytes.get(e), Some(b'+' | b'-')) {
            work.tick()?;
            e += 1;
        }
        let start = e;
        while bytes.get(e).is_some_and(u8::is_ascii_digit) {
            work.tick()?;
            e += 1;
        }
        if e > start {
            i = e;
        }
    }
    Ok(i)
}
pub(crate) fn lex(
    g: &Grammar,
    input: &str,
    work: &mut Work<'_>,
) -> Result<(Vec<Token>, Point), Error> {
    let mut point = Point::default();
    let mut previous_cr = false;
    let mut tokens = Vec::new();
    while point.byte < input.len() {
        work.tick()?;
        let tail = &input[point.byte..];
        let bytes = tail.as_bytes();
        let (kind, hidden, len) = if whitespace(tail.chars().next().unwrap()) {
            let mut n = 0;
            for c in tail.chars() {
                work.tick()?;
                if !whitespace(c) {
                    break;
                }
                n += c.len_utf8();
            }
            ("WS".to_owned(), true, n)
        } else if tail.starts_with("//*") && find_comment_end(tail, 3, work)?.is_some() {
            let end = find_comment_end(tail, 3, work)?.unwrap();
            ("ML_NOTE".to_owned(), true, end)
        } else if let Some(line) = tail.strip_prefix("//") {
            let mut n = 2;
            for c in line.chars() {
                work.tick()?;
                if matches!(c, '\r' | '\n') {
                    break;
                }
                n += c.len_utf8();
            }
            ("SL_NOTE".to_owned(), true, n)
        } else if tail.starts_with("/*") && find_comment_end(tail, 2, work)?.is_some() {
            let end = find_comment_end(tail, 2, work)?.unwrap();
            ("REGULAR_COMMENT".to_owned(), false, end)
        } else if bytes[0] == b'\'' {
            let n = quoted_name_len(tail, point, work)?;
            ("UNRESTRICTED_NAME".to_owned(), false, n)
        } else if bytes[0] == b'"' {
            let mut end = None;
            for (i, c) in tail[1..].char_indices() {
                work.tick()?;
                if c == '"' {
                    end = Some(i + 2);
                    break;
                }
            }
            let n = end.ok_or_else(|| Error::Lexical {
                span: Span {
                    start: point,
                    end: point,
                },
                message: "unterminated STRING_VALUE".into(),
                expected: vec!["double quote".into()],
            })?;
            ("STRING_VALUE".to_owned(), false, n)
        } else if id_start(bytes[0]) {
            let mut n = 1;
            while bytes.get(n).is_some_and(|b| id_continue(*b)) {
                work.tick()?;
                n += 1;
            }
            ("ID".to_owned(), false, n)
        } else {
            let mut n = numeric_len(bytes, work)?;
            let mut word = "NUMBER".to_owned();
            for keyword in &g.punctuation {
                work.tick()?;
                if keyword.len() > n && tail.starts_with(keyword) {
                    n = keyword.len();
                    word.clear();
                    word.push_str(keyword);
                }
            }
            if n == 0 {
                let c = tail.chars().next().unwrap();
                let end = advance(point, &tail[..c.len_utf8()], &mut previous_cr, work)?;
                return Err(Error::Lexical {
                    span: Span { start: point, end },
                    message: format!("unrecognized character {c:?}"),
                    expected: vec!["keyword or one of the eight non-fragment terminals".into()],
                });
            }
            (word, false, n)
        };
        let text = &tail[..len];
        let is_keyword = g.keywords.contains(text)
            && (!hidden)
            && (kind == "ID" || g.punctuation.contains(&kind));
        let token_kind = if is_keyword {
            TokenKind::Keyword(text.into())
        } else {
            TokenKind::Terminal(kind)
        };
        let end = advance(point, text, &mut previous_cr, work)?;
        work.limit("tokens", tokens.len() + 1, work.options.max_tokens)?;
        tokens.push(Token {
            kind: token_kind,
            hidden,
            span: Span { start: point, end },
        });
        point = end;
    }
    Ok((tokens, point))
}
fn find_comment_end(tail: &str, start: usize, work: &mut Work<'_>) -> Result<Option<usize>, Error> {
    let bytes = tail.as_bytes();
    let mut i = start;
    while i + 1 < bytes.len() {
        work.tick()?;
        if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            return Ok(Some(i + 2));
        }
        i += 1;
    }
    Ok(None)
}
fn quoted_name_len(tail: &str, start: Point, work: &mut Work<'_>) -> Result<usize, Error> {
    let mut chars = tail.char_indices();
    chars.next();
    while let Some((i, c)) = chars.next() {
        work.tick()?;
        if c == '\'' {
            return Ok(i + 1);
        }
        if c == '\\' {
            let Some((_, escape)) = chars.next() else {
                break;
            };
            if !matches!(escape, '\'' | '"' | 'b' | 'f' | 't' | 'n' | 'r' | '\\') {
                return Err(Error::Lexical {
                    span: Span { start, end: start },
                    message: format!("invalid unrestricted-name escape {escape:?}"),
                    expected: vec!["' \" b f t n r or backslash".into()],
                });
            }
        }
    }
    Err(Error::Lexical {
        span: Span { start, end: start },
        message: "unterminated UNRESTRICTED_NAME".into(),
        expected: vec!["single quote".into()],
    })
}

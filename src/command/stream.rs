use std::sync::Arc;

use crate::platform::text::lexer::{Span, Token, lex};

#[derive(Clone, Debug)]
pub struct Stream {
    pub input: Arc<str>,
    tokens: Vec<Token>,
    cursor: usize,
}

impl Stream {
    pub fn new(input: impl Into<Arc<str>>, from: usize) -> Self {
        let input: Arc<str> = input.into();
        let tokens = lex(input.get(from..).unwrap_or_default())
            .into_iter()
            .map(|mut token| {
                token.span.start += from;
                token
            })
            .collect();

        Self {
            input,
            tokens,
            cursor: 0,
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    pub fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.cursor)?.clone();

        self.cursor += 1;

        Some(token)
    }

    pub fn remaining(&self) -> &[Token] {
        &self.tokens[self.cursor.min(self.tokens.len())..]
    }

    pub fn cursor_span(&self) -> Span {
        match self.peek() {
            Some(token) => token.span,
            None => Span {
                start: self.input.len(),
                len: 0,
                index: self.tokens.len(),
                quoted: false,
            },
        }
    }

    pub fn take_rest(&mut self) -> Option<(String, Span)> {
        let first = self.peek()?.span;
        let last = self.tokens.last()?.span;
        let mut text = String::new();
        let mut wrote = first.start;

        for token in self.remaining() {
            let between = self.input.get(wrote..token.span.start).unwrap_or_default();

            if !text.is_empty() {
                match between.chars().all(char::is_whitespace) {
                    true => text.push_str(between),
                    false => text.push(' '),
                }
            }

            text.push_str(&token.raw);
            wrote = token.span.end();
        }

        self.cursor = self.tokens.len();

        Some((
            text,
            Span {
                start: first.start,
                len: last.end() - first.start,
                index: first.index,
                quoted: false,
            },
        ))
    }

    pub fn without(&self, omitted: &[usize]) -> Self {
        let taken = self.cursor.min(self.tokens.len());
        let kept = self
            .remaining()
            .iter()
            .enumerate()
            .filter(|(index, _)| !omitted.contains(index))
            .map(|(_, token)| token.clone());

        Self {
            input: Arc::clone(&self.input),
            tokens: self.tokens[..taken].iter().cloned().chain(kept).collect(),
            cursor: taken,
        }
    }
}

use std::collections::HashMap;

use crate::command::args::{ArgKind, Field};
use crate::command::stream::Stream;
use crate::platform::text::lexer::{Span, Token};

#[derive(Clone, Debug)]
struct Flag {
    at: Span,
    value: Option<Token>,
}

#[derive(Clone, Debug, Default)]
pub struct Flags {
    values: HashMap<&'static str, Flag>,
}

impl Flags {
    pub fn is_set(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn value(&self, name: &str) -> Option<&Token> {
        self.values.get(name).and_then(|flag| flag.value.as_ref())
    }

    pub fn after(&self, name: &str) -> Span {
        let Some(flag) = self.values.get(name) else {
            return Span::default();
        };

        Span {
            start: flag.at.end(),
            len: 0,
            index: flag.at.index + 1,
            quoted: false,
        }
    }
}

fn matched<'a>(raw: &str, fields: &'a [Field]) -> Option<&'a Field> {
    let name = raw.strip_prefix(['-', '+'])?.trim_start_matches(['-', '+']);

    if name.is_empty() {
        return None;
    }

    fields.iter().find(|field| {
        field.is_flag()
            && (field.name == name || field.short.is_some_and(|c| name == c.to_string()))
    })
}

pub fn split(stream: &Stream, fields: &[Field]) -> (Flags, Stream) {
    if !fields.iter().any(Field::is_flag) {
        return (Flags::default(), stream.clone());
    }

    let tokens = stream.remaining().to_vec();
    let mut values = HashMap::new();
    let mut consumed = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        let Some(field) = matched(&tokens[index].raw, fields) else {
            index += 1;

            continue;
        };

        consumed.push(index);

        let at = tokens[index].span;

        if field.kind == ArgKind::Boolean {
            values.insert(field.name, Flag { at, value: None });
            index += 1;

            continue;
        }

        let value = tokens.get(index + 1).cloned();

        if value.is_some() {
            consumed.push(index + 1);
        }

        values.insert(field.name, Flag { at, value });
        index += 2;
    }

    (Flags { values }, stream.without(&consumed))
}

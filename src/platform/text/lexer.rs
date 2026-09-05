use std::mem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub len: usize,
    pub index: usize,
    pub quoted: bool,
}

impl Span {
    pub fn end(&self) -> usize {
        self.start + self.len
    }
}

#[derive(Debug, Clone, Default)]
pub struct Token {
    pub raw: String,
    pub span: Span,
}

fn emit(tokens: &mut Vec<Token>, current: &mut String, start: usize, quoted: &mut bool) {
    if current.is_empty() {
        return;
    }

    let index = tokens.len();

    tokens.push(token(current, start, index, *quoted));
    *quoted = false;
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut chars = input.char_indices();
    let mut tokens: Vec<Token> = Vec::new();
    let mut current = String::new();
    let mut start = 0;
    let mut open = None;
    let mut quoted = false;

    while let Some((index, ch)) = chars.next() {
        if ch == '\\' {
            current.extend(chars.next().map(|(_, escaped)| escaped));

            continue;
        }

        if open == Some(ch) {
            open = None;

            continue;
        }

        if open.is_some() {
            current.push(ch);

            continue;
        }

        if ch.is_whitespace() {
            emit(&mut tokens, &mut current, start, &mut quoted);

            continue;
        }

        if (ch == '"' || ch == '\'') && current.is_empty() {
            open = Some(ch);
            quoted = true;
            start = index + ch.len_utf8();

            continue;
        }

        if ch == '<'
            && let Some(close) = input[index..]
                .split('\n')
                .next()
                .and_then(|line| line.find('>'))
        {
            let last = index + close;

            if current.is_empty() {
                start = index;
            }

            current.push(ch);

            for (at, inside) in chars.by_ref() {
                current.push(inside);

                if at == last {
                    break;
                }
            }

            continue;
        }

        if current.is_empty() {
            start = index;
        }

        current.push(ch);
    }

    emit(&mut tokens, &mut current, start, &mut quoted);

    tokens
}

fn token(current: &mut String, start: usize, index: usize, quoted: bool) -> Token {
    let raw = mem::take(current);

    Token {
        span: Span {
            start,
            len: raw.len(),
            index,
            quoted,
        },
        raw,
    }
}

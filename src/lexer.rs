use crate::commands::CommandArgument;

#[derive(Debug, Clone)]
pub struct Token {
    pub contents: Option<CommandArgument>,
    pub raw: String,
    pub position: usize,
    pub length: usize,
    pub iteration: usize,
}

pub fn lex(input: String) -> Vec<Token> {
    let mut iter = input.chars().into_iter().enumerate().peekable();
    let mut tokens: Vec<Token> = vec![];
    let mut in_quotes = false;
    let mut token_start = 0;
    let mut token_count = 0;
    let mut current_token = String::new();

    while let Some((i, char)) = iter.next() {
        if char == '\\' {
            if iter.peek().is_some() {
                current_token.push(iter.next().unwrap().1);
            }

            continue;
        }

        if in_quotes && (char == '"' || char == '\'') {
            in_quotes = false;
            continue;
        } else if in_quotes {
            current_token.push(char);
            continue;
        }

        if char.is_whitespace() {
            if !current_token.is_empty() {
                tokens.push(Token { contents: None, raw: current_token.clone(), position: token_start, length: current_token.len(), iteration: token_count });
                token_count += 1;
                current_token = String::new();
            }

            continue;
        }

        if char == '"' || char == '\'' {
            in_quotes = true;
            token_start = i + 1;
            continue;
        }

        if current_token.is_empty() {
            token_start = i;
        }

        current_token.push(char);
    }

    if !current_token.is_empty() {
        tokens.push(Token { contents: None, raw: current_token.clone(), position: token_start, length: current_token.len(), iteration: token_count });
    }

    tokens
}

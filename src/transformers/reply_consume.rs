use std::{iter::Peekable, vec::IntoIter};

use serenity::all::{Context, Message};

use crate::{commands::{CommandArgument, TransformerError, TransformerReturn}, event_handler::MissingArgumentError, lexer::Token, transformers::Transformers};

impl Transformers {
    pub fn reply_consume<'a>(_ctx: &'a Context, msg: &'a Message, args: &'a mut Peekable<IntoIter<Token>>) -> TransformerReturn<'a> {
        Box::pin(async move {
            let mut new_token = Token {
                contents: None,
                raw: String::new(),
                position: 0,
                length: 0,
                iteration: 0
            };

            let reason: String = {
                if let Some(t) = args.next() && !t.raw.chars().all(char::is_whitespace) {
                    new_token.position = t.position;
                    new_token.iteration = t.iteration;
                    msg.content[t.position + 1..].to_string().clone()
                } else if let Some(reply) = msg.referenced_message.clone() {
                    println!("EXPLODE!");
                    format!("Message: {}", reply.content)
                } else {
                    return Err(TransformerError::MissingArgumentError(MissingArgumentError(String::from("Consume"))))
                }
            };

            new_token.length = reason.len();
            new_token.raw = reason;
            new_token.contents = Some(CommandArgument::String(new_token.raw.clone()));
            Ok(new_token)
        })
    }
}

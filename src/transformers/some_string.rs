use std::{iter::Peekable, vec::IntoIter};

use serenity::all::{Context, Message};

use crate::{commands::{CommandArgument, TransformerError, TransformerReturn}, event_handler::{CommandError, MissingArgumentError}, lexer::Token, transformers::Transformers};

impl Transformers {
    pub fn some_string<'a>(_ctx: &'a Context, _msg: &'a Message, args: &'a mut Peekable<IntoIter<Token>>) -> TransformerReturn<'a> {
        Box::pin(async move {
            let Some(mut input) = args.next() else {
                return Err(TransformerError::MissingArgumentError(MissingArgumentError(String::from("String"))))
            };

            if input.raw.chars().all(char::is_whitespace) || input.raw.is_empty() {
                return Err(TransformerError::CommandError(CommandError {
                    arg: Some(input),
                    title: String::from("String must not be empty and not be whitespace"),
                    hint: None
                }))
            }

            input.contents = Some(CommandArgument::String(input.raw.clone()));
            Ok(input)
        })
    }
}

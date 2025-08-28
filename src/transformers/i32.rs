use std::{iter::Peekable, vec::IntoIter};

use serenity::all::{Context, Message};

use crate::{
    commands::{CommandArgument, TransformerError, TransformerReturn},
    event_handler::{CommandError, MissingArgumentError},
    lexer::Token,
    transformers::Transformers,
};

impl Transformers {
    pub fn i32<'a>(
        _ctx: &'a Context,
        _msg: &'a Message,
        args: &'a mut Peekable<IntoIter<Token>>,
    ) -> TransformerReturn<'a> {
        Box::pin(async move {
            let Some(mut input) = args.next() else {
                return Err(TransformerError::MissingArgumentError(
                    MissingArgumentError(String::from("String")),
                ));
            };

            if let Ok(n) = input.raw.clone().parse::<i32>() {
                input.contents = Some(CommandArgument::i32(n));
                Ok(input)
            } else {
                Err(TransformerError::CommandError(CommandError {
                    arg: Some(input),
                    title: String::from("Could not turn input to a <Number>"),
                    hint: Some(String::from("provide a valid number")),
                }))
            }
        })
    }
}

use std::{iter::Peekable, vec::IntoIter};

use serenity::all::{Context, Message};

use crate::{
    commands::{CommandArgument, TransformerError, TransformerReturn},
    event_handler::MissingArgumentError,
    lexer::Token,
    transformers::Transformers,
};

impl Transformers {
    pub fn string<'a>(
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
            input.contents = Some(CommandArgument::String(input.raw.clone()));
            Ok(input)
        })
    }
}

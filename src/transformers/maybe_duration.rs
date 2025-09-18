use std::{iter::Peekable, vec::IntoIter};

use chrono::Duration;
use serenity::all::{Context, Message};

use crate::{
    commands::{CommandArgument, TransformerError, TransformerReturn},
    event_handler::MissingArgumentError,
    lexer::Token,
    transformers::Transformers,
};

impl Transformers {
    pub fn maybe_duration<'a>(
        ctx: &'a Context,
        msg: &'a Message,
        args: &'a mut Peekable<IntoIter<Token>>,
    ) -> TransformerReturn<'a> {
        Box::pin(async move {
            let Some(input) = args.peek() else {
                return Err(TransformerError::MissingArgumentError(
                    MissingArgumentError(String::from("Duration")),
                ));
            };

            let mut fake_args = vec![input.clone()].into_iter().peekable();

            let input = match Self::duration(ctx, msg, &mut fake_args).await {
                Ok(t) => {
                    args.next();
                    t
                }

                _ => Token {
                    contents: Some(CommandArgument::Duration(Duration::zero())),
                    raw: String::new(),
                    position: 0,
                    length: 0,
                    iteration: 0,
                    quoted: false,
                },
            };

            Ok(input)
        })
    }
}

use serenity::all::{Context, Message};

use crate::{commands::{CommandArgument, TransformerReturn}, lexer::Token, transformers::Transformers};

impl Transformers {
    pub fn string<'a>(_ctx: &'a Context, _msg: &'a Message, mut input: Token) -> TransformerReturn<'a> {
        Box::pin(async move {
            input.contents = Some(CommandArgument::String(input.raw.clone()));
            Ok(input)
        })
    }
}

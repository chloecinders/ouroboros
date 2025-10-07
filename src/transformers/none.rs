use std::{iter::Peekable, vec::IntoIter};

use serenity::all::{Context, Message};

use crate::{
    commands::{CommandArgument, TransformerReturn},
    lexer::Token,
    transformers::Transformers,
};

impl Transformers {
    pub fn none<'a>(
        _ctx: &'a Context,
        _msg: &'a Message,
        _args: &'a mut Peekable<IntoIter<Token>>,
    ) -> TransformerReturn<'a> {
        Box::pin(async move {
            Ok(Token {
                contents: Some(CommandArgument::None),
                raw: String::new(),
                position: 0,
                length: 0,
                iteration: 0,
                quoted: false,
                inferred: None,
            })
        })
    }
}

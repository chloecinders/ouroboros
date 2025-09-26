use std::{iter::Peekable, vec::IntoIter};

use serenity::all::{Context, Message, MessageType};

use crate::{
    commands::{CommandArgument, TransformerError, TransformerReturn},
    event_handler::CommandError,
    lexer::{InferType, Token},
    transformers::Transformers,
};

impl Transformers {
    pub fn reply_user<'a>(
        ctx: &'a Context,
        msg: &'a Message,
        args: &'a mut Peekable<IntoIter<Token>>,
    ) -> TransformerReturn<'a> {
        Box::pin(async move {
            if msg.guild_id.is_none() {
                return Err(TransformerError::CommandError(CommandError {
                    title: String::from("Server only command"),
                    hint: Some(String::from("stop trying to run this in dms!")),
                    arg: None,
                }));
            }

            if let Some(reply) = msg.referenced_message.clone() {
                let infer_type = if matches!(reply.kind, MessageType::AutoModAction) {
                    InferType::SystemMessage
                } else {
                    InferType::Message
                };

                Ok(Token {
                    contents: Some(CommandArgument::User(reply.author)),
                    raw: String::new(),
                    position: 0,
                    length: 0,
                    iteration: 0,
                    quoted: false,
                    inferred: Some(infer_type),
                })
            } else {
                return Transformers::user(ctx, msg, args).await;
            }
        })
    }
}

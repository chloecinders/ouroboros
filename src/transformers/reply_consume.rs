use std::{iter::Peekable, vec::IntoIter};

use serenity::all::{Context, Message};

use crate::{
    commands::{CommandArgument, TransformerError, TransformerReturn},
    event_handler::MissingArgumentError,
    lexer::{InferType, Token},
    transformers::Transformers,
};

impl Transformers {
    pub fn reply_consume<'a>(
        ctx: &'a Context,
        msg: &'a Message,
        args: &'a mut Peekable<IntoIter<Token>>,
    ) -> TransformerReturn<'a> {
        Box::pin(async move {
            if args.peek().is_some() {
                return Transformers::consume(ctx, msg, args).await;
            } else if let Some(reply) = msg.referenced_message.clone() {
                let mut is_log_message = false;
                if let Ok(Some(_)) =
                    sqlx::query("SELECT 1 FROM log_messages_context WHERE message_id = $1")
                        .bind(reply.id.get() as i64)
                        .fetch_optional(&*crate::SQL)
                        .await
                {
                    is_log_message = true;
                }

                let is_bot_reply = reply.author.id == ctx.cache.current_user().id;
                let (content, infer_type) = if let Some(embed) = reply.embeds.first() {
                    let desc = embed.clone().description.unwrap_or_default();

                    if is_bot_reply || is_log_message {
                        (String::new(), InferType::Bot)
                    } else if embed.clone().kind.unwrap_or_default() == "auto_moderation_message" {
                        (desc, InferType::SystemMessage)
                    } else {
                        (String::new(), InferType::Message)
                    }
                } else if is_bot_reply || is_log_message {
                    (String::new(), InferType::Bot)
                } else {
                    (String::new(), InferType::Message)
                };

                Ok(Token {
                    contents: Some(CommandArgument::String(content)),
                    raw: String::new(),
                    position: 0,
                    length: 0,
                    iteration: 0,
                    quoted: false,
                    inferred: Some(infer_type),
                })
            } else {
                Err(TransformerError::MissingArgumentError(
                    MissingArgumentError(String::from("String")),
                ))
            }
        })
    }
}

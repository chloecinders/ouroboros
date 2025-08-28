use std::{iter::Peekable, vec::IntoIter};

use serenity::all::{Context, Message};

use crate::{commands::{CommandArgument, TransformerError, TransformerReturn}, event_handler::CommandError, lexer::Token, transformers::Transformers};

impl Transformers {
    pub fn reply_member<'a>(ctx: &'a Context, msg: &'a Message, args: &'a mut Peekable<IntoIter<Token>>) -> TransformerReturn<'a> {
        Box::pin(async move {
            if msg.guild_id.is_none() {
                return Err(TransformerError::CommandError(CommandError {
                    title: String::from("Server only command"),
                    hint: Some(String::from("stop trying to run this in dms!")),
                    arg: None
                }))
            }

            if let Some(reply) = msg.referenced_message.clone() {
                let Ok(member) = msg.guild_id.unwrap().member(&ctx.http, reply.author.clone()).await else {
                    return Err(TransformerError::CommandError(CommandError {
                        title: String::from("Replied member not in server"),
                        hint: Some(String::from("the member you replied to isn't in the server anymore. Urge them to join back!")),
                        arg: None
                    }))
                };

                Ok(Token {
                    contents: Some(CommandArgument::Member(member)),
                    raw: String::new(),
                    position: 0,
                    length: 0,
                    iteration: 0
                })
            } else {
                return Transformers::member(ctx, msg, args).await
            }
        })
    }
}

use std::{iter::Peekable, vec::IntoIter};

use serenity::all::{Context, Message};

use crate::{
    commands::{CommandArgument, TransformerError, TransformerReturn},
    event_handler::MissingArgumentError,
    lexer::Token,
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
                let content = if let Some(embed) = reply.embeds.first()
                    && embed.clone().kind.unwrap_or(String::new()) == "auto_moderation_message"
                {
                    let reason_type = if let Some(field) =
                        embed.fields.iter().find(|f| f.name == "quarantine_user")
                    {
                        if field.value == "display_name" {
                            String::from("Name: ")
                        } else if field.value == "clan_tag" {
                            String::from("Tag: ")
                        } else {
                            String::from("Automod: ")
                        }
                    } else {
                        String::from("Message: ")
                    };

                    let content = embed.clone().description.unwrap_or(msg.content.clone());

                    format!("{reason_type}{content}")
                } else {
                    format!("Message: {}", reply.content)
                };

                return Ok(Token {
                    contents: Some(CommandArgument::String(content)),
                    raw: String::new(),
                    position: 0,
                    length: 0,
                    iteration: 0,
                });
            } else {
                return Err(TransformerError::MissingArgumentError(
                    MissingArgumentError(String::from("String")),
                ));
            }
        })
    }
}

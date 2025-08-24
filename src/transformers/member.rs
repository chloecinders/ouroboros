use std::{iter::Peekable, vec::IntoIter};

use serenity::all::{Context, Message};

use crate::{commands::{CommandArgument, TransformerError, TransformerReturn}, event_handler::{CommandError, MissingArgumentError}, lexer::Token, transformers::Transformers};

impl Transformers {
    pub fn member<'a>(ctx: &'a Context, msg: &'a Message, args: &'a mut Peekable<IntoIter<Token>>) -> TransformerReturn<'a> {
        Box::pin(async move {
            let Some(mut input) = args.next() else {
                return Err(TransformerError::MissingArgumentError(MissingArgumentError(String::from("Member"))))
            };

            let id = if let Ok(id) = input.raw.parse::<u64>() {
                id
            } else if input.raw.starts_with("<@") && input.raw.ends_with(">") {
                let new_input = input.raw.strip_prefix("<@").unwrap().strip_suffix(">").unwrap();

                if let Ok(id) = new_input.parse::<u64>() {
                    id
                } else {
                    return Err(TransformerError::CommandError(CommandError {
                        arg: Some(input),
                        title: String::from("Could not turn input to a <Discord Member>"),
                        hint: Some(String::from("provide a valid ID or mention")),
                    }));
                }
            } else {
                let Ok(users) = msg.guild_id.unwrap_or_else(|| unreachable!()).members(&ctx.http, None, None).await else {
                    return Err(TransformerError::CommandError(CommandError {
                        arg: Some(input),
                        title: String::from("Could not turn input to a <Discord User>"),
                        hint: Some(String::from("provide a valid ID or mention")),
                    }));
                };

                let opt_user = users.iter().find(|u| u.user.name == input.raw);

                if let Some(user) = opt_user {
                    input.contents = Some(CommandArgument::Member(user.clone()));
                    return Ok(input);
                }

                return Err(TransformerError::CommandError(CommandError {
                    arg: Some(input),
                    title: String::from("Could not turn input to a <Discord User>"),
                    hint: Some(String::from("provide a valid ID or mention")),
                }));
            };

            let member = {
                if let Ok(member) = msg.guild_id.unwrap_or_else(|| unreachable!()).member(&ctx.http, id).await {
                    member.clone()
                } else {
                    return Err(TransformerError::CommandError(CommandError {
                        arg: Some(input),
                        title: String::from("Could not find the <Discord Member>"),
                        hint: Some(String::from("make sure the ID or mention you provided is valid and that the member is in this server!")),
                    }));
                }
            };

            input.contents = Some(CommandArgument::Member(member));
            Ok(input)
        })
    }
}

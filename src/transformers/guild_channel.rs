use std::{iter::Peekable, vec::IntoIter};

use serenity::all::{Context, Message};

use crate::{commands::{CommandArgument, TransformerError, TransformerReturn}, event_handler::{CommandError, MissingArgumentError}, lexer::Token, transformers::Transformers};

impl Transformers {
    pub fn guild_channel<'a>(ctx: &'a Context, msg: &'a Message, args: &'a mut Peekable<IntoIter<Token>>) -> TransformerReturn<'a> {
        Box::pin(async move {
            let Some(mut input) = args.next() else {
                return Err(TransformerError::MissingArgumentError(MissingArgumentError(String::from("Member"))))
            };

            let Some(guild) = msg.guild_id else {
                return Err(TransformerError::CommandError(CommandError {
                    title: String::from("Server only command"),
                    hint: Some(String::from("stop trying to run this in dms!")),
                    arg: None
                }))
            };

            let Ok(channels) = guild.channels(&ctx.http).await else {
                return Err(TransformerError::CommandError(CommandError {
                    title: String::from("Couldn't get guild channels"),
                    hint: Some(String::from("please try again later.")),
                    arg: None
                }))
            };

            let id = if let Ok(id) = input.raw.parse::<u64>() {
                id
            } else if input.raw.starts_with("<#") && input.raw.ends_with(">") {
                let new_input = input.raw.strip_prefix("<#").unwrap().strip_suffix(">").unwrap();

                if let Ok(id) = new_input.parse::<u64>() {
                    id
                } else {
                    return Err(TransformerError::CommandError(CommandError {
                        arg: Some(input),
                        title: String::from("Could not turn input to a <Guild Channel>"),
                        hint: Some(String::from("provide a valid ID or mention")),
                    }));
                }
            } else {
                0
            };

            for (channel_id, channel) in channels.into_iter() {
                if id == channel_id.get() || channel.name == input.raw {
                    input.contents = Some(CommandArgument::GuildChannel(channel));
                    return Ok(input);
                }
            }

            Err(TransformerError::CommandError(CommandError {
                title: String::from("Could not find channel in guild"),
                hint: Some(String::from("make sure to input the channel id or the exact name.")),
                arg: None
            }))
        })
    }
}

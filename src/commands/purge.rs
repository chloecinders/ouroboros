use std::sync::Arc;

use chrono::{Duration, Utc};
use serenity::{
    all::{Context, CreateEmbed, CreateMessage, GetMessages, Mentionable, Message, Permissions},
    async_trait,
};

use crate::{
    commands::{
        Command, CommandArgument, CommandCategory, CommandPermissions, CommandSyntax, TransformerFn,
    }, constants::BRAND_BLUE, event_handler::CommandError, lexer::Token, transformers::Transformers, utils::guild_mod_log
};
use ouroboros_macros::command;

pub struct Purge;

impl Purge {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Purge {
    fn get_name(&self) -> String {
        String::from("purge")
    }

    fn get_short(&self) -> String {
        String::from("Mass deletes a specific amount of messages")
    }

    fn get_full(&self) -> String {
        String::from(
            "Mass deletes a specific amount of messages from a channel. \
            Messages older than 2 weeks are ignored. \
            Count must be between 2 and 100.",
        )
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![CommandSyntax::Number("count", true)]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::Moderation
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::i32] count: i32,
    ) -> Result<(), CommandError> {
        if !(2..=100).contains(&count) {
            return Err(CommandError {
                title: String::from("Message count must be between 2 and 100"),
                hint: None,
                arg: Some(args.first().unwrap().clone()),
            });
        }

        let count = count as u8;

        let Ok(messages) = msg
            .channel_id
            .messages(&ctx.http, GetMessages::new().limit(count))
            .await
        else {
            return Err(CommandError {
                title: String::from("Could not get channel messages"),
                hint: Some(String::from(
                    "make sure the bot has enough permissions to view the messages of this channel",
                )),
                arg: None,
            });
        };

        let now = Utc::now();
        let two_weeks = Duration::weeks(2);

        let filtered = messages.iter().filter_map(|m| {
            let diff = (now - *m.timestamp).num_seconds().abs();

            if diff <= two_weeks.num_seconds() {
                Some(m.id)
            } else {
                None
            }
        });

        let ids = filtered.clone().map(|m| m.get().to_string()).collect::<Vec<_>>();

        if msg
            .channel_id
            .delete_messages(&ctx.http, filtered)
            .await
            .is_err()
        {
            return Err(CommandError {
                title: String::from("Could not delete channel messages"),
                hint: Some(String::from(
                    "make sure the bot has enough permissions to delete the messages of this channel",
                )),
                arg: None,
            });
        };

        guild_mod_log(
            &ctx.http,
            msg.guild_id.unwrap(),
            CreateMessage::new()
                .add_embed(
                    CreateEmbed::new()
                        .description(format!(
                            "**MESSAGES PURGED**\n-# Actor: {} `{}` | Channel: <#{}> | Count: {}\n```\n{}\n```",
                            msg.author.mention(),
                            msg.author.id.get(),
                            msg.channel_id.get(),
                            count,
                            ids.join("\n")
                        ))
                        .color(BRAND_BLUE)
                )
        ).await;

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![Permissions::MANAGE_MESSAGES],
            one_of: vec![],
        }
    }
}

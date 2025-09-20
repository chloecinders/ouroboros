use std::sync::Arc;

use serenity::{
    all::{Context, CreateEmbed, CreateMessage, Mentionable, Message, Permissions},
    async_trait,
};
use sqlx::query;
use tracing::{error, warn};

use crate::{
    SQL, commands::{
        Command, CommandArgument, CommandCategory, CommandPermissions, CommandSyntax, TransformerFn,
    }, constants::BRAND_BLUE, event_handler::CommandError, lexer::Token, transformers::Transformers, utils::{guild_mod_log, message_and_dm, tinyid}
};
use ouroboros_macros::command;

pub struct Softban;

impl Softban {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Softban {
    fn get_name(&self) -> String {
        String::from("softban")
    }

    fn get_short(&self) -> String {
        String::from("Softbans a member from the server")
    }

    fn get_full(&self) -> String {
        String::from("
            Bans and immediately unbans a member from the server and leaves a note in the users log. \
            Useful for clearing out messages without permanent consequences. \
            Clears 1 day of messages.")
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![
            CommandSyntax::Member("user", true),
            CommandSyntax::Consume("reason"),
        ]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::Moderation
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::reply_member] member: Member,
        #[transformers::reply_consume] reason: Option<String>,
    ) -> Result<(), CommandError> {
        let mut reason = reason
            .map(|s| {
                if s.is_empty() || s.chars().all(char::is_whitespace) {
                    String::from("No reason provided")
                } else {
                    s
                }
            })
            .unwrap_or(String::from("No reason provided"));

        if reason.len() > 500 {
            reason.truncate(500);
            reason.push_str("...");
        }

        let db_id = tinyid().await;

        let res = query!(
            "INSERT INTO actions (id, type, guild_id, user_id, moderator_id, reason) VALUES ($1, 'softban', $2, $3, $4, $5)",
            db_id,
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            member.user.id.get() as i64,
            msg.author.id.get() as i64,
            reason.as_str()
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while softbanning; err = {err:?}");
            return Err(CommandError {
                title: String::from("Could not softban member"),
                hint: Some(String::from("please try again later")),
                arg: None,
            });
        }

        if let Err(err) = member.ban_with_reason(&ctx.http, 1, &reason).await {
            warn!("Got error while softbanning; err = {err:?}");

            if query!("DELETE FROM actions WHERE id = $1", db_id)
                .execute(SQL.get().unwrap())
                .await
                .is_err()
            {
                error!(
                    "Got an error while softbanning and an error with the database! Stray softban entry in DB & manual action required; id = {db_id}; err = {err:?}"
                );
            }

            return Err(CommandError {
                title: String::from("Could not softban member"),
                hint: Some(String::from(
                    "check if the bot has the ban members permission or try again later",
                )),
                arg: None,
            });
        }

        if let Err(err) = member.unban(&ctx.http).await {
            warn!("Got error while softunbanning; err = {err:?}");

            // leave the entry in the db since they have still faced the consequences
            return Err(CommandError {
                title: String::from("Member banned, but bot ran into an error trying to unban"),
                hint: Some(String::from(
                    "manually unban the member and check if the bot has the ban members permission",
                )),
                arg: None,
            });
        }

        message_and_dm(
            &ctx,
            &msg,
            &member.user,
            |a| format!(
                "**{} SOFTBANNED**\n-# Log ID: `{db_id}`{a}\n```\n{reason}\n```",
                member.mention()
            ),
            format!(
                "You have been kicked from {}\n```\n{}\n```",
                msg.guild(&ctx.cache)
                    .map(|g| g.name.clone())
                    .unwrap_or(String::from("UNKNOWN_GUILD")),
                reason
            ),
        ).await;

        guild_mod_log(
            &ctx.http,
            msg.guild_id.unwrap(),
            CreateMessage::new()
                .add_embed(
                    CreateEmbed::new()
                        .description(format!(
                            "**MEMBER SOFTBANNED**\n-# Log ID: `{db_id}` | Actor: {} `{}` | Target: {} `{}`\n```\n{reason}\n```",
                            msg.author.mention(),
                            msg.author.id.get(),
                            member.mention(),
                            member.user.id.get()
                        ))
                        .color(BRAND_BLUE)
                )
        ).await;

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![Permissions::KICK_MEMBERS],
            one_of: vec![],
        }
    }
}

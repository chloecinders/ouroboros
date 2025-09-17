use std::sync::Arc;

use ouroboros_macros::command;
use serenity::{
    all::{
        Context, CreateAllowedMentions, CreateEmbed, CreateEmbedFooter, CreateMessage, Mentionable, Message, Permissions
    },
    async_trait,
};
use sqlx::query;
use tracing::{error, warn};

use crate::{
    SQL,
    commands::{Command, CommandArgument, CommandCategory, CommandPermissions, CommandSyntax, TransformerFn},
    constants::BRAND_BLUE,
    event_handler::CommandError,
    lexer::Token,
    transformers::Transformers,
    utils::tinyid,
};

pub struct Unmute;

impl Unmute {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Unmute {
    fn get_name(&self) -> String {
        String::from("unmute")
    }

    fn get_short(&self) -> String {
        String::from("Unmutes a member in the server")
    }

    fn get_full(&self) -> String {
        String::from("Unmutes a member in the server.")
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![
            CommandSyntax::Member("member", true),
            CommandSyntax::String("reason", false),
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
        #[transformers::reply_member] mut member: Member,
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

        let res = query!(
            "UPDATE actions SET active = false, expires_at = NULL WHERE guild_id = $1 AND user_id = $2 AND type = 'mute' AND active = true;",
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            member.user.id.get() as i64,
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while unmuting; err = {err:?}");
            return Err(CommandError {
                title: String::from("Could not unmute member"),
                hint: Some(String::from("please try again later")),
                arg: None,
            });
        }

        let db_id = tinyid().await;

        let res = query!(
            "INSERT INTO actions (id, type, guild_id, user_id, moderator_id, reason) VALUES ($1, 'unmute', $2, $3, $4, $5)",
            db_id,
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            member.user.id.get() as i64,
            msg.author.id.get() as i64,
            reason.clone()
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while unmuting; err = {err:?}");
            return Err(CommandError {
                title: String::from("Could not unmute member"),
                hint: Some(String::from("please try again later")),
                arg: None,
            });
        }

        if let Err(err) = member.enable_communication(&ctx.http).await {
            warn!("Got error while unmuting; err = {err:?}");

            if query!("DELETE FROM actions WHERE id = $1", db_id)
                .execute(SQL.get().unwrap())
                .await
                .is_err()
            {
                error!(
                    "Got an error while unmuting and an error with the database! Stray unmute entry in DB & manual action required; id = {db_id}; err = {err:?}"
                );
            }

            return Err(CommandError {
                title: String::from("Could not unmute member"),
                hint: Some(String::from(
                    "check if the bot has the timeout members permission or try again later",
                )),
                arg: None,
            });
        }

        let reply = CreateMessage::new()
            .add_embed(
                CreateEmbed::new()
                    .description(format!(
                        "Unmuted {}\n```\n{}\n```",
                        member.mention(),
                        reason
                    ))
                    .color(BRAND_BLUE)
                    .footer(CreateEmbedFooter::new(format!("Log ID: {db_id}")))
            )
            .reference_message(&msg)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

        if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {err:?}");
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![Permissions::MODERATE_MEMBERS],
            one_of: vec![],
        }
    }
}

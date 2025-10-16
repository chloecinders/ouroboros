use std::{sync::Arc, time::Duration};

use ouroboros_macros::command;
use serenity::{
    all::{
        Context, CreateAllowedMentions, CreateEmbed, CreateMessage, Mentionable, Message,
        Permissions,
    },
    async_trait,
};
use sqlx::query;
use tokio::time::sleep;
use tracing::{error, warn};

use crate::{
    SQL,
    commands::{
        Command, CommandArgument, CommandCategory, CommandParameter, CommandPermissions,
        CommandSyntax, TransformerFnArc,
    },
    constants::BRAND_BLUE,
    event_handler::CommandError,
    lexer::{InferType, Token},
    transformers::Transformers,
    utils::{LogType, guild_log, tinyid},
};

pub struct Unmute;

impl Unmute {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Unmute {
    fn get_name(&self) -> &'static str {
        "unmute"
    }

    fn get_short(&self) -> &'static str {
        "Unmutes a member in the server"
    }

    fn get_full(&self) -> &'static str {
        "Unmutes a member in the server."
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

    fn get_params(&self) -> Vec<&'static CommandParameter<'static>> {
        vec![]
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::reply_member] mut member: Member,
        #[transformers::reply_consume] reason: Option<String>,
    ) -> Result<(), CommandError> {
        let inferred = args
            .first()
            .map(|a| matches!(a.inferred, Some(InferType::Message)))
            .unwrap_or(false);
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
                        "**{} UNMUTED**\n-# Log ID: `{db_id}`\n```\n{reason}\n```",
                        member.mention()
                    ))
                    .color(BRAND_BLUE),
            )
            .reference_message(&msg)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

        let reply_msg = msg.channel_id.send_message(&ctx.http, reply).await;

        guild_log(
            &ctx.http,
            LogType::MemberUnmute,
            msg.guild_id.unwrap(),
            CreateMessage::new()
                .add_embed(
                    CreateEmbed::new()
                        .description(format!(
                            "**MEMBER UNMUTED**\n-# Log ID: `{db_id}` | Actor: {} `{}` | Target: {} `{}`\n```\n{reason}\n```",
                            msg.author.mention(),
                            msg.author.id.get(),
                            member.mention(),
                            member.user.id.get()
                        ))
                        .color(BRAND_BLUE)
                )
        ).await;

        let reply_msg = match reply_msg {
            Ok(m) => m,
            Err(err) => {
                warn!("Could not send message; err = {err:?}");
                return Ok(());
            }
        };

        if inferred && let Some(reply) = msg.referenced_message.clone() {
            let _ = reply.delete(&ctx.http).await;
        }

        if inferred {
            let http = ctx.http.clone();

            tokio::spawn(async move {
                sleep(Duration::from_secs(5)).await;
                let _ = msg.delete(&http).await;
                let _ = reply_msg.delete(&http).await;
            });
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![Permissions::MODERATE_MEMBERS],
            one_of: vec![],
            bot: [
                CommandPermissions::baseline().as_slice(),
                CommandPermissions::moderation().as_slice(),
            ]
            .concat(),
        }
    }
}

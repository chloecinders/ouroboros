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

pub struct Unban;

impl Unban {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Unban {
    fn get_name(&self) -> &'static str {
        "unban"
    }

    fn get_short(&self) -> &'static str {
        "Unbans a member from the server"
    }

    fn get_full(&self) -> &'static str {
        "Unbans a member from the server."
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![
            CommandSyntax::User("user", true),
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
        #[transformers::reply_user] user: User,
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
            "UPDATE actions SET active = false, expires_at = NULL WHERE guild_id = $1 AND user_id = $2 AND type = 'ban' AND active = true;",
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            user.id.get() as i64,
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while unbanning; err = {err:?}");
            return Err(CommandError {
                title: String::from("Could not unban member"),
                hint: Some(String::from("please try again later")),
                arg: None,
            });
        }

        let db_id = tinyid().await;

        let res = query!(
            "INSERT INTO actions (id, type, guild_id, user_id, moderator_id, reason) VALUES ($1, 'unban', $2, $3, $4, $5)",
            db_id,
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            user.id.get() as i64,
            msg.author.id.get() as i64,
            reason.clone()
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while unbanning; err = {err:?}");
            return Err(CommandError {
                title: String::from("Could not unban member"),
                hint: Some(String::from("please try again later")),
                arg: None,
            });
        }

        if let Err(err) = ctx
            .http
            .as_ref()
            .remove_ban(msg.guild_id.unwrap(), user.id, Some(&reason))
            .await
        {
            warn!("Got error while unbanning; err = {err:?}");

            if query!("DELETE FROM actions WHERE id = $1", db_id)
                .execute(SQL.get().unwrap())
                .await
                .is_err()
            {
                error!(
                    "Got an error while unbanning and an error with the database! Stray unban entry in DB & manual action required; id = {db_id}; err = {err:?}"
                );
            }

            return Err(CommandError {
                title: String::from("Could not unban member"),
                hint: Some(String::from(
                    "check if the bot has the ban members permission or try again later",
                )),
                arg: None,
            });
        }

        let reply = CreateMessage::new()
            .add_embed(
                CreateEmbed::new()
                    .description(format!(
                        "**{} UNBANNED**\n-# Log ID: `{db_id}`\n```\n{reason}\n```",
                        user.mention()
                    ))
                    .color(BRAND_BLUE),
            )
            .reference_message(&msg)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

        let reply_msg = msg.channel_id.send_message(&ctx, reply).await;

        guild_log(
            &ctx,
            LogType::MemberUnban,
            msg.guild_id.unwrap(),
            CreateMessage::new()
                .add_embed(
                    CreateEmbed::new()
                        .description(format!(
                            "**MEMBER UNBANNED**\n-# Log ID: `{db_id}` | Actor: {} `{}` | Target: {} `{}`\n```\n{reason}\n```",
                            msg.author.mention(),
                            msg.author.id.get(),
                            user.mention(),
                            user.id.get()
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
            let _ = reply.delete(&ctx).await;
        }

        if inferred {
            tokio::spawn(async move {
                sleep(Duration::from_secs(5)).await;
                let _ = msg.delete(&ctx).await;
                let _ = reply_msg.delete(&ctx).await;
            });
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![Permissions::BAN_MEMBERS],
            one_of: vec![],
            bot: [
                CommandPermissions::baseline().as_slice(),
                CommandPermissions::moderation().as_slice(),
            ]
            .concat(),
        }
    }
}

use std::sync::Arc;

use serenity::{
    all::{Context, CreateEmbed, CreateMessage, Mentionable, Message, Permissions},
    async_trait,
};
use sqlx::query;
use tracing::warn;

use crate::{
    SQL,
    commands::{
        Command, CommandArgument, CommandCategory, CommandParameter, CommandPermissions, CommandSyntax, TransformerFnArc
    },
    constants::BRAND_BLUE,
    event_handler::CommandError,
    lexer::{InferType, Token},
    transformers::Transformers,
    utils::{LogType, guild_log, message_and_dm, tinyid},
};
use ouroboros_macros::command;

pub struct Warn;

impl Warn {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Warn {
    fn get_name(&self) -> &'static str {
        "warn"
    }

    fn get_short(&self) -> &'static str {
        "Warns a member of the server"
    }

    fn get_full(&self) -> &'static str {
        "Warns a member, storing a note in the users log."
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

    fn get_params(&self) -> Vec<&'static CommandParameter<'static>> {
        vec![]
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::reply_member] member: Member,
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

        let db_id = tinyid().await;

        let res = query!(
            "INSERT INTO actions (id, type, guild_id, user_id, moderator_id, reason) VALUES ($1, 'warn', $2, $3, $4, $5)",
            db_id,
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            member.user.id.get() as i64,
            msg.author.id.get() as i64,
            reason.as_str()
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while warning; err = {err:?}");
            return Err(CommandError {
                title: String::from("Could not warn member"),
                hint: Some(String::from("please try again later")),
                arg: None,
            });
        }

        if inferred && let Some(reply) = msg.referenced_message.clone() {
            let _ = reply.delete(&ctx.http).await;
        }

        message_and_dm(
            &ctx,
            &msg,
            &member.user,
            |a| {
                format!(
                    "**{} WARNED**\n-# Log ID: `{db_id}`{a}\n```\n{reason}\n```",
                    member.mention()
                )
            },
            format!(
                "**WARNED**\n-# Server: {}\n```\n{}\n```",
                msg.guild(&ctx.cache)
                    .map(|g| g.name.clone())
                    .unwrap_or(String::from("UNKNOWN_GUILD")),
                reason
            ),
            inferred,
            false
        )
        .await;

        guild_log(
            &ctx.http,
            LogType::MemberWarn,
            msg.guild_id.unwrap(),
            CreateMessage::new()
                .add_embed(
                    CreateEmbed::new()
                        .description(format!(
                            "**MEMBER WARNED**\n-# Log ID: `{db_id}` | Actor: {} `{}` | Target: {} `{}`\n```\n{reason}\n```",
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
            required: vec![Permissions::MANAGE_NICKNAMES],
            one_of: vec![],
        }
    }
}

use std::sync::Arc;

use chrono::{Duration, Utc};
use serenity::{
    all::{Context, CreateEmbed, CreateMessage, EditMember, Mentionable, Message, Permissions},
    async_trait,
};
use sqlx::query;
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
    utils::{LogType, guild_log, message_and_dm, tinyid},
};
use ouroboros_macros::command;

pub struct Mute;

impl Mute {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Mute {
    fn get_name(&self) -> &'static str {
        "mute"
    }

    fn get_short(&self) -> &'static str {
        "Uses the Discord timeout feature on a member"
    }

    fn get_full(&self) -> &'static str {
        "Uses the Discord timeout feature on a member and leaves a note in the users log. \
        Has a max duration of 28 days. Duration (including the removal of the timeout) is managed by Discord"
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![
            CommandSyntax::Member("member", true),
            CommandSyntax::Duration("duration", true),
            CommandSyntax::Reason("reason"),
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
        #[transformers::maybe_duration] duration: Option<Duration>,
        #[transformers::reply_consume] reason: Option<String>,
    ) -> Result<(), CommandError> {
        let inferred = args
            .first()
            .map(|a| matches!(a.inferred, Some(InferType::Message)))
            .unwrap_or(false);
        let duration = duration.unwrap_or(Duration::zero());
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

        let time_string = if !duration.is_zero() {
            let (time, mut unit) = match () {
                _ if (duration.num_days() as f64 / 365.0).fract() == 0.0
                    && duration.num_days() >= 365 =>
                {
                    (duration.num_days() / 365, String::from("year"))
                }
                _ if (duration.num_days() as f64 / 30.0).fract() == 0.0
                    && duration.num_days() >= 30 =>
                {
                    (duration.num_days() / 30, String::from("month"))
                }
                _ if duration.num_days() != 0 => (duration.num_days(), String::from("day")),
                _ if duration.num_hours() != 0 => (duration.num_hours(), String::from("hour")),
                _ if duration.num_minutes() != 0 => {
                    (duration.num_minutes(), String::from("minute"))
                }
                _ if duration.num_seconds() != 0 => {
                    (duration.num_seconds(), String::from("second"))
                }
                _ => (0, String::new()),
            };

            if time > 1 {
                unit += "s";
            }

            format!("{time} {unit}")
        } else {
            String::from("permanent")
        };

        let duration = if duration.is_zero() {
            None
        } else {
            Some(Utc::now() + duration)
        };

        let res = query!(
            "INSERT INTO actions (id, type, guild_id, user_id, moderator_id, reason, expires_at, last_reapplied_at) VALUES ($1, 'mute', $2, $3, $4, $5, $6, NOW())",
            db_id,
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            member.user.id.get() as i64,
            msg.author.id.get() as i64,
            reason.as_str(),
            duration.map(|d| d.naive_utc()),
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while timing out; err = {err:?}");
            return Err(CommandError {
                title: String::from("Could not time member out"),
                hint: Some(String::from("please try again later")),
                arg: None,
            });
        }

        let audit_reason = format!(
            "Ouroboros Managed Mute: log id `{db_id}`. Please use Ouroboros to unmute to avoid accidental re-application!"
        );

        let edit = if let Some(duration) = duration {
            EditMember::new()
                .audit_log_reason(audit_reason.as_str())
                .disable_communication_until_datetime(duration.into())
        } else {
            EditMember::new()
                .audit_log_reason(&reason)
                .disable_communication_until_datetime((Utc::now() + Duration::days(27)).into())
        };

        if let Err(err) = member.guild_id.edit_member(&ctx.http, &member, edit).await {
            warn!("Got error while timinng out; err = {err:?}");

            if query!("DELETE FROM actions WHERE id = $1", db_id)
                .execute(SQL.get().unwrap())
                .await
                .is_err()
            {
                error!(
                    "Got an error while timing out and an error with the database! Stray timeout entry in DB & manual action required; id = {db_id}; err = {err:?}"
                );
            }

            return Err(CommandError {
                title: String::from("Could not time member out"),
                hint: Some(String::from(
                    "check if the bot has the timeout members permission or try again later",
                )),
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
            |a| format!(
                "**{} TIMEOUT**\n-# Log ID: `{db_id}` | Duration: {time_string}{a}\n```\n{reason}\n```",
                member.mention()
            ),
            format!(
                "**TIMEOUT**\n-# Server: {} | Duration: {}\n```\n{}\n```",
                msg.guild(&ctx.cache)
                    .map(|g| g.name.clone())
                    .unwrap_or(String::from("UNKNOWN_GUILD")),
                time_string,
                reason
            ),
            inferred,
            false
        )
        .await;

        guild_log(
            &ctx.http,
            LogType::MemberMute,
            msg.guild_id.unwrap(),
            CreateMessage::new()
                .add_embed(
                    CreateEmbed::new()
                        .description(format!(
                            "**MEMBER TIMEOUT**\n-# Log ID: `{db_id}` | Actor: {} `{}` | Target: {} `{}` | Duration: {time_string}\n```\n{reason}\n```",
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
            required: vec![Permissions::MODERATE_MEMBERS],
            one_of: vec![],
            bot: CommandPermissions::baseline(),
        }
    }
}

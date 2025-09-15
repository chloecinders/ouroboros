use std::sync::Arc;

use chrono::{Duration, Utc};
use serenity::{
    all::{Context, Mentionable, Message, Permissions},
    async_trait,
};
use sqlx::query;
use tracing::{error, warn};

use crate::{
    SQL,
    commands::{Command, CommandArgument, CommandPermissions, CommandSyntax, TransformerFn},
    event_handler::CommandError,
    lexer::Token,
    transformers::Transformers,
    utils::{message_and_dm, tinyid},
};
use ouroboros_macros::command;

pub struct Ban;

impl Ban {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Ban {
    fn get_name(&self) -> String {
        String::from("ban")
    }

    fn get_short(&self) -> String {
        String::from("Bans a member from the server")
    }

    fn get_full(&self) -> String {
        String::from(
            "Bans from the server and leaves a note in the users log. \
            Defaults to permanent if no duration is provided. \
            Use 0 for the duration to make the ban permanent. \
            Ban expiry is checked every 5 minutes.",
        )
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![
            CommandSyntax::Member("member", true),
            CommandSyntax::Duration("duration", false),
            CommandSyntax::Reason("reason"),
        ]
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::reply_user] user: User,
        #[transformers::maybe_duration] duration: Option<Duration>,
        #[transformers::reply_consume] reason: Option<String>,
    ) -> Result<(), CommandError> {
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

            format!("for {time} {unit}")
        } else {
            String::from("permanently")
        };

        let duration = if duration.is_zero() {
            None
        } else {
            Some((Utc::now() + duration).naive_utc())
        };

        let res = query!(
            "UPDATE actions SET active = false WHERE guild_id = $1 AND user_id = $2 AND type = 'ban'",
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            user.id.get() as i64,
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while banning; err = {err:?}");
            return Err(CommandError {
                title: String::from("Could not ban member"),
                hint: Some(String::from("please try again later")),
                arg: None,
            });
        }

        let res = query!(
            "INSERT INTO actions (id, type, guild_id, user_id, moderator_id, reason, expires_at) VALUES ($1, 'ban', $2, $3, $4, $5, $6)",
            db_id,
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            user.id.get() as i64,
            msg.author.id.get() as i64,
            reason.as_str(),
            duration
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while banning; err = {err:?}");
            return Err(CommandError {
                title: String::from("Could not ban member"),
                hint: Some(String::from("please try again later")),
                arg: None,
            });
        }

        if let Err(err) = msg
            .guild_id
            .unwrap()
            .ban_with_reason(&ctx.http, &user, 0, &reason)
            .await
        {
            warn!("Got error while banning; err = {err:?}");

            // cant do much here...
            if query!("DELETE FROM actions WHERE id = $1", db_id)
                .execute(SQL.get().unwrap())
                .await
                .is_err()
            {
                error!(
                    "Got an error while banning and an error with the database! Stray ban entry in DB & manual action required; id = {db_id}; err = {err:?}"
                );
            }

            return Err(CommandError {
                title: String::from("Could not ban member"),
                hint: Some(String::from(
                    "check if the bot has the ban members permission or try again later",
                )),
                arg: None,
            });
        }

        message_and_dm(
            &ctx,
            &msg,
            &user,
            format!(
                "Banned {} {}\n```\n{}\n```",
                user.mention(),
                time_string,
                reason
            ),
            format!(
                "You have been banned from {} {}\n```\n{}\n```",
                msg.guild(&ctx.cache)
                    .map(|g| g.name.clone())
                    .unwrap_or(String::from("UNKNOWN_GUILD")),
                time_string,
                reason
            ),
        )
        .await;

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![Permissions::BAN_MEMBERS],
            one_of: vec![],
        }
    }
}

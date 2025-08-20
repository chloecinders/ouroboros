use std::{sync::Arc};

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use serenity::{all::{Context, CreateEmbed, CreateMessage, Mentionable, Message, Permissions}, async_trait};
use sqlx::query;
use tracing::{error, warn};

use crate::{commands::{Command, CommandArgument, CommandPermissions, CommandSyntax, TransformerFn}, constants::BRAND_BLUE, database::ActionType, event_handler::CommandError, lexer::Token, transformers::Transformers, SQL};
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
        String::from("
            Bans from the server and leaves a note in the users log. \
            Use 0 for the duration to make the ban permanent. \
            Ban expiry is checked every 5 minutes.")
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![
            CommandSyntax::Member("member", true),
            CommandSyntax::Duration("user", true),
            CommandSyntax::Consume("reason")
        ]
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::member] member: Member,
        #[transformers::duration] duration: Duration
    ) -> Result<(), CommandError> {
        let mut reason: String = {
            if let Some(t) = args_iter.next() {
                msg.content[t.position + 1..].to_string().clone()
            } else {
                String::from("No reason provided")
            }
        };

        if reason.len() > 500 {
            reason.truncate(500);
            reason.push_str("...");
        }

        let db_id = nanoid::nanoid!();

        let time_string = if !duration.is_zero() {
            let (time, mut unit) = match () {
                _ if (duration.num_days() as f64 / 365.0).fract() == 0.0 && duration.num_days() >= 365 => (duration.num_days() / 365, String::from("year")),
                _ if (duration.num_days() as f64 / 30.0).fract() == 0.0 && duration.num_days() >= 30 => (duration.num_days() / 30, String::from("month")),
                _ if duration.num_days() != 0 => (duration.num_days(), String::from("day")),
                _ if duration.num_hours() != 0 => (duration.num_hours(), String::from("hour")),
                _ if duration.num_minutes() != 0 => (duration.num_minutes(), String::from("minute")),
                _ if duration.num_seconds() != 0 => (duration.num_seconds(), String::from("second")),
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
            "INSERT INTO actions (id, type, guild_id, user_id, moderator_id, reason, expires_at) VALUES ($1, $2::action_type, $3, $4, $5, $6, $7)",
            db_id,
            ActionType::Ban as ActionType,
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            member.user.id.get() as i64,
            msg.author.id.get() as i64,
            reason.as_str(),
            duration
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while banning; err = {err:?}");
            return Err(CommandError { title: String::from("Could not ban member"), hint: Some(String::from("please try again later")), arg: None });
        }

        if let Err(err) = member.ban_with_reason(&ctx.http, 1, &reason).await {
            warn!("Got error while banning; err = {err:?}");

            // cant do much here...
            if let Err(_) = query!("DELETE FROM actions WHERE id = $1", db_id).execute(SQL.get().unwrap()).await {
                error!("Got an error while banning and an error with the database! Stray ban entry in DB & manual action required; id = {db_id}; err = {err:?}");
            }

            return Err(CommandError { title: String::from("Could not ban member"), hint: Some(String::from("Check if the bot has the ban members permission or try again later")), arg: None });
        }

        let reply = CreateMessage::new()
            .add_embed(CreateEmbed::new().description(format!("Banned {} {}\n```\n{}\n```", member.mention(), time_string, reason)).color(BRAND_BLUE.clone()))
            .reference_message(&msg);

        if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {err:?}");
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions { required: vec![Permissions::BAN_MEMBERS], one_of: vec![] }
    }
}

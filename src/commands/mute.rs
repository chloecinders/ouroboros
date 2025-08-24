use std::{sync::Arc};

use chrono::{Duration, Utc};
use serenity::{all::{Context, CreateEmbed, CreateMessage, EditMember, Mentionable, Message, Permissions}, async_trait};
use sqlx::query;
use tracing::{error, warn};

use crate::{commands::{Command, CommandArgument, CommandPermissions, CommandSyntax, TransformerFn}, constants::BRAND_BLUE, event_handler::CommandError, lexer::Token, transformers::Transformers, SQL};
use ouroboros_macros::command;

pub struct Mute;

impl Mute {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Mute {
    fn get_name(&self) -> String {
        String::from("mute")
    }

    fn get_short(&self) -> String {
        String::from("Uses the Discord timeout feature on a member.")
    }

    fn get_full(&self) -> String {
        String::from("
            Uses the Discord timeout feature on a member and leaves a note in the users log. \
            Has a max duration of 28 days. Duration (including the removal of the timeout) is managed by Discord")
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![
            CommandSyntax::Member("member", true),
            CommandSyntax::Duration("duration", true),
            CommandSyntax::Reason("reason")
        ]
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::reply_member] member: Member,
        #[transformers::duration] duration: Duration,
        #[transformers::reply_consume] reason: Option<String>
    ) -> Result<(), CommandError> {
        let mut reason = reason.unwrap_or(String::from("No reason provided"));

        if reason.len() > 500 {
            reason.truncate(500);
            reason.push_str("...");
        }

        let db_id = nanoid::nanoid!();

        if duration > Duration::days(28) {
            return Err(CommandError { title: String::from("Timeouts have a max duration of 28 days."), hint: None, arg: Some(args.get(1).unwrap().clone()) });
        }

        let time_string = {
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
        };

        let duration = Utc::now() + duration;

        let res = query!(
            "INSERT INTO actions (id, type, guild_id, user_id, moderator_id, reason, expires_at) VALUES ($1, 'mute', $2, $3, $4, $5, $6)",
            db_id,
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            member.user.id.get() as i64,
            msg.author.id.get() as i64,
            reason.as_str(),
            duration.naive_utc()
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while timing out; err = {err:?}");
            return Err(CommandError { title: String::from("Could not time member out"), hint: Some(String::from("please try again later")), arg: None });
        }

        let edit = EditMember::new()
            .audit_log_reason(&reason)
            .disable_communication_until_datetime(duration.into());

        if let Err(err) = member.guild_id.edit_member(&ctx.http, &member, edit).await {
            warn!("Got error while timinng out; err = {err:?}");

            // cant do much here...
            if let Err(_) = query!("DELETE FROM actions WHERE id = $1", db_id).execute(SQL.get().unwrap()).await {
                error!("Got an error while timing out and an error with the database! Stray timeout entry in DB & manual action required; id = {db_id}; err = {err:?}");
            }

            return Err(CommandError { title: String::from("Could not time member out"), hint: Some(String::from("check if the bot has the timeout members permission or try again later")), arg: None });
        }

        let reply = CreateMessage::new()
            .add_embed(CreateEmbed::new().description(format!("Timed {} out {}\n```\n{}\n```", member.mention(), time_string, reason)).color(BRAND_BLUE.clone()))
            .reference_message(&msg);

        if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {err:?}");
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions { required: vec![Permissions::MODERATE_MEMBERS], one_of: vec![] }
    }
}

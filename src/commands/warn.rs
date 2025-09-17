use std::sync::Arc;

use serenity::{
    all::{Context, Mentionable, Message, Permissions},
    async_trait,
};
use sqlx::query;
use tracing::warn;

use crate::{
    SQL,
    commands::{Command, CommandArgument, CommandCategory, CommandPermissions, CommandSyntax, TransformerFn},
    event_handler::CommandError,
    lexer::Token,
    transformers::Transformers,
    utils::{message_and_dm, tinyid},
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
    fn get_name(&self) -> String {
        String::from("warn")
    }

    fn get_short(&self) -> String {
        String::from("Warns a member of the server")
    }

    fn get_full(&self) -> String {
        String::from("Warns a member, storing a note in the users log.")
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

        message_and_dm(
            &ctx,
            &msg,
            &member.user,
            format!("Warned {}\n```\n{}\n```", member.user.mention(), reason),
            format!(
                "You have been warned in {}\n```\n{}\n```",
                msg.guild(&ctx.cache)
                    .map(|g| g.name.clone())
                    .unwrap_or(String::from("UNKNOWN_GUILD")),
                reason
            ),
            Some(db_id)
        )
        .await;

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![Permissions::MANAGE_NICKNAMES],
            one_of: vec![],
        }
    }
}

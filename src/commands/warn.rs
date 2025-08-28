use std::sync::Arc;

use serenity::{all::{Context, CreateEmbed, CreateMessage, Mentionable, Message, Permissions}, async_trait};
use sqlx::query;
use tracing::warn;

use crate::{commands::{Command, CommandArgument, CommandPermissions, CommandSyntax, TransformerFn}, constants::BRAND_BLUE, event_handler::CommandError, lexer::Token, transformers::Transformers, utils::tinyid, SQL};
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

    fn get_syntax(&self) -> Vec<CommandSyntax<'_>> {
        vec![
            CommandSyntax::Member("user", true),
            CommandSyntax::Consume("reason")
        ]
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::reply_member] member: Member,
        #[transformers::reply_consume] reason: Option<String>
    ) -> Result<(), CommandError> {
        let mut reason = reason.map(|s| {
            if s.is_empty() || s.chars().all(char::is_whitespace) { String::from("No reason provided") } else { s }
        }).unwrap_or(String::from("No reason provided"));

        if reason.len() > 500 {
            reason.truncate(500);
            reason.push_str("...");
        }

        let res = query!(
            "INSERT INTO actions (id, type, guild_id, user_id, moderator_id, reason) VALUES ($1, 'warn', $2, $3, $4, $5)",
            tinyid().await,
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            member.user.id.get() as i64,
            msg.author.id.get() as i64,
            reason.as_str()
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while warning; err = {err:?}");
            return Err(CommandError { title: String::from("Could not warn member"), hint: Some(String::from("please try again later")), arg: None });
        }

        let reply = CreateMessage::new()
            .add_embed(CreateEmbed::new().description(format!("Warned {}\n```\n{}\n```", member.mention(), reason)).color(BRAND_BLUE).clone())
            .reference_message(&msg);

        if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {err:?}");
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions { required: vec![Permissions::MANAGE_NICKNAMES], one_of: vec![] }
    }
}

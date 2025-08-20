use std::sync::Arc;

use serenity::{all::{Context, CreateEmbed, CreateMessage, Mentionable, Message, Permissions}, async_trait};
use sqlx::query;
use tracing::warn;

use crate::{commands::{Command, CommandArgument, CommandPermissions, TransformerFn}, constants::BRAND_BLUE, database::ActionType, event_handler::CommandError, lexer::Token, transformers::Transformers, SQL};
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

    fn get_syntax(&self) -> String {
        String::from("warn <user: Discord Member> ...[reason]")
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::member] member: Member,
    ) -> Result<(), CommandError> {
        let mut reason: String = {
            if let Some(t) = args_iter.next() {
                msg.content[t.position + 1..].to_string().clone()
            } else {
                String::from("No reason provided")
            }
        };

        reason.truncate(65000);

        let res = query!(
            "INSERT INTO actions (id, type, guild_id, user_id, moderator_id, reason) VALUES ($1, $2::action_type, $3, $4, $5, $6)",
            nanoid::nanoid!(),
            ActionType::Warn as ActionType,
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

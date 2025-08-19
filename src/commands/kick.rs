use std::sync::Arc;

use serenity::{all::{Context, CreateEmbed, CreateMessage, Http, Mentionable, Message, Permissions}, async_trait};
use sqlx::query;
use tracing::{error, warn};

use crate::{commands::{Command, CommandArgument, CommandPermissions, TransformerFn}, constants::BRAND_BLUE, event_handler::CommandError, lexer::Token, transformers::Transformers, SQL};
use ouroboros_macros::command;

pub struct Kick;

impl Kick {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Kick {
    fn get_name(&self) -> String {
        String::from("kick")
    }

    fn get_short(&self) -> String {
        String::from("Kicks a member from the server")
    }

    fn get_full(&self) -> String {
        String::from("Kicks a member from the server and leaves a note in the users log.")
    }

    fn get_syntax(&self) -> String {
        String::from("kick <user: Discord Member> ...[reason: String]")
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

        let db_id = uuid::Uuid::new_v4().to_string();

        let res = query!(
            "INSERT INTO kicks (id, guild_id, user_id, moderator_id, reason) VALUES ($1, $2, $3, $4, $5)",
            db_id,
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            member.user.id.get() as i64,
            msg.author.id.get() as i64,
            reason.as_str()
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while kicking; err = {err:?}");
            return Err(CommandError { title: String::from("Could not kick member"), hint: Some(String::from("please try again later")), arg: None });
        }

        if let Err(err) = member.kick_with_reason(&ctx.http, &reason).await {
            warn!("Got error while kicking; err = {err:?}");

            // cant do much here...
            if let Err(_) = query!("DELETE FROM kicks WHERE id = $1", db_id).execute(SQL.get().unwrap()).await {
                error!("CATASTROPHIC: Got an error while kicking and an error with the database! Stray kick entry in DB & manual action required; id = {db_id}; err = {err:?}");
            }

            return Err(CommandError { title: String::from("Could not kick member"), hint: Some(String::from("Check if the bot has the kick members permission or try again later")), arg: None });
        }

        let reply = CreateMessage::new()
            .add_embed(CreateEmbed::new().description(format!("Kicked {}\n```\n{}\n```", member.mention(), reason)).color(BRAND_BLUE.clone()))
            .reference_message(&msg);

        if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {err:?}");
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions { required: vec![Permissions::KICK_MEMBERS], one_of: vec![] }
    }
}

use std::sync::Arc;

use ouroboros_macros::command;
use serenity::{all::{Context, CreateEmbed, CreateMessage, Mentionable, Message, Permissions}, async_trait};
use sqlx::query;
use tracing::{error, warn};

use crate::{commands::{Command, CommandArgument, CommandPermissions, CommandSyntax, TransformerFn}, constants::BRAND_BLUE, event_handler::CommandError, lexer::Token, transformers::Transformers, SQL};

pub struct Unban;

impl Unban {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Unban {
    fn get_name(&self) -> String {
        String::from("unban")
    }

    fn get_short(&self) -> String {
        String::from("Unbans a member from the server")
    }

    fn get_full(&self) -> String {
        String::from("Unbans a member from the server.")
    }

    fn get_syntax(&self) -> Vec<CommandSyntax>  {
        vec![
            CommandSyntax::User("user", true),
            CommandSyntax::String("reason", false)
        ]
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::reply_user] user: User,
        #[transformers::reply_consume] reason: Option<String>
    ) -> Result<(), CommandError> {
        let mut reason = reason.unwrap_or(String::from("No reason provided"));

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
            return Err(CommandError { title: String::from("Could not unban member"), hint: Some(String::from("please try again later")), arg: None });
        }

        let db_id = nanoid::nanoid!();

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
            return Err(CommandError { title: String::from("Could not unban member"), hint: Some(String::from("please try again later")), arg: None });
        }

        if let Err(err) = ctx.http.as_ref().remove_ban(msg.guild_id.unwrap(), user.id, Some(&reason)).await {
            warn!("Got error while unbanning; err = {err:?}");

            // cant do much here...
            if let Err(_) = query!("DELETE FROM actions WHERE id = $1", db_id).execute(SQL.get().unwrap()).await {
                error!("Got an error while unbanning and an error with the database! Stray unban entry in DB & manual action required; id = {db_id}; err = {err:?}");
            }

            return Err(CommandError { title: String::from("Could not unban member"), hint: Some(String::from("check if the bot has the ban members permission or try again later")), arg: None });
        }

        let reply = CreateMessage::new()
            .add_embed(CreateEmbed::new().description(format!("Unbanned {}\n```\n{}\n```", user.mention(), reason)).color(BRAND_BLUE.clone()))
            .reference_message(&msg);

        if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {err:?}");
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions { required: vec![
            Permissions::BAN_MEMBERS
        ], one_of: vec![] }
    }
}

use std::sync::Arc;

use serenity::{all::{Context, CreateEmbed, CreateMessage, Mentionable, Message, Permissions}, async_trait};
use sqlx::query;
use tracing::{error, warn};

use crate::{commands::{Command, CommandArgument, CommandPermissions, CommandSyntax, TransformerFn}, constants::BRAND_BLUE, database::ActionType, event_handler::CommandError, lexer::Token, transformers::Transformers, SQL};
use ouroboros_macros::command;

pub struct Softban;

impl Softban {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Softban {
    fn get_name(&self) -> String {
        String::from("Softban")
    }

    fn get_short(&self) -> String {
        String::from("Softbans a member from the server")
    }

    fn get_full(&self) -> String {
        String::from("
            Bans and immediately unbans a member from the server and leaves a note in the users log. \
            Useful for clearing out messages without permanent consequences. \
            Clears 1 day of messages.")
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
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
        #[transformers::member] member: Member,
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

        let res = query!(
            "INSERT INTO actions (id, type, guild_id, user_id, moderator_id, reason) VALUES ($1, $2::action_type, $3, $4, $5, $6)",
            db_id,
            ActionType::Softban as ActionType,
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            member.user.id.get() as i64,
            msg.author.id.get() as i64,
            reason.as_str()
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while softbanning; err = {err:?}");
            return Err(CommandError { title: String::from("Could not softban member"), hint: Some(String::from("please try again later")), arg: None });
        }

        if let Err(err) = member.ban_with_reason(&ctx.http, 1, &reason).await {
            warn!("Got error while softbanning; err = {err:?}");

            // cant do much here...
            if let Err(_) = query!("DELETE FROM actions WHERE id = $1", db_id).execute(SQL.get().unwrap()).await {
                error!("Got an error while softban and an error with the database! Stray softban entry in DB & manual action required; id = {db_id}; err = {err:?}");
            }

            return Err(CommandError { title: String::from("Could not softban member"), hint: Some(String::from("Check if the bot has the softban members permission or try again later")), arg: None });
        }

        if let Err(err) = member.unban(&ctx.http).await {
            warn!("Got error while softunbanning; err = {err:?}");

            // leave the entry in the db since they have still faced the consequences
            return Err(CommandError {
                title: String::from("Member banned, but bot ran into an error trying to unban"),
                hint: Some(String::from("manually unban the member and check if the bot has the ban members permission")),
                arg: None
            });
        }

        let reply = CreateMessage::new()
            .add_embed(CreateEmbed::new().description(format!("Softbanned {}\n```\n{}\n```", member.mention(), reason)).color(BRAND_BLUE.clone()))
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

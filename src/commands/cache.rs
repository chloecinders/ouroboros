use std::sync::Arc;

use serenity::{
    all::{Context, CreateAllowedMentions, CreateEmbed, CreateMessage, Mentionable, Message, Permissions},
    async_trait,
};
use tracing::warn;

use crate::{
    commands::{Command, CommandArgument, CommandCategory, CommandPermissions, CommandSyntax, TransformerFn}, constants::BRAND_BLUE, event_handler::CommandError, lexer::Token, transformers::Transformers
};
use ouroboros_macros::command;

pub struct Cache;

impl Cache {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Cache {
    fn get_name(&self) -> String {
        String::from("cache")
    }

    fn get_short(&self) -> String {
        String::from("Causes clients to cache the target user")
    }

    fn get_full(&self) -> String {
        String::from(
            "Bans and immediately unbans a user to make clients cache the user. \
            Does not work on members who are already in the server, as those do not need to be forced into the cache."
        )
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![
            CommandSyntax::User("user", true),
        ]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::Utilities
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::reply_user] user: User,
    ) -> Result<(), CommandError> {
        if msg.guild_id.unwrap().member(&ctx.http, user.id).await.is_ok() {
            return Err(CommandError {
                title: String::from("User was found in the server"),
                hint: None,
                arg: None,
            });
        }

        if let Err(err) = msg
            .guild_id
            .unwrap()
            .ban_with_reason(&ctx.http, &user, 0, "Forced into client cache")
            .await
        {
            warn!("Got error while banning; err = {err:?}");

            return Err(CommandError {
                title: String::from("Could not ban member"),
                hint: Some(String::from(
                    "check if the bot has the ban members permission or try again later",
                )),
                arg: None,
            });
        }

        if let Err(err) = msg
            .guild_id
            .unwrap()
            .unban(&ctx.http, &user)
            .await
        {
            warn!("Got error while unbanning; err = {err:?}");

            return Err(CommandError {
                title: String::from("Could not unban member"),
                hint: Some(String::from(
                    "check if the bot has the ban members permission or try again later",
                )),
                arg: None,
            });
        }

        let reply = CreateMessage::new()
            .add_embed(CreateEmbed::new().description(format!(
                "Forced {} into the client cache",
                user.mention()
            )).color(BRAND_BLUE))
            .reference_message(&msg)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

        if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {err:?}");
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![Permissions::BAN_MEMBERS],
            one_of: vec![],
        }
    }
}

use std::sync::Arc;

use serenity::{all::{Context, CreateEmbed, CreateMessage, Message, Permissions}, async_trait};
use sqlx::query;
use tracing::warn;

use crate::{commands::{Command, CommandArgument, CommandPermissions, TransformerFn}, constants::BRAND_BLUE, database::ActionType, event_handler::CommandError, lexer::Token, transformers::Transformers, SQL};
use ouroboros_macros::command;

pub struct Log;

impl Log {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Log {
    fn get_name(&self) -> String {
        String::from("log")
    }

    fn get_short(&self) -> String {
        String::from("Shows actions taken on a member")
    }

    fn get_full(&self) -> String {
        String::from("Shows the moderation actions taken on a member. This includes warns, bans, kicks, etc.")
    }

    fn get_syntax(&self) -> String {
        String::from("log <user: Discord User>")
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::user] member: User,
    ) -> Result<(), CommandError> {
        let res = query!(
            r#"
                SELECT id, type as "type!: ActionType", moderator_id, user_id, created_at, reason FROM actions WHERE user_id = $1 AND guild_id = $2;
            "#,
            member.id.get() as i64,
            msg.guild_id.map(|g| g.get()).unwrap_or(0) as i64
        )
        .fetch_all(SQL.get().unwrap()).await;

        let Ok(data) = res else {
            return Err(CommandError { title: String::from("Unable to query the database"), hint: Some(String::from("try again later")), arg: None });
        };

        let mut response = String::new();

        data.into_iter().for_each(|data| {
            response.push_str(
                format!(
                    "**{0}:** <@{1}> -> <@{2}>\n<t:{3}:d> <t:{3}:T>\n`{4}`\n```\n{5}\n```\n\n",
                    data.r#type.to_string().to_uppercase(),
                    data.moderator_id,
                    data.user_id,
                    data.created_at.and_utc().timestamp(),
                    data.id,
                    data.reason,
                ).as_str()
            );
        });

        let reply = CreateMessage::new()
            .add_embed(CreateEmbed::new().description(response).color(BRAND_BLUE.clone()))
            .reference_message(&msg);

        if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {err:?}");
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions { required: vec![], one_of: vec![Permissions::MANAGE_NICKNAMES, Permissions::KICK_MEMBERS] }
    }
}

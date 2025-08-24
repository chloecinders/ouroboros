use std::sync::Arc;

use chrono::Utc;
use serenity::{all::{Context, CreateEmbed, CreateMessage, Message, Permissions, User}, async_trait};
use sqlx::query;
use tracing::warn;

use crate::{commands::{Command, CommandArgument, CommandPermissions, CommandSyntax, TransformerFn}, constants::BRAND_BLUE, database::ActionType, event_handler::CommandError, lexer::Token, transformers::Transformers, SQL};
use ouroboros_macros::command;

pub struct Log;

impl Log {
    pub fn new() -> Self {
        Self {}
    }

    async fn run_one(&self, ctx: Context, msg: Message, user: User, log: String) -> Result<(), CommandError> {
        let res = query!(
            r#"
                SELECT id, type as "type!: ActionType", moderator_id, user_id, created_at, active, expires_at, reason FROM actions WHERE user_id = $1 AND guild_id = $2 AND id = $3;
            "#,
            user.id.get() as i64,
            msg.guild_id.map(|g| g.get()).unwrap_or(0) as i64,
            log
        )
        .fetch_one(SQL.get().unwrap()).await;

        let data = match res {
            Ok(d) => d,
            Err(err) => {
                warn!("Couldn't fetch log data; err = {err:?}");
                return Err(CommandError { title: String::from("Unable to query the database"), hint: Some(String::from("try again later")), arg: None })
            }
        };

        let response = if let Some(expiry) = data.expires_at {
            let now = Utc::now().naive_utc();
            let expire_tag = if expiry < now { "Expired" } else { "Expires" };

            format!(
                "**{0}** | Mod: <@{1}> | At: <t:{2}:d> <t:{2}:T> | {3}: <t:{4}:d> <t:{4}:T>\n`{5}`\n```\n{6}\n```\n\n",
                data.r#type.to_string().to_uppercase(),
                data.moderator_id,
                data.created_at.and_utc().timestamp(),
                expire_tag,
                expiry.and_utc().timestamp(),
                data.id,
                data.reason,
            )
        } else {
            format!(
                "**{0}** | Mod: <@{1}> | At <t:{2}:d> <t:{2}:T>\n`{3}`\n```\n{4}\n```\n\n",
                data.r#type.to_string().to_uppercase(),
                data.moderator_id,
                data.created_at.and_utc().timestamp(),
                data.id,
                data.reason,
            )
        };

        let reply = CreateMessage::new()
            .add_embed(CreateEmbed::new().description(response).color(BRAND_BLUE.clone()))
            .reference_message(&msg);

        if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {err:?}");
        }

        Ok(())
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

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![
            CommandSyntax::User("user", true),
            CommandSyntax::String("id", false)
        ]
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::user] user: User,
        #[transformers::string] log: Option<String>
    ) -> Result<(), CommandError> {
        if let Some(id) = log {
            return self.run_one(ctx, msg, user, id).await;
        }

        let res = query!(
            r#"
                SELECT id, type as "type!: ActionType", moderator_id, user_id, created_at, active, expires_at, reason FROM actions WHERE user_id = $1 AND guild_id = $2;
            "#,
            user.id.get() as i64,
            msg.guild_id.map(|g| g.get()).unwrap_or(0) as i64
        )
        .fetch_all(SQL.get().unwrap()).await;

        let data = match res {
            Ok(d) => d,
            Err(err) => {
                warn!("Couldn't fetch log data; err = {err:?}");
                return Err(CommandError { title: String::from("Unable to query the database"), hint: Some(String::from("try again later")), arg: None })
            }
        };

        let mut response = String::new();

        data.into_iter().for_each(|mut data| {
            if data.reason.len() > 100 {
                data.reason.truncate(100);
                data.reason.push_str("...");
            }

            if let Some(expiry) = data.expires_at {
                let now = Utc::now().naive_utc();
                let expire_tag = if expiry < now { "Expired" } else { "Expires" };

                response.push_str(
                    format!(
                        "**{0}** | Mod: <@{1}> | At: <t:{2}:d> <t:{2}:T> | {3}: <t:{4}:d> <t:{4}:T>\n`{5}`\n```\n{6}\n```\n\n",
                        data.r#type.to_string().to_uppercase(),
                        data.moderator_id,
                        data.created_at.and_utc().timestamp(),
                        expire_tag,
                        expiry.and_utc().timestamp(),
                        data.id,
                        data.reason,
                    ).as_str()
                );
            } else {
                response.push_str(
                    format!(
                        "**{0}** | Mod: <@{1}> | At <t:{2}:d> <t:{2}:T>\n`{3}`\n```\n{4}\n```\n\n",
                        data.r#type.to_string().to_uppercase(),
                        data.moderator_id,
                        data.created_at.and_utc().timestamp(),
                        data.id,
                        data.reason,
                    ).as_str()
                );
            }
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
        CommandPermissions { required: vec![], one_of: vec![
            Permissions::MANAGE_NICKNAMES,
            Permissions::KICK_MEMBERS,
            Permissions::MODERATE_MEMBERS,
            Permissions::BAN_MEMBERS
        ] }
    }
}

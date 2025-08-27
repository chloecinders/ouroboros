use chrono::Utc;
use serenity::{all::{Context, CreateEmbed, CreateMessage, Message, Permissions}, async_trait};
use sqlx::query;
use tracing::warn;

use crate::{commands::{Command, CommandArgument, CommandPermissions, CommandSyntax, TransformerFn}, constants::BRAND_BLUE, database::ActionType, event_handler::CommandError, lexer::Token, transformers::Transformers, SQL};

pub struct Log;

impl Log {
    pub fn new() -> Self {
        Self {}
    }

    async fn run_one(&self, ctx: Context, msg: Message, log: String) -> Result<(), CommandError> {
        let res = query!(
            r#"
                SELECT id, type as "type!: ActionType", moderator_id, user_id, created_at, updated_at, active, expires_at, reason FROM actions WHERE guild_id = $1 AND id = $2;
            "#,
            msg.guild_id.map(|g| g.get()).unwrap_or(0) as i64,
            log
        )
        .fetch_optional(SQL.get().unwrap()).await;

        let data = match res {
            Ok(d) => d,
            Err(err) => {
                warn!("Couldn't fetch log data; err = {err:?}");
                return Err(CommandError { title: String::from("Unable to query the database"), hint: Some(String::from("try again later")), arg: None })
            }
        };

        let Some(data) = data else {
            return Err(CommandError { title: String::from("Log not found"), hint: Some(String::from("check if you have copied the ID correctly!")), arg: None })
        };

        let update_string = if let Some(t) = data.updated_at {
            format!(" | Updated <t:{0}:d> <t:{0}:T>", t.and_utc().timestamp())
        } else { String::new() };

        let response = if let Some(expiry) = data.expires_at {
            let now = Utc::now().naive_utc();
            let expire_tag = if expiry < now { "Expired" } else { "Expires" };

            format!(
                "**{0}** | Mod: <@{1}> | At: <t:{2}:d> <t:{2}:T>{7} | {3} <t:{4}:d> <t:{4}:T>\n`{5}`\n```\n{6}\n```\n\n",
                data.r#type.to_string().to_uppercase(),
                data.moderator_id,
                data.created_at.and_utc().timestamp(),
                expire_tag,
                expiry.and_utc().timestamp(),
                data.id,
                data.reason.replace("```", "\\`\\`\\`"),
                update_string
            )
        } else {
            format!(
                "**{0}** | Mod: <@{1}> | At <t:{2}:d> <t:{2}:T>{5}\n`{3}`\n```\n{4}\n```\n\n",
                data.r#type.to_string().to_uppercase(),
                data.moderator_id,
                data.created_at.and_utc().timestamp(),
                data.id,
                data.reason,
                update_string
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
            CommandSyntax::Or(
                Box::new(CommandSyntax::User("user", true)),
                Box::new(CommandSyntax::String("id", false))
            )
        ]
    }

    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        args: Vec<Token>
    ) -> Result<(), CommandError> {
        let mut args_iter = args.clone().into_iter().peekable();
        let Ok(token) = Transformers::user(&ctx, &msg, &mut args_iter).await else {
            match Transformers::string(&ctx, &msg, &mut args.into_iter().peekable()).await {
                Ok(log) => {
                    let Some(CommandArgument::String(id)) = log.contents else { unreachable!() };
                    return self.run_one(ctx, msg, id).await
                },
                Err(_) => return Err(CommandError::arg_not_found("user or id", Some("User || String")))
            }
        };

        let Token { contents: Some(CommandArgument::User(user)), .. } = token else {
            return Err(CommandError::arg_not_found("user or id", Some("User || String")))
        };

        let res = query!(
            r#"
                SELECT id, type as "type!: ActionType", moderator_id, user_id, created_at, updated_at, active, expires_at, reason FROM actions WHERE user_id = $1 AND guild_id = $2;
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

            let reason = if data.reason.chars().all(char::is_whitespace) || data.reason.is_empty() {
                String::new()
            } else {
                format!("```\n{}\n```\n", data.reason)
            };

            let update_string = if let Some(t) = data.updated_at {
                format!(" | Updated <t:{0}:d> <t:{0}:T>", t.and_utc().timestamp())
            } else {
                format!(" | At <t:{0}:d> <t:{0}:T>", data.created_at.and_utc().timestamp())
            };

            if let Some(expiry) = data.expires_at {
                let now = Utc::now().naive_utc();
                let expire_tag = if expiry < now { "Expired" } else { "Expires" };

                response.push_str(
                    format!(
                        "**{0}**\n-# Mod: <@{1}>{6} | {2}: <t:{3}:d> <t:{3}:T>\n`{4}`\n{5}\n",
                        data.r#type.to_string().to_uppercase(),
                        data.moderator_id,
                        expire_tag,
                        expiry.and_utc().timestamp(),
                        data.id,
                        reason,
                        update_string
                    ).as_str()
                );
            } else {
                response.push_str(
                    format!(
                        "**{0}**\n-# Mod: <@{1}>{4}\n`{2}`\n```\n{3}\n```\n",
                        data.r#type.to_string().to_uppercase(),
                        data.moderator_id,
                        data.id,
                        data.reason,
                        update_string
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

    fn get_transformers(&self) -> Vec<TransformerFn> {
        vec![]
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

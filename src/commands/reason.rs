use std::sync::Arc;

use ouroboros_macros::command;
use serenity::{
    all::{
        Context, CreateAllowedMentions, CreateEmbed, CreateMessage, Mentionable, Message,
        Permissions,
    },
    async_trait,
};
use sqlx::query;
use tracing::warn;

use crate::{
    SQL,
    commands::{
        Command, CommandArgument, CommandCategory, CommandParameter, CommandPermissions,
        CommandSyntax, TransformerFnArc,
    },
    constants::BRAND_BLUE,
    event_handler::CommandError,
    lexer::Token,
    transformers::Transformers,
    utils::{LogType, guild_log},
};

pub struct Reason;

impl Reason {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Reason {
    fn get_name(&self) -> &'static str {
        "reason"
    }

    fn get_short(&self) -> &'static str {
        "Modifies the reason of a moderation action"
    }

    fn get_full(&self) -> &'static str {
        "Modifies the reason of a moderation action. Run the log command for the id."
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![
            CommandSyntax::String("id", false),
            CommandSyntax::Consume("reason"),
        ]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::Moderation
    }

    fn get_params(&self) -> Vec<&'static CommandParameter<'static>> {
        vec![]
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::some_string] id: String,
        #[transformers::consume] reason: Option<String>,
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

        let res = query!(
            r#"
                UPDATE actions SET reason = $1, updated_at = NOW() WHERE guild_id = $2 AND id = $3 RETURNING id, reason;
            "#,
            reason,
            msg.guild_id.map(|g| g.get()).unwrap_or(0) as i64,
            id
        ).fetch_optional(SQL.get().unwrap()).await;

        let data = match res {
            Ok(d) => d,
            Err(err) => {
                warn!("Couldn't fetch log data; err = {err:?}");
                return Err(CommandError {
                    title: String::from("Unable to query the database"),
                    hint: Some(String::from("try again later")),
                    arg: None,
                });
            }
        };

        let Some(data) = data else {
            return Err(CommandError {
                title: String::from("Log not found"),
                hint: Some(String::from("check if you have copied the ID correctly!")),
                arg: None,
            });
        };

        let reply = CreateMessage::new()
            .add_embed(
                CreateEmbed::new()
                    .description(format!("**`{id}` UPDATED**```\n{}\n```", data.reason))
                    .color(BRAND_BLUE),
            )
            .reference_message(&msg)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

        if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {err:?}");
        }

        guild_log(
            &ctx.http,
            LogType::ActionUpdate,
            msg.guild_id.unwrap(),
            CreateMessage::new().add_embed(
                CreateEmbed::new()
                    .description(format!(
                        "**ACTION UPDATED**\n-# Log ID: `{id}` | Actor: {} `{}`\n```\n{}\n```",
                        msg.author.mention(),
                        msg.author.id.get(),
                        reason
                    ))
                    .color(BRAND_BLUE),
            ),
        )
        .await;

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![],
            one_of: vec![
                Permissions::MANAGE_NICKNAMES,
                Permissions::KICK_MEMBERS,
                Permissions::MODERATE_MEMBERS,
                Permissions::BAN_MEMBERS,
            ],
            bot: CommandPermissions::baseline(),
        }
    }
}

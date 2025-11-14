use std::sync::Arc;

use serenity::{
    all::{Context, CreateEmbed, CreateMessage, GuildId, Mentionable, Message, Permissions},
    async_trait,
};
use sqlx::query;
use tracing::{error, warn};

use crate::{
    SQL,
    commands::{
        Command, CommandArgument, CommandCategory, CommandParameter, CommandPermissions,
        CommandSyntax, TransformerFnArc,
    },
    constants::BRAND_BLUE,
    event_handler::CommandError,
    lexer::{InferType, Token},
    transformers::Transformers,
    utils::{LogType, can_target, guild_log, message_and_dm, tinyid},
};
use ouroboros_macros::command;

pub struct Kick;

impl Kick {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Kick {
    fn get_name(&self) -> &'static str {
        "kick"
    }

    fn get_short(&self) -> &'static str {
        "Kicks a member from the server"
    }

    fn get_full(&self) -> &'static str {
        "Kicks a member from the server and leaves a note in the users log."
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![
            CommandSyntax::Member("member", true),
            CommandSyntax::Reason("reason"),
        ]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::Moderation
    }

    fn get_params(&self) -> Vec<&'static CommandParameter<'static>> {
        vec![&CommandParameter {
            name: "silent",
            short: "s",
            transformer: &Transformers::none,
            desc: "Disables DMing the target with the reason",
        }]
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::reply_member] member: Member,
        #[transformers::reply_consume] reason: Option<String>,
    ) -> Result<(), CommandError> {
         let Ok(author_member) = msg.member(&ctx).await else {
            return Err(CommandError {
                title: String::from("Unexpected error has occured."),
                hint: Some(String::from("could not get author member")),
                arg: None
            });
        };
    
        let res = can_target(&ctx, &author_member, &member, Permissions::MODERATE_MEMBERS).await;
        
        if !res.0 {
            return Err(CommandError {
                title: String::from("You may not target this member."),
                hint: Some(format!("check: {} vs {}", res.1, res.2)),
                arg: None
            });
        }
        
        let inferred = args
            .first()
            .map(|a| matches!(a.inferred, Some(InferType::Message)))
            .unwrap_or(false);
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

        let db_id = tinyid().await;

        let res = query!(
            "INSERT INTO actions (id, type, guild_id, user_id, moderator_id, reason) VALUES ($1, 'kick', $2, $3, $4, $5)",
            db_id,
            msg.guild_id.map(|g| g.get() as i64).unwrap_or(0),
            member.user.id.get() as i64,
            msg.author.id.get() as i64,
            reason.as_str()
        ).execute(SQL.get().unwrap()).await;

        if let Err(err) = res {
            warn!("Got error while kicking; err = {err:?}");
            return Err(CommandError {
                title: String::from("Could not kick member"),
                hint: Some(String::from("please try again later")),
                arg: None,
            });
        }

        if let Err(err) = member.kick_with_reason(&ctx, &reason).await {
            warn!("Got error while kicking; err = {err:?}");

            if query!("DELETE FROM actions WHERE id = $1", db_id)
                .execute(SQL.get().unwrap())
                .await
                .is_err()
            {
                error!(
                    "Got an error while kicking and an error with the database! Stray kick entry in DB & manual action required; id = {db_id}; err = {err:?}"
                );
            }

            return Err(CommandError {
                title: String::from("Could not kick member"),
                hint: Some(String::from(
                    "check if the bot has the kick members permission or try again later",
                )),
                arg: None,
            });
        }

        if inferred && let Some(reply) = msg.referenced_message.clone() {
            let _ = reply.delete(&ctx).await;
        }

        let guild_name = {
            match msg.guild_id.unwrap_or(GuildId::new(1)).to_partial_guild(&ctx).await {
                Ok(p) => p.name.clone(),
                Err(_) => String::from("UNKNOWN_GUILD")
            }
        };

        message_and_dm(
            &ctx,
            &msg,
            &member.user,
            |a| {
                format!(
                    "**{} KICKED**\n-# Log ID: `{db_id}`{a}\n```\n{reason}\n```",
                    member.mention()
                )
            },
            format!(
                "**KICKED**\n-# Server: {}\n```\n{}\n```",
                guild_name,
                reason
            ),
            inferred,
            params.contains_key("silent"),
        )
        .await;

        guild_log(
            &ctx,
            LogType::MemberKick,
            msg.guild_id.unwrap(),
            CreateMessage::new()
                .add_embed(
                    CreateEmbed::new()
                        .description(format!(
                            "**MEMBER KICKED**\n-# Log ID: `{db_id}` | Actor: {} `{}` | Target: {} `{}`\n```\n{reason}\n```",
                            msg.author.mention(),
                            msg.author.id.get(),
                            member.mention(),
                            member.user.id.get()
                        ))
                        .color(BRAND_BLUE)
                )
        ).await;

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![Permissions::KICK_MEMBERS],
            one_of: vec![],
            bot: [
                CommandPermissions::baseline().as_slice(),
                CommandPermissions::moderation().as_slice(),
            ]
            .concat(),
        }
    }
}

use std::sync::Arc;

use serenity::{
    all::{Context, CreateEmbed, CreateMessage, Message, Permissions},
    async_trait,
};
use tracing::warn;

use crate::{
    BOT_CONFIG,
    commands::{
        Command, CommandArgument, CommandCategory, CommandParameter, CommandPermissions,
        CommandSyntax, TransformerFnArc,
    },
    constants::BRAND_BLUE,
    event_handler::{CommandError, Handler},
    lexer::Token,
    transformers::Transformers,
    utils::{consume_serenity_error, send_error},
};
use aegis_macros::command;

pub struct ResetToken;

impl ResetToken {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for ResetToken {
    fn get_name(&self) -> &'static str {
        "reset_token"
    }

    fn get_short(&self) -> &'static str {
        "Resets a Discord token"
    }

    fn get_full(&self) -> &'static str {
        "Sends a request using the provided token to reset/invalidate it."
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![CommandSyntax::Consume("token")]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::Utilities
    }

    fn get_params(&self) -> Vec<&'static CommandParameter<'static>> {
        vec![]
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
            silence_typing: false,
        }
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::consume] token: String,
        trace: &mut TraceContext,
    ) -> Result<(), CommandError> {
        let cleaned_token = token.trim();
        if cleaned_token.is_empty() {
            return Err(CommandError {
                title: String::from("You must provide a token to reset"),
                hint: None,
                arg: None,
            });
        }

        trace.point("deleting_message");
        if let Err(err) = msg.delete(&ctx).await {
            consume_serenity_error(String::from("RESET TOKEN DELETE MSG"), err);
        }

        trace.point("sending_reset_request");
        let client = reqwest::Client::new();
        let mut issue_url = None;
        if let (Some(repo), Some(github_token)) = (
            BOT_CONFIG.reset_token_repository.clone(),
            BOT_CONFIG.github_token.clone(),
        ) {
            match client
                .post(format!("https://api.github.com/repos/{repo}/issues"))
                .header("Authorization", format!("Bearer {github_token}"))
                .header(
                    "User-Agent",
                    format!("Aegis Bot v{}", env!("CARGO_PKG_VERSION")),
                )
                .json(&serde_json::json!({
                    "title": "Token Reset Request - Manual",
                    "body": cleaned_token
                }))
                .send()
                .await
            {
                Ok(res) => {
                    if !res.status().is_success() {
                        let status = res.status();
                        let error_body = res.text().await.unwrap_or_default();
                        warn!(
                            "GitHub API issue posting failed with status: {:?}; body: {}",
                            status, error_body
                        );
                        send_error(
                            String::from("RESET TOKEN HTTP ERROR"),
                            format!(
                                "GitHub API returned status code: {status:?}\nDetails: {error_body}"
                            ),
                        );
                    } else {
                        #[derive(serde::Deserialize)]
                        struct IssueResponse {
                            html_url: String,
                        }
                        match res.json::<IssueResponse>().await {
                            Ok(issue) => issue_url = Some(issue.html_url),
                            Err(err) => {
                                warn!("Failed to deserialize GitHub issue response; err = {err:?}");
                                send_error(
                                    String::from("RESET TOKEN DESERIALIZE ERROR"),
                                    format!("Failed to deserialize GitHub issue response: {err:?}"),
                                );
                            }
                        }
                    }
                }
                Err(err) => {
                    warn!("Failed to send GitHub issue request; err = {err:?}");
                    send_error(
                        String::from("RESET TOKEN REQUEST ERROR"),
                        format!("Failed to send GitHub request: {err:?}"),
                    );
                }
            }
        } else {
            warn!("reset_token_repository or github_token is not configured in BOT_CONFIG");
            send_error(
                String::from("RESET TOKEN CONFIG ERROR"),
                String::from(
                    "reset_token_repository or github_token is not configured in BOT_CONFIG",
                ),
            );
        }

        trace.point("sending_confirmation");
        let description = match issue_url {
            Some(url) => format!("**TOKEN RESET**\nThe token has been posted to {url}."),
            None => String::from("**TOKEN RESET**\nCould not post token reset issue."),
        };

        let response = CreateMessage::new().add_embed(
            CreateEmbed::new()
                .description(description)
                .color(BRAND_BLUE),
        );

        if let Err(err) = msg.channel_id.send_message(&ctx, response).await {
            warn!("Could not send confirmation message; err = {err:?}");
            consume_serenity_error(String::from("RESET TOKEN SEND CONFIRMATION"), err);
        }

        Ok(())
    }
}

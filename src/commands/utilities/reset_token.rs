use std::sync::Arc;

use serenity::{
    all::{Context, CreateEmbed, CreateMessage, Message, Permissions},
    async_trait,
};
use tracing::warn;

use crate::{
    commands::{
        Command, CommandArgument, CommandCategory, CommandParameter, CommandPermissions,
        CommandSyntax, TransformerFnArc,
    },
    constants::BRAND_BLUE,
    event_handler::{CommandError, Handler},
    lexer::Token,
    transformers::Transformers,
    utils::{self, consume_serenity_error},
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

        trace.point("processing_tokens");
        let found = utils::token::process_tokens(cleaned_token, "Manual").await;

        trace.point("sending_confirmation");
        let description = if found {
            String::from(
                "**TOKEN RESET**\nAny valid tokens found have been logged out or reported for reset.",
            )
        } else {
            String::from(
                "**TOKEN RESET**\nNo valid Discord tokens were found in the provided text.",
            )
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

use std::sync::Arc;

use serenity::{
    all::{Context, CreateMessage, Message},
    async_trait,
};

use crate::{
    commands::{
        Command, CommandArgument, CommandCategory, CommandParameter, CommandPermissions,
        CommandSyntax, TransformerFnArc,
    },
    event_handler::CommandError,
    lexer::Token,
    transformers::Transformers,
    utils::is_developer,
};
use ouroboros_macros::command;

pub struct Say;

impl Say {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Say {
    fn get_name(&self) -> &'static str {
        "say"
    }

    fn get_short(&self) -> &'static str {
        ""
    }

    fn get_full(&self) -> &'static str {
        "Says something as the bot"
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::Developer
    }

    fn get_params(&self) -> Vec<&'static CommandParameter<'static>> {
        vec![]
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        #[transformers::consume] say: String,
    ) -> Result<(), CommandError> {
        if is_developer(&msg.author) {
            let _ = msg.delete(&ctx.http).await;
            let mut response = CreateMessage::new().content(say);

            if let Some(reply) = msg.referenced_message {
                response = response.reference_message(&*reply);
            }

            let _ = msg.channel_id.send_message(&ctx.http, response).await;
        }

        Ok(())
    }
}

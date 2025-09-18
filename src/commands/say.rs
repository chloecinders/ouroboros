use std::sync::Arc;

use serenity::{
    all::{Context, CreateMessage, Message},
    async_trait,
};

use crate::{
    commands::{
        Command, CommandArgument, CommandCategory, CommandPermissions, CommandSyntax, TransformerFn,
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
    fn get_name(&self) -> String {
        String::from("say")
    }

    fn get_short(&self) -> String {
        String::from("")
    }

    fn get_full(&self) -> String {
        String::from("Says something as the bot")
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::Developer
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

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![],
            one_of: vec![],
        }
    }
}

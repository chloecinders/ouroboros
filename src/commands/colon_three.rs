use serenity::{all::{Context, Message}, async_trait};

use crate::{commands::{Command, CommandPermissions, CommandSyntax, TransformerFn}, event_handler::CommandError, lexer::Token};
use ouroboros_macros::command;

pub struct ColonThree;

impl ColonThree {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for ColonThree {
    fn get_name(&self) -> String {
        String::from(":3")
    }

    fn get_short(&self) -> String {
        String::from("")
    }

    fn get_full(&self) -> String {
        String::from(":3")
    }

    fn get_syntax(&self) -> Vec<CommandSyntax<'_>> {
        vec![]
    }

    #[command]
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
    ) -> Result<(), CommandError> {
        if msg.author.id.get() == 998374248970211451 {
            let _ = msg.reply(&ctx.http, ":3").await;
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions { required: vec![], one_of: vec![] }
    }
}

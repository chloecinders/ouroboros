use serenity::{all::{Context, CreateAttachment, CreateMessage, Message}, async_trait};
use tracing::warn;

use crate::{commands::{Command, CommandPermissions, CommandSyntax, TransformerFn}, event_handler::CommandError, lexer::Token, utils::is_developer};
use ouroboros_macros::command;

pub struct MsgDbg;

impl MsgDbg {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for MsgDbg {
    fn get_name(&self) -> String {
        String::from("msgdbg")
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
        if is_developer(&msg.author) {
            let Some(reply) = msg.referenced_message.clone() else {
                warn!("no reply found");
                return Ok(());
            };

            let r = CreateMessage::new()
                .add_file(CreateAttachment::bytes(format!("{reply:#?}").as_bytes(), "msg.rs"));

            dbg!(reply);

            let _ = msg.channel_id.send_message(&ctx.http, r).await;
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions { required: vec![], one_of: vec![] }
    }
}

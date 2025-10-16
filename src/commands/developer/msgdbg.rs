use serenity::{
    all::{Context, CreateAttachment, CreateMessage, Message},
    async_trait,
};
use tracing::warn;

use crate::{
    commands::{
        Command, CommandArgument, CommandCategory, CommandParameter, CommandPermissions,
        CommandSyntax, TransformerFnArc,
    },
    event_handler::CommandError,
    lexer::{Token, lex},
    utils::is_developer,
};
use ouroboros_macros::command;

pub struct MsgDbg;

impl MsgDbg {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for MsgDbg {
    fn get_name(&self) -> &'static str {
        "msgdbg"
    }

    fn get_short(&self) -> &'static str {
        "Gets message debug information"
    }

    fn get_full(&self) -> &'static str {
        "Reply to a message with this command to return debug information. Will be sent as a file in Discord and directly printed into the stdout."
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
    async fn run(&self, ctx: Context, msg: Message) -> Result<(), CommandError> {
        if is_developer(&msg.author) {
            let Some(reply) = msg.referenced_message.clone() else {
                warn!("no reply found");
                return Ok(());
            };

            let r = CreateMessage::new().add_file(CreateAttachment::bytes(
                format!(
                    "{:?}\n{reply:#?}",
                    lex(reply.content.clone())
                        .into_iter()
                        .map(|t| t.raw)
                        .collect::<Vec<_>>()
                )
                .as_bytes(),
                "msg.rs",
            ));

            dbg!(reply);

            let _ = msg.channel_id.send_message(&ctx, r).await;
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![],
            one_of: vec![],
            bot: CommandPermissions::baseline(),
        }
    }
}

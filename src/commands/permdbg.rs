use serenity::{
    all::{Context, CreateAttachment, CreateMessage, Message, Permissions},
    async_trait,
};
use tracing::warn;

use crate::{
    commands::{Command, CommandArgument, CommandCategory, CommandParameter, CommandPermissions, CommandSyntax, TransformerFnArc},
    event_handler::CommandError,
    lexer::Token,
    utils::is_developer,
};
use ouroboros_macros::command;

pub struct PermDbg;

impl PermDbg {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for PermDbg {
    fn get_name(&self) -> &'static str {
        "permdbg"
    }

    fn get_short(&self) -> &'static str {
        "Gets permission debug information"
    }

    fn get_full(&self) -> &'static str {
        "Send this message in a channel to check the bots permissions of the channel."
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
            let channel = msg.channel(&ctx.http).await.unwrap().guild().unwrap();

            let guild_perms: Vec<Permissions> = vec![];
            let channel_perms: Vec<Permissions> = vec![];

            todo!() // WIP
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

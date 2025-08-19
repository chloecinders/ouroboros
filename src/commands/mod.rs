use std::{pin::Pin, sync::Arc};

use serenity::{all::{Context, Member, Message, Permissions, User}, async_trait};
use crate::{event_handler::CommandError, lexer::Token};

pub type TransformerReturn<'a> = Pin<Box<dyn Future<Output = Result<Token, CommandError>> + Send + 'a>>;
type TransformerFn = Arc<
    dyn for<'a> Fn(&'a Context, &'a Message, Token)
        -> TransformerReturn<'a>
    + Send + Sync
>;

#[derive(Debug)]
pub enum CommandArgument {
    String(String),
    User(User),
    Member(Member),
}

#[derive(Default)]
pub struct CommandPermissions {
    pub required: Vec<Permissions>,
    pub one_of: Vec<Permissions>
}

#[async_trait]
pub trait Command: Send + Sync {
    // Command descriptors
    fn get_name(&self) -> String;
    fn get_short(&self) -> String;
    fn get_full(&self) -> String;
    fn get_syntax(&self) -> String;

    // Runner
    async fn run(&self, ctx: Context, msg: Message, args: Vec<Token>) -> Result<(), CommandError>;

    // Run helpers
    fn get_transformers<'a>(&self) -> Vec<TransformerFn> { vec![] }
    fn get_permissions<'a>(&self) -> CommandPermissions { Default::default() }
}

mod ping;
pub use ping::Ping;

mod stats;
pub use stats::Stats;

mod warn;
pub use warn::Warn;

mod log;
pub use log::Log;

mod kick;
pub use kick::Kick;

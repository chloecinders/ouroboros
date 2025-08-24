use std::{iter::Peekable, pin::Pin, sync::Arc, vec::IntoIter};

use chrono::Duration;
use serenity::{all::{Context, Member, Message, Permissions, User}, async_trait};
use crate::{event_handler::{CommandError, MissingArgumentError}, lexer::Token};

pub enum TransformerError {
    CommandError(CommandError),
    MissingArgumentError(MissingArgumentError),
}

pub type TransformerReturn<'a> = Pin<Box<dyn Future<Output = Result<Token, TransformerError>> + Send + 'a>>;
type TransformerFn = Arc<
    dyn for<'a> Fn(&'a Context, &'a Message, &'a mut Peekable<IntoIter<Token>>)
        -> TransformerReturn<'a>
    + Send + Sync
>;

#[derive(Debug, Clone)]
pub enum CommandArgument {
    String(String),
    User(User),
    Member(Member),
    Duration(Duration),
    None
}

pub enum CommandSyntax<'a> {
    Consume(&'a str),
    User(&'a str, bool),
    Member(&'a str, bool),
    String(&'a str, bool),
    Duration(&'a str, bool),
    Reason(&'a str),
}

impl<'a> CommandSyntax<'a> {
    pub fn get_def(&'a self) -> String {
        let (inner, required) = match self {
            Self::Consume(name) | Self::Reason(name) => (format!("...[{name}]"), None),
            Self::User(name, opt) => (format!("{name}: Discord User"), Some(opt)),
            Self::Member(name, opt) => (format!("{name}: Discord Member"), Some(opt)),
            Self::String(name, opt) => (format!("{name}: String"), Some(opt)),
            Self::Duration(name, opt) => (format!("{name}: Duration"), Some(opt)),
        };

        if let Some(is_required) = required {
            if *is_required {
                format!("<{inner}>")
            } else {
                format!("[{inner}]")
            }
        } else {
            inner
        }
    }

    pub fn get_example(&'a self) -> String {
        match self {
            CommandSyntax::Consume(_) => "Some Text",
            CommandSyntax::User(_, _) => "123456789",
            CommandSyntax::Member(_, _) => "123456789",
            CommandSyntax::String(_, _) => "\"String\"",
            CommandSyntax::Duration(_, _) => "15m",
            CommandSyntax::Reason(_) => "user broke a rule"
        }.to_string()
    }
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
    fn get_syntax(&self) -> Vec<CommandSyntax>;

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

mod softban;
pub use softban::Softban;

mod ban;
pub use ban::Ban;

mod mute;
pub use mute::Mute;

mod unban;
pub use unban::Unban;

mod unmute;
pub use unmute::Unmute;

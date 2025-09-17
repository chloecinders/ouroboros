use core::fmt;
use std::{fmt::Debug, iter::Peekable, pin::Pin, sync::Arc, vec::IntoIter};

use crate::{
    event_handler::{CommandError, MissingArgumentError},
    lexer::Token,
};
use serenity::{
    all::{Context, GuildChannel, Member, Message, Permissions, User},
    async_trait,
};

#[allow(clippy::large_enum_variant)]
pub enum TransformerError {
    CommandError(CommandError),
    MissingArgumentError(MissingArgumentError),
}

pub type TransformerReturn<'a> =
    Pin<Box<dyn Future<Output = Result<Token, TransformerError>> + Send + 'a>>;
type TransformerFn = Arc<
    dyn for<'a> Fn(
            &'a Context,
            &'a Message,
            &'a mut Peekable<IntoIter<Token>>,
        ) -> TransformerReturn<'a>
        + Send
        + Sync,
>;

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum CommandArgument {
    String(String),
    User(User),
    Member(Member),
    Duration(chrono::Duration),
    None,
    i32(i32),
    GuildChannel(GuildChannel),
}

pub enum CommandSyntax {
    Consume(&'static str),
    User(&'static str, bool),
    Member(&'static str, bool),
    String(&'static str, bool),
    Duration(&'static str, bool),
    Reason(&'static str),
    Number(&'static str, bool),
    Or(Box<CommandSyntax>, Box<CommandSyntax>),
}

#[derive(PartialEq, Eq, Hash)]
pub enum CommandCategory {
    Misc,
    Utilities,
    Moderation,
    Admin,
    Developer,
}

impl fmt::Display for CommandCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CommandCategory::Misc => "Misc",
                CommandCategory::Utilities => "Utilities",
                CommandCategory::Moderation => "Moderation",
                CommandCategory::Admin => "Admin",
                CommandCategory::Developer => "Developer",
            }
        )
    }
}

impl CommandSyntax {
    pub fn get_def(&self) -> String {
        let (inner, required) = match self {
            Self::Consume(name) | Self::Reason(name) => (format!("...[{name}]"), None),
            Self::Or(a, b) => (format!("({} || {})", a.get_def(), b.get_def()), None),
            Self::User(name, opt) => (format!("{name}: Discord User"), Some(opt)),
            Self::Member(name, opt) => (format!("{name}: Discord Member"), Some(opt)),
            Self::String(name, opt) => (format!("{name}: String"), Some(opt)),
            Self::Duration(name, opt) => (format!("{name}: Duration"), Some(opt)),
            Self::Number(name, opt) => (format!("{name}: Number"), Some(opt)),
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

    pub fn get_example(&self) -> String {
        match self {
            CommandSyntax::Consume(_) => String::from("Some Text"),
            CommandSyntax::User(_, _) => String::from("123456789"),
            CommandSyntax::Member(_, _) => String::from("123456789"),
            CommandSyntax::String(_, _) => String::from("\"something\""),
            CommandSyntax::Duration(_, _) => String::from("15m"),
            CommandSyntax::Reason(_) => String::from("user broke a rule"),
            CommandSyntax::Number(_, _) => String::from("5"),
            CommandSyntax::Or(a, b) => format!("({} || {})", a.get_example(), b.get_example()),
        }
    }
}

#[derive(Default)]
pub struct CommandPermissions {
    pub required: Vec<Permissions>,
    pub one_of: Vec<Permissions>,
}

#[async_trait]
pub trait Command: Send + Sync {
    // Command descriptors
    fn get_name(&self) -> String;
    fn get_short(&self) -> String;
    fn get_full(&self) -> String;
    fn get_syntax(&self) -> Vec<CommandSyntax>;
    fn get_category(&self) -> CommandCategory;

    // Runner
    async fn run(&self, ctx: Context, msg: Message, args: Vec<Token>) -> Result<(), CommandError>;

    // Run helpers
    fn get_transformers(&self) -> Vec<TransformerFn> {
        vec![]
    }
    fn get_permissions(&self) -> CommandPermissions {
        Default::default()
    }
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

mod cban;
pub use cban::CBan;

mod purge;
pub use purge::Purge;

mod msgdbg;
pub use msgdbg::MsgDbg;

mod colon_three;
pub use colon_three::ColonThree;

mod reason;
pub use reason::Reason;

mod update;
pub use update::Update;

mod config;
pub use config::Config;

mod say;
pub use say::Say;

mod about;
pub use about::About;

mod duration;
pub use duration::Duration;

mod extract_id;
pub use extract_id::ExtractId;

mod cache;
pub use cache::Cache;

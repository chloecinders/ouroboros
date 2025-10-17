use core::fmt;
use std::{collections::HashMap, fmt::Debug, iter::Peekable, pin::Pin, sync::Arc, vec::IntoIter};

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
pub type TransformerFn = dyn for<'a> Fn(&'a Context, &'a Message, &'a mut Peekable<IntoIter<Token>>) -> TransformerReturn<'a>
    + Send
    + Sync;
pub type TransformerFnArc = Arc<TransformerFn>;

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum CommandArgument {
    String(String),
    User(User),
    // ManyUsers(Vec<User>),
    Member(Member),
    Duration(chrono::Duration),
    None,
    i32(i32),
    GuildChannel(GuildChannel),
    bool(bool),
}

pub enum CommandSyntax {
    Consume(&'static str),
    User(&'static str, bool),
    Member(&'static str, bool),
    Channel(&'static str, bool),
    String(&'static str, bool),
    Duration(&'static str, bool),
    Reason(&'static str),
    Number(&'static str, bool),
    Filters,
    Or(Box<CommandSyntax>, Box<CommandSyntax>),
}

pub struct CommandParameter<'a> {
    pub name: &'a str,
    pub short: &'a str,
    pub transformer: &'a TransformerFn,
    pub desc: &'a str,
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
            Self::Filters => (String::from("...[filters]"), None),
            Self::Channel(name, opt) => (format!("{name}: Channel"), Some(opt)),
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
            CommandSyntax::Channel(_, _) => String::from("#some-channel"),
            CommandSyntax::Filters => String::from("+user @ouroboros"),
            CommandSyntax::Or(a, b) => format!("({} || {})", a.get_example(), b.get_example()),
        }
    }
}

#[derive(Default)]
pub struct CommandPermissions {
    pub required: Vec<Permissions>,
    pub one_of: Vec<Permissions>,
    pub bot: Vec<Permissions>,
}

impl CommandPermissions {
    pub fn baseline() -> Vec<Permissions> {
        vec![
            Permissions::VIEW_CHANNEL,
            Permissions::SEND_MESSAGES,
            Permissions::SEND_MESSAGES_IN_THREADS,
            Permissions::READ_MESSAGE_HISTORY,
            Permissions::ATTACH_FILES,
            Permissions::EMBED_LINKS,
            Permissions::ADD_REACTIONS,
        ]
    }

    pub fn moderation() -> Vec<Permissions> {
        vec![
            Permissions::KICK_MEMBERS,
            Permissions::MODERATE_MEMBERS,
            Permissions::BAN_MEMBERS,
            Permissions::MANAGE_MESSAGES,
            Permissions::MANAGE_NICKNAMES,
            Permissions::MODERATE_MEMBERS,
        ]
    }
}

#[async_trait]
pub trait Command: Send + Sync {
    // Command descriptors
    fn get_name(&self) -> &'static str;
    fn get_short(&self) -> &'static str;
    fn get_full(&self) -> &'static str;
    fn get_syntax(&self) -> Vec<CommandSyntax>;
    fn get_category(&self) -> CommandCategory;
    fn get_params(&self) -> Vec<&'static CommandParameter<'static>>;

    // Runner
    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        args: Vec<Token>,
        params: HashMap<&str, (bool, CommandArgument)>,
    ) -> Result<(), CommandError>;

    // Run helpers
    fn get_transformers(&self) -> Vec<TransformerFnArc> {
        vec![]
    }
    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            bot: CommandPermissions::baseline(),
            ..Default::default()
        }
    }
}

mod admin;
pub use admin::Config;
pub use admin::DefineLog;

mod developer;
pub use developer::MsgDbg;
pub use developer::PermDbg;
pub use developer::Say;
pub use developer::Update;

mod misc;
pub use misc::About;
pub use misc::ColonThree;
pub use misc::Ping;
pub use misc::Stats;

mod moderation;
pub use moderation::Ban;
pub use moderation::Duration;
pub use moderation::Kick;
pub use moderation::Log;
pub use moderation::Mute;
pub use moderation::Purge;
pub use moderation::Reason;
pub use moderation::Softban;
pub use moderation::Unban;
pub use moderation::Unmute;
pub use moderation::Warn;

mod utilities;
pub use utilities::Cache;
pub use utilities::ExtractId;

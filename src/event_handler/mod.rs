use std::sync::Arc;

use serenity::{
    all::{
        ChannelId, Context, CreateAllowedMentions, CreateEmbed, CreateMessage, EventHandler, Guild, GuildId, GuildMemberUpdateEvent, Member, Message, MessageId, MessageUpdateEvent
    },
    async_trait,
};
use tracing::warn;

use crate::{
    commands::{
        About, Ban, CBan, Cache, ColonThree, Command, Config, Duration, ExtractId, Kick, Log, MsgDbg, Mute, Ping, Purge, Reason, Say, Softban, Stats, Unban, Unmute, Update, Warn
    },
    constants::BRAND_RED,
    lexer::Token,
};

#[derive(Debug)]
pub struct CommandError {
    pub title: String,
    pub hint: Option<String>,
    pub arg: Option<Token>,
}

impl CommandError {
    pub fn arg_not_found(arg_type: &str, name: Option<&str>) -> Self {
        let name = name.map(|n| format!(": {n}")).unwrap_or_default();

        Self {
            arg: None,
            title: format!("Missing argument, expected {arg_type}{name}"),
            hint: Some(String::from("for more information run !help (command)")),
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Command Error: {}; hint: {}",
            self.title,
            self.hint.clone().unwrap_or(String::from("(None)"))
        )
    }
}

impl std::error::Error for CommandError {}

#[derive(Debug)]
pub struct MissingArgumentError(pub String);

impl std::fmt::Display for MissingArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing Argument Error: {}", self.0)
    }
}

impl std::error::Error for MissingArgumentError {}

mod help_cmd;

// events
mod guild_create;
mod guild_member_update;
mod message;
mod message_delete;
mod message_update;
mod shards_ready;

pub struct Handler {
    prefix: String,
    commands: Vec<Arc<dyn Command>>,
}

impl Handler {
    pub fn new(prefix: String) -> Self {
        let commands: Vec<Arc<dyn Command>> = vec![
            Arc::new(Ping::new()),
            Arc::new(Stats::new()),
            Arc::new(Warn::new()),
            Arc::new(Log::new()),
            Arc::new(Kick::new()),
            Arc::new(Softban::new()),
            Arc::new(Ban::new()),
            Arc::new(Mute::new()),
            Arc::new(Unban::new()),
            Arc::new(Unmute::new()),
            Arc::new(CBan::new()),
            Arc::new(Purge::new()),
            Arc::new(MsgDbg::new()),
            Arc::new(ColonThree::new()),
            Arc::new(Reason::new()),
            Arc::new(Update::new()),
            Arc::new(Config::new()),
            Arc::new(Say::new()),
            Arc::new(About::new()),
            Arc::new(Duration::new()),
            Arc::new(ExtractId::new()),
            Arc::new(Cache::new())
        ];

        Self { prefix, commands }
    }
}

impl Handler {
    pub async fn send_error(&self, ctx: Context, msg: Message, input: String, err: CommandError) {
        let error_message;

        if let Some(arg) = err.arg {
            let mut hint = String::new();

            if let Some(h) = err.hint {
                hint = format!("**hint:** {h}");
            }

            error_message = format!(
                "**error:** argument {}\n```\n{input}\n{}{}\n{}\n```\n{}",
                arg.iteration,
                " ".repeat(arg.position + 1),
                "^".repeat(arg.length),
                err.title,
                hint
            );
        } else {
            let mut hint = String::new();

            if let Some(h) = err.hint {
                hint = format!("**hint:** {h}");
            }

            error_message = format!(
                "**error:** command failed to run```\n{input}\n\n{}\n```\n{}",
                err.title, hint
            );
        }

        let reply = CreateMessage::new()
            .add_embed(
                CreateEmbed::new()
                    .description(error_message)
                    .color(BRAND_RED),
            )
            .reference_message(&msg)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

        if let Err(e) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {e:?}")
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        message::message(self, ctx, msg).await
    }
    async fn message_update(
        &self,
        ctx: Context,
        old_if_available: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        message_update::message_update(self, ctx, old_if_available, new, event).await
    }
    async fn message_delete(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        guild_id: Option<GuildId>,
    ) {
        message_delete::message_delete(self, ctx, channel_id, deleted_message_id, guild_id).await
    }
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        guild_create::guild_create(self, ctx, guild, is_new).await
    }
    async fn shards_ready(&self, ctx: Context, total_shards: u32) {
        shards_ready::shards_ready(self, ctx, total_shards).await
    }
    async fn guild_member_update(
        &self,
        ctx: Context,
        old_if_available: Option<Member>,
        new: Option<Member>,
        event: GuildMemberUpdateEvent,
    ) {
        guild_member_update::guild_member_update(self, ctx, old_if_available, new, event).await
    }
}

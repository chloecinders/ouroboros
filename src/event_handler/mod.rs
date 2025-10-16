use std::{collections::HashMap, sync::Arc, time::Duration};

use serenity::{
    all::{
        ChannelId, Context, CreateAllowedMentions, CreateEmbed, CreateMessage, EventHandler, Guild,
        GuildId, GuildMemberUpdateEvent, Member, Message, MessageId, MessageUpdateEvent, User,
    },
    async_trait,
};
use tokio::{
    sync::{Mutex, MutexGuard},
    time::sleep,
};
use tracing::{info, warn};

use crate::{
    SQL,
    commands::{
        About, Ban, Cache, ColonThree, Command, Config, DefineLog, Duration as DurationCommand,
        ExtractId, Kick, Log, MsgDbg, Mute, PermDbg, Ping, Purge, Reason, Say, Softban, Stats,
        Unban, Unmute, Update, Warn,
    },
    constants::BRAND_RED,
    event_handler::message_cache::MessageCache,
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

// incredibly annoying, Serenity's event is marked as non-exhaustive with no method to construct it manually!
struct MessageDeleteEvent {
    // guild_id: Option<GuildId>, unused
    channel_id: ChannelId,
    message_id: MessageId,
}

mod help_cmd;
mod message_cache;

// events
mod guild_create;
mod guild_member_removal;
mod guild_member_update;
mod message;
mod message_delete;
mod message_update;
mod shards_ready;

pub struct Handler {
    prefix: String,
    commands: Vec<Arc<dyn Command>>,
    message_cache: Arc<Mutex<MessageCache>>,
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
            Arc::new(Purge::new()),
            Arc::new(MsgDbg::new()),
            Arc::new(ColonThree::new()),
            Arc::new(Reason::new()),
            Arc::new(Update::new()),
            Arc::new(Config::new()),
            Arc::new(Say::new()),
            Arc::new(About::new()),
            Arc::new(DurationCommand::new()),
            Arc::new(ExtractId::new()),
            Arc::new(Cache::new()),
            Arc::new(DefineLog::new()),
            Arc::new(PermDbg::new()),
        ];

        let cache = Arc::new(Mutex::new(MessageCache::new()));
        let cache_clone = cache.clone();

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(43200)).await;
                let lock = cache_clone.lock().await;
                Self::update_cache_size(lock).await;
            }
        });

        Self {
            prefix,
            commands,
            message_cache: cache,
        }
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
                    .description(error_message.clone())
                    .color(BRAND_RED),
            )
            .reference_message(&msg)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

        if let Err(e) = msg.channel_id.send_message(&ctx, reply).await {
            let _ = msg
                .channel_id
                .send_message(
                    &ctx,
                    CreateMessage::new().content(format!(
                        "{error_message}\n-# Bot does not have embed perms in this channel."
                    )),
                )
                .await;
            warn!("Could not send message; err = {e:?}")
        }
    }

    pub async fn update_cache_size(mut cache: MutexGuard<'_, MessageCache>) {
        info!("Updating message cache sizes...");

        let inserts = cache.get_inserts();
        let mut sizes = cache.get_sizes();
        let actions: HashMap<u64, i16> = HashMap::new();

        for (channel, count) in inserts {
            let count = count as f32;
            let size = *sizes.entry(channel).or_insert(100) as f32;

            if count > size * 0.4 {
                sizes.insert(channel, (size * 1.2).round() as usize);
            } else if (count) < size * 0.20 {
                sizes.insert(channel, (size * 0.8).round() as usize);
            }
        }

        let rows: Vec<(i64, i64, i16)> = sizes
            .iter()
            .map(|(&channel_id, &count)| {
                let prev_action = actions.get(&channel_id).copied().unwrap_or(0);
                (channel_id as i64, count as i64, prev_action)
            })
            .collect();

        let channel_ids: Vec<i64> = rows.iter().map(|r| r.0).collect();
        let message_counts: Vec<i64> = rows.iter().map(|r| r.1).collect();
        let previous_actions: Vec<i16> = rows.iter().map(|r| r.2).collect();

        if let Err(err) = sqlx::query!(
            r#"
                INSERT INTO message_cache_store (channel_id, message_count, previous_action)
                SELECT * FROM UNNEST($1::BIGINT[], $2::BIGINT[], $3::SMALLINT[])
                ON CONFLICT (channel_id) DO UPDATE
                SET message_count = EXCLUDED.message_count,
                    previous_action = EXCLUDED.previous_action
            "#,
            &channel_ids,
            &message_counts,
            &previous_actions,
        )
        .execute(SQL.get().unwrap())
        .await
        {
            warn!("Got error updating message cache store; err = {err:?}");
        }

        cache.clear_inserts();
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        {
            let mut lock = self.message_cache.lock().await;
            let cloned = msg.clone();
            lock.store_message(cloned.channel_id.get(), cloned);
        }

        message::message(self, ctx, msg).await;
    }

    async fn message_update(
        &self,
        ctx: Context,
        _old_if_available: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        let mut lock = self.message_cache.lock().await;
        let old_if_available = lock
            .get_message(event.channel_id.get(), event.id.get())
            .cloned();
        message_update::message_update(self, ctx, old_if_available, new, event).await
    }

    async fn message_delete(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        _guild_id: Option<GuildId>,
    ) {
        let mut lock = self.message_cache.lock().await;
        let event = MessageDeleteEvent {
            channel_id,
            message_id: deleted_message_id,
        };
        let old_if_available = lock
            .get_message(event.channel_id.get(), event.message_id.get())
            .cloned();
        message_delete::message_delete(self, ctx, event, old_if_available).await
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

    async fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        user: User,
        member_data_if_available: Option<Member>,
    ) {
        guild_member_removal::guild_member_removal(
            self,
            ctx,
            guild_id,
            user,
            member_data_if_available,
        )
        .await
    }
}

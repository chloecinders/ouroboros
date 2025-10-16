use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serenity::all::{CacheHttp, ChannelId, CreateMessage, GuildId};
use tracing::warn;

use crate::GUILD_SETTINGS;

#[derive(Hash, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogType {
    MemberBan,
    MemberUnban,
    MemberCache,
    MemberKick,
    MemberMute,
    MemberUnmute,
    MemberWarn,
    MemberSoftban,
    MemberUpdate,
    ActionUpdate,
    MessageDelete,
    MessageEdit,
}

impl LogType {
    pub fn title(&self) -> String {
        String::from(match self {
            LogType::MemberBan => "Member Ban",
            LogType::MemberUnban => "Member Unban",
            LogType::MemberCache => "Member Cache",
            LogType::MemberKick => "Member Kick",
            LogType::MemberMute => "Member Mute",
            LogType::MemberUnmute => "Member Unmute",
            LogType::MemberWarn => "Member Warn",
            LogType::MemberSoftban => "Member Softban",
            LogType::MemberUpdate => "Member Update",
            LogType::ActionUpdate => "Action Update",
            LogType::MessageDelete => "Message Delete",
            LogType::MessageEdit => "Message Edit",
        })
    }

    pub fn all() -> Vec<LogType> {
        vec![
            LogType::MemberBan,
            LogType::MemberUnban,
            LogType::MemberCache,
            LogType::MemberKick,
            LogType::MemberMute,
            LogType::MemberUnmute,
            LogType::MemberWarn,
            LogType::MemberSoftban,
            LogType::MemberUpdate,
            LogType::ActionUpdate,
            LogType::MessageDelete,
            LogType::MessageEdit,
        ]
    }

    pub async fn channel_id(&self, guild: GuildId) -> Option<ChannelId> {
        let mut lock = GUILD_SETTINGS.get().unwrap().lock().await;
        let settings = lock.get(guild.get()).await.ok()?;

        settings
            .log
            .log_channel_ids
            .get(self)
            .map(|c| ChannelId::new(*c))
    }
}

pub async fn guild_log(http: impl CacheHttp, log_type: LogType, guild: GuildId, msg: CreateMessage) {
    let Some(channel) = log_type.channel_id(guild).await else {
        return;
    };

    if let Err(err) = channel.send_message(http, msg).await {
        warn!("Cannot not send log message; err = {err:?}");
    }
}

pub fn snowflake_to_timestamp(snowflake: u64) -> chrono::DateTime<chrono::Utc> {
    let discord_epoch: i64 = 1420070400000;
    let timestamp = ((snowflake >> 22) as i64) + discord_epoch;

    DateTime::from_naive_utc_and_offset(
        DateTime::from_timestamp_millis(timestamp)
            .unwrap()
            .naive_utc(),
        chrono::Utc,
    )
}

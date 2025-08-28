use chrono::DateTime;
use serenity::all::{CreateMessage, GuildId, Http};
use tracing::warn;

use crate::GUILD_SETTINGS;

pub async fn guild_log(http: &Http, guild: GuildId, msg: CreateMessage) {
    let mut settings = GUILD_SETTINGS.get().unwrap().lock().await;
    let Ok(guild_settings) = settings.get(guild.get()).await else {
        warn!("Found guild with no cached settings; Id = {}", guild.get());
        return;
    };

    if guild_settings.log.channel.is_none() {
        return;
    }

    let Ok(channel) = http
        .get_channel(guild_settings.log.channel.unwrap_or(1).into())
        .await
    else {
        warn!(
            "Cannot get log channel; guild = {}, channel = {}",
            guild.get(),
            guild_settings.log.channel.unwrap_or(1)
        );
        return;
    };

    if let Err(err) = channel.id().send_message(http, msg).await {
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

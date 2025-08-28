use std::sync::Arc;

use serenity::all::{Guild, Http};
use sqlx::query;
use tracing::{error, info, warn};

use crate::SQL;

pub async fn check_expiring_bans(http: &Arc<Http>) {
    info!("check_expiring_bans asynchronous task running...");

    let data = match query!(
        r#"
        SELECT id, guild_id, user_id FROM actions WHERE type = 'ban' AND active = true AND expires_at < NOW();
        "#
    ).fetch_all(SQL.get().unwrap()).await {
        Ok(d) => d,
        Err(e) => {
            error!("task check_expiring_bans couldnt fetch necessary data; Err = {e:?}");
            return;
        }
    };

    let mut updated: Vec<String> = vec![];

    for entry in data {
        let Ok(guild) = Guild::get(http, entry.guild_id as u64).await else {
            warn!("task check_expiring_bans couldnt fetch guild; Id = {:?}", entry.guild_id);
            continue;
        };

        if guild.unban(http, entry.user_id as u64).await.is_err() {
            warn!("task check_expiring_bans couldnt unban user; Guild = {:?} Id = {:?}", entry.guild_id, entry.user_id);
            continue;
        } else {
            updated.push(entry.id);
        }
    }

    if query!(
        r#"
        UPDATE actions SET active = false WHERE id = ANY($1);
        "#,
        &updated
    ).execute(SQL.get().unwrap()).await.is_err() {
        error!("task check_expiring_bans couldnt update entries; entries = {:?}", updated);
    } else {
        info!("task check_expiring_bans finished");
    }
}

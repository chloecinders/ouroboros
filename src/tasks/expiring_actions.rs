use std::sync::Arc;

use serenity::all::{Cache, Http};
use tracing::info;

pub fn check_expiring_bans(cache: &Arc<Cache>, http: &Arc<Http>) {
    info!("check_expiring_bans asynchronous task running...");
}

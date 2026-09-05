use std::sync::Arc;

use tokio::sync::Notify;
use tracing::info;

#[derive(Clone, Default)]
pub struct Requested(Arc<Notify>);

impl Requested {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ask(&self) {
        self.0.notify_waiters();
    }

    pub async fn asked(&self) {
        self.0.notified().await;
    }
}

pub async fn requested_or_signalled(requested: &Requested) {
    tokio::select! {
        _ = signal() => {},
        _ = requested.asked() => info!("shutdown requested from a command"),
    }
}

#[cfg(unix)]
pub async fn signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut terminate = match signal(SignalKind::terminate()) {
        Ok(listener) => listener,
        Err(err) => {
            info!("no sigterm listener, falling back to ctrl-c only; err = {err:?}");

            let _ = tokio::signal::ctrl_c().await;

            return;
        }
    };

    tokio::select! {
        _ = tokio::signal::ctrl_c() => info!("received ctrl-c, shutting down"),
        _ = terminate.recv() => info!("received sigterm, shutting down"),
    }
}

#[cfg(not(unix))]
pub async fn signal() {
    let _ = tokio::signal::ctrl_c().await;

    info!("received ctrl-c, shutting down");
}

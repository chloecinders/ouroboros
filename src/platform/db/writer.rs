use std::future::Future;
use std::time::Duration;

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::timeout;
use tracing::warn;

pub struct Batched<T> {
    outbox: Sender<T>,
    label: &'static str,
}

impl<T: Send + 'static> Batched<T> {
    pub fn spawn<F, Fut>(label: &'static str, flush: F) -> Self
    where
        F: Fn(Vec<T>) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let (outbox, inbox) = mpsc::channel(4096);

        tokio::spawn(drain(inbox, flush));

        Self { outbox, label }
    }

    pub fn send(&self, item: T) {
        if self.outbox.try_send(item).is_err() {
            warn!("{} writer is saturated, dropping a row", self.label);
        }
    }
}

async fn drain<T, F, Fut>(mut inbox: Receiver<T>, flush: F)
where
    F: Fn(Vec<T>) -> Fut + Send,
    Fut: Future<Output = ()> + Send + 'static,
{
    let window = Duration::from_millis(250);

    loop {
        let Some(first) = inbox.recv().await else {
            return;
        };

        let mut batch = Vec::with_capacity(256);

        batch.push(first);

        while batch.len() < 256 {
            match timeout(window, inbox.recv()).await {
                Ok(Some(next)) => batch.push(next),
                Ok(None) => break,
                Err(_) => break,
            }
        }

        flush(batch).await;
    }
}

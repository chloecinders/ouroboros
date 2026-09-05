use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::domain::Snowflake;
use crate::platform::cache::lru::Lru;
use crate::platform::discord::partial::PartialMessage;

pub type Cached = Arc<PartialMessage>;

#[derive(Default)]
pub struct Recent {
    channels: Mutex<HashMap<Snowflake, Lru<Snowflake, Cached>>>,
}

impl Recent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn remember(&self, message: Cached) {
        let Ok(mut channels) = self.channels.lock() else {
            return;
        };

        channels
            .entry(message.channel_id)
            .or_insert_with(|| Lru::new(100))
            .insert(message.id, message);
    }

    pub fn take(&self, channel: Snowflake, message: Snowflake) -> Option<Cached> {
        self.channels
            .lock()
            .ok()?
            .get_mut(&channel)?
            .remove(&message)
    }

    pub fn peek(&self, channel: Snowflake, message: Snowflake) -> Option<Cached> {
        self.channels
            .lock()
            .ok()?
            .get(&channel)?
            .peek(&message)
            .map(Arc::clone)
    }
}

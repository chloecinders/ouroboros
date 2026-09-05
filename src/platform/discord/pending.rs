use std::time::Duration;

use crate::domain::Snowflake;
use crate::platform::cache::TtlSet;

pub struct Pending {
    deletions: TtlSet<(Snowflake, Snowflake)>,
    silenced: TtlSet<(Snowflake, Snowflake)>,
    timeouts: TtlSet<(Snowflake, Snowflake)>,
}

impl Default for Pending {
    fn default() -> Self {
        Self::new()
    }
}

impl Pending {
    pub fn new() -> Self {
        let window = Duration::from_secs(60);

        Self {
            deletions: TtlSet::new(4096, window),
            silenced: TtlSet::new(4096, window),
            timeouts: TtlSet::new(4096, window),
        }
    }

    pub fn expect_deletion(&self, channel: Snowflake, message: Snowflake) {
        self.deletions.insert((channel, message));
    }

    pub fn expect_deletions(
        &self,
        channel: Snowflake,
        messages: impl IntoIterator<Item = Snowflake>,
    ) {
        for message in messages {
            self.expect_deletion(channel, message);
        }
    }

    pub fn claim_deletion(&self, channel: Snowflake, message: Snowflake) -> bool {
        self.deletions.take(&(channel, message))
    }

    pub fn silence(&self, channel: Snowflake, message: Snowflake) {
        self.silenced.insert((channel, message));
    }

    pub fn claim_silence(&self, channel: Snowflake, message: Snowflake) -> bool {
        self.silenced.take(&(channel, message))
    }

    pub fn expect_timeout(&self, guild: Snowflake, member: Snowflake) {
        self.timeouts.insert((guild, member));
    }

    pub fn claim_timeout(&self, guild: Snowflake, member: Snowflake) -> bool {
        self.timeouts.take(&(guild, member))
    }

    pub fn sweep(&self) {
        self.deletions.sweep();
        self.silenced.sweep();
        self.timeouts.sweep();
    }
}

pub mod commands;
pub mod executor;
pub mod external;
pub mod lifecycle;
pub mod notices;
pub mod scheduled;
pub mod store;
pub mod sync;

use std::sync::Arc;

use crate::command::registry::Registry;
use crate::platform::discord::dispatch::Dispatch;
use crate::register;

pub fn observe(dispatch: &mut Dispatch) {
    dispatch.add(Arc::new(sync::Sync));
}

pub fn register(registry: &mut Registry) {
    register!(
        registry,
        commands::warn::Warn,
        commands::kick::Kick,
        commands::ban::Ban,
        commands::softban::Softban,
        commands::mute::Mute,
        commands::unban::Unban,
        commands::unmute::Unmute,
    );
}

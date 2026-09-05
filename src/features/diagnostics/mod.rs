pub mod commands;
pub mod store;
pub mod ui;

use crate::command::registry::Registry;
use crate::register;

pub fn register(registry: &mut Registry) {
    register!(
        registry,
        commands::downtime::ScheduleDowntime,
        commands::restart::Restart,
        commands::say::Say,
        commands::trace::Trace,
    );

    #[cfg(feature = "self-update")]
    register!(registry, commands::update::Update);
}

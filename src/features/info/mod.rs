pub mod commands;

use crate::command::registry::Registry;
use crate::register;

pub fn register(registry: &mut Registry) {
    register!(
        registry,
        commands::about::About,
        commands::colon_three::ColonThree,
        commands::ping::Ping,
        commands::stats::Stats,
    );
}

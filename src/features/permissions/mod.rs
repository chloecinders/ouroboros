pub mod cache;
pub mod commands;
pub mod resolve;
pub mod rule;
pub mod store;
pub mod ui;

use crate::command::registry::Registry;
use crate::register;

pub fn register(registry: &mut Registry) {
    register!(registry, commands::perms::Perms,);
}

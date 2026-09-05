pub mod cache;
pub mod commands;
pub mod controls;
pub mod store;
pub mod ui;

use crate::command::registry::Registry;
use crate::platform::discord::interact::Router;
use crate::register;

pub fn control(router: &mut Router) {
    controls::register(router);
}

pub fn register(registry: &mut Registry) {
    register!(registry, commands::definelog::DefineLog);
}

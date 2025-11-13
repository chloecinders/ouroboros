use serenity::all::{Context, Message};

use crate::{event_handler::Handler, utils::command_processing::process};

pub async fn message(handler: &Handler, ctx: Context, msg: Message) {
    process(handler, ctx, msg).await
}

use std::time::{Duration, Instant};

use serenity::{all::{Context, CreateEmbed, CreateMessage, Message as DiscordMessage}, async_trait};
use tracing::warn;

use crate::{commands::{Command, CommandSyntax}, constants::BRAND_BLUE, event_handler::CommandError, lexer::Token, ShardManagerContainer};

pub struct Ping;

impl Ping {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Ping {
    fn get_name(&self) -> String {
        String::from("ping")
    }

    fn get_short(&self) -> String {
        String::from("Gets the bots current latency")
    }

    fn get_full(&self) -> String {
        String::from("Gets the bots HTTP and gateway latency. Useful for checking if the bot is lagging.")
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![]
    }

    async fn run(&self, ctx: Context, msg: DiscordMessage, _args: Vec<Token>) -> Result<(), CommandError> {
        let http_ping = {
            let start = Instant::now();
            let _ = ctx.http.get_current_user().await;
            start.elapsed()
        };

        let gateway_ping = {
            let data_read = ctx.data.read().await;
            let shard_manager = data_read.get::<ShardManagerContainer>().unwrap().clone();
            let runners = shard_manager.runners.lock().await;
            let shard_info = runners.get(&ctx.shard_id).unwrap();
            shard_info.latency.unwrap_or(Duration::default())
        };

        let message = CreateMessage::new()
        .embed(
            CreateEmbed::new()
                .description(format!("HTTP: {}ms\nGateway: {}ms", http_ping.as_millis(), gateway_ping.as_millis()))
                .color(BRAND_BLUE.clone())
        )
        .reference_message(&msg);

        if let Err(e) = msg.channel_id.send_message(&ctx.http, message).await {
            warn!("Could not send message; err = {e:?}");
        }

        Ok(())
    }
}

use std::time::{Duration, Instant};

use reqwest::{Client, redirect::Policy};
use serenity::{
    all::{Context, CreateAllowedMentions, CreateEmbed, CreateMessage, Message as DiscordMessage},
    async_trait,
};
use tracing::warn;

use crate::{
    ShardManagerContainer,
    commands::{Command, CommandCategory, CommandSyntax},
    constants::BRAND_BLUE,
    event_handler::CommandError,
    lexer::Token,
};

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
        String::from(
            "Gets the bots HTTP and gateway latency. Useful for checking if the bot is lagging.",
        )
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::Misc
    }

    async fn run(
        &self,
        ctx: Context,
        msg: DiscordMessage,
        _args: Vec<Token>,
    ) -> Result<(), CommandError> {
        let http = {
            let start = Instant::now();
            let _ = ctx.http.get_current_user().await;
            start.elapsed().as_millis()
        };

        let gateway = {
            let data_read = ctx.data.read().await;
            let shard_manager = data_read.get::<ShardManagerContainer>().unwrap().clone();
            let runners = shard_manager.runners.lock().await;
            let shard_info = runners.get(&ctx.shard_id).unwrap();
            shard_info
                .latency
                .unwrap_or(Duration::default())
                .as_millis()
        };

        let ping = {
            let client = Client::builder().redirect(Policy::none()).build().unwrap();

            let start = Instant::now();
            let _ = client
                .get("https://discord.com/api/v10/gateway")
                .send()
                .await;
            start.elapsed().as_millis()
        };

        let message = CreateMessage::new()
            .embed(
                CreateEmbed::new()
                    .description(format!(
                        "HTTP: {http}ms\nGateway: {gateway}ms\nPing: {ping}ms",
                    ))
                    .color(BRAND_BLUE),
            )
            .reference_message(&msg)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

        if let Err(e) = msg.channel_id.send_message(&ctx.http, message).await {
            warn!("Could not send message; err = {e:?}");
        }

        Ok(())
    }
}

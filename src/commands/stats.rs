use std::time::Instant;

use serenity::{
    all::{Context, CreateAllowedMentions, CreateEmbed, CreateMessage, Message},
    async_trait,
};
use sysinfo::System;
use tracing::warn;

use crate::{
    START_TIME,
    commands::{Command, CommandCategory, CommandSyntax},
    constants::BRAND_BLUE,
    event_handler::CommandError,
    lexer::Token,
};

pub struct Stats;

impl Stats {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Command for Stats {
    fn get_name(&self) -> String {
        String::from("stats")
    }

    fn get_short(&self) -> String {
        String::from("Gets various bot statistics")
    }

    fn get_full(&self) -> String {
        String::from("Shows various statistics of the bot. Useful for nerds!")
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::Misc
    }

    async fn run(&self, ctx: Context, msg: Message, _args: Vec<Token>) -> Result<(), CommandError> {
        let guild_count = ctx.cache.guild_count();

        let uptime = {
            let elapsed = START_TIME.get().unwrap_or(&Instant::now()).elapsed();
            let seconds = elapsed.as_secs();

            (seconds / 3600, (seconds % 3600) / 60, seconds % 60)
        };

        let memory = {
            let mut sys = System::new_all();
            sys.refresh_all();

            sys.process((std::process::id() as usize).into())
                .map(|p| p.memory() as f64 / 1024.0 / 1024.0)
                .unwrap_or(0.0)
        };

        let description = {
            let uptime = if uptime.0 != 0 {
                format!("{}h {}m {}s", uptime.0, uptime.1, uptime.2)
            } else if uptime.1 != 0 {
                format!("{}m {}s", uptime.1, uptime.2)
            } else {
                format!("{}s", uptime.2)
            };

            format!("**STATS**\nServers: {guild_count}\nUptime: {uptime}\nMemory: {memory:.2}MB")
        };

        let reply = CreateMessage::new()
            .add_embed(
                CreateEmbed::new()
                    .description(description)
                    .color(BRAND_BLUE),
            )
            .reference_message(&msg)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

        if let Err(e) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {e:?}");
        }

        Ok(())
    }
}

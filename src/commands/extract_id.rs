use serenity::{
    all::{Context, CreateAllowedMentions, CreateEmbed, CreateMessage, Message},
    async_trait,
};
use tracing::warn;

use crate::{
    commands::{Command, CommandSyntax},
    constants::BRAND_BLUE,
    event_handler::CommandError,
    lexer::Token,
};

pub struct ExtractId;

impl ExtractId {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Command for ExtractId {
    fn get_name(&self) -> String {
        String::from("eid")
    }

    fn get_short(&self) -> String {
        String::from("Extracts an id from a message")
    }

    fn get_full(&self) -> String {
        String::from("Checks a replied to message for ids and sends them in separate messages. Useful for people on mobile who don't want to fight with their phone about copying out an id.")
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![]
    }

    async fn run(&self, ctx: Context, msg: Message, _args: Vec<Token>) -> Result<(), CommandError> {
        let Some(reply) = &msg.referenced_message else {
            return Err(CommandError { title: String::from("You must reply to a message to use this command"), hint: None, arg: None });
        };

        let mut search_text = reply.content.clone();

        for embed in &reply.embeds {
            let mut embed_locations = vec![embed.title.clone(), embed.description.clone(), embed.footer.clone().map(|f| f.text)];
            embed.fields.iter().for_each(|f| { embed_locations.push(Some(f.name.clone())); embed_locations.push(Some(f.value.clone())); });
            embed_locations.into_iter().for_each(|s| { search_text.push_str("\n"); search_text.push_str(&s.unwrap_or_default()); });
        }

         let mut ids = Vec::new();
        let mut current = String::new();

        for ch in search_text.chars() {
            if ch.is_ascii_digit() {
                current.push(ch);
            } else {
                if !current.is_empty() {
                    if current.len() >= 5 && current.len() <= 20 {
                        ids.push(current.clone());
                    }
                    current.clear();
                }
            }
        }

        if !current.is_empty() && current.len() >= 5 && current.len() <= 20 {
            ids.push(current);
        }

        if ids.is_empty() {
            let reply = CreateMessage::new()
                .add_embed(CreateEmbed::new().description("No IDs found in the referenced message.").color(BRAND_BLUE))
                .reference_message(&msg)
                .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

            if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
                warn!("Could not send message; err = {err:?}");
            }
        } else {
            let mut iter = ids.into_iter();
            let first_id = iter.next().unwrap();

            let reply = CreateMessage::new()
                .content(first_id)
                .reference_message(&msg)
                .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

            if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
                warn!("Could not send message; err = {err:?}");
            }

            for id in iter.take(4) {
                let reply = CreateMessage::new().content(id);

                if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
                    warn!("Could not send message; err = {err:?}");
                }
            }
        }

        Ok(())
    }
}

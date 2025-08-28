use serenity::all::{Context, CreateAllowedMentions, CreateEmbed, CreateMessage, Message};
use tracing::warn;

use crate::{
    constants::BRAND_BLUE,
    event_handler::{CommandError, Handler},
    lexer::Token,
};

impl Handler {
    pub async fn help_run(
        &self,
        ctx: Context,
        msg: Message,
        args: Vec<Token>,
    ) -> Result<(), CommandError> {
        let mut args_iter = args.into_iter();
        if let Some(name_tok) = args_iter.next() {
            let Some(cmd) = self
                .commands
                .iter()
                .find(|c| c.get_name() == name_tok.raw.to_lowercase())
            else {
                return Err(CommandError {
                    title: String::from("Command not found"),
                    hint: Some(String::from(
                        "double check if the command name provided is a valid command.",
                    )),
                    arg: Some(name_tok),
                });
            };

            let cmd_perms = cmd.get_permissions();

            let perms = if cmd_perms.one_of.is_empty() && cmd_perms.required.is_empty() {
                ""
            } else {
                let mut result = String::new();

                if !cmd_perms.required.is_empty() {
                    let string = cmd_perms
                        .required
                        .iter()
                        .map(|p| {
                            let names = p
                                .get_permission_names()
                                .into_iter()
                                .map(|n| n.to_uppercase().replace(" ", "_"))
                                .collect::<Vec<_>>();
                            names.join(" && ")
                        })
                        .collect::<Vec<_>>()
                        .join(" && ");
                    result.push_str(&string);
                }

                if !cmd_perms.one_of.is_empty() {
                    let string = cmd_perms
                        .one_of
                        .iter()
                        .map(|p| {
                            let names = p
                                .get_permission_names()
                                .into_iter()
                                .map(|n| n.to_uppercase().replace(" ", "_"))
                                .collect::<Vec<_>>();
                            names.join(" || ")
                        })
                        .collect::<Vec<_>>()
                        .join(" || ");

                    if !result.is_empty() {
                        result.push_str(&format!(" && ({string})"));
                    } else {
                        result.push_str(&string);
                    }
                }

                &format!("\nRequired Permissions:\n`{result}`")
            };

            let mut hint_text = String::from(
                "-# <name: type>, <> = required, [] = optional, ...[] = all text after last argument",
            );

            if !perms.is_empty() {
                hint_text.push_str("\n-# && = AND, || = OR");
            }

            let syntax = {
                let command_syntax = cmd.get_syntax();

                let mut def = vec![];
                let mut example = vec![];

                for syn in command_syntax {
                    def.push(syn.get_def());
                    example.push(syn.get_example());
                }

                format!(
                    "Syntax:\n```\n{0}{1} {2}\n```\nExample:\n```{0}{1} {3}```",
                    self.prefix,
                    cmd.get_name(),
                    def.join(" "),
                    example.join(" ")
                )
            };

            let reply = CreateMessage::new()
                .add_embed(
                    CreateEmbed::new()
                        .description(format!(
                            "**{}**\n{}\n\n{}{}\n\n{}",
                            cmd.get_name().to_uppercase(),
                            cmd.get_full(),
                            syntax,
                            perms,
                            hint_text
                        ))
                        .color(BRAND_BLUE),
                )
                .reference_message(&msg)
                .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

            if let Err(e) = msg.channel_id.send_message(&ctx.http, reply).await {
                warn!("Could not send message; err = {e:?}")
            }

            return Ok(());
        }

        let mut full_msg = String::new();

        self.commands.iter().for_each(|c| {
            if !c.get_short().is_empty() {
                full_msg.push_str(format!("`{}` - {}\n", c.get_name(), c.get_short()).as_str());
            }
        });

        let reply = CreateMessage::new()
            .add_embed(CreateEmbed::new().description(full_msg).color(BRAND_BLUE))
            .reference_message(&msg)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

        if let Err(e) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {e:?}")
        }

        Ok(())
    }
}

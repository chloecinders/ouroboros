use std::sync::Arc;

use serenity::{all::{Context, CreateEmbed, CreateMessage, EventHandler, Message}, async_trait};
use tracing::warn;

use crate::{commands::{Ban, Command, CommandArgument, Kick, Log, Ping, Softban, Stats, Warn}, constants::{BRAND_BLUE, BRAND_RED}, lexer::{lex, Token}, utils::check_guild_permission};

#[derive(Debug)]
pub struct CommandError {
    pub title: String,
    pub hint: Option<String>,
    pub arg: Option<Token>,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Command Error: {}; hint: {}", self.title, self.hint.clone().unwrap_or(String::from("(None)")))
    }
}

impl std::error::Error for CommandError {}

pub struct Handler {
    prefix: String,
    commands: Vec<Arc<dyn Command>>
}

impl Handler {
    pub fn new(prefix: String) -> Self {
        let commands: Vec<Arc<dyn Command>> = vec![
            Arc::new(Ping::new()),
            Arc::new(Stats::new()),
            Arc::new(Warn::new()),
            Arc::new(Log::new()),
            Arc::new(Kick::new()),
            Arc::new(Softban::new()),
            Arc::new(Ban::new()),
        ];

        Self {
            prefix,
            commands
        }
    }
}

impl Handler {
    async fn help_run(&self, ctx: Context, msg: Message, args: Vec<Token>) -> Result<(), CommandError> {
        let mut args_iter = args.into_iter();
        if let Some(name_tok) = args_iter.next() {
            let Some(cmd) = self.commands.iter().find(|c| c.get_name() == name_tok.raw.to_lowercase()) else {
                return Err(CommandError {
                    title: String::from("Command not found"),
                    hint: Some(String::from("double check if the command name provided is a valid command.")),
                    arg: Some(name_tok)
                })
            };

            let cmd_perms = cmd.get_permissions();

            let perms = if cmd_perms.one_of.len() == 0 && cmd_perms.required.len() == 0 { "" } else {
                let mut result = String::new();

                if cmd_perms.required.len() != 0 {
                    let string = cmd_perms.required.iter().map(|p| {
                        let names = p.get_permission_names().into_iter().map(|n| n.to_uppercase().replace(" ", "_")).collect::<Vec<_>>();
                        names.join(" && ")
                    }).collect::<Vec<_>>().join(" && ");
                    result.push_str(&string);
                }

                if cmd_perms.one_of.len() != 0 {
                    let string = cmd_perms.one_of.iter().map(|p| {
                        let names = p.get_permission_names().into_iter().map(|n| n.to_uppercase().replace(" ", "_")).collect::<Vec<_>>();
                        names.join(" || ")
                    }).collect::<Vec<_>>().join(" || ");

                    if result != "" {
                        result.push_str(&format!(" && ({string})"));
                    } else {
                        result.push_str(&string);
                    }
                }

                &format!("\nRequired Permissions:\n`{result}`")
            };

            let mut hint_text = String::from("-# <name: type>, <> = required, [] = optional, ...[] = all text after last argument");

            if perms.len() != 0 {
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
                        .description(
                            format!(
                                "**{}**\n{}\n\n{}{}\n\n{}",
                                cmd.get_name().to_uppercase(),
                                cmd.get_full(),
                                syntax,
                                perms,
                                hint_text
                            )
                        )
                        .color(BRAND_BLUE.clone())
                    )
                .reference_message(&msg);

            if let Err(e) = msg.channel_id.send_message(&ctx.http, reply).await {
                warn!("Could not send message; err = {e:?}")
            }

            return Ok(());
        }

        let mut full_msg = String::new();

        self.commands.iter().for_each(|c| {
            full_msg.push_str(format!("`{}` - {}\n", c.get_name(), c.get_short()).as_str());
        });

        let reply = CreateMessage::new()
            .add_embed(
                CreateEmbed::new()
                    .description(full_msg)
                    .color(BRAND_BLUE.clone())
                )
            .reference_message(&msg);

        if let Err(e) = msg.channel_id.send_message(&ctx.http, reply).await {
            warn!("Could not send message; err = {e:?}")
        }

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.content.starts_with(self.prefix.as_str()) || msg.guild_id.is_none() {
            return;
        }

        let contents = msg.content.clone();
        let strip = contents.strip_prefix(self.prefix.as_str()).unwrap_or("");
        let lex = lex(String::from(strip));
        let mut parts = lex.into_iter().enumerate();
        let command_name = parts.next().map(|(_, s)| s.raw).unwrap_or(String::new());

        if command_name == "help" {
            if let Err(err) = self.help_run(ctx.clone(), msg.clone(), parts.map(|(_, t)| t).collect()).await {
                send_error(ctx, msg, contents, err).await;
            }

            return;
        }

        let command = self.commands.iter().find(|c| c.get_name() == command_name.to_lowercase());

        if let Some(c) = command {
            let permissions = c.get_permissions();

            if permissions.required.len() != 0 || permissions.one_of.len() != 0 {
                let Ok(member) = msg.member(&ctx.http).await else {
                    send_error(ctx, msg, contents, CommandError {
                        title: String::from("You do not have permissions to execute this command."),
                        hint: Some(String::from("Consider begging for more permissions at your local Discord administrator!")),
                        arg: None
                    }).await;
                    return;
                };

                for permission in permissions.required {
                    if !check_guild_permission(&ctx, &member, permission).await {
                        send_error(ctx, msg, contents, CommandError {
                            title: String::from("You do not have permissions to execute this command."),
                            hint: Some(String::from("Consider begging for more permissions at your local Discord administrator!")),
                            arg: None
                        }).await;
                        return;
                    }
                }

                let mut pass = true;

                for permission in permissions.one_of {
                    if !check_guild_permission(&ctx, &member, permission).await {
                        pass = false;
                        break;
                    }
                }

                if !pass {
                    send_error(ctx, msg, contents, CommandError {
                        title: String::from("You do not have permissions to execute this command."),
                        hint: Some(String::from("Consider begging for more permissions at your local Discord administrator!")),
                        arg: None
                    }).await;
                    return;
                }
            }

            let transformers = c.get_transformers();
            let mut args: Vec<Token> = vec![];

            for (i, mut arg) in parts {
                if let Some(transformer) = transformers.get(i - 1) {
                    let result = transformer(&ctx, &msg, arg).await;

                    match result {
                        Ok(r) => {
                            args.push(r);
                        },
                        Err(err) => {
                            send_error(ctx, msg, contents, err).await;
                            return;
                        },
                    }
                } else {
                    arg.contents = Some(CommandArgument::String(arg.raw.clone()));
                    args.push(arg);
                }
            }

            let res = c.run(ctx.clone(), msg.clone(), args).await;

            if let Err(err) = res {
                send_error(ctx, msg, contents, err).await;
            }
        }
    }
}

async fn send_error(ctx: Context, msg: Message, input: String, err: CommandError) {
    let error_message;

    if let Some(arg) = err.arg {
        let mut hint = String::new();

        if let Some(h) = err.hint {
            hint = format!("**hint:** {h}");
        }

        error_message = format!(
            "**error:** argument {}\n```\n{input}\n{}{}\n{}\n```\n{}",
            arg.iteration,
            " ".repeat(arg.position + 1),
            "^".repeat(arg.length),
            err.title,
            hint
        );
    } else {
        let mut hint = String::new();

        if let Some(h) = err.hint {
            hint = format!("**hint:** {h}");
        }

        error_message = format!("**error:** command failed to run```\n{input}\n\n{}\n```\n{}", err.title, hint);
    }

    let reply = CreateMessage::new()
        .add_embed(CreateEmbed::new()
        .description(
            error_message
        ).color(BRAND_RED.clone()))
        .reference_message(&msg);

    if let Err(e) = msg.channel_id.send_message(&ctx.http, reply).await {
        warn!("Could not send message; err = {e:?}")
    }
}

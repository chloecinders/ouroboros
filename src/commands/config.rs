use std::sync::Arc;

use serenity::{all::{Context, CreateEmbed, CreateMessage, Message, Permissions}, async_trait, json::{self, Value}};
use sqlx::query;
use tracing::{error, warn};

use crate::{commands::{Command, CommandArgument, CommandPermissions, CommandSyntax, TransformerError, TransformerFn}, constants::BRAND_BLUE, event_handler::CommandError, lexer::Token, transformers::Transformers, utils::Settings, GUILD_SETTINGS, SQL};

pub struct Config;

impl Config {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for Config {
    fn get_name(&self) -> String {
        String::from("config")
    }

    fn get_short(&self) -> String {
        String::from("Configures functions of the bot.")
    }

    fn get_full(&self) -> String {
        String::from("Configures functions of the bot. \
            Available subcommands: list set get;\n \
            `list [group]` lists all groups/keys in a group\n \
            `set <group>.<key> <value>` sets a setting to value\n \
            `get <group>.<key>` gets the value of a setting \
            To clear a setting set its value to `none`.")
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![
            CommandSyntax::String("subcommand", true),
            CommandSyntax::String("argument1", false),
            CommandSyntax::String("argument2", false),
        ]
    }

    async fn run(
        &self,
        ctx: Context,
        msg: Message,
        args: Vec<Token>
    ) -> Result<(), CommandError> {
        let mut args_iter = args.into_iter();

        let Some(subcommand_token) = args_iter.next() else { return Err(CommandError::arg_not_found("String", Some("subcommand"))) };
        let Token { contents: Some(CommandArgument::String(subcommand)), .. } = subcommand_token.clone() else {
            return Err(CommandError::arg_not_found("String", Some("subcommand")));
        };

        let arg1_token = args_iter.next();
        let arg1: Option<String> = match arg1_token.clone() {
            Some(arg) => match arg.contents {
                Some(CommandArgument::String(inner)) => Some(inner),
                _ => None,
            },
            None => None,
        };

        let arg2_token = args_iter.next();
        let arg2: Option<String> = match arg2_token.clone() {
            Some(arg) => match arg.contents {
                Some(CommandArgument::String(inner)) => Some(inner),
                _ => None,
            },
            None => None,
        };

        let mut global = GUILD_SETTINGS.get().unwrap().lock().await;
        let settings = match global.get(msg.guild_id.map(|g| g.get() as u64).unwrap_or(1)).await {
            Ok(s) => s,
            Err(_) => {
                let settings = Settings {
                    ..Default::default()
                };

                settings
            }
        };

        if subcommand == "list" {
            let Ok(Value::Object(json_rep)) = json::to_value(&settings) else {
                error!("Json serialization went wrong on guild settings");
                return Err(CommandError { title: String::from("Could not fetch guild settings"), hint: Some(String::from("please try again later")), arg: None })
            };

            let description = if let Some(group_key) = arg1 {
                let Some(Value::Object(group)) = json_rep.get(&group_key) else {
                    return Err(CommandError { title: String::from("Could not find group"), hint: Some(String::from("run `config list` for a list of all groups")), arg: None })
                };

                format!(
                    "**Available Config Groups**\n{}",
                    group.keys().into_iter().map(|k| format!("`{k}`")).collect::<Vec<String>>().join("\n")
                )
            } else {
                format!(
                    "**Available Settings In Group**\n{}",
                    json_rep.keys().into_iter().map(|k| format!("`{k}`")).collect::<Vec<String>>().join("\n")
                )
            };

            let reply = CreateMessage::new()
                .add_embed(CreateEmbed::new().description(description).color(BRAND_BLUE.clone()))
                .reference_message(&msg);

            if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
                warn!("Could not send message; err = {err:?}");
            }

            Ok(())
        } else if subcommand == "get" {
            let Some(setting) = arg1 else {
                return Err(CommandError::arg_not_found("String", Some("arg1")));
            };

            let value = match setting.as_str() {
                "log.channel" => settings.log.channel.map(|c| format!("<#{c}>")).unwrap_or(String::from("None")),
                _ => {
                    return Err(CommandError { title: String::from("Could not find setting"), hint: Some(String::from("run config list for a list of valid settings")), arg: Some(arg1_token.unwrap()) })
                }
            };

            let reply = CreateMessage::new()
                .add_embed(CreateEmbed::new().description(format!("{}: {}", setting, value)).color(BRAND_BLUE.clone()))
                .reference_message(&msg);

            if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
                warn!("Could not send message; err = {err:?}");
            }

            Ok(())
        } else if subcommand == "set" {
            let Some(setting) = arg1 else {
                return Err(CommandError::arg_not_found("String", Some("arg1")));
            };

            let Some(value) = arg2 else {
                return Err(CommandError::arg_not_found("String", Some("arg1")));
            };

            let mut iter = vec![arg2_token.clone().unwrap()].into_iter().peekable();

            let query = match setting.as_str() {
                "log.channel" => {
                    if iter.peek().map(|t| t.raw.clone()).unwrap_or(String::new()).to_lowercase() == "none" {
                        query!(
                            "UPDATE guild_settings SET log_channel = $2 WHERE guild_id = $1;",
                            msg.guild_id.map(|g| g.get()).unwrap_or(1) as i64,
                            None as Option<i64>
                        )
                    } else {
                        match Transformers::guild_channel(&ctx, &msg, &mut iter).await {
                            Ok(Token {contents: Some(CommandArgument::GuildChannel(channel)), .. }) => query!(
                                "UPDATE guild_settings SET log_channel = $2 WHERE guild_id = $1;",
                                msg.guild_id.map(|g| g.get()).unwrap_or(1) as i64,
                                channel.id.get() as i64
                            ),
                            Err(TransformerError::CommandError(mut err)) => {
                                err.arg = Some(arg2_token.unwrap());
                                return Err(err);
                            }
                            _ => unreachable!()
                        }
                    }

                },
                _ => {
                    return Err(CommandError { title: String::from("Could not find setting"), hint: Some(String::from("run `config list` for a list of valid settings")), arg: Some(arg1_token.unwrap()) })
                }
            };

            if let Err(err) = query.execute(SQL.get().unwrap()).await {
                warn!("Could not update guild settings; err = {err:?}");
                return Err(CommandError { title: String::from("Could not update settings"), hint: Some(String::from("please try again later.")), arg: Some(arg1_token.unwrap()) })
            }

            global.invalidate();

            let reply = CreateMessage::new()
                .add_embed(CreateEmbed::new().description(format!("Successfully set {} to {}", setting, value)).color(BRAND_BLUE.clone()))
                .reference_message(&msg);

            if let Err(err) = msg.channel_id.send_message(&ctx.http, reply).await {
                warn!("Could not send message; err = {err:?}");
            }

            Ok(())
        } else {
            Err(CommandError { title: String::from("Subcommand not found"), hint: Some(String::from("available subcommands: list, get, set")), arg: Some(subcommand_token) })
        }
    }

    fn get_transformers(&self) -> Vec<TransformerFn> {
        vec![
            Arc::new(Transformers::some_string),
            Arc::new(Transformers::string),
            Arc::new(Transformers::some_string)
        ]
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions { required: vec![Permissions::ADMINISTRATOR], one_of: vec![] }
    }
}

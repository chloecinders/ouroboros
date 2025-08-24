use serenity::all::{Context, Message};

use crate::{commands::{CommandArgument, TransformerError}, event_handler::{CommandError, Handler}, lexer::{lex, Token}, utils::check_guild_permission};

pub async fn message(handler: &Handler, ctx: Context, msg: Message) {
    if !msg.content.starts_with(handler.prefix.as_str()) || msg.guild_id.is_none() {
        return;
    }

    let contents = msg.content.clone();
    let strip = contents.strip_prefix(handler.prefix.as_str()).unwrap_or("");
    let lex = lex(String::from(strip));
    let mut parts = lex.into_iter().peekable();
    let command_name = parts.next().map(|s| s.raw).unwrap_or(String::new());

    if command_name == "help" {
        if let Err(err) = handler.help_run(ctx.clone(), msg.clone(), parts.map(|t| t).collect()).await {
            handler.send_error(ctx, msg, contents, err).await;
        }

        return;
    }

    let command = handler.commands.iter().find(|c| c.get_name() == command_name.to_lowercase());

    if let Some(c) = command {
        let permissions = c.get_permissions();

        if permissions.required.len() != 0 || permissions.one_of.len() != 0 {
            let Ok(member) = msg.member(&ctx.http).await else {
                handler.send_error(ctx, msg, contents, CommandError {
                    title: String::from("You do not have permissions to execute this command."),
                    hint: Some(String::from("consider begging for more permissions at your local Discord administrator!")),
                    arg: None
                }).await;
                return;
            };

            for permission in permissions.required {
                if !check_guild_permission(&ctx, &member, permission).await {
                    handler.send_error(ctx, msg, contents, CommandError {
                        title: String::from("You do not have permissions to execute this command."),
                        hint: Some(String::from("consider begging for more permissions at your local Discord administrator!")),
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
                handler.send_error(ctx, msg, contents, CommandError {
                    title: String::from("You do not have permissions to execute this command."),
                    hint: Some(String::from("consider begging for more permissions at your local Discord administrator!")),
                    arg: None
                }).await;
                return;
            }
        }

        let mut transformers = c.get_transformers().into_iter();
        let mut args: Vec<Token> = vec![];

        while let Some(_) = parts.peek() {
            if let Some(transformer) = transformers.next() {
                let result = transformer(&ctx, &msg, &mut parts).await;

                match result {
                    Ok(r) => {
                        args.push(r);
                    },
                    Err(TransformerError::MissingArgumentError(err)) => {
                        handler.send_error(ctx, msg, contents, CommandError::arg_not_found(&err.0, None)).await;
                        return;
                    },
                    Err(TransformerError::CommandError(err)) => {
                        handler.send_error(ctx, msg, contents, err).await;
                        return;
                    }
                }
            } else if let Some(mut arg) = parts.next() {
                arg.contents = Some(CommandArgument::String(arg.raw.clone()));
                args.push(arg);
            }
        }

        while let Some(transformer) = transformers.next() {
            let result = transformer(&ctx, &msg, &mut parts).await;

            match result {
                Ok(r) => {
                    args.push(r);
                },
                Err(TransformerError::CommandError(err)) => {
                    handler.send_error(ctx, msg, contents, err).await;
                    return;
                },
                Err(TransformerError::MissingArgumentError(_)) => {
                    args.push(Token { contents: Some(CommandArgument::None), raw: String::new(), position: 0, length: 0, iteration: 0 });
                }
            }
        }

        let res = c.run(ctx.clone(), msg.clone(), args).await;

        if let Err(err) = res {
            handler.send_error(ctx, msg, contents, err).await;
        }
    }
}

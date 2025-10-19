use std::collections::HashMap;

use serenity::all::{Context, CreateAllowedMentions, CreateMessage, Message};
use tracing::warn;

use crate::{
    commands::{CommandArgument, TransformerError},
    event_handler::{CommandError, Handler},
    lexer::{Token, lex},
    utils::{
        check_channel_permission, check_guild_permission, extract_command_parameters, is_developer,
    },
};

pub async fn message(handler: &Handler, ctx: Context, mut msg: Message) {
    if !msg.content.starts_with(handler.prefix.as_str()) || msg.guild_id.is_none() {
        return;
    }

    let mut contents = msg.content.clone();
    let strip = contents.strip_prefix(handler.prefix.as_str()).unwrap_or("");
    let tokens = lex(String::from(strip));
    let mut parts = tokens.into_iter().peekable();
    let command_name = parts.next().map(|s| s.raw).unwrap_or_default();

    if command_name == "help" {
        if let Err(err) = handler
            .help_run(ctx.clone(), msg.clone(), parts.collect())
            .await
        {
            handler.send_error(ctx, msg, contents, err).await;
        }

        return;
    } else if command_name == "cachedbg" && is_developer(&msg.author) {
        let lock = handler.message_cache.lock().await;
        let mut sizes = lock.get_sizes();
        let size = sizes.entry(msg.channel_id.get()).or_insert(100);
        let count = lock.get_channel_len(msg.channel_id.get());
        let mut inserts = lock.get_inserts();
        let insert_count = inserts.entry(msg.channel_id.get()).or_insert(0);

        let reply = CreateMessage::new()
            .content(format!(
                "Size: {}; Count: {}; Inserts: {}",
                *size, count, *insert_count
            ))
            .reference_message(&msg)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

        if let Err(err) = msg.channel_id.send_message(&ctx, reply).await {
            warn!("Could not send message; err = {err:?}");
        }

        return;
    }

    let command = handler
        .commands
        .iter()
        .find(|c| c.get_name() == command_name.to_lowercase());

    if let Some(c) = command {
        let permissions = c.get_permissions();

        // Self user permission check commented out till i figure out what is actually causing the issues

        // let channel_id = msg.channel_id;
        // let guild_id = msg.guild_id.unwrap();
        // let current_user_id = ctx.cache.current_user().id;

        // let channel = channel_id.to_channel(&ctx).await.unwrap().guild().unwrap();

        // let member = match guild_id.member(&ctx, current_user_id).await {
        //     Ok(m) => m,
        //     Err(err) => {
        //         warn!("Couldnt get current bot member object; err = {err:?}");
        //         return;
        //     }
        // };

        // for perm in permissions.bot.iter() {
        //     if !check_channel_permission(&ctx, &channel, &member, *perm).await {
        //         handler
        //             .send_error(
        //                 ctx,
        //                 msg,
        //                 contents,
        //                 CommandError {
        //                     title: format!("I do not have a required permission to execute this command."),
        //                     hint: Some(format!("missing permission: {perm}")),
        //                     arg: None,
        //                 },
        //             )
        //             .await;
        //         return;
        //     }
        // }

        if !permissions.required.is_empty() || !permissions.one_of.is_empty() {
            let Ok(member) = msg.member(&ctx).await else {
                handler.send_error(ctx, msg, contents, CommandError {
                    title: String::from("You do not have permissions to execute this command."),
                    hint: Some(String::from("could not get user member object")),
                    arg: None
                }).await;
                return;
            };

            for permission in permissions.required {
                if !check_guild_permission(&ctx, &member, permission) {
                    handler.send_error(ctx, msg, contents, CommandError {
                        title: String::from("You do not have permissions to execute this command."),
                        hint: Some(format!("guild required permission check fail: {permission}")),
                        arg: None
                    }).await;
                    return;
                }
            }

            let mut pass = true;

            for permission in permissions.one_of {
                if !check_guild_permission(&ctx, &member, permission) {
                    pass = false;
                    break;
                }
            }

            if !pass {
                handler.send_error(ctx, msg, contents, CommandError {
                    title: String::from("You do not have permissions to execute this command."),
                    hint: Some(format!("guild one of permission check fail")),
                    arg: None
                }).await;
                return;
            }
        }

        let mut command_params = HashMap::new();

        if !c.get_params().is_empty() {
            let params = c.get_params();
            let res = extract_command_parameters(&ctx, &msg, strip.to_string(), params).await;

            if let Ok(params) = res {
                command_params = params.0;
                contents = format!("{}{}", handler.prefix, params.1.clone());
                msg.content = contents.clone();
                parts = lex(params.1).into_iter().peekable();
                parts.next();
            }
        }

        let mut transformers = c.get_transformers().into_iter();
        let mut args: Vec<Token> = vec![];

        while parts.peek().is_some() {
            if let Some(transformer) = transformers.next() {
                let result = transformer(&ctx, &msg, &mut parts).await;

                match result {
                    Ok(r) => {
                        args.push(r);
                    }
                    Err(TransformerError::MissingArgumentError(err)) => {
                        handler
                            .send_error(
                                ctx,
                                msg,
                                contents,
                                CommandError::arg_not_found(&err.0, None),
                            )
                            .await;
                        return;
                    }
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

        for transformer in transformers {
            let result = transformer(&ctx, &msg, &mut parts).await;

            match result {
                Ok(r) => {
                    args.push(r);
                }
                Err(TransformerError::CommandError(err)) => {
                    handler.send_error(ctx, msg, contents, err).await;
                    return;
                }
                Err(TransformerError::MissingArgumentError(_)) => {
                    args.push(Token {
                        contents: Some(CommandArgument::None),
                        raw: String::new(),
                        position: 0,
                        length: 0,
                        iteration: 0,
                        quoted: false,
                        inferred: None,
                    });
                }
            }
        }

        let res = c.run(ctx.clone(), msg.clone(), args, command_params).await;

        if let Err(err) = res {
            handler.send_error(ctx, msg, contents, err).await;
        }
    }
}

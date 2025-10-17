use std::{collections::HashMap, time::Duration};

use serenity::all::{Context, CreateAllowedMentions, CreateEmbed, CreateMessage, Message, User};
use tokio::time::sleep;
use tracing::warn;

use crate::{
    commands::{CommandArgument, CommandParameter, TransformerError},
    constants::BRAND_BLUE,
    lexer::lex,
};

pub async fn message_and_dm(
    ctx: &Context,
    command_msg: &Message,
    dm_user: &User,
    server_msg: impl Fn(String) -> String,
    dm_msg: String,
    automatically_delete: bool,
    silent: bool,
) {
    let mut addition = String::new();

    if !silent {
        let dm = CreateMessage::new()
            .add_embed(CreateEmbed::new().description(dm_msg).color(BRAND_BLUE));

        if dm_user.direct_message(&ctx, dm).await.is_err() {
            addition = String::from(" | DM failed");
        }
    } else {
        addition = String::from(" | silent")
    }

    let embed = CreateEmbed::new()
        .description(server_msg(addition))
        .color(BRAND_BLUE);

    let reply = CreateMessage::new()
        .add_embed(embed)
        .reference_message(command_msg)
        .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

    let msg = match command_msg.channel_id.send_message(&ctx, reply).await {
        Ok(m) => m,
        Err(err) => {
            warn!("Could not send message; err = {err:?}");
            return;
        }
    };

    if automatically_delete {
        let ctx = ctx.clone();
        let cmd_msg = command_msg.clone();

        tokio::spawn(async move {
            sleep(Duration::from_secs(5)).await;
            let _ = msg.delete(&ctx).await;
            let _ = cmd_msg.delete(&ctx).await;
        });
    }
}

pub async fn extract_command_parameters<'a>(
    context: &Context,
    msg: &Message,
    contents: String,
    params: Vec<&CommandParameter<'static>>,
) -> Result<(HashMap<&'a str, (bool, CommandArgument)>, String), TransformerError> {
    let mut found_args: HashMap<&str, (bool, CommandArgument)> = HashMap::default();
    let mut lex = lex(contents.clone()).into_iter().peekable();
    let mut to_remove = Vec::new();

    while let Some(token) = lex.next() {
        let Some((positive, arg_name)) = ({
            token
                .raw
                .strip_prefix("-")
                .map(|a| (false, a))
                .or(token.raw.strip_prefix("+").map(|a| (true, a)))
        }) else {
            continue;
        };

        for param in params.iter() {
            if param.name == arg_name || param.short == arg_name {
                let cloned = lex.clone();
                let contents_arg = (*param.transformer)(context, msg, &mut lex)
                    .await
                    .map(|t| t.contents.unwrap_or(CommandArgument::None))
                    .unwrap_or(CommandArgument::None);

                if lex.len() == cloned.len() {
                    to_remove.push((token.position, token.position + token.length));
                    found_args.insert(param.name, (positive, contents_arg));
                    continue;
                }

                let mut last_consumed = None;
                let mut cloned_iter = cloned.rev().into_iter();
                let pos_to_search = lex.peek().map(|t| t.position).unwrap_or(0);

                while let Some(token) = cloned_iter.next() {
                    let curr_pos = token.position;
                    last_consumed = Some(token);

                    if curr_pos == pos_to_search {
                        last_consumed = Some(cloned_iter.next().or(last_consumed).unwrap());
                        break;
                    }
                }

                found_args.insert(param.name, (positive, contents_arg));
                let last_position = last_consumed
                    .clone()
                    .map(|t| t.position)
                    .unwrap_or(token.position);
                let last_length = last_consumed.map(|t| t.length).unwrap_or(token.length);
                to_remove.push((token.position, last_position + last_length));
            }
        }
    }

    to_remove.sort_by_key(|r| r.0);
    let mut stripped = String::new();
    let mut last_end = 0;
    for (start, end) in to_remove {
        if start > last_end {
            stripped.push_str(&contents[last_end..start]);
        }
        last_end = end;
    }
    if last_end < contents.len() {
        stripped.push_str(&contents[last_end..]);
    }

    Ok((found_args, stripped.trim().to_string()))
}

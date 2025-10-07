use std::{collections::HashMap, time::Duration};

use serenity::all::{Context, CreateAllowedMentions, CreateEmbed, CreateMessage, Message, User};
use tokio::time::sleep;
use tracing::warn;

use crate::{commands::{CommandArgument, CommandParameter, TransformerError}, constants::BRAND_BLUE, lexer::lex};

pub async fn message_and_dm(
    ctx: &Context,
    command_msg: &Message,
    dm_user: &User,
    server_msg: impl Fn(String) -> String,
    dm_msg: String,
    automatically_delete: bool,
    silent: bool
) {
    let mut addition = String::new();

    if !silent {
        let dm =
            CreateMessage::new().add_embed(CreateEmbed::new().description(dm_msg).color(BRAND_BLUE));

        if dm_user.direct_message(&ctx.http, dm).await.is_err() {
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

    let msg = match command_msg.channel_id.send_message(&ctx.http, reply).await {
        Ok(m) => m,
        Err(err) => {
            warn!("Could not send message; err = {err:?}");
            return;
        }
    };

    if automatically_delete {
        let http = ctx.http.clone();
        let cmd_msg = command_msg.clone();

        tokio::spawn(async move {
            sleep(Duration::from_secs(5)).await;
            let _ = msg.delete(&http).await;
            let _ = cmd_msg.delete(&http).await;
        });
    }
}

pub async fn get_args<'a>(
    context: &Context,
    msg: &Message,
    contents: String,
    params: Vec<&CommandParameter<'static>>
) -> Result<(HashMap<&'a str, (bool, CommandArgument)>, String), TransformerError> {
    let mut found_args: HashMap<&str, (bool, CommandArgument)> = HashMap::default();
    let mut lex = lex(contents.clone()).into_iter().peekable();
    let mut to_remove = Vec::new();

    while let Some(token) = lex.next() {
        let Some((positive, arg_name)) = ({
            if let Some(arg) = token.raw.strip_prefix("-") {
                Some((false, arg))
            } else if let Some(arg) = token.raw.strip_prefix("+") {
                Some((true, arg))
            } else { None }
        }) else { continue };

        for param in params.iter() {
            if param.name == arg_name || param.short == arg_name {
                let cloned = lex.clone();
                let contents_arg = (*param.transformer)(context, msg, &mut lex.clone()).await
                    .map(|t| t.contents.unwrap_or(CommandArgument::None))
                    .unwrap_or(CommandArgument::None);

                let last_cloned = cloned.clone().last();
                let last_consumed = cloned.zip(lex.clone()).find(|(a, b)| a.position == b.position).map(|(a, _)| a).or(last_cloned);

                found_args.insert(param.name, (positive, contents_arg));
                let last_position = last_consumed.clone().map(|t| t.position).unwrap_or(token.position);
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


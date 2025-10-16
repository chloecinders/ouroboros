use serenity::{
    all::{Context, CreateAllowedMentions, CreateMessage, Message},
    async_trait,
};
use tracing::warn;

use crate::{
    commands::{
        Command, CommandArgument, CommandCategory, CommandParameter, CommandPermissions,
        CommandSyntax, TransformerFnArc,
    },
    event_handler::CommandError,
    lexer::Token,
    utils::{is_developer, permissions_for_channel},
};
use ouroboros_macros::command;

pub struct PermDbg;

impl PermDbg {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command for PermDbg {
    fn get_name(&self) -> &'static str {
        "permdbg"
    }

    fn get_short(&self) -> &'static str {
        "Gets permission debug information"
    }

    fn get_full(&self) -> &'static str {
        "Send this message in a channel to check the bots permissions of the channel."
    }

    fn get_syntax(&self) -> Vec<CommandSyntax> {
        vec![]
    }

    fn get_category(&self) -> CommandCategory {
        CommandCategory::Developer
    }

    fn get_params(&self) -> Vec<&'static CommandParameter<'static>> {
        vec![]
    }

    #[command]
    async fn run(&self, ctx: Context, msg: Message) -> Result<(), CommandError> {
        if is_developer(&msg.author) {
            let channel_id = msg.channel_id;
            let http = ctx.http.clone();
            let cache = ctx.cache.clone();

            let channel = channel_id.to_channel(&http).await.unwrap().guild().unwrap();
            let guild = {
                let cache_ref = cache.clone();
                channel.guild(&cache_ref).unwrap().clone()
            };

            let current_user_id = cache.current_user().id;
            let member = guild
                .member(&http, current_user_id)
                .await
                .unwrap()
                .into_owned();

            let permissions = permissions_for_channel(&ctx, channel, &member);

            let mut perms = permissions
                .iter()
                .map(|p| (p.0.bits(), p))
                .collect::<Vec<_>>();

            perms.sort_by_key(|k| k.0);

            let strings = perms
                .into_iter()
                .map(|(_, p)| {
                    format!(
                        "`{} (1<<{}) {}`",
                        if p.0.to_string().is_empty() {
                            String::from("UNKNOWN")
                        } else {
                            p.0.to_string()
                        },
                        p.0.bits().trailing_zeros(),
                        if *p.1 { "[x]" } else { "[ ]" }
                    )
                })
                .collect::<Vec<_>>();

            let reply = CreateMessage::new()
                .content(format!(
                    "Current Channel Permissions:\n{}",
                    strings.join(", ")
                ))
                .reference_message(&msg)
                .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

            if let Err(e) = msg.channel_id.send_message(&ctx.http, reply).await {
                warn!("{e:?}");
            }
        }

        Ok(())
    }

    fn get_permissions(&self) -> CommandPermissions {
        CommandPermissions {
            required: vec![],
            one_of: vec![],
            bot: CommandPermissions::baseline(),
        }
    }
}

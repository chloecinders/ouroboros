use std::{sync::Arc, time::Duration};

use serenity::all::{
    ButtonStyle, Context, CreateActionRow, CreateAllowedMentions, CreateButton, CreateEmbed,
    CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
    EditMessage, Message,
};
use tracing::warn;

use crate::{
    commands::{Command, CommandCategory},
    constants::BRAND_BLUE,
    event_handler::{CommandError, Handler},
    lexer::Token,
    utils::is_developer,
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

            let params = {
                if cmd.get_params().is_empty() {
                    String::new()
                } else {
                    let params = cmd
                        .get_params()
                        .iter()
                        .map(|p| format!("`+{}/+{}` -> {}", p.short, p.name, p.desc))
                        .collect::<Vec<_>>();

                    format!("\n\nOptional Parameters:\n{}", params.join("\n"))
                }
            };

            let reply = CreateMessage::new()
                .add_embed(
                    CreateEmbed::new()
                        .description(format!(
                            "**{}**\n{}{}\n\n{}{}",
                            cmd.get_name().to_uppercase(),
                            cmd.get_full(),
                            params,
                            syntax,
                            perms,
                        ))
                        .color(BRAND_BLUE),
                )
                .reference_message(&msg)
                .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

            if let Err(e) = msg.channel_id.send_message(&ctx.http, reply).await {
                warn!("Could not send message; err = {e:?}");
                return Err(CommandError {
                    title: String::from("Could not send message"),
                    hint: None,
                    arg: None,
                });
            }

            return Ok(());
        }

        let mut command_pages: Vec<(CommandCategory, Vec<&Arc<dyn Command>>)> = vec![];
        let mut current_page: usize = 0;

        self.commands.iter().for_each(|c| {
            if c.get_category() == CommandCategory::Developer && !is_developer(&msg.author) {
                return;
            }

            if let Some((_, vec)) = command_pages
                .iter_mut()
                .find(|(k, _)| k == &c.get_category())
            {
                vec.push(c);
            } else {
                command_pages.push((c.get_category(), vec![c]));
            }
        });

        let get_page_body = |page: usize| {
            let Some(page) = command_pages.get(page) else {
                return String::new();
            };

            let mut body = format!("**{}**\n", page.0.to_string().to_uppercase());

            page.1.iter().for_each(|c| {
                if !c.get_short().is_empty() {
                    body.push_str(format!("`{}` - {}\n", c.get_name(), c.get_short()).as_str());
                }
            });

            body
        };

        let mut page_buttons = vec![
            CreateButton::new("first")
                .style(ButtonStyle::Secondary)
                .label("<<")
                .disabled(true),
            CreateButton::new("prev")
                .style(ButtonStyle::Secondary)
                .label("<")
                .disabled(true),
            CreateButton::new("page")
                .style(ButtonStyle::Secondary)
                .label(format!("1/{}", command_pages.len()))
                .disabled(true),
            CreateButton::new("next")
                .style(ButtonStyle::Secondary)
                .label(">"),
            CreateButton::new("last")
                .style(ButtonStyle::Secondary)
                .label(">>"),
        ];

        let get_updated_buttons = |page: usize, disabled: [bool; 4]| -> Vec<CreateButton> {
            let disabled: [bool; 5] = [disabled[0], disabled[1], true, disabled[2], disabled[3]];
            let mut new = page_buttons.clone();

            new = new
                .into_iter()
                .enumerate()
                .map(|(i, mut d)| {
                    if i == 2 {
                        d = d.label(format!("{}/{}", page + 1, command_pages.len()));
                    }

                    d.disabled(disabled[i])
                })
                .collect();

            new
        };

        let reply = CreateMessage::new()
            .add_embed(
                CreateEmbed::new()
                    .description(get_page_body(current_page))
                    .footer(CreateEmbedFooter::new(format!(
                        "View additional information using {}help <command: String>",
                        self.prefix
                    )))
                    .color(BRAND_BLUE),
            )
            .components(vec![CreateActionRow::Buttons(page_buttons.clone())])
            .reference_message(&msg)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(false));

        let mut new_msg = match msg.channel_id.send_message(&ctx.http, reply).await {
            Ok(m) => m,
            Err(e) => {
                warn!("Could not send message; err = {e:?}");
                return Err(CommandError {
                    title: String::from("Could not send message"),
                    hint: None,
                    arg: None,
                });
                return Ok(());
            }
        };

        loop {
            let interaction = match new_msg
                .await_component_interaction(&ctx.shard)
                .timeout(Duration::from_secs(60 * 5))
                .await
            {
                Some(i) => i,
                None => {
                    page_buttons = page_buttons.into_iter().map(|b| b.disabled(true)).collect();
                    let _ = new_msg
                        .edit(
                            &ctx.http,
                            EditMessage::new()
                                .components(vec![CreateActionRow::Buttons(page_buttons)]),
                        )
                        .await;
                    return Ok(());
                }
            };

            if interaction.user.id != msg.author.id {
                if let Err(e) = interaction
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("You are not the author of the original message!")
                                .ephemeral(true),
                        ),
                    )
                    .await
                {
                    warn!("Could not send message; err = {e:?}");
                    return Err(CommandError {
                        title: String::from("Could not send message"),
                        hint: None,
                        arg: None,
                    });
                }

                continue;
            }

            match interaction.data.custom_id.as_str() {
                "first" => {
                    current_page = 0;
                    let response = get_page_body(current_page);
                    if interaction
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::UpdateMessage(
                                CreateInteractionResponseMessage::default()
                                    .add_embed(
                                        CreateEmbed::new()
                                            .description(response)
                                            .footer(CreateEmbedFooter::new(format!("View additional information using {}help <command: String>", self.prefix)))
                                            .color(BRAND_BLUE),
                                    )
                                    .components(vec![
                                        CreateActionRow::Buttons(get_updated_buttons(
                                            current_page,
                                            [true, true, false, false],
                                        )),
                                    ]),
                            ),
                        )
                        .await
                        .is_err()
                    {
                        return Ok(());
                    }
                }

                "prev" => {
                    current_page -= 1;
                    let response = get_page_body(current_page);
                    let none_prev = if current_page == 0 {
                        true
                    } else {
                        command_pages.get(current_page - 1).is_none()
                    };
                    if interaction
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::UpdateMessage(
                                CreateInteractionResponseMessage::default()
                                    .add_embed(
                                        CreateEmbed::new()
                                            .description(response)
                                            .footer(CreateEmbedFooter::new(format!("View additional information using {}help <command: String>", self.prefix)))
                                            .color(BRAND_BLUE),
                                    )
                                    .components(vec![
                                        CreateActionRow::Buttons(get_updated_buttons(
                                            current_page,
                                            [none_prev, none_prev, false, false],
                                        )),
                                    ]),
                            ),
                        )
                        .await
                        .is_err()
                    {
                        return Ok(());
                    }
                }

                "next" => {
                    current_page += 1;
                    let response = get_page_body(current_page);
                    let none_next = command_pages.get(current_page + 1).is_none();
                    if interaction
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::UpdateMessage(
                                CreateInteractionResponseMessage::default()
                                    .add_embed(
                                        CreateEmbed::new()
                                            .description(response)
                                            .footer(CreateEmbedFooter::new(format!("View additional information using {}help <command: String>", self.prefix)))
                                            .color(BRAND_BLUE),
                                    )
                                    .components(vec![
                                        CreateActionRow::Buttons(get_updated_buttons(
                                            current_page,
                                            [false, false, none_next, none_next],
                                        )),
                                    ]),
                            ),
                        )
                        .await
                        .is_err()
                    {
                        return Ok(());
                    }
                }

                "last" => {
                    current_page = command_pages.len() - 1;
                    let response = get_page_body(current_page);
                    if interaction
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::UpdateMessage(
                                CreateInteractionResponseMessage::default()
                                    .add_embed(
                                        CreateEmbed::new()
                                            .description(response)
                                            .footer(CreateEmbedFooter::new(format!("View additional information using {}help <command: String>", self.prefix)))
                                            .color(BRAND_BLUE),
                                    )
                                    .components(vec![
                                        CreateActionRow::Buttons(get_updated_buttons(
                                            current_page,
                                            [false, false, true, true],
                                        )),
                                    ]),
                            ),
                        )
                        .await
                        .is_err()
                    {
                        return Ok(());
                    }
                }

                _ => {}
            };
        }
    }
}

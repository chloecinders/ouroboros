use std::{iter::Peekable, vec::IntoIter};

use serenity::all::{Context, Message};

use crate::{
    commands::{CommandArgument, TransformerError, TransformerReturn},
    event_handler::{CommandError, MissingArgumentError},
    lexer::Token,
    transformers::Transformers,
};

impl Transformers {
    pub fn user<'a>(
        ctx: &'a Context,
        _msg: &'a Message,
        args: &'a mut Peekable<IntoIter<Token>>,
    ) -> TransformerReturn<'a> {
        Box::pin(async move {
            let Some(mut input) = args.next() else {
                return Err(TransformerError::MissingArgumentError(
                    MissingArgumentError(String::from("User")),
                ));
            };

            let id = if let Ok(id) = input.raw.parse::<u64>() {
                id
            } else if input.raw.starts_with("<@") && input.raw.ends_with(">") {
                let new_input = input
                    .raw
                    .strip_prefix("<@")
                    .unwrap()
                    .strip_suffix(">")
                    .unwrap();

                if let Ok(id) = new_input.parse::<u64>() {
                    id
                } else {
                    return Err(TransformerError::CommandError(CommandError {
                        arg: Some(input),
                        title: String::from("Could not turn input to a <Discord User>"),
                        hint: Some(String::from("provide a valid ID or mention")),
                    }));
                }
            } else {
                let users = ctx.cache.users();
                let opt_user = users.iter().find(|u| u.name == input.raw);

                if let Some(user) = opt_user {
                    input.contents = Some(CommandArgument::User(user.clone()));
                    return Ok(input);
                }

                return Err(TransformerError::CommandError(CommandError {
                    arg: Some(input),
                    title: String::from("Could not turn input to a <Discord User>"),
                    hint: Some(String::from("provide a valid ID or mention")),
                }));
            };

            let user = {
                if let Some(user) = ctx.cache.user(id) {
                    user.clone()
                } else if let Ok(user) = ctx.http.get_user(id.into()).await {
                    user.clone()
                } else {
                    return Err(TransformerError::CommandError(CommandError {
                        arg: Some(input),
                        title: String::from("Could not find the <Discord User>"),
                        hint: Some(String::from(
                            "make sure the ID or mention you provided is valid and that its associated user exists!",
                        )),
                    }));
                }
            };

            input.contents = Some(CommandArgument::User(user));
            Ok(input)
        })
    }
}

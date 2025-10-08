// use std::{iter::Peekable, vec::IntoIter};

// use serenity::all::{Context, Message, User};

// use crate::{
//     commands::{CommandArgument, TransformerReturn},
//     lexer::Token,
//     transformers::Transformers,
// };

// impl Transformers {
//     pub fn many_users<'a>(
//         ctx: &'a Context,
//         _msg: &'a Message,
//         args: &'a mut Peekable<IntoIter<Token>>,
//     ) -> TransformerReturn<'a> {
//         Box::pin(async move {
//             let mut users: Vec<User> = vec![];

//             let mut out = Token {
//                 contents: None,
//                 raw: String::new(),
//                 position: 0,
//                 length: 0,
//                 iteration: 0,
//                 quoted: false,
//                 inferred: None,
//             };

//             while let Some(input) = args.peek() {
//                 let id = if let Ok(id) = input.raw.parse::<u64>() {
//                     id
//                 } else if input.raw.starts_with("<@") && input.raw.ends_with(">") {
//                     let new_input = input
//                         .raw
//                         .strip_prefix("<@")
//                         .unwrap()
//                         .strip_suffix(">")
//                         .unwrap();

//                     if let Ok(id) = new_input.parse::<u64>() {
//                         id
//                     } else {
//                         break;
//                     }
//                 } else {
//                     let user_cache = ctx.cache.users();
//                     let opt_user = user_cache.iter().find(|u| u.name == input.raw);

//                     if let Some(user) = opt_user {
//                         users.push(user.clone());
//                         args.next();
//                         continue;
//                     }

//                     continue;
//                 };

//                 let user = {
//                     if let Some(user) = ctx.cache.user(id) {
//                         user.clone()
//                     } else if let Ok(user) = ctx.http.get_user(id.into()).await {
//                         user.clone()
//                     } else {
//                         continue;
//                     }
//                 };

//                 args.next();
//                 users.push(user);
//             }

//             out.contents = Some(CommandArgument::ManyUsers(users));
//             Ok(out)
//         })
//     }
// }

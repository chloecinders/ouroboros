// use std::{iter::Peekable, vec::IntoIter};

// use serenity::all::{Context, Message};

// use crate::{
//     commands::{CommandArgument, TransformerError, TransformerReturn},
//     event_handler::MissingArgumentError,
//     lexer::Token,
//     transformers::Transformers,
// };

// impl Transformers {
//     pub fn bool<'a>(
//         _ctx: &'a Context,
//         _msg: &'a Message,
//         args: &'a mut Peekable<IntoIter<Token>>,
//     ) -> TransformerReturn<'a> {
//         Box::pin(async move {
//             let Some(mut input) = args.next() else {
//                 return Err(TransformerError::MissingArgumentError(
//                     MissingArgumentError(String::from("String")),
//                 ));
//             };

//             let res = matches!(
//                 input.raw.to_lowercase().as_str(),
//                 "true"
//                     | "y"
//                     | "yes"
//                     | "yeah"
//                     | "t"
//                     | "ok"
//                     | "on"
//                     | "enabled"
//                     | "1"
//                     | "enable"
//                     | "check"
//                     | "checked"
//                     | "sure"
//                     | "yep"
//                     | "aye"
//                     | "valid"
//                     | "correct"
//             );

//             input.contents = Some(CommandArgument::bool(res));
//             Ok(input)
//         })
//     }
// }

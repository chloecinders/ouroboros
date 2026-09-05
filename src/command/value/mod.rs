mod discord;
mod scalar;

use crate::command::args::{Arg, ArgKind, Inferred};
use crate::command::cx::Cx;
use crate::command::error::{Error, Result};
use crate::command::flags::Flags;
use crate::command::stream::Stream;
use crate::platform::text::lexer::{Span, Token};

pub trait FromArgs: Sized + Send {
    const KIND: ArgKind;

    fn from_token(
        cx: &Cx,
        field: &'static str,
        token: &Token,
    ) -> impl Future<Output = Result<Self>> + Send;

    fn from_message(_cx: &Cx) -> impl Future<Output = Result<Option<Self>>> + Send {
        async { Ok(None) }
    }
}

pub trait FromRest: FromArgs {
    fn from_rest(text: &str, span: Span) -> Option<Self>;
}

pub trait FromReply: FromArgs {
    fn from_reply(
        cx: &Cx,
        field: &'static str,
        span: Span,
    ) -> impl Future<Output = Result<Option<(Self, Inferred)>>> + Send;
}

pub trait Snapshot {
    fn snapshot(&self) -> serde_json::Value;
}

pub async fn positional<T: FromArgs>(
    cx: &Cx,
    stream: &mut Stream,
    field: &'static str,
) -> Result<T> {
    let span = stream.cursor_span();
    let Some(token) = stream.advance() else {
        return match T::from_message(cx).await? {
            Some(value) => Ok(value),
            None => Err(Error::missing(cx.input(), field, T::KIND, span)),
        };
    };

    T::from_token(cx, field, &token).await
}

pub async fn optional<T: FromArgs>(
    cx: &Cx,
    stream: &mut Stream,
    field: &'static str,
) -> Result<Option<T>> {
    let Some(token) = stream.advance() else {
        return T::from_message(cx).await;
    };

    T::from_token(cx, field, &token).await.map(Some)
}

pub async fn skippable<T: FromArgs>(
    cx: &Cx,
    stream: &mut Stream,
    field: &'static str,
) -> Result<Option<T>> {
    let Some(token) = stream.peek().cloned() else {
        return T::from_message(cx).await;
    };

    let Ok(value) = T::from_token(cx, field, &token).await else {
        return Ok(None);
    };

    let _ = stream.advance();

    Ok(Some(value))
}

pub async fn rest<T: FromRest>(cx: &Cx, stream: &mut Stream, field: &'static str) -> Result<T> {
    let span = stream.cursor_span();
    let (text, span) = stream.take_rest().unwrap_or((String::new(), span));

    T::from_rest(&text, span).ok_or_else(|| Error::missing(cx.input(), field, T::KIND, span))
}

pub async fn rest_optional<T: FromRest>(
    _cx: &Cx,
    stream: &mut Stream,
    _field: &'static str,
) -> Result<Option<T>> {
    let span = stream.cursor_span();
    let (text, span) = stream.take_rest().unwrap_or((String::new(), span));

    Ok(T::from_rest(&text, span))
}

pub async fn reply<T: FromReply>(
    cx: &Cx,
    stream: &mut Stream,
    field: &'static str,
) -> Result<Arg<T>> {
    if let Some((value, source)) = T::from_reply(cx, field, stream.cursor_span()).await? {
        return Ok(Arg::from_reply(value, source));
    }

    positional::<T>(cx, stream, field).await.map(Arg::stated)
}

#[allow(dead_code)]
pub async fn reply_optional<T: FromReply>(
    cx: &Cx,
    stream: &mut Stream,
    field: &'static str,
) -> Result<Option<Arg<T>>> {
    if let Some((value, source)) = T::from_reply(cx, field, stream.cursor_span()).await? {
        return Ok(Some(Arg::from_reply(value, source)));
    }

    Ok(optional::<T>(cx, stream, field).await?.map(Arg::stated))
}

pub async fn flag<T: FromArgs>(cx: &Cx, flags: &Flags, name: &'static str) -> Result<Option<T>> {
    if !flags.is_set(name) {
        return Ok(None);
    }

    let Some(token) = flags.value(name) else {
        return match T::from_message(cx).await? {
            Some(value) => Ok(Some(value)),
            None => Err(Error::missing(cx.input(), name, T::KIND, flags.after(name))),
        };
    };

    T::from_token(cx, name, token).await.map(Some)
}

pub fn switch(flags: &Flags, name: &str) -> bool {
    flags.is_set(name)
}

impl<T: Snapshot> Snapshot for Option<T> {
    fn snapshot(&self) -> serde_json::Value {
        match self {
            Some(value) => value.snapshot(),
            None => serde_json::Value::Null,
        }
    }
}

impl<T: Snapshot> Snapshot for Arg<T> {
    fn snapshot(&self) -> serde_json::Value {
        self.value.snapshot()
    }
}

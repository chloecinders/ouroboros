use chrono::Duration;

use crate::command::args::{ArgKind, Inferred};
use crate::command::cx::Cx;
use crate::command::error::{Error, Result};
use crate::command::value::{FromArgs, FromReply, FromRest, Snapshot};
use crate::domain::Snowflake;
use crate::domain::ids::{ActionId, MessageId};
use crate::domain::reason::{Note, Reason};
use crate::features::records::store;
use crate::platform::text::duration;
use crate::platform::text::lexer::{Span, Token};

impl FromArgs for String {
    const KIND: ArgKind = ArgKind::Text;

    async fn from_token(_cx: &Cx, _field: &'static str, token: &Token) -> Result<Self> {
        Ok(token.raw.clone())
    }
}

impl FromRest for String {
    fn from_rest(text: &str, _span: Span) -> Option<Self> {
        match text.is_empty() {
            true => None,
            false => Some(text.to_string()),
        }
    }
}

impl Snapshot for String {
    fn snapshot(&self) -> serde_json::Value {
        serde_json::Value::String(self.clone())
    }
}

impl FromArgs for bool {
    const KIND: ArgKind = ArgKind::Boolean;

    async fn from_token(_cx: &Cx, _field: &'static str, token: &Token) -> Result<Self> {
        Ok([
            "true", "y", "yes", "yeah", "t", "ok", "on", "enabled", "1", "enable", "check",
            "checked", "sure", "yep", "aye", "valid", "correct",
        ]
        .contains(&token.raw.to_lowercase().as_str()))
    }
}

impl Snapshot for bool {
    fn snapshot(&self) -> serde_json::Value {
        serde_json::Value::Bool(*self)
    }
}

macro_rules! whole_number {
    ($ty:ty) => {
        impl FromArgs for $ty {
            const KIND: ArgKind = ArgKind::Number;

            async fn from_token(cx: &Cx, field: &'static str, token: &Token) -> Result<Self> {
                token
                    .raw
                    .parse::<$ty>()
                    .map_err(|_| Error::invalid(cx.input(), field, ArgKind::Number, token.span))
            }
        }

        impl Snapshot for $ty {
            fn snapshot(&self) -> serde_json::Value {
                serde_json::Value::from(*self)
            }
        }
    };
}

whole_number!(u8);
whole_number!(u32);
whole_number!(i32);
whole_number!(i64);
whole_number!(u64);

impl FromArgs for Duration {
    const KIND: ArgKind = ArgKind::Duration;

    async fn from_token(cx: &Cx, field: &'static str, token: &Token) -> Result<Self> {
        duration::parse(&token.raw)
            .ok_or_else(|| Error::invalid(cx.input(), field, ArgKind::Duration, token.span))
    }
}

impl Snapshot for Duration {
    fn snapshot(&self) -> serde_json::Value {
        serde_json::Value::from(self.num_seconds())
    }
}

impl FromArgs for Reason {
    const KIND: ArgKind = ArgKind::Reason;

    async fn from_token(_cx: &Cx, _field: &'static str, token: &Token) -> Result<Self> {
        Ok(Reason::new(&token.raw))
    }
}

impl FromRest for Reason {
    fn from_rest(text: &str, _span: Span) -> Option<Self> {
        Some(Reason::new(text))
    }
}

impl Snapshot for Reason {
    fn snapshot(&self) -> serde_json::Value {
        serde_json::Value::String(self.as_str().to_string())
    }
}

impl FromArgs for Note {
    const KIND: ArgKind = ArgKind::Note;

    async fn from_token(cx: &Cx, field: &'static str, token: &Token) -> Result<Self> {
        Note::new(&token.raw)
            .ok_or_else(|| Error::invalid(cx.input(), field, ArgKind::Note, token.span))
    }
}

impl FromRest for Note {
    fn from_rest(text: &str, _span: Span) -> Option<Self> {
        Note::new(text)
    }
}

impl Snapshot for Note {
    fn snapshot(&self) -> serde_json::Value {
        serde_json::Value::String(self.as_str().to_string())
    }
}

impl FromArgs for ActionId {
    const KIND: ArgKind = ArgKind::ActionId;

    async fn from_token(cx: &Cx, field: &'static str, token: &Token) -> Result<Self> {
        if token.raw.len() != 6 {
            return Err(Error::invalid(
                cx.input(),
                field,
                ArgKind::ActionId,
                token.span,
            ));
        }

        Ok(ActionId::from(token.raw.clone()))
    }
}

impl FromReply for ActionId {
    async fn from_reply(
        cx: &Cx,
        _field: &'static str,
        _span: Span,
    ) -> Result<Option<(Self, Inferred)>> {
        let Some(reply) = cx.msg.referenced_message.as_deref() else {
            return Ok(None);
        };

        let found = store::action_for_message(cx.pool(), reply.id.get()).await?;

        Ok(found.map(|id| (id, Inferred::Bot)))
    }
}

impl Snapshot for ActionId {
    fn snapshot(&self) -> serde_json::Value {
        serde_json::Value::String(self.as_str().to_string())
    }
}

impl FromArgs for MessageId {
    const KIND: ArgKind = ArgKind::MessageId;

    async fn from_token(cx: &Cx, field: &'static str, token: &Token) -> Result<Self> {
        let invalid = || Error::invalid(cx.input(), field, ArgKind::MessageId, token.span);

        if !(17..=20).contains(&token.raw.len()) {
            return Err(invalid());
        }

        token
            .raw
            .parse::<Snowflake>()
            .map(MessageId::new)
            .map_err(|_| invalid())
    }
}

impl FromReply for MessageId {
    async fn from_reply(
        cx: &Cx,
        _field: &'static str,
        _span: Span,
    ) -> Result<Option<(Self, Inferred)>> {
        let Some(reply) = cx.msg.referenced_message.as_deref() else {
            return Ok(None);
        };

        Ok(Some((MessageId::new(reply.id.get()), Inferred::Message)))
    }
}

impl Snapshot for MessageId {
    fn snapshot(&self) -> serde_json::Value {
        serde_json::Value::from(self.get())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgKind {
    Member,
    User,
    Channel,
    Role,
    Duration,
    Number,
    Text,
    Color,
    Reason,
    Note,
    Reference,
    Boolean,
    ActionId,
    MessageId,
}

impl ArgKind {
    pub fn label(&self) -> &'static str {
        match self {
            ArgKind::Member => "Discord Member",
            ArgKind::User => "Discord User",
            ArgKind::Channel => "Channel",
            ArgKind::Role => "Role",
            ArgKind::Duration => "Duration",
            ArgKind::Number => "Number",
            ArgKind::Text => "String",
            ArgKind::Color => "Hex Color",
            ArgKind::Reason => "Reason",
            ArgKind::Note => "Note",
            ArgKind::Reference => "Reference",
            ArgKind::Boolean => "Yes/No",
            ArgKind::ActionId => "Log ID",
            ArgKind::MessageId => "Message ID",
        }
    }

    pub fn example(&self) -> &'static str {
        match self {
            ArgKind::Member | ArgKind::User => "@someone",
            ArgKind::Channel => "#some-channel",
            ArgKind::Role => "@some-role",
            ArgKind::Duration => "15m",
            ArgKind::Number => "5",
            ArgKind::Text => "\"something\"",
            ArgKind::Color => "#04D9B2",
            ArgKind::Reason => "user broke a rule",
            ArgKind::Note => "keep an eye on them",
            ArgKind::Reference => "https://discord.com/channels/1/2/3",
            ArgKind::Boolean => "yes",
            ArgKind::ActionId => "a7f3kQ",
            ArgKind::MessageId => "1329461552194687047",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Positional,
    Optional,
    Rest,
    Reply,
    Flag,
}

#[derive(Clone, Copy, Debug)]
pub struct Field {
    pub name: &'static str,
    pub kind: ArgKind,
    pub shape: Shape,
    pub short: Option<char>,
    pub desc: &'static str,
    pub amend: crate::domain::action::Amendment,
}

impl Field {
    pub fn syntax(&self) -> String {
        let inner = format!("{}: {}", self.name, self.kind.label());

        match self.shape {
            Shape::Positional => format!("<{inner}>"),
            Shape::Optional => format!("[{inner}]"),
            Shape::Rest => format!("...[{}]", self.name),
            Shape::Reply => format!("(<{inner}> || reply)"),
            Shape::Flag => match self.short {
                Some(short) => format!("[+{short}/+{}]", self.name),
                None => format!("[+{}]", self.name),
            },
        }
    }

    pub fn switch(&self) -> String {
        match self.short {
            Some(short) => format!("+{short}/+{}", self.name),
            None => format!("+{}", self.name),
        }
    }

    pub fn example(&self) -> Option<&'static str> {
        match self.shape {
            Shape::Flag => None,
            _ => Some(self.kind.example()),
        }
    }

    pub fn is_flag(&self) -> bool {
        matches!(self.shape, Shape::Flag)
    }
}

#[derive(Clone, Debug)]
pub struct Arg<T> {
    pub value: T,
    pub inferred: Option<Inferred>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inferred {
    Message,
    SystemMessage,
    Bot,
}

impl<T> Arg<T> {
    pub fn stated(value: T) -> Self {
        Self {
            value,
            inferred: None,
        }
    }

    pub fn from_reply(value: T, inferred: Inferred) -> Self {
        Self {
            value,
            inferred: Some(inferred),
        }
    }

    pub fn into_value(self) -> T {
        self.value
    }

    pub fn was_inferred(&self) -> bool {
        self.inferred.is_some()
    }
}

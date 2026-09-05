use serde::{Deserialize, Serialize};

#[derive(Hash, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogType {
    MemberModeration,
    MemberUpdate,
    MemberJoinLeave,
    MessageUpdate,
    AegisAnnouncements,
    Channels,
    Roles,
    VoiceActivity,
    Expressions,
    Errors,
}

pub const ALL: [LogType; 10] = [
    LogType::MemberModeration,
    LogType::MemberUpdate,
    LogType::MemberJoinLeave,
    LogType::MessageUpdate,
    LogType::AegisAnnouncements,
    LogType::Channels,
    LogType::Roles,
    LogType::VoiceActivity,
    LogType::Expressions,
    LogType::Errors,
];

impl LogType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogType::MemberModeration => "member_moderation",
            LogType::MemberUpdate => "member_update",
            LogType::MemberJoinLeave => "member_join_leave",
            LogType::MessageUpdate => "message_update",
            LogType::AegisAnnouncements => "aegis_announcements",
            LogType::Channels => "channels",
            LogType::Roles => "roles",
            LogType::VoiceActivity => "voice_activity",
            LogType::Expressions => "expressions",
            LogType::Errors => "errors",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        ALL.into_iter().find(|kind| kind.as_str() == raw)
    }

    pub fn title(&self) -> &'static str {
        match self {
            LogType::MemberModeration => "Member Moderation",
            LogType::MemberUpdate => "Member Update",
            LogType::MemberJoinLeave => "Member Join/Leave",
            LogType::MessageUpdate => "Message Update",
            LogType::AegisAnnouncements => "Aegis Announcements",
            LogType::Channels => "Channels",
            LogType::Roles => "Roles",
            LogType::VoiceActivity => "Voice Activity",
            LogType::Expressions => "Expressions",
            LogType::Errors => "Errors",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            LogType::MemberModeration => "New warns, bans, mutes, etc.",
            LogType::MemberUpdate => "Nickname, role changes",
            LogType::MemberJoinLeave => "Member joins and leaves",
            LogType::MessageUpdate => "Message deletions and edits",
            LogType::AegisAnnouncements => "Scheduled bot downtime, updates",
            LogType::Channels => "Channel create/update/delete events",
            LogType::Roles => "Role create/update/delete events",
            LogType::VoiceActivity => "Voice joins, leaves, moves, mutes, deafens",
            LogType::Expressions => "Emoji/sticker create, update, delete events",
            LogType::Errors => "Problems Aegis hit while working in this server",
        }
    }
}

use std::{
    collections::HashMap,
    sync::Arc,
};
use serenity::all::{Member, PartialGuild, Permissions};
use tokio::sync::Mutex;

use crate::{commands::Command, event_handler::Handler, utils::check_guild_permission};

pub struct PermissionCache {
    inner: HashMap<u64, GuildPermissionCache>,
}

impl PermissionCache {
    pub fn new() -> Self {
        Self { inner: Default::default() }
    }

    pub async fn can_run(&mut self, request: CommandPermissionRequest) -> bool {
        let guild_entry = self.inner.entry(request.guild.id.get()).or_default();
        guild_entry.can_run(request).await
    }

    pub async fn invalidate(&mut self, guild_id: u64, user_id: u64) {
        self.inner.entry(guild_id).or_default().invalidate(user_id).await;
    }

    pub async fn invalidate_guild(&mut self, guild_id: u64) {
        self.inner.insert(guild_id, Default::default());
    }
}

#[derive(Default)]
struct GuildPermissionCache {
    inner: HashMap<u64, Arc<Mutex<CommandPermissionCacheInfo>>>,
}

impl GuildPermissionCache {
    pub async fn can_run(&mut self, request: CommandPermissionRequest) -> bool {
        let perms = request.command.get_permissions();

        if perms.one_of.is_empty() && perms.required.is_empty() {
            return true;
        }

        let user_id = request.member.user.id.get();
        let command_name = request.command.get_name().to_string();
        let user_entry_arc = Arc::clone(self.inner.entry(user_id).or_default());
        let mut user_entry = user_entry_arc.lock().await;

        if !user_entry.valid {
            let allowed = Self::evaluate_permissions(request.clone()).await;
            user_entry.allowed.insert(command_name.clone(), allowed);
            user_entry.valid = true;

            let handler = request.handler.clone();
            let member = request.member.clone();
            let guild = request.guild.clone();
            let entry_ref = Arc::clone(&user_entry_arc);

            tokio::spawn(async move {
                let member = member;
                let guild = guild;

                for command in handler.commands.clone() {
                    let perms = command.get_permissions();

                    if perms.one_of.is_empty() && perms.required.is_empty() {
                        continue;
                    }

                    let req = CommandPermissionRequest {
                        command: command.clone(),
                        member: member.clone(),
                        guild: guild.clone(),
                        handler: handler.clone(),
                    };

                    let ok = Self::evaluate_permissions(req).await;

                    let mut lock = entry_ref.lock().await;
                    lock.allowed.insert(command.get_name().to_string(), ok);
                }
            });

            return allowed;
        }

        dbg!(&user_entry);

        match user_entry.allowed.get(&command_name) {
            Some(v) => *v,
            None => {
                let allowed = Self::evaluate_permissions(request).await;
                user_entry.allowed.insert(command_name, allowed);
                allowed
            }
        }
    }

    pub async fn invalidate(&mut self, user_id: u64) {
        let mut entry = self.inner.entry(user_id).or_default().lock().await;
        entry.valid = false;
        entry.allowed = Default::default();
    }

    async fn evaluate_permissions(request: CommandPermissionRequest) -> bool {
        let permissions = request.command.get_permissions();
        let guild = request.guild;
        let member = request.member;

        if check_guild_permission(&guild, &member, Permissions::ADMINISTRATOR).await
            || guild.owner_id.get() == member.user.id.get()
        {
            return true;
        }

        for permission in permissions.required {
            if !check_guild_permission(&guild, &member, permission).await {
                return false;
            }
        }

        if !permissions.one_of.is_empty() {
            for permission in permissions.one_of {
                if check_guild_permission(&guild, &member, permission).await {
                    return true;
                }
            }
            return false;
        }

        true
    }
}

#[derive(Default, Debug, Clone)]
struct CommandPermissionCacheInfo {
    pub allowed: HashMap<String, bool>,
    pub valid: bool,
}

#[derive(Clone)]
pub struct CommandPermissionRequest {
    pub command: Arc<dyn Command>,
    pub member: Member,
    pub guild: PartialGuild,
    pub handler: Handler,
}

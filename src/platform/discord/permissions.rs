use std::collections::HashMap;

use serenity::all::{
    GuildId, PermissionOverwrite, PermissionOverwriteType, Permissions, RoleId, UserId,
};

#[derive(Clone, Copy, Debug)]
pub struct Role {
    pub permissions: Permissions,
    pub position: i64,
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub guild: GuildId,
    pub owner: UserId,
    pub roles: HashMap<RoleId, Role>,
}

#[derive(Clone, Copy, Debug)]
pub struct Actor<'a> {
    pub id: UserId,
    pub roles: &'a [RoleId],
}

impl Snapshot {
    pub fn everyone(&self) -> RoleId {
        RoleId::new(self.guild.get())
    }

    pub fn base(&self, actor: Actor<'_>) -> Permissions {
        if actor.id == self.owner {
            return Permissions::all();
        }

        let mut granted = self
            .roles
            .get(&self.everyone())
            .map(|role| role.permissions)
            .unwrap_or_else(Permissions::empty);

        for id in actor.roles {
            if let Some(role) = self.roles.get(id) {
                granted |= role.permissions;
            }
        }

        granted
    }

    pub fn in_channel(&self, actor: Actor<'_>, overwrites: &[PermissionOverwrite]) -> Permissions {
        let mut granted = self.base(actor);

        if granted.contains(Permissions::ADMINISTRATOR) {
            return Permissions::all();
        }

        if let Some(everyone) = overwrites
            .iter()
            .find(|rule| rule.kind == PermissionOverwriteType::Role(self.everyone()))
        {
            granted = (granted & !everyone.deny) | everyone.allow;
        }

        let mut allow = Permissions::empty();
        let mut deny = Permissions::empty();

        for role in actor.roles {
            if let Some(rule) = overwrites
                .iter()
                .find(|rule| rule.kind == PermissionOverwriteType::Role(*role))
            {
                allow |= rule.allow;
                deny |= rule.deny;
            }
        }

        granted = (granted & !deny) | allow;

        if let Some(personal) = overwrites
            .iter()
            .find(|rule| rule.kind == PermissionOverwriteType::Member(actor.id))
        {
            granted = (granted & !personal.deny) | personal.allow;
        }

        granted
    }

    pub fn allows(
        &self,
        actor: Actor<'_>,
        overwrites: &[PermissionOverwrite],
        wanted: Permissions,
    ) -> bool {
        let granted = self.in_channel(actor, overwrites);

        granted.contains(Permissions::ADMINISTRATOR) || granted.contains(wanted)
    }

    pub fn can_target(&self, actor: Actor<'_>, target: Actor<'_>, wanted: Permissions) -> bool {
        if actor.id == self.owner {
            return true;
        }

        if target.id == self.owner {
            return false;
        }

        self.authority(actor, wanted) > self.authority(target, wanted)
    }

    pub fn can_enforce(&self, actor: Actor<'_>, target: Actor<'_>, wanted: Permissions) -> bool {
        if target.id == self.owner {
            return false;
        }

        let granted = self.base(actor);

        if !granted.contains(Permissions::ADMINISTRATOR) && !granted.contains(wanted) {
            return false;
        }

        if wanted.contains(Permissions::MODERATE_MEMBERS)
            && self.base(target).contains(Permissions::ADMINISTRATOR)
        {
            return false;
        }

        actor.id == self.owner || self.rank(actor) > self.rank(target)
    }

    fn rank(&self, actor: Actor<'_>) -> i64 {
        actor
            .roles
            .iter()
            .filter_map(|id| self.roles.get(id))
            .map(|role| role.position)
            .max()
            .unwrap_or(-1)
    }

    fn authority(&self, actor: Actor<'_>, wanted: Permissions) -> i64 {
        actor
            .roles
            .iter()
            .filter_map(|id| self.roles.get(id))
            .filter(|role| {
                role.permissions.contains(wanted)
                    || role.permissions.contains(Permissions::ADMINISTRATOR)
            })
            .map(|role| role.position)
            .max()
            .unwrap_or(-1)
    }
}

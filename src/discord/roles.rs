use std::collections::HashMap;

use serenity::{
    model::{
        prelude::{GuildId, Member, Role},
        Permissions,
    },
    prelude::Context,
};

pub async fn get_guild_roles(ctx: &Context, guild: GuildId) -> Vec<Role> {
    let roles = guild.roles(&ctx.http).await.expect("Error getting roles");

    let mut roles = roles.values().cloned().collect::<Vec<Role>>();
    roles.sort_by(|a, b| b.position.cmp(&a.position));

    roles
}

pub async fn get_user_roles(ctx: &Context, user: &Member) -> Vec<Role> {
    let roles = get_guild_roles(ctx, user.guild_id).await;

    roles
        .into_iter()
        .filter(|role| user.roles.contains(&role.id))
        .collect()
}

async fn get_all_permissions(ctx: &Context, member: &Member) -> Permissions {
    let mut permissions = Permissions::empty();
    let roles = get_user_roles(ctx, member).await;

    for role in roles {
        permissions |= role.permissions;
    }

    permissions
}

pub async fn get_permissions(ctx: &Context, member: &Member) -> HashMap<String, bool> {
    let mut permissions: HashMap<String, bool> = HashMap::new();

    let all = Permissions::all().get_permission_names();
    let user = get_all_permissions(&ctx, &member)
        .await
        .get_permission_names();

    if user.contains(&"Administrator") {
        for permission in all {
            permissions.insert(permission.to_string(), true);
        }
        return permissions;
    }

    for permission in all {
        let has_permission = user.contains(&permission);
        permissions.insert(permission.to_string(), has_permission);
    }

    permissions
}

pub trait RoleExt {
    fn as_mention(&self) -> String;
}

impl RoleExt for Role {
    fn as_mention(&self) -> String {
        if self.name == "@everyone" {
            return "@everyone".to_string();
        }

        format!("<@&{}>", self.id)
    }
}

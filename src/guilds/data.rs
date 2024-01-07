use std::{collections::HashMap, sync::Arc};

use serenity::{all::GuildId, prelude::TypeMapKey};
use tokio::sync::Mutex;

use super::music::manager::MusicManager;

pub struct GuildManager {
    guilds: HashMap<u64, GuildData>,
}

impl GuildManager {
    pub fn new() -> Self {
        GuildManager {
            guilds: HashMap::new(),
        }
    }

    pub fn get(&mut self, guild_id: &GuildId) -> &mut GuildData {
        self.guilds.entry(guild_id.get()).or_default()
    }
}

#[derive(Default)]
pub struct GuildData {
    pub music: Arc<Mutex<MusicManager>>,
}

pub struct GuildContext;

impl TypeMapKey for GuildContext {
    type Value = GuildManager;
}

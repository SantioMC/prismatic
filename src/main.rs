use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use config::Config;
use diesel::{Connection, SqliteConnection};

use invidious::ClientAsync;
use reqwest::Client as HttpClient;
use serenity::framework::standard::Configuration;
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::prelude::*;
use serenity::{prelude::GatewayIntents, Client};
use songbird::SerenityInit;

use crate::guilds::data::{GuildContext, GuildManager};

pub mod command;
pub mod config;
pub mod discord;
pub mod guilds;
pub mod helper;
// pub mod models;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::get_config();
    let http = Http::new(&config.token);

    // taken from https://github.com/serenity-rs/serenity/blob/current/examples/e05_command_framework/src/main.rs#L221
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .group(&command::general::GENERAL_GROUP)
        .group(&command::music::MUSIC_GROUP);

    framework.configure(
        Configuration::new()
            .prefix("!")
            .case_insensitivity(true)
            .on_mention(Some(bot_id))
            .owners(owners)
            .allow_dm(false),
    );

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::MESSAGE_CONTENT;

    let connection = SqliteConnection::establish("data.db")
        .unwrap_or_else(|_| panic!("Failed to connect to database"));

    let mut client = Client::builder(&config.token, intents)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .type_map_insert::<DatabaseKey>(Arc::new(Mutex::new(connection)))
        .type_map_insert::<ConfigKey>(config)
        .type_map_insert::<GuildContext>(GuildManager::new())
        .type_map_insert::<YoutubeKey>(ClientAsync::default())
        .await
        .expect("Error creating client");

    println!(
        "Started running as {}",
        client.http.get_current_user().await?.name
    );

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}

pub struct Bot;
pub struct DatabaseKey;
pub struct ConfigKey;
pub struct HttpKey;
pub struct YoutubeKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

impl TypeMapKey for DatabaseKey {
    type Value = Arc<Mutex<SqliteConnection>>;
}

impl TypeMapKey for ConfigKey {
    type Value = Config;
}

impl TypeMapKey for YoutubeKey {
    type Value = ClientAsync;
}

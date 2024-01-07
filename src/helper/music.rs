//! A simple wrapper around songbird

use std::sync::Arc;

use anyhow::Result;

use invidious::{hidden::SearchItem, ClientAsyncTrait, CommonVideo};
use reqwest::Client;
use serenity::{
    all::{ChannelId, GuildId, Message},
    async_trait,
    client::Context,
};
use songbird::{
    input::YoutubeDl, typemap::TypeMap, Event, EventContext, EventHandler, Songbird, TrackEvent,
};
use tokio::sync::{Mutex, RwLockWriteGuard};

use crate::{
    guilds::music::{handler::TrackEndNotifier, manager::MusicManager, track::Track},
    YoutubeKey,
};

/// Connects if the bot is not connected to a voice channel, otherwise nothing happens
pub async fn ensure_connected(
    ctx: &Context,
    songbird: Arc<Songbird>,
    handler: Arc<Mutex<MusicManager>>,
    msg: &Message,
) -> Result<()> {
    if !is_connected(Arc::clone(&songbird), msg.guild_id.unwrap()).await {
        let channel_id = get_voice_channel(ctx, msg).expect("Could not retrieve voice channel");
        connect_to(songbird, handler, msg.guild_id.unwrap(), channel_id).await?;
    }

    Ok(())
}

/// Get the channel id of the user's currently connected channel
fn get_voice_channel(ctx: &Context, msg: &Message) -> Option<ChannelId> {
    let guild = msg.guild(&ctx.cache).expect("Could not retrieve guild");

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    channel_id
}

/// Check if the bot is connected to a voice channel
async fn is_connected(songbird: Arc<Songbird>, guild: GuildId) -> bool {
    songbird.get(guild).is_some()
}

/// Connect the bot to the specified voice channel
async fn connect_to(
    songbird: Arc<Songbird>,
    music_handler: Arc<Mutex<MusicManager>>,
    guild: GuildId,
    channel: ChannelId,
) -> Result<()> {
    if let Ok(handler_lock) = songbird.join(guild, channel).await {
        let mut handler = handler_lock.lock().await;

        handler.add_global_event(TrackEvent::End.into(), TrackEndNotifier::new(music_handler));
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);

        if !handler.is_deaf() {
            let _ = handler.deafen(true).await;
        }
    }

    Ok(())
}

/// Stops the current track completely
pub async fn stop_playing(songbird: Arc<Songbird>, guild: GuildId) -> Result<()> {
    if let Some(handler_lock) = songbird.get(guild) {
        let mut handler = handler_lock.lock().await;
        handler.stop();
    }

    Ok(())
}

/// Query youtube for videos that match the query
pub async fn query_youtube(
    data: &RwLockWriteGuard<'_, TypeMap>,
    query: &str,
) -> Result<Vec<CommonVideo>> {
    let do_search = !query.starts_with("http");
    let client = data
        .get::<YoutubeKey>()
        .expect("Expected YoutubeKey in TypeMap.")
        .clone();

    let videos = if do_search {
        let search = client
            .search(Some(&format!("q={}", query)))
            .await
            .expect("Failed to search for tracks");

        search.items.iter().filter_map(is_video).collect::<Vec<_>>()
    } else {
        let id = query.split("?v=").last().unwrap();
        let video: CommonVideo = client
            .video(id, None)
            .await
            .expect("Failed to find video")
            .into();
        vec![video]
    };

    Ok(videos)
}

pub async fn play_track(
    songbird: Arc<Songbird>,
    client: Arc<Client>,
    guild: GuildId,
    track: &Track,
) -> Result<()> {
    let client = Arc::as_ref(&client);

    if let Some(handler_lock) = songbird.get(guild) {
        let mut handler = handler_lock.lock().await;

        let source = track.source.get_url();

        let youtubedl = YoutubeDl::new(client.clone(), source);
        let _ = handler.play_input(youtubedl.into());
    }

    Ok(())
}

fn is_video(item: &SearchItem) -> Option<CommonVideo> {
    return match item {
        SearchItem::Video(v) => Some(v.clone()),
        _ => None,
    };
}

struct TrackErrorNotifier;

#[async_trait]
impl EventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }

        None
    }
}

//! Handles communication between the music manager and songbird.

use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use serenity::{
    all::{ChannelId, GuildId},
    client::Context,
};
use songbird::{Event, EventContext, EventHandler, Songbird};
use tokio::sync::Mutex;

use crate::helper::{embed, emoji, helper::SendEmbed, music};

use super::{
    event::MusicEventHandler, manager::Event as MusicEvent, manager::MusicManager, track::Track,
};

pub struct MusicHandler {
    guild: GuildId,
    channel: ChannelId,
    context: Context,
    config: MusicConfig,
    songbird: Arc<Songbird>,
    client: Arc<Client>,
}

pub struct MusicConfig {
    pub announce_songs: bool,
}

impl MusicConfig {
    pub fn new() -> Self {
        MusicConfig {
            announce_songs: true,
        }
    }
}

impl MusicHandler {
    pub fn new(
        ctx: Context,
        songbird: Arc<Songbird>,
        client: Arc<Client>,
        guild: GuildId,
        channel: ChannelId,
    ) -> Self {
        Self {
            config: MusicConfig::new(),
            context: ctx,
            guild,
            channel,
            client,
            songbird,
        }
    }
}

#[async_trait]
impl MusicEventHandler for MusicHandler {
    async fn on_track_start(&mut self, track: &Track) {
        let _ = music::play_track(
            Arc::clone(&self.songbird),
            Arc::clone(&self.client),
            self.guild,
            &track,
        )
        .await;

        if self.config.announce_songs {
            let thumbnail = track.thumbnail.clone();
            let embed = embed::build(format!(
                "{} Now playing **{}**",
                emoji::get_bot_emote(&self.context, "p_music")
                    .await
                    .unwrap(),
                track.title
            ));

            if let Some(thumbnail) = thumbnail {
                let _ = self
                    .channel
                    .send_embed(&self.context.http, embed.image(thumbnail))
                    .await;
            } else {
                let _ = self.channel.send_embed(&self.context.http, embed).await;
            };
        }
    }

    async fn on_queue_added(&mut self, track: &Track) {
        let _ = self
            .channel
            .send_embed(
                &self.context.http,
                embed::build(format!("Added **{}** to the queue", track.title)),
            )
            .await;
    }

    async fn on_track_skipped(&mut self) {
        let _ = music::stop_playing(Arc::clone(&self.songbird), self.guild).await;
    }
}

pub struct TrackEndNotifier {
    handler: Arc<Mutex<MusicManager>>,
}

impl TrackEndNotifier {
    pub fn new(handler: Arc<Mutex<MusicManager>>) -> Self {
        TrackEndNotifier { handler }
    }
}

#[async_trait]
impl EventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_) = ctx {
            let mut manager = self.handler.lock().await;
            manager.emit(MusicEvent::TrackEnded).await;
            manager.next().await;
        }

        None
    }
}

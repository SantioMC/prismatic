use async_trait::async_trait;

use super::track::Track;

/// Handles music events and makes it so when music needs to play, it actually plays
#[async_trait]
#[allow(unused_variables)]
pub trait MusicEventHandler: Send {
    async fn on_track_start(&mut self, track: &Track) {}
    async fn on_track_end(&mut self, track: &Track) {}
    async fn on_track_skipped(&mut self) {}
    async fn on_queue_added(&mut self, track: &Track) {}
    async fn on_queue_emptied(&mut self) {}
}

use super::{event::MusicEventHandler, track::Track};

pub struct MusicManager {
    queue: Vec<Track>,
    previous: Vec<Track>,
    playing: Option<Track>,
    handler: Option<Box<dyn MusicEventHandler>>,
    music_loop: Loop,
}

#[allow(dead_code)]
impl MusicManager {
    pub async fn add(&mut self, track: &Track) {
        self.queue.push(track.clone());
        self.emit(Event::QueueAdded(track.clone())).await;

        // If we aren't playing, let's start playing
        if self.playing.is_none() {
            self.next().await;
        }
    }

    pub fn remove(&mut self, index: usize) {
        self.queue.remove(index);
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn get(&self, index: usize) -> Option<&Track> {
        self.queue.get(index)
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn now_playing(&self) -> Option<&Track> {
        self.playing.as_ref()
    }

    pub fn set_loop(&mut self, loop_type: Loop) {
        self.music_loop = loop_type;
    }

    pub fn get_loop(&self) -> &Loop {
        &self.music_loop
    }

    pub async fn skip(&mut self) {
        self.emit(Event::TrackSkipped).await;
        self.next().await;
    }

    pub async fn next(&mut self) -> Option<Track> {
        if self.queue.is_empty() {
            self.playing = None;
            return None;
        }

        let track = self.queue.remove(0);
        self.playing = Some(track.clone());
        self.emit(Event::TrackStarted(track.clone())).await;

        self.previous.push(track.clone());
        if self.previous.len() > 10 {
            self.previous.remove(0);
        }

        match self.music_loop {
            Loop::Queue => {
                self.queue.push(track.clone());
                Some(track)
            }
            Loop::Track => {
                self.queue.insert(0, track.clone());
                Some(track)
            }
            _ => Some(track),
        }
    }

    pub fn previous(&mut self) -> Option<Track> {
        if let Some(track) = self.previous.pop() {
            self.queue.insert(0, track.clone());
            Some(track)
        } else {
            None
        }
    }

    pub fn event_handler(&mut self, handler: Box<dyn MusicEventHandler>) {
        self.handler = Some(handler);
    }

    pub fn has_handler(&self) -> bool {
        self.handler.is_some()
    }

    pub async fn emit(&mut self, event: Event) {
        if let Some(handler) = &mut self.handler {
            match event {
                Event::TrackStarted(track) => {
                    let _ = handler.on_track_start(&track).await;
                }
                Event::TrackEnded => {
                    let currently_playing = self.playing.clone().expect("No track playing");
                    let _ = handler.on_track_end(&currently_playing).await;
                }
                Event::QueueAdded(track) => {
                    let _ = handler.on_queue_added(&track).await;
                }
                Event::TrackSkipped => {
                    let _ = handler.on_track_skipped().await;
                }
                Event::QueueEmptied => {
                    let _ = handler.on_queue_emptied().await;
                }
            }
        }
    }
}

impl Default for MusicManager {
    fn default() -> Self {
        MusicManager {
            queue: Vec::new(),
            previous: Vec::new(),
            playing: None,
            music_loop: Loop::None,
            handler: None,
        }
    }
}

#[derive(PartialEq)]
pub enum Loop {
    None,
    Queue,
    Track,
}

pub enum Event {
    /// Sent when a track starts playing
    TrackStarted(Track),
    /// Sent when the current track ends
    TrackEnded,
    /// Sent when a track is added to the queue
    QueueAdded(Track),
    /// Sent when the queue has no more tracks to play
    QueueEmptied,
    /// Sent when the current track is skipped
    TrackSkipped,
}

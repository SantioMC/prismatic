use std::time::Duration;

use futures::Future;
use serenity::{
    all::{ChannelId, Message},
    builder::{CreateEmbed, CreateMessage},
    http::CacheHttp,
    Result,
};

pub trait SendEmbed {
    fn send_embed(
        &self,
        cache_http: impl CacheHttp,
        content: CreateEmbed,
    ) -> impl Future<Output = Result<Message>> + Send;
}

impl SendEmbed for ChannelId {
    #[inline]
    async fn send_embed(
        &self,
        cache_http: impl CacheHttp,
        content: CreateEmbed,
    ) -> Result<Message> {
        let content = CreateMessage::new().content("").embed(content);
        self.send_message(cache_http, content).await
    }
}

pub fn to_ms(duration: Duration) -> String {
    return format!("{}ms", duration.as_millis());
}

use anyhow::Result;
use serenity::{all::GuildId, client::Context};

pub async fn get_emoji(ctx: &Context, guild: GuildId, emoji: &str) -> Result<String> {
    let emote = guild
        .emojis(&ctx.http)
        .await?
        .iter()
        .find(|e| e.name == emoji)
        .map(|e| e.to_string());

    match emote {
        Some(e) => Ok(e),
        None => Ok("".to_string()),
    }
}

pub async fn get_bot_emote(ctx: &Context, emoji: &str) -> Result<String> {
    let guild = GuildId::new(765558158390984705);
    let emote = get_emoji(ctx, guild, emoji).await?;

    Ok(emote)
}

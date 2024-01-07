use std::sync::Arc;

use serenity::{
    all::Message,
    client::Context,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
};
use songbird::SongbirdKey;

use crate::{
    guilds::{
        data::GuildContext,
        music::{handler::MusicHandler, track::Track},
    },
    helper::{embed, emoji, helper::SendEmbed, music},
    HttpKey,
};

#[group]
#[commands(play, countdown, skip)]
struct Music;

#[command]
async fn countdown(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    msg.channel_id
        .send_embed(
            &ctx.http,
            embed::build(format!(
                "Doing something in {}",
                emoji::get_bot_emote(&ctx, "p_countdown").await?
            )),
        )
        .await?;

    Ok(())
}

#[command]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.raw().collect::<Vec<&str>>().join(" ");
    if query.is_empty() {
        msg.channel_id
            .send_embed(
                &ctx.http,
                embed::error("You need to provide a search query!"),
            )
            .await?;

        return Ok(());
    }

    let mut typemap = ctx.data.write().await;
    let videos = music::query_youtube(&typemap, &query).await;

    let http_client = typemap
        .get::<HttpKey>()
        .expect("Expected HttpKey in TypeMap.")
        .clone();

    let songbird = typemap
        .get::<SongbirdKey>()
        .expect("Expected SongbirdKey in TypeMap.")
        .clone();

    let manager = typemap
        .get_mut::<GuildContext>()
        .expect("Expected GuildManager in TypeMap.");

    let data = manager.get(&msg.guild_id.unwrap());
    let videos = match videos {
        Ok(v) => v,
        Err(e) => {
            msg.channel_id
                .send_embed(&ctx.http, embed::error(&format!("{}", e)))
                .await?;

            return Ok(());
        }
    };

    let video = if videos.is_empty() {
        msg.channel_id
            .send_embed(&ctx.http, embed::error("No tracks found!"))
            .await?;

        return Ok(());
    } else if videos.len() > 1 {
        // for now, default to just the first one
        videos[0].clone()
    } else {
        videos[0].clone()
    };

    let track = Track::from_youtube(video);

    music::ensure_connected(&ctx, Arc::clone(&songbird), Arc::clone(&data.music), &msg)
        .await
        .expect("Failed to connect to voice channel");

    {
        let mut music = data.music.lock().await;

        if !music.has_handler() {
            let handler = Box::new(MusicHandler::new(
                ctx.clone(),
                Arc::clone(&songbird),
                Arc::from(http_client),
                msg.guild_id.unwrap(),
                msg.channel_id,
            ));

            music.event_handler(handler);
        }

        music.add(&track).await;
    }

    Ok(())
}

#[command]
pub async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let mut typemap = ctx.data.write().await;
    let manager = typemap
        .get_mut::<GuildContext>()
        .expect("Expected GuildManager in TypeMap.");

    let data = manager.get(&msg.guild_id.unwrap());
    let mut music = data.music.lock().await;

    music.skip().await;
    Ok(())
}

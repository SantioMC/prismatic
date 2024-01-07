use std::time::Duration;

use serenity::{
    all::Message,
    builder::EditMessage,
    client::Context,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::Timestamp,
};

use crate::helper::{
    embed, emoji,
    helper::{to_ms, SendEmbed},
};

#[group]
#[commands(ping)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let loading = emoji::get_bot_emote(&ctx, "p_music").await?;
    let time_started = Timestamp::now();

    let mut message = msg
        .channel_id
        .send_embed(
            &ctx.http,
            embed::build(format!(
                "**Bot Ping**
				
				One way ping: {loading}
				Two way ping: {loading}",
            )),
        )
        .await?;

    let one_way = get_duration(message.timestamp, time_started);
    let time_ended = Timestamp::now();
    let two_way = get_duration(time_ended, time_started);

    message
        .edit(
            &ctx,
            EditMessage::new().add_embed(embed::build(format!(
                "**Bot Ping**

				One way ping: {}
				Two way ping: {}",
                to_ms(one_way),
                to_ms(two_way)
            ))),
        )
        .await?;

    Ok(())
}

fn get_duration(left: Timestamp, right: Timestamp) -> Duration {
    Duration::from_millis(left.millisecond() as u64 - right.millisecond() as u64)
}

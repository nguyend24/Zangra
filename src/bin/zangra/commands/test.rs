use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::client::Context;
use chrono::{Utc};
use chrono_tz::US::Eastern;

#[command]
pub async fn timestamp(ctx: &Context, message: &Message, _args: Args) -> CommandResult {
    let time = Utc::now().with_timezone(&Eastern).format("%F %r");
    if let Err(why) = message.channel_id.send_message(ctx, |m| m
        .embed(|e| e
            .title("Current time")
            .field("Time", time, false))).await {
        println!("Error sending timestamp: {}", why);
    }

    Ok(())
}
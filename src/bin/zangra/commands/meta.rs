use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

// #[command]
// async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
//     let _ = msg.channel_id.say(&ctx.http, "Pong!");
//
//     Ok(())
// }

#[command]
async fn invis(ctx: &Context, _msg: &Message) -> CommandResult {
    ctx.invisible().await;
    Ok(())
}

#[command]
async fn online(ctx: &Context, _msg: &Message) -> CommandResult {
    ctx.online().await;
    Ok(())
}
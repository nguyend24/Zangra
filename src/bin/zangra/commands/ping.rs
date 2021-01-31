use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};

//Responds to ping with "Pong!"
#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(why) =
    msg.channel_id.send_message(&ctx.http, |m|
        m
            .content("Pong!")
            .reactions(vec![ReactionType::Unicode(String::from("✅"))])).await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

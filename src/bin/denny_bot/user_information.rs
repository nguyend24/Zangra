use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::utils::{content_safe, ContentSafeOptions};

#[command]
pub fn birthday(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let settings = if let Some(guild_id) = msg.guild_id {
        // By default roles, users, and channel mentions are cleaned.
        ContentSafeOptions::default()
            // We do not want to clean channal mentions as they
            // do not ping users.
            .clean_channel(false)
            // If it's a guild channel, we want mentioned users to be displayed
            // as their display name.
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
    };

    let echo = content_safe(&ctx.cache, &args.rest(), &settings);
    msg.channel_id.say(&ctx.http, format!("Your birthday is {}!", echo));

    Ok(())
}
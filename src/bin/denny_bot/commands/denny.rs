use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::*,
    prelude::*,
};
use serenity::framework::standard::Args;
use serenity::utils::{ContentSafeOptions, Content, content_safe};

#[command]
pub fn nowplaying(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let status = args.rest();
    ctx.set_activity(Activity::playing(status));

    Ok(())
}

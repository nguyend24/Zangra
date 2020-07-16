use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::client::Context;
use serenity::model::channel::Message;

//Multiplies 2 f64
#[command]
pub fn multiply(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let one = args.single::<f64>().unwrap();
    let two = args.single::<f64>().unwrap();

    let product = one * two;

    let _ = msg.channel_id.say(&ctx.http, product);

    Ok(())
}

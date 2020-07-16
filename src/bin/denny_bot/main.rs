use serenity::{
    framework::{standard::macros::group, StandardFramework},
    Client,
};

use std::env;

use commands::{math::*, ping::*};
use user_information::BIRTHDAY_COMMAND;

use discord_event_handler::Handler;
use crate::twitch_webhook_handler::set_up_twitch_webhooks;

mod commands;
mod discord_event_handler;
mod four_horseman_server;
mod misc;
mod twitch;
mod twitch_webhook_handler;
mod test_server;
mod introductions;
mod user_information;

static VERSION: &str = "0.1.0";

#[group]
#[commands(ping, birthday)]
struct General;

#[group]
#[commands(multiply)]
struct Math;

fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler).expect("Error creating client");

    client.with_framework(StandardFramework::new().configure(|c|
        c
            .with_whitespace(true)
            .ignore_bots(true)
            .prefix("~"))
        .group(&GENERAL_GROUP)
        .group(&MATH_GROUP));

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

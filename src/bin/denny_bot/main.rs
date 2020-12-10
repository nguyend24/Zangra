use serenity::{
    framework::{standard::macros::group, StandardFramework},
    Client,
};

use std::env;

use commands::{math::*, ping::*};

use discord_event_handler::Handler;
use crate::twitch_webhook_handler::set_up_twitch_webhooks;

mod commands;
mod discord_event_handler;
mod limited_budgetworks_server;
mod misc;
mod twitch;
mod twitch_webhook_handler;
mod test_server;
mod config;

static VERSION: &str = "0.1.0";

#[group]
#[commands(ping)]
struct General;

#[group]
#[commands(multiply)]
struct Math;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let framework = StandardFramework::new().configure(
        |c| c
            .prefix("~")
            .ignore_bots(true)
            .with_whitespace(true)
    )
        .group(&GENERAL_GROUP)
        .group(&MATH_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

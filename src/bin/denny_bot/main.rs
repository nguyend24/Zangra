use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::macros::group,
        StandardFramework},
    model::{
        id::{ChannelId, GuildId},
        channel::Reaction,
        guild::Member,
        gateway::Ready},

    utils::Color,
};

use std::env;

use commands::{math::*, ping::*};

use crate::limited_budgetworks_server::utils::{add_member_join_role, add_role_rules_verified};

mod commands;
mod config;
mod limited_budgetworks_server;
mod misc;
mod test_server;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[group]
#[commands(ping)]
struct General;

#[group]
#[commands(multiply)]
struct Math;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, new_member: Member) {
        if guild_id.as_u64() == &713889872359981076 {
            add_member_join_role(&ctx, new_member).await;
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        add_role_rules_verified(&ctx, &reaction).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        println!("Version: {}", VERSION);

        if let Err(why) = ChannelId(773036830580408330).send_message(&ctx, |m| {
            m
                .embed(|e| {
                    e
                        .author(|a| {
                            a.icon_url(&ready.user.face())
                                .name(&ready.user.name)
                        })
                        .description(format!("\
                        {} is connected!\n\
                        Version: {}
                        ", &ready.user.name, &VERSION))
                        .color(Color::from_rgb(255, 128, 0))
                })
        }).await {
            println!("{}", why)
        };
    }
}

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

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

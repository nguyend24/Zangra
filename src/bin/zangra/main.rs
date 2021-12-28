use serenity::{
    async_trait,
    client::{
        bridge::gateway::GatewayIntents,
        Client,
        Context,
        EventHandler
    },
    framework::{
        standard::macros::group,
        StandardFramework},
    model::{
        id::{ChannelId, GuildId},
        channel::Reaction,
        guild::Member,
        gateway::Ready,
        interactions::Interaction,
        voice::VoiceState,
    },

    utils::Color,
};

use std::env;

use commands::{math::*, ping::*, messages::*, meta::*, test::*};

use crate::limited_budgetworks_server::utils::{add_member_join_role, add_role_rules_verified, add_member_welcome_message};
use std::fs::File;
use std::io::{Read, Write};

use serde::{
    Deserialize, // To deserialize data into structures
    Serialize,
};

use crate::utils::database::{DatabasePool, get_sqlite_pool};

mod commands;
mod config;
mod edbh;
mod limited_budgetworks_server;
mod misc;
mod test_server;
mod utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const TEST_SERVER_ID: &u64 = &373993407741427713_u64;

#[group]
#[commands(ping, invis, online, timestamp)]
struct General;

#[group]
#[commands(multiply)]
struct Math;

#[group]
#[commands(createroleselection)]
struct Moderation;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigurationData {
    pub discord_token: String,
    pub application_id: String,
}

fn read_configuration() -> Option<ConfigurationData> {
    match File::open("config.toml") {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let configuration = toml::from_str::<ConfigurationData>(&contents).unwrap();

            Some(configuration)
        }

        Err(why) => {
            println!("Unable to open config file. Why: {}", why);
            None
        }
    }
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, new_member: Member) {
        if guild_id.as_u64() == &713889872359981076 {
            add_member_join_role(&ctx, &new_member).await;
            add_member_welcome_message(&ctx, &new_member).await;
        }
    }

    async fn interaction_create(&self, _ctx: Context, _interaction: Interaction) {
        if autorole_selections(&_ctx, _interaction).await {
            return;
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
                .embed(|e| e
                    .author(|a|  a
                        .icon_url(&ready.user.face())
                        .name(&ready.user.name)
                    )
                    .description(format!("\
                      {} is connected!\n\
                      Version: {}
                      ", &ready.user.name, &VERSION))
                    .color(Color::from_rgb(255, 128, 0))
                )
        }).await {
            println!("{}", why)
        };
    }

    async fn voice_state_update(&self, ctx: Context, guild_id: Option<GuildId>, old: Option<VoiceState>, new: VoiceState) {
        if let Some(ref guild_id) = guild_id {
            if guild_id.as_u64() == &687876072045412560_u64 {
               edbh::utils::voice_state_changed(&ctx, &guild_id, &old, &new).await;
            }
        }

        // if let Some(ref guild_id) = guild_id{
        //     if guild_id.as_u64() == &373993407741427713_u64 {
        //         if let Some(state_change) = identify_state(guild_id, &old, &new) {
        //             println!("{}", &state_change);
        //
        //             match state_change {
        //                 VoiceStateChange::LeftVoiceChannel => {
        //                     if let Err(why) = ChannelId(805612483951722496).send_message(&ctx, |m| m
        //                         .embed(|e| e
        //                             .title("User left Voice Channel")
        //                             .author(|a| a
        //                                 .name(format!("{}#{}", &new.member.as_ref().unwrap().user.name, &new.member.as_ref().unwrap().user.discriminator))
        //                                 .icon_url(new.member.as_ref().unwrap().user.avatar_url().unwrap())
        //                             )
        //                             .field("Member", &new.member.as_ref().unwrap().user.name, false)
        //                             .timestamp(Utc::now().to_rfc3339()))).await {
        //                         println!("Error sending test voice log. Why: {}", why);
        //                     };
        //                 }
        //                 VoiceStateChange::JoinedVoiceChannel => {}
        //                 VoiceStateChange::ServerDeafened => {}
        //                 VoiceStateChange::ServerMuted => {}
        //                 VoiceStateChange::SelfDeafened => {}
        //                 VoiceStateChange::SelfMuted => {}
        //                 VoiceStateChange::SelfStream => {}
        //                 VoiceStateChange::SelfVideo => {}
        //                 VoiceStateChange::Suppress => {}
        //             }
        //         }
        //     }
        // }



    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
    let configuration = read_configuration().unwrap();

    let framework = StandardFramework::new().configure(
        |c| c
            .prefix("~")
            .ignore_bots(true)
            .with_whitespace(true)
            .case_insensitivity(true)
    )
        .group(&GENERAL_GROUP)
        .group(&MATH_GROUP)
        .group(&MODERATION_GROUP);

    let mut client = Client::builder(configuration.discord_token)
        .event_handler(Handler)
        .framework(framework)
        .intents(GatewayIntents::all())
        .application_id(configuration.application_id.parse().unwrap())
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        let pool = get_sqlite_pool("sqlite://zangra.db").await?;
        data.insert::<DatabasePool>(pool);
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}

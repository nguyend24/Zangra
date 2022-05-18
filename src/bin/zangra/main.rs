use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{standard::macros::group, StandardFramework},
    model::{
        channel::{ChannelType, Message, Reaction},
        gateway::{GatewayIntents, Ready},
        guild::Member,
        id::{ChannelId, GuildId},
        interactions::{
            application_command::{ApplicationCommand, ApplicationCommandOptionType, ApplicationCommandType},
            Interaction,
        },
        permissions::Permissions,
        voice::VoiceState,
    },
    utils::Color,
};

use std::env;

use commands::{math::*, messages::*, meta::*, ping::*, test::*};

use crate::limited_budgetworks_server::utils::{add_member_join_role, add_member_welcome_message, add_role_rules_verified};
use std::fs::File;
use std::io::Read;

use serde::{
    Deserialize, // To deserialize data into structures
    Serialize,
};

use crate::commands::webblock::{edit_interaction, webblock, webblock_check_message};

use crate::utils::database::{get_sqlite_pool, DatabasePool};

mod commands;
mod config;
mod edbh;
mod error;
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
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        if new_member.guild_id.as_u64() == &713889872359981076 {
            add_member_join_role(&ctx, &new_member).await;
            add_member_welcome_message(&ctx, &new_member).await;
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = &interaction {
            // if let Err(why) = command.create_interaction_response(&ctx, |re| {
            //     re.kind(InteractionResponseType::DeferredChannelMessageWithSource)
            // }).await {
            //     println!("Cannot respond to slash command: {}", why);
            // };

            match command.data.name.as_str() {
                "createroleselector" => {
                    // command.get_interaction_response(&ctx).await.unwrap().delete(&ctx).await;
                    createroleselectorslash(&ctx, &command).await;
                }
                "webblock" => {
                    webblock(&ctx, command).await.unwrap();
                }
                "Edit Role Selector" => {
                    if let Err(why) = edit_role_selector(&ctx, &command).await {
                        println!("Unable to edit role selector: {}", why);
                    };
                }

                _ => {}
            }
        } else if let Interaction::MessageComponent(mc) = &interaction {
            match mc.data.custom_id.as_str() {
                "selectmenu" => {
                    autorole_selections(&ctx, &interaction).await.unwrap();
                }
                _ => {}
            }
        } else if let Interaction::ModalSubmit(msi) = &interaction {
            match msi.data.custom_id.as_str().split(" ").next().unwrap_or("{}") {
                "webblockedit" => {
                    edit_interaction(&ctx, &msi).await.unwrap();
                }
                _ => {}
            }
        } else {
        }
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        if !new_message.author.bot {
            if let Err(why) = webblock_check_message(&ctx, &new_message).await {
                println!("Error webblock_check_message: {}", why);
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        add_role_rules_verified(&ctx, &reaction).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        println!("Version: {}", VERSION);

        if let Err(why) = ChannelId(773036830580408330)
            .send_message(&ctx, |m| {
                m.embed(|e| {
                    e.author(|a| a.icon_url(&ready.user.face()).name(&ready.user.name))
                        .description(format!(
                            "\
                      {} is connected!\n\
                      Version: {}
                      ",
                            &ready.user.name, &VERSION
                        ))
                        .color(Color::from_rgb(255, 128, 0))
                })
            })
            .await
        {
            println!("{}", why)
        };

        if let Err(why) = ApplicationCommand::create_global_application_command(&ctx, |command| {
            command.name("Edit Role Selector");
            command.kind(ApplicationCommandType::Message)
        })
        .await
        {
            println!("Unable to create slash command: {}", why);
        }

        if let Err(why) = ApplicationCommand::create_global_application_command(&ctx, |c| {
            c.name("webblock");
            c.description("Create a block list for unwanted links");
            c.default_member_permissions(Permissions::SEND_MESSAGES);
            c.create_option(|o| {
                o.kind(ApplicationCommandOptionType::SubCommand);
                o.name("help");
                o.description("Instructions for using link blocking feature")
            });
            c.create_option(|o| {
                o.kind(ApplicationCommandOptionType::SubCommand);
                o.name("enable");
                o.description("Turn on site blocking")
            });
            c.create_option(|o| {
                o.kind(ApplicationCommandOptionType::SubCommand);
                o.name("disable");
                o.description("Turn off site blocking")
            });
            c.create_option(|o| {
                o.kind(ApplicationCommandOptionType::SubCommand);
                o.name("edit");
                o.description("Edit the blocklist")
            });
            c.create_option(|logging| {
                logging.kind(ApplicationCommandOptionType::SubCommandGroup);
                logging.name("log");
                logging.description("Log when actions are taken");
                logging.create_sub_option(|enable| {
                    enable.kind(ApplicationCommandOptionType::SubCommand);
                    enable.name("enable");
                    enable.description("Turn on logging of actions taken");
                    enable.create_sub_option(|channel| {
                        channel.kind(ApplicationCommandOptionType::Channel);
                        channel.name("channel");
                        channel.description("Choose channel to send log messages");
                        channel.required(true);
                        channel.channel_types(&[ChannelType::Text])
                    })
                });
                logging.create_sub_option(|disable| {
                    disable.kind(ApplicationCommandOptionType::SubCommand);
                    disable.name("disable");
                    disable.description("Turn off logging of actions taken")
                })
            });
            c.create_option(|o| {
                o.kind(ApplicationCommandOptionType::SubCommand);
                o.name("status");
                o.description("Current configuration status for this server")
            })
        })
        .await
        {
            println!("Unable to create slash command: {}", why);
        }
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        if let Some(ref guild_id) = new.guild_id {
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
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let configuration = read_configuration().unwrap();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~").ignore_bots(true).with_whitespace(true).case_insensitivity(true))
        .group(&GENERAL_GROUP)
        .group(&MATH_GROUP)
        .group(&MODERATION_GROUP);

    let mut client = Client::builder(configuration.discord_token, GatewayIntents::all())
        .event_handler(Handler)
        .framework(framework)
        .application_id(configuration.application_id.parse().unwrap())
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        let pool = get_sqlite_pool("sqlite://zangra.sqlite").await?;
        data.insert::<DatabasePool>(pool);
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}

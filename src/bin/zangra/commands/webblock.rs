use std::collections::BTreeMap;
use anyhow::Result;
use linkify::Links;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use serenity::model::interactions::application_command::{ApplicationCommandInteraction, ApplicationCommandInteractionDataOption};
use serenity::model::interactions::InteractionResponseType;
use serenity::model::interactions::message_component::{ActionRowComponent};
use serenity::model::interactions::message_component::InputTextStyle;
use serenity::model::interactions::modal::ModalSubmitInteraction;
use serenity::model::prelude::InteractionApplicationCommandCallbackDataFlags;
use serenity::utils::Color;
use sqlx::{Error, Row};
use url::Url;

use crate::DatabasePool;

pub async fn webblock<>(ctx: &Context, aci: &ApplicationCommandInteraction) -> Result<()> {
    for option in &aci.data.options {
        match option.name.as_str() {
            "enable" => {
                if let Err(why) = enable(&ctx, &aci).await {
                    println!("WebBlock enable, why: {}", why);
                }
            }
            "disable" => {
                if let Err(why) = disable(&ctx, &aci).await {
                    println!("WebBlock disable, why: {}", why);
                }
            }
            "edit" => {
                if let Err(why) = edit_command(&ctx, &aci).await {
                    println!("WebBlock edit, why: {}", why);
                }
            }
            "log" => {
                if let Err(why) = log(&ctx, &aci, &option).await {
                    println!("WebBlock log, why: {}", why);
                }
            }
            "help" => {
                if let Err(why) = help(&ctx, &aci).await {
                    println!("WebBlock help, why: {}", why);
                }
            }
            "status" => {
                if let Err(why) = status(&ctx, &aci).await {
                    println!("WebBlock status, why:{}", why);
                }
            }
            _ => {}
        }
    }

    Ok(())
}

pub async fn webblock_check_message(ctx: &Context, message: &Message) -> Result<()> {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().unwrap().clone();

    let guild_id = match message.guild_id {
        Some(gid) => { gid }
        None => {
            return Ok(());
        }
    };

    let guild_webblock_info = sqlx::query(
        "SELECT Enabled, DeleteMode, LogMode, LogChannelId FROM WebBlockInformation WHERE GuildId=?")
        .bind(*guild_id.as_u64() as i64)
        .fetch_one(&pool)
        .await;

    match guild_webblock_info {
        Ok(row) => {
            let enabled = match row.get("Enabled") {
                "true" => {true}
                "false" => {false}
                _ => {false}
            };
            let delete_messages = match row.get("DeleteMode") {
                "true" => {true}
                "false" => {false}
                _ => {false}
            };
            let log_offence = match row.get("LogMode") {
                "true" => {true}
                "false" => {false}
                _ => {false}
            };

            let log_channel_id: i64 = row.get("LogChannelId");
            let log_channel_id: u64 = log_channel_id as u64;

            let mut offending: bool = false;

            if enabled {
                let blocked_sites: Vec<Url> = sqlx::query(
                    "SELECT Site FROM WebBlockSite WHERE GuildId=?")
                    .bind(*guild_id.as_u64() as i64)
                    .fetch_all(&pool)
                    .await?
                    .iter()
                    .map(|row| {
                        let mut site: String = row.get("Site");
                        if ! (site.contains("http") || site.contains("https")) {
                            site = "https://".to_owned() + site.as_str();
                        }
                        site
                    })
                    .map(|s| Url::parse(&s).unwrap())
                    .collect();


                let mut urls: Vec<Url> = Vec::new();

                //get all the links in the message
                {
                    let finder = linkify::LinkFinder::new();
                    let links: Links = finder.links(&message.content);
                    for link in links {
                        match Url::parse(link.as_str()) {
                            Ok(url) => {
                                urls.push(url);
                            }
                            Err(_parse_error) => {}
                        }
                    }
                }
                //check if the domain of any of the links matches the domains in the blocklist
                for u in &urls {
                    for bl in &blocked_sites {
                        match bl.domain() {
                            Some(bd) => {
                                match u.domain() {
                                    Some(ud) => {
                                        if ud != "" && bd.split(".").next().unwrap() == ud.split(".").next().unwrap() {
                                            offending = true;
                                        }
                                    }
                                    None => {}
                                }
                            }
                            None => {}
                        }
                    }
                }

                if offending {
                    if log_offence {
                        ChannelId(log_channel_id).send_message(&ctx, |m| {
                            m.embed(|e| {
                                e.title("Message containing blocked link");
                                e.description(&message.content);
                                e.author(|a| {
                                    a.icon_url(message.author.face());
                                    a.name(&message.author.name)
                                });
                                e.color(Color::RED)
                            })
                        }).await.unwrap();
                    }

                    if delete_messages {
                        message.delete(&ctx).await?;
                    }
                }
            }
        }
        Err(sqlite_error) => {
            //Feature is not enabled for this server so don't do anything
            println!("{}", sqlite_error);
            return Ok(());
        }
    }

    // let msg_content = message.content.to_owned();
    // let finder = linkify::LinkFinder::new();
    // let links: linkify::Links = finder.links(&msg_content);
    //
    // for link in links {
    //     let url = link.as_str().to_string();
    //     let slice = &url[link.start()..link.end()];
    //     println!("{}", slice);
    // }

    Ok(())
}

async fn enable(ctx: &Context, aci: &ApplicationCommandInteraction) -> Result<()> {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().unwrap().clone();

    sqlx::query(
        "INSERT INTO WebBlockInformation (GuildId, Enabled) VALUES(?, 'true')\
             ON CONFLICT (GuildId) DO UPDATE SET Enabled='true'")
        .bind(*aci.guild_id.unwrap().as_u64() as i64)
        .execute(&pool)
        .await?;

    aci.create_interaction_response(&ctx, |re| {
        re.kind(InteractionResponseType::ChannelMessageWithSource);
        re.interaction_response_data(|d| {
            d.embed(|e| {
                e.title("WebBlock Enabled");
                e.color(Color::DARK_GREEN)
            });
            d.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
        })
    }).await?;

    Ok(())
}

async fn disable(ctx: &Context, aci: &ApplicationCommandInteraction) -> Result<()> {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().unwrap().clone();

    let _ = sqlx::query(
        "INSERT INTO WebBlockInformation (GuildId, Enabled) VALUES(?, 'true')\
                    ON CONFLICT (GuildId) DO UPDATE SET Enabled='false'")
        .bind(*aci.guild_id.unwrap().as_u64() as i64)
        .execute(&pool)
        .await.unwrap();

    aci.create_interaction_response(&ctx, |re| {
        re.kind(InteractionResponseType::ChannelMessageWithSource);
        re.interaction_response_data(|d| {
            d.embed(|e| {
                e.title("WebBlock Disabled");
                e.color(Color::RED)
            });
            d.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
        })
    }).await?;

    Ok(())
}

pub async fn edit_interaction(ctx: &Context, mc: &ModalSubmitInteraction) -> Result<()> {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().unwrap().clone();
    if !mc.data.custom_id.contains("webblockedit") {
        return Ok(());
    }

    let guild_id: u64 = mc.data.custom_id.replace("webblockedit", "").trim().parse().unwrap();
    let mut invalid_urls: Vec<&str> = Vec::new();
    for ar in &mc.data.components {
        for com in &ar.components {
            if let ActionRowComponent::InputText(it) = com {
                let _ = sqlx::query("DELETE FROM WebBlockSite WHERE GuildId=?").bind(guild_id as i64).execute(&pool).await;
                let lines = it.value.split("\n");
                for (i, line) in lines.enumerate() {
                    if line.trim() == "" {
                        continue;
                    }

                    match Url::parse(line) {
                        Ok(_) => {
                            sqlx::query("INSERT INTO WebBlockSite (GuildId, Site, SiteOrder) VALUES (?, ?, ?)")
                            .bind(guild_id as i64)
                            .bind(line.trim())
                            .bind(i as i64)
                            .execute(&pool)
                            .await.unwrap();
                        }
                        Err(_) => {
                            invalid_urls.push(line);
                        }
                    }
                }
            }
        }
    }

    mc.create_interaction_response(&ctx, |re| {
        re.kind(InteractionResponseType::ChannelMessageWithSource);
        re.interaction_response_data(|d| {
            let invalid_url_message = "The following lines are not valid URLs:\n".to_string() + &invalid_urls.iter().fold("".to_string(), |a, b| a + b);
            d.content(invalid_url_message);
            d.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
        })
    }).await?;
    Ok(())
}

async fn edit_command(ctx: &Context, aci: &ApplicationCommandInteraction) -> Result<()> {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().unwrap().clone();

    let guild_id = &aci.guild_id.unwrap();
    let rows = sqlx::query(
        "SELECT Site, SiteOrder FROM WebBlockSite WHERE GuildId = ?")
        .bind(*guild_id.as_u64() as i64)
        .fetch_all(&pool)
        .await;

    match rows {
        Ok(sql_rows) => {
            let mut site_list: BTreeMap<u64, String> = BTreeMap::new();
            for row in sql_rows {
                //build sorted map of sites and order
                let site: String = row.get("Site");
                let order: i64 = row.get("SiteOrder");

                site_list.insert(order as u64, site);
            }

            aci.create_interaction_response(&ctx, |re| {
                re.kind(InteractionResponseType::Modal);
                re.interaction_response_data(|d| {
                    d.custom_id(format!("webblockedit {}", guild_id.as_u64()));
                    d.title("Edit blocklist");
                    d.components(|c| {
                        c.create_action_row(|ar| {
                            ar.create_input_text(|t| {
                                let mut text = String::from("");
                                for (_key, site) in &site_list {
                                    text += *&site;
                                    text += "\n";
                                }

                                t.custom_id("sites_list");
                                t.label("One site per line");
                                t.style(InputTextStyle::Paragraph);
                                t.value(text)
                            })
                        })
                    })
                })
            }).await.unwrap();
        }
        Err(_why) => { //There are no blocked sites
            aci.create_interaction_response(&ctx, |re| {
                re.kind(InteractionResponseType::Modal);
                re.interaction_response_data(|d| {
                    d.custom_id(format!("webblockedit {}", guild_id.as_u64()));
                    d.title("Edit blocklist");
                    d.components(|c| {
                        c.create_action_row(|ar| {
                            ar.create_input_text(|t| {
                                t.custom_id("site_list");
                                t.label("One site per line");
                                t.style(InputTextStyle::Paragraph);
                                t.min_length(0);
                                t.value("")
                            })
                        })
                    })
                })
            }).await.unwrap();
        }
    }

    Ok(())
}

async fn log(ctx: &Context, aci: &ApplicationCommandInteraction, logging_option: &ApplicationCommandInteractionDataOption) -> Result<()> {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().unwrap().clone();
    println!("option: {}", &logging_option.name);

    for option in &logging_option.options {
        if option.name == "enable" {
            let channel: u64 = option.options[0].value.as_ref().unwrap().as_str().unwrap().parse().unwrap();
            println!("{}", &channel);
            sqlx::query("INSERT INTO WebBlockInformation (GuildId, LogMode, LogChannelId)\
                             VALUES (?, 'true', ?)\
                             ON CONFLICT DO UPDATE SET LogMode='true', LogChannelId=?")
                .bind(*aci.guild_id.unwrap().as_u64() as i64)
                .bind(channel as i64)
                .bind(channel as i64)
                .execute(&pool)
                .await?;
            let channel_name = ChannelId(channel).name(&ctx).await.unwrap_or(String::from(""));
            aci.create_interaction_response(&ctx, |re| {
                re.kind(InteractionResponseType::ChannelMessageWithSource);
                re.interaction_response_data(|d| {
                    d.embed(|e| {
                        e.title(format!("Logging channel set to {}", channel_name))
                    });
                    d.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                })
            }).await?;

        } else if option.name == "disable" {
            sqlx::query("INSERT INTO WebBlockInformation (GuildId, LogMode) VALUES (?, 'false')\
                         ON CONFLICT DO UPDATE SET LogMode='false'")
                .bind(*aci.guild_id.unwrap().as_u64() as i64)
                .execute(&pool)
                .await?;

            aci.create_interaction_response(&ctx, |re| {
                re.kind(InteractionResponseType::ChannelMessageWithSource);
                re.interaction_response_data(|d| {
                    d.embed(|e| {
                        e.title("Logging disabled")
                    });
                    d.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                })
            }).await?;
        }
    }

    Ok(())
}

async fn help(ctx: &Context, aci: &ApplicationCommandInteraction) -> Result<()> {
    aci.create_interaction_response(&ctx, |re| {
        re.kind(InteractionResponseType::ChannelMessageWithSource);
        re.interaction_response_data(|d| {
            d.embed(|e| {
                e.title("WebBlock Commands");
                e.color(Color::DARK_BLUE);
                e.field("/webblock enable/disable", "Turn on link filter feature for this server", false);
                e.field("/webblock edit", "Edit the custom filter list for this server", false);
                e.field("/webblock log", "Configure logging options for this server", false);
                e.field("/webblock delete", "Configure deleting options for this server", false)
            });
            d.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
        })
    }).await?;

    Ok(())
}

async fn status(ctx: &Context, aci: &ApplicationCommandInteraction) -> Result<()> {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().unwrap().clone();

    let guild_id = aci.guild_id.unwrap();
    let guild_row = sqlx::query(
        "SELECT Enabled, DeleteMode, LogMode, LogChannelId FROM WebBlockInformation WHERE GuildId=?")
        .bind(*guild_id.as_u64() as i64)
        .fetch_one(&pool)
        .await;

    match guild_row {
        Ok(row) => {
            let status = match row.get("Enabled") {
                "true" => { "enabled" }
                "false" => { "disabled" }
                _ => { "disabled" }
            };
            let deletion = match row.get("DeleteMode") {
                "true" => { "enabled" }
                "false" => { "disabled" }
                _ => { "disabled" }
            };
            let logging = match row.get("LogMode") {
                "true" => { "enabled" }
                "false" => { "disabled" }
                _ => { "disabled" }
            };

            let logging_channel_id: i64 = row.get("LogChannelId");
            let logging_channel_id: u64 = logging_channel_id as u64;
            let logging_channel = match ChannelId(logging_channel_id).name(&ctx).await {
                Some(channel_name) => { channel_name }
                None => { "Not configured".to_string() }
            };

            aci.create_interaction_response(&ctx, |re| {
                re.kind(InteractionResponseType::ChannelMessageWithSource);
                re.interaction_response_data(|d| {
                    d.embed(|e| {
                        e.color(Color::DARK_BLUE);
                        e.field("Status", status, false);
                        e.field("Delete Messages", deletion, false);
                        e.field("Log offenses", logging, false);
                        e.field("Logging channel", logging_channel, false)
                    });
                    d.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                })
            }).await.unwrap();
        }
        Err(sqlite_error) => {
            if let Error::RowNotFound = sqlite_error {
                aci.create_interaction_response(&ctx, |re| {
                    re.kind(InteractionResponseType::ChannelMessageWithSource);
                    re.interaction_response_data(|d| {
                        d.embed(|e| {
                            e.field("Status", "disabled", false);
                            e.field("Delete Messages", "disabled", false);
                            e.field("Log offenses", "disabled", false);
                            e.field("Logging channel", "Not configured", false)
                        });
                        d.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    })
                }).await.unwrap();
            }

        }
    }

    Ok(())
}
use std::{
    collections::HashMap,
    time::Duration,
};

use std::collections::BTreeMap;

use serenity::{
    builder::{CreateButton, CreateSelectMenu, CreateSelectMenuOption},
    client::Context,
    collector::EventCollectorBuilder,
    framework::standard::{macros::command, CommandResult},
    futures::StreamExt,
    model::{
        channel::{Message},
        event::{Event, EventType},
        guild::{Emoji, Role},
        id::{EmojiId, RoleId},
        interactions::{
            Interaction,
            InteractionResponseType,
            message_component::{ButtonStyle, ComponentType},
        },
    },
    utils::Color,
};

use crate::DatabasePool;
use itertools::Itertools;
use rand::distributions::{Distribution, Uniform};

fn random_color() -> Color {
    let mut rng = rand::thread_rng();
    let between = Uniform::from(0..255);
    Color::from_rgb(between.sample(&mut rng), between.sample(&mut rng), between.sample(&mut rng))
}

pub async fn autorole_selections(ctx: &Context, interaction: Interaction) -> bool {
    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().unwrap().clone();

    let message_id = interaction.clone().message_component().unwrap().message.id();

    let query =
        sqlx::query("select AutoRoleMessageId from AutoRoleMessage where AutoRoleMessageId = ?")
            .bind(i64::try_from(message_id.as_u64().clone()).unwrap())
            .fetch_one(&pool)
            .await;

    if let Ok(_query) = query {
        if let Some(mc) = interaction.clone().message_component() {
            let role_ids: Vec<RoleId> = mc.data.values.iter().map(|rid| RoleId(rid.parse().unwrap())).collect();
            mc.member.as_ref().unwrap().clone().add_roles(&ctx, &*role_ids).await.expect("autorole_selections");
            mc.create_interaction_response(&ctx, |re| {
                re.kind(InteractionResponseType::DeferredUpdateMessage)
            }).await.expect("autorole_selections");
            return true;
        }
    }

    false
}

#[command]
pub async fn createroleselection(ctx: &Context, msg: &Message) -> CommandResult {
    msg.delete(&ctx).await.unwrap();

    let guild_roles: HashMap<RoleId, Role> = msg.guild_id.unwrap().roles(&ctx).await.expect("Error getting guild roles in createroleselection");
    let cancel_button = {
        let mut b = CreateButton::default();
        b.custom_id("cancel");
        b.label("Cancel");
        b.style(ButtonStyle::Danger);
        b
    };
    //Send setup message
    let mut setup_message: Message = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title("Select Roles");
            e.color(random_color())
        });
        m.components(|com| {
            com.create_action_row(|ar| {
                ar.add_button(cancel_button.clone())
            });
            com.create_action_row(|ar| {
                ar.create_select_menu(|sm| {
                    sm.custom_id("GuildRoles");
                    sm.placeholder("Select Roles");
                    sm.min_values(0);
                    sm.max_values({
                        guild_roles.len()
                    } as u64);
                    sm.options(|ops| { //List all roles available in guild
                        let options: Vec<CreateSelectMenuOption> = guild_roles.values().sorted().map(|role| {
                            let mut option = CreateSelectMenuOption::default();
                            option.label(role.name.as_str());
                            option.value(role.id.as_u64());
                            option
                        }).collect();

                        ops.set_options(options)
                    })
                })
            })
        })
    }).await.expect("Error sending setup message in createroleselection");


    let guild_emojis: HashMap<EmojiId, Emoji> = msg.guild(&ctx).await.unwrap().emojis;
    let emojis_select_menu = { //select menu contain all custom emojis in the guild
        let mut select_menu: CreateSelectMenu = CreateSelectMenu::default();
        select_menu.custom_id("emoji select menu");
        select_menu.placeholder("Select emoji");
        select_menu.min_values(1);
        select_menu.max_values(1);
        select_menu.options(|ops| {
            ops.set_options({
                guild_emojis.values().map(|e| {
                    CreateSelectMenuOption::default()
                        .emoji(e.id.into())
                        .label(&e.name)
                        .value(e.id.as_u64())
                        .clone()
                }).collect()
            });
            ops
        });

        select_menu
    };

    //select roles
    let mut role_selections: Vec<RoleId> = Vec::new();
    match setup_message.await_component_interaction(&ctx).timeout(Duration::from_secs(60 * 10)).await {
        Some(mc) => {
            match mc.data.component_type {
                ComponentType::Button => {
                    setup_message.delete(&ctx).await.unwrap();
                    return Ok(());
                }
                ComponentType::SelectMenu => {  //Select emoji for each selection
                    role_selections = mc.data.values.iter().map(|rid| {
                        RoleId(rid.parse::<u64>().unwrap())
                    }).sorted().collect();
                    //finish roles selection interaction
                    mc.create_interaction_response(&ctx, |re| {
                        re.kind(InteractionResponseType::DeferredUpdateMessage)
                    }).await.unwrap();
                }
                _ => { println!("???") }
            }
        }
        None => { //Timeout
            println!("setup timeout");
            setup_message.delete(&ctx).await.unwrap();
        }
    }

    //for each selected role, edit msg to say "Pick emoji for this role"
    //store <role, emoji>
    let mut roles_and_emojis: BTreeMap<RoleId, EmojiId> = BTreeMap::new();
    for role_id in &role_selections {
        //edit setup message and have user select an emoji for the specified role
        setup_message.edit(&ctx, |m| {
            m.embed(|e| {
                e.title(format!("Select emoji for {}", guild_roles.get(&role_id).unwrap().name));
                e.color(random_color())
            });
            m.components(|c| {
                c.create_action_row(|ar| {
                    ar.add_button(cancel_button.clone())
                });
                c.create_action_row(|ar| {
                    ar.add_select_menu(emojis_select_menu.clone())
                })
            })
        }).await.unwrap();

        //wait for the emoji selection for the next role
        match setup_message.await_component_interaction(&ctx).timeout(Duration::from_secs(60 * 10)).await {
            Some(mc) => {
                match mc.data.component_type {
                    ComponentType::Button => {
                        setup_message.delete(&ctx).await.unwrap();
                        return Ok(());
                    }
                    ComponentType::SelectMenu => {
                        let emoji_id: EmojiId = EmojiId(mc.data.values.first().unwrap().parse::<u64>().unwrap());
                        roles_and_emojis.insert(role_id.clone(), emoji_id);
                    }
                    _ => {}
                }

                mc.create_interaction_response(&ctx, |re| {
                    re.kind(InteractionResponseType::UpdateMessage);
                    re.interaction_response_data(|d| {
                        d.create_embed(|e| {
                            e.title("Add a message?");
                            e.color(random_color())
                        });
                        d.components(|c| {
                            c.create_action_row(|ar| {
                                ar.create_button(|but| {
                                    but.custom_id("yes");
                                    but.style(ButtonStyle::Success);
                                    but.label("Yes")
                                });
                                ar.create_button(|but| {
                                    but.custom_id("no");
                                    but.style(ButtonStyle::Danger);
                                    but.label("No")
                                })
                            })
                        })
                    })
                }).await.unwrap();
            }
            None => {}
        }
    }

//add a message to role select message

    let select_menu = {
        let mut sm = CreateSelectMenu::default();
        sm.custom_id("selectmenu");
        sm.min_values(1);
        sm.max_values(roles_and_emojis.len() as u64);
        sm.options(|ops| {
            let options: Vec<CreateSelectMenuOption> = roles_and_emojis.keys().map(|rid| {
                let mut o = CreateSelectMenuOption::default();
                let role_name = guild_roles.get(rid).unwrap().name.as_str();
                let emoji_id = roles_and_emojis.get(rid).unwrap();
                o.label(role_name);
                o.value(rid);
                o.emoji(emoji_id.clone().into());
                o
            }).collect();
            ops.set_options(options)
        });
        sm
    };

    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().unwrap().clone();

    match setup_message.await_component_interaction(&ctx).timeout(Duration::from_secs(60 * 10)).await {
        Some(mc) => {
            match mc.data.custom_id.as_str() {
                "yes" => {
                    mc.create_interaction_response(&ctx, |re| {
                        re.kind(InteractionResponseType::UpdateMessage);
                        re.interaction_response_data(|d| {
                            d.create_embed(|e| {
                                e.title("Reply to me with the message");
                                e.color(random_color())
                            });
                            d.components(|c| {
                                c.set_action_rows(Vec::new())
                            })
                        })
                    }).await.unwrap();

                    //Wait for the reply containing an extra message
                    let event_builder = EventCollectorBuilder::new(&ctx)
                        .add_event_type(EventType::MessageCreate)
                        .add_channel_id(&setup_message.channel_id)
                        .filter(move |f| {
                            match f.as_ref() {
                                Event::MessageCreate(e) => {
                                    if let Some(mid) = e.message.referenced_message.clone() {
                                        return mid.id == setup_message.id;
                                    }
                                }
                                _ => { return false; }
                            }
                            false
                        })
                        .timeout(Duration::from_secs(60 * 10));

                    match event_builder.await.unwrap().next().await.unwrap().as_ref() {
                        Event::MessageCreate(e) => {
                            setup_message.edit(&ctx, |edit| {//Display finished list and Done/Cancel buttons
                                edit.embed(|embed| { //Display finished list
                                    embed.color(random_color());
                                    embed.description(e.message.content.as_str())
                                });
                                edit.components(|c| { //done/dancel buttons
                                    c.create_action_row(|ar| {
                                        ar.create_button(|but| {
                                            but.custom_id("done");
                                            but.label("Done");
                                            but.style(ButtonStyle::Primary)
                                        });
                                        ar.add_button(cancel_button.clone())
                                    });
                                    c.create_action_row(|ar| {
                                        ar.add_select_menu(select_menu.clone())
                                    })
                                })
                            }).await.unwrap();

                            e.message.delete(&ctx).await.unwrap();

                            match setup_message.await_component_interaction(&ctx).timeout(Duration::from_secs(60 * 10)).await {
                                Some(mc) => {
                                    match mc.data.custom_id.as_str() {
                                        "done" => {
                                            //remove buttons, set message and set list
                                            mc.create_interaction_response(&ctx, |re| {
                                                re.kind(InteractionResponseType::UpdateMessage);
                                                re.interaction_response_data(|d| {
                                                    d.components(|c| {
                                                        c.create_action_row(|ar| {
                                                            ar.add_select_menu(select_menu.clone())
                                                        })
                                                    })
                                                })
                                            }).await.unwrap();
                                        }
                                        "cancel" => {
                                            setup_message.delete(&ctx).await.unwrap();
                                        }
                                        _ => {}
                                    }
                                }
                                None => {}
                            }
                        }
                        _ => {}
                    };
                }
                "no" => {
                    setup_message.edit(&ctx, |edit| {//Display finished list and Done/Cancel buttons
                        edit.embed(|embed| { //Display finished list
                            embed.color(random_color())
                        });
                        edit.components(|c| { //done/dancel buttons
                            c.create_action_row(|ar| {
                                ar.add_select_menu(select_menu.clone())
                            })
                        })
                    }).await.unwrap();
                }
                _ => {}
            }

            sqlx::query("insert into AutoRoleMessage (AutoRoleMessageId) values (?)")
                .bind(i64::try_from(setup_message.id.as_u64().clone()).unwrap())
                .execute(&pool)
                .await
                .unwrap();
        }
        None => {}
    }

    Ok(())
}
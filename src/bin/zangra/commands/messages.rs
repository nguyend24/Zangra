use std::{
    collections::HashMap,
    time::Duration,
};
use std::cmp::min;

use itertools::Itertools;
use rand::distributions::{Distribution, Uniform};
use serenity::{
    builder::{CreateActionRow, CreateEmbed, CreateSelectMenu, CreateSelectMenuOption},
    client::Context,
    collector::EventCollectorBuilder,
    framework::standard::{CommandResult, macros::command},
    futures::StreamExt,
    model::{
        channel::Message,
        event::{Event, EventType},
        guild::{Emoji, Role},
        id::{ChannelId, EmojiId, GuildId, RoleId},
        interactions::{
            Interaction,
            InteractionResponseType,
            message_component::{ActionRowComponent, ButtonStyle},
        },
    },
    utils::Color,
};

use crate::DatabasePool;

fn random_color() -> Color {
    let mut rng = rand::thread_rng();
    let between = Uniform::from(0..255);
    Color::from_rgb(between.sample(&mut rng), between.sample(&mut rng), between.sample(&mut rng))
}

struct RoleSelector {
    setup_message: Message,
    content: String,
    embeds: Vec<CreateEmbed>,
    action_rows: Vec<CreateActionRow>,
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
        if let Some(ref mc) = interaction.message_component() {
            mc.create_interaction_response(&ctx, |re| {
                re.kind(InteractionResponseType::DeferredUpdateMessage)
            }).await.expect("autorole_selections");

            let mut msg = mc.message.clone().regular().unwrap();
            let member = mc.member.clone().expect("Can't access member");

            match mc.data.custom_id.as_str() {
                "clear_roles" => {
                    for ar in msg.components {
                        for com in ar.components {
                            match com {
                                ActionRowComponent::SelectMenu(sm) => {
                                    for role in sm.options {
                                        let mut member = mc.member.clone().unwrap();
                                        let _ = member.remove_role(&ctx, RoleId(role.value.parse().unwrap())).await;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                "edit" => {
                    let permissions = member.permissions.expect("Unable to read a member's permissions");
                    if permissions.administrator() {
                        let guild_id = mc
                            .guild_id
                            .expect("Unable to retrieve guildId from interaction");

                        let channel_id = mc.channel_id;
                        let role_selector = role_selection_message_setup(ctx, guild_id, channel_id, Some(msg.clone()))
                            .await
                            .expect("Unable to setup role selector message");

                        msg.edit(&ctx, |m| {
                            m.content(role_selector.content);
                            m.set_embeds(role_selector.embeds);
                            m.components(|c| {
                                c.set_action_rows(role_selector.action_rows)
                            })
                        }).await.expect("Unable to edit original role selector message");

                        role_selector.setup_message.delete(&ctx).await.expect("Unable to delete role selector set up message")
                    }
                }
                "selectmenu" => {
                    let role_ids: Vec<RoleId> = mc.data.values.iter().map(|rid| RoleId(rid.parse().unwrap())).collect();
                    let _ = mc.member.clone().unwrap().clone().add_roles(&ctx, &*role_ids).await;
                    msg.clone().edit(&ctx, |m| {
                        m.content(msg.content.clone())
                    }).await.unwrap();
                }
                _ => {}
            }
            return true;
        }
    }

    false
}

#[command]
pub async fn createroleselection(ctx: &Context, msg: &Message) -> CommandResult {
    // msg.delete(&ctx).await.unwrap();

    if let Some(role_selector) = role_selection_message_setup(ctx, msg.guild_id.unwrap().clone(), msg.channel_id.clone(), None).await {
        let data = ctx.data.read().await;
        let pool = data.get::<DatabasePool>().unwrap().clone();

        let role_selector_msg = role_selector.setup_message;
        sqlx::query("insert into AutoRoleMessage (AutoRoleMessageId) values (?)")
            .bind(role_selector_msg.id.as_u64().clone() as i64)
            .execute(&pool)
            .await
            .unwrap();
    }

    Ok(())
}

async fn role_selection_message_setup(ctx: &Context, guild_id: GuildId, channel_id: ChannelId, edit_message: Option<Message>) -> Option<RoleSelector> {
    let guild_roles: HashMap<RoleId, Role> = guild_id.roles(&ctx).await.expect("Error getting guild roles in createroleselection");

    //Send setup message
    let mut page_index = 0;
    let list_max = 25;
    let mut selected_roles: Vec<RoleId> = Vec::new();

    let role_selection_menu = |guild_roles: &HashMap<RoleId, Role>, page_index: &usize, list_max: &usize, selected_roles: &Vec<RoleId>| {
        CreateActionRow::default().create_select_menu(|sm| {
            let mut list_length = 0;
            sm
                .custom_id("select_guild_roles")
                .placeholder("Select Roles")
                .min_values(0)
                .options(|ops| { //List all roles available in guild
                    let options: Vec<CreateSelectMenuOption> = guild_roles
                        .iter()
                        .sorted()
                        .skip(page_index * list_max)
                        .take(25)
                        .map(|(_roleid, role)| {
                            CreateSelectMenuOption::default()
                                .label(role.name.as_str())
                                .value(role.id.as_u64())
                                .to_owned()
                        }).collect();
                    list_length = options.len();
                    ops.set_options(options)
                })
                .max_values(min((list_max - selected_roles.len()) as u64, list_length as u64))
        })
            .clone()
    };

    let role_selection_prompt = |page_index: &usize, selected_roles: &Vec<RoleId>| {
        let mut embed = CreateEmbed::default();
        embed.title("Select Roles for this Role Selector");
        embed.color(random_color());

        let mut description = "Selected roles:\n".to_owned();

        let selected_roles_text = selected_roles
            .iter()
            .map(|r| guild_roles.get(r).unwrap().name.clone())
            .reduce(|curr, next| curr + "\n" + &next)
            .unwrap_or("".parse().unwrap());

        description += selected_roles_text.as_str();
        embed.description(description);
        embed.footer(|f| {
            f.text(format!("Page {}/{}", page_index + 1, (guild_roles.len() / list_max) + 1))
        });
        embed
    };

    let role_selection_buttons_action_row = |page_index: &usize| {
        CreateActionRow::default()
            .create_button(|b| {
                b.custom_id("previous_page");
                b.label("Previous Page");
                b.style(ButtonStyle::Primary);
                if page_index == &0 {
                    b.disabled(true);
                }
                b
            })
            .create_button(|b| {
                b.custom_id("next_page");
                b.label("Next Page");
                b.style(ButtonStyle::Primary);
                if (page_index + 1) * list_max > guild_roles.len() {
                    b.disabled(true);
                }
                b
            })
            .create_button(|b| {
                b.custom_id("continue");
                b.label("Continue");
                b.style(ButtonStyle::Primary)
            })
            .create_button(|b| {
                b.custom_id("cancel");
                b.label("Cancel");
                b.style(ButtonStyle::Danger)
            })
            .clone()
    };


    let mut setup_message: Message = channel_id.send_message(&ctx, |m| {
        if let Some(msg) = edit_message {
            m.reference_message(&msg);
        }
        m.set_embed(role_selection_prompt(&page_index, &selected_roles));
        m.components(|c| {
            c.add_action_row(role_selection_buttons_action_row(&page_index));
            c.add_action_row(role_selection_menu(&guild_roles, &page_index, &list_max, &selected_roles))
        })
    }).await.expect("Error sending setup message in createroleselection");

    let mut interaction = setup_message.await_component_interaction(&ctx).timeout(Duration::from_secs(60 * 10)).await;

    //while continue button is not pressed
    while let Some(ref mc) = interaction {
        match mc.data.custom_id.as_str() {
            "previous_page" => {
                //retreat the selection list, everything else stays the same
                page_index -= 1;
            }
            "next_page" => {
                //advance the selection list, everything else stays the same
                page_index += 1;
            }
            "continue" => {
                mc.create_interaction_response(&ctx, |re| {
                    re.kind(InteractionResponseType::DeferredUpdateMessage)
                }).await.unwrap();
                break;
            }
            "cancel" => {
                setup_message.delete(&ctx).await.expect("Error deleting setup message");
                return None;
            }
            "select_guild_roles" => {
                //user has selected items from the list, update selected roles and update display
                mc.data.values.iter()
                    .for_each(|r| {
                        println!("{}", guild_roles.get(&RoleId(r.parse().unwrap())).unwrap().name.as_str());
                        if !selected_roles.contains(&RoleId(r.parse().unwrap())) {
                            selected_roles.push(RoleId(r.parse().unwrap()));
                        }
                    });

                for (i, roleid) in guild_roles.keys().sorted().enumerate() {
                    if page_index * list_max <= i && i < (page_index * list_max) + list_max {

                        //if roleid isn't in values, remove from selected_roles
                        if !mc.data.values.contains(&roleid.to_string()) {
                            //remove from selected_roles
                            if let Some(i) = selected_roles.iter().position(|p| p == roleid) {
                                selected_roles.remove(i);
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        mc.create_interaction_response(&ctx, |re| {
            re.kind(InteractionResponseType::UpdateMessage);
            re.interaction_response_data(|d| {
                d.embeds(vec![role_selection_prompt(&page_index, &selected_roles)]);
                d.components(|c| {
                    c.add_action_row(role_selection_buttons_action_row(&page_index));
                    c.add_action_row(role_selection_menu(&guild_roles, &page_index, &list_max, &selected_roles))
                })
            })
        }).await.unwrap();

        interaction = setup_message.await_component_interaction(&ctx).timeout(Duration::from_secs(60 * 10)).await;
    }

    let guild_emojis: HashMap<EmojiId, Emoji> = guild_id.to_guild_cached(ctx).await.unwrap().emojis;
    let emoji_selection_buttons_action_row = |page_index: &usize| {
        CreateActionRow::default()
            .create_button(|b| {
                b.custom_id("previous_page");
                b.label("Previous Page");
                b.style(ButtonStyle::Primary);
                if page_index == &0 {
                    b.disabled(true);
                }
                b
            })
            .create_button(|b| {
                b.custom_id("next_page");
                b.label("Next Page");
                b.style(ButtonStyle::Primary);
                if (page_index + 1) * list_max > guild_emojis.len() {
                    b.disabled(true);
                }
                b
            })
            .create_button(|b| {
                b.custom_id("cancel");
                b.label("Cancel");
                b.style(ButtonStyle::Danger)
            })
            .clone()
    };

    let mut page_index = 0;
    let emojis_select_menu = |page_index: &usize| {
        CreateActionRow::default().create_select_menu(|sm| {
            let mut list_length = 0;
            sm
                .custom_id("select_emoji")
                .placeholder("Select emoji")
                .min_values(1)
                .max_values(1)
                .options(|ops| { //List all roles available in guild
                    let options: Vec<CreateSelectMenuOption> = guild_emojis
                        .iter()
                        .skip(page_index * list_max)
                        .take(25)
                        .map(|(emojiid, emoji)| {
                            CreateSelectMenuOption::default()
                                .label(emoji.name.as_str())
                                .value(emoji.id.as_u64())
                                .emoji(emojiid.clone().into())
                                .to_owned()
                        }).collect();
                    list_length = options.len();
                    ops.set_options(options)
                })
        })
            .clone()
    };

    let mut roles_and_emojis: HashMap<RoleId, EmojiId> = HashMap::new();
    for roleid in &selected_roles {
        let emoji_selection_embed = |page_index: &usize| {
            let mut embed = CreateEmbed::default();
            embed.title(format!("Pick an emoji to represent this role: *{}*", guild_roles.get(&roleid).unwrap().name));
            embed.color(random_color());
            embed.footer(|f| {
                f.text(format!("Page {}/{}", page_index + 1, (guild_emojis.len() / list_max) + 1))
            });
            embed
        };

        setup_message.edit(&ctx, |m| {
            m.set_embed(emoji_selection_embed(&page_index));
            m.components(|c| {
                c.add_action_row(emoji_selection_buttons_action_row(&page_index));
                c.add_action_row(emojis_select_menu(&page_index))
            })
        }).await.unwrap(); //error handle this later


        let mut done = false;
        while !done {
            match setup_message.await_component_interaction(&ctx).timeout(Duration::from_secs(60 * 10)).await {
                Some(mc) => {
                    match mc.data.custom_id.as_str() {
                        "previous_page" => {
                            page_index -= 1;
                        }
                        "next_page" => {
                            page_index += 1;
                        }
                        "cancel" => {
                            setup_message.delete(&ctx).await.unwrap();
                            return None;
                        }
                        "select_emoji" => {
                            let emoji_id = EmojiId(mc.data.values.first().unwrap().parse().unwrap());
                            roles_and_emojis.insert(roleid.clone(), emoji_id);
                            done = true;
                        }
                        _ => {}
                    }

                    mc.create_interaction_response(&ctx, |re| {
                        re.kind(InteractionResponseType::UpdateMessage);
                        re.interaction_response_data(|d| {
                            d.embeds(vec![emoji_selection_embed(&page_index)]);
                            d.components(|c| {
                                c.add_action_row(emoji_selection_buttons_action_row(&page_index));
                                c.add_action_row(emojis_select_menu(&page_index))
                            })
                        })
                    }).await.unwrap();
                }
                None => {}
            }
        }
    }

//add a message to role select message

    let select_menu = {
        let mut sm = CreateSelectMenu::default();
        sm.custom_id("selectmenu");
        sm.min_values(1);
        sm.max_values(roles_and_emojis.len() as u64);
        sm.options(|ops| {
            let options: Vec<CreateSelectMenuOption> = selected_roles.iter().map(|rid| {
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

    setup_message.edit(&ctx, |m| {
        m.embed(|e| {
            e.description("Reply to me with instructions for this role selection")
        });
        m.components(|c| {
            c.set_action_rows(vec![])
        })
    }).await.unwrap();

    let response = match await_message_reply(&ctx, setup_message.clone()).await {
        Ok(msg) => { msg }
        Err(_why) => {
            "Select your roles!".parse().unwrap()
        }
    };

    let mut action_rows: Vec<CreateActionRow> = Vec::new();

    let buttons = CreateActionRow::default()
        .create_button(|b| {
            b.custom_id("clear_roles");
            b.label("Clear Roles");
            b.style(ButtonStyle::Primary)
        })
        .create_button(|b| {
            b.custom_id("edit");
            b.label("Edit");
            b.style(ButtonStyle::Primary)
        })
        .clone();
    let selection_menu = CreateActionRow::default()
        .add_select_menu(select_menu.clone())
        .clone();

    action_rows.push(buttons);
    action_rows.push(selection_menu);

    setup_message.edit(&ctx, |m| {
        m.content(&response);
        m.set_embeds(vec![]);
        m.components(|c| {
            c.create_action_row(|ar| {
                ar.create_button(|b| {
                    b.custom_id("clear_roles");
                    b.label("Clear Roles");
                    b.style(ButtonStyle::Primary)
                });
                ar.create_button(|b| {
                    b.custom_id("edit");
                    b.label("Edit");
                    b.style(ButtonStyle::Primary)
                })
            });
            c.create_action_row(|ar| {
                ar.add_select_menu(select_menu.clone())
            })
        })
    }).await.unwrap();

    let role_selector = RoleSelector {
        setup_message: setup_message,
        content: response,
        embeds: vec![],
        action_rows: action_rows,
    };
    Some(role_selector)
}

async fn await_message_reply(ctx: &Context, parent_message: Message) -> Result<String, String> {
    let event_builder = EventCollectorBuilder::new(&ctx)
        .add_event_type(EventType::MessageCreate)
        .add_channel_id(parent_message.channel_id)
        .filter(move |e| {
            match e.as_ref() {
                Event::MessageCreate(mce) => {
                    if let Some(mid) = mce.message.referenced_message.clone() {
                        return mid.id == parent_message.id;
                    }
                }
                _ => { return false; }
            }
            false
        })
        .timeout(Duration::from_secs(60 * 10)).await;

    let mut msg: String = String::from("");
    //Reply containing message
    match event_builder {
        Ok(mut ec) => {
            if let Some(event) = ec.next().await {
                match event.as_ref() {
                    Event::MessageCreate(m) => {
                        msg = m.message.content.clone();

                        if let Err(why) = m.message.delete(&ctx).await {
                            println!("{}", why);
                        }
                    }
                    _ => {}
                }
            }
        }
        Err(err) => { return Err(err.to_string()); }
    }

    Ok(msg)
}
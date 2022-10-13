use std::cmp::min;
use std::collections::HashSet;
use std::{collections::HashMap, time::Duration};

use anyhow::anyhow;
use anyhow::Result;

use itertools::Itertools;
use rand::distributions::{Distribution, Uniform};

use serde_json::{json, Value};
use serenity::{
    builder::{CreateActionRow, CreateEmbed, CreateSelectMenu, CreateSelectMenuOption},
    client::Context,
    collector::EventCollectorBuilder,
    framework::standard::{macros::command, CommandResult},
    futures::StreamExt,
    model::{
        application::{
            component::{ActionRowComponent, ButtonStyle},
            interaction,
            interaction::{
                application_command::ApplicationCommandInteraction,
                message_component::MessageComponentInteraction, InteractionResponseType,
            },
        },
        channel,
        channel::{Embed, Message, MessageReference},
        event::{Event, EventType},
        guild::Role,
        id::{ChannelId, GuildId, RoleId},
    },
    utils::Color,
};

use crate::DatabasePool;

fn random_color() -> Color {
    let mut rng = rand::thread_rng();
    let between = Uniform::from(0..255);
    Color::from_rgb(
        between.sample(&mut rng),
        between.sample(&mut rng),
        between.sample(&mut rng),
    )
}

struct RoleSelector {
    setup_message: Message,
    content: String,
    embeds: Vec<CreateEmbed>,
    action_rows: Vec<CreateActionRow>,
}

pub async fn edit_role_selector<'a, C: Into<&'a Context>>(
    ctx: C,
    command: &ApplicationCommandInteraction,
) -> Result<()> {
    let ctx = ctx.into();

    command
        .create_interaction_response(&ctx, |response| {
            response.kind(InteractionResponseType::ChannelMessageWithSource);
            response.interaction_response_data(|data| {
                data.content("Edit below");
                data.flags(interaction::MessageFlags::EPHEMERAL);

                data
            });

            response
        })
        .await?;

    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().unwrap().clone();

    let messages = &command.data.resolved.messages;
    let member = match command.member {
        Some(ref m) => m,
        None => {
            return Err(anyhow!("Unable to retrieve new member"));
        }
    };
    let guild_id = match command.guild_id {
        Some(gid) => gid,
        None => {
            return Err(anyhow!("Unable to find guild id"));
        }
    };

    for (message_id, message) in messages {
        match sqlx::query(
            "select AutoRoleMessageId from AutoRoleMessage where AutoRoleMessageId = ?",
        )
        .bind(*message_id.as_u64() as i64)
        .fetch_one(&pool)
        .await
        {
            Ok(_row) => {
                let permissions = member
                    .permissions
                    .expect("Unable to read a member's permissions");
                if permissions.administrator() {
                    let channel_id = &command.channel_id;

                    let role_selector = role_selection_message_setup(
                        ctx,
                        guild_id,
                        channel_id.clone(),
                        Some(message.clone()),
                    )
                    .await
                    .expect("Unable to setup role selector message");

                    message
                        .clone()
                        .edit(&ctx, |m| {
                            m.content(role_selector.content);
                            m.set_embeds(role_selector.embeds);
                            m.components(|c| c.set_action_rows(role_selector.action_rows))
                        })
                        .await?;
                    role_selector
                        .setup_message
                        .delete(&ctx)
                        .await
                        .expect("Unable to delete role selector set up message");
                }
            }
            Err(why) => {
                println!("{}", why);
            }
        }
    }

    Ok(())
}

pub async fn autorole_selections(ctx: &Context, mc: &MessageComponentInteraction) -> Result<()> {
    mc.create_interaction_response(&ctx, |response| {
        response.kind(InteractionResponseType::DeferredUpdateMessage);

        response
    })
    .await?;

    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().unwrap().clone();

    let message_id = mc.message.id;
    let query =
        sqlx::query("select AutoRoleMessageId from AutoRoleMessage where AutoRoleMessageId = ?")
            .bind(*message_id.as_u64() as i64)
            .fetch_one(&pool)
            .await;

    if query.is_ok() {
        let msg = &mc.message;
        let mut member = mc.member.clone().ok_or(anyhow!("can't retrieve member"))?;
        let member_roles = member.roles.clone();

        if mc.data.custom_id.as_str() == "selectmenu" {
            for msg_component in &msg.components {
                for ar_component in &msg_component.components {
                    match ar_component {
                        ActionRowComponent::SelectMenu(sm) => {
                            let all_items: HashSet<RoleId> = sm
                                .options
                                .iter()
                                .map(|op| {
                                    let role_id_u64: u64 = op.value.parse().unwrap();
                                    RoleId(role_id_u64)
                                })
                                .collect();

                            let selected_items: HashSet<RoleId> = mc
                                .data
                                .values
                                .iter()
                                .map(|op| {
                                    let role_id_u64 = op.parse().unwrap();
                                    RoleId(role_id_u64)
                                })
                                .collect();

                            let remove: Vec<RoleId> = all_items
                                .difference(&selected_items)
                                .filter(|role| member_roles.contains(role))
                                .map(|role| role.clone())
                                .collect();
                            let add: Vec<RoleId> = selected_items
                                .into_iter()
                                .filter(|role| !member_roles.contains(role))
                                .collect();

                            println!("remove: {:?}, add: {:?}", &remove, &add);

                            member.remove_roles(&ctx, &remove).await?;
                            member.add_roles(&ctx, &add).await?;

                            println!("after");
                        }
                        _ => todo!(),
                    }
                }
            }
        }
    }

    Ok(())
}

#[command]
pub async fn createroleselection(ctx: &Context, msg: &Message) -> CommandResult {
    if let Ok(role_selector) = role_selection_message_setup(
        ctx,
        msg.guild_id.unwrap().clone(),
        msg.channel_id.clone(),
        None,
    )
    .await
    {
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

pub async fn createroleselectorslash(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<()> {
    command
        .create_interaction_response(&ctx, |re| {
            re.kind(InteractionResponseType::ChannelMessageWithSource);
            re.interaction_response_data(|data| data.content("."))
        })
        .await?;

    command
        .get_interaction_response(&ctx)
        .await?
        .delete(&ctx)
        .await?;

    let guild_id = command.guild_id.unwrap();
    let channel_id = command.channel_id;

    if let Ok(role_selector) =
        role_selection_message_setup(&ctx, guild_id.clone(), channel_id.clone(), None).await
    {
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

async fn role_selection_message_setup(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
    edit_message: Option<Message>,
) -> Result<RoleSelector> {
    let guild_roles: HashMap<RoleId, Role> = guild_id.roles(&ctx).await?;
    let mut selected_roles: Vec<RoleId> = Vec::new(); //Roles available for a user to choose from

    let mut page_index = 0;
    let list_max = 25;

    let role_selection_menu = |guild_roles: &HashMap<RoleId, Role>,
                               page_index: &usize,
                               list_max: &usize,
                               selected_roles: &Vec<RoleId>| {
        CreateActionRow::default()
            .create_select_menu(|sm| {
                let mut list_length = 0;
                sm.custom_id("select_guild_roles")
                    .placeholder("Select Roles")
                    .min_values(0)
                    .options(|ops| {
                        //List all roles available in guild
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
                            })
                            .collect();
                        list_length = options.len();
                        ops.set_options(options)
                    })
                    .max_values(min(
                        (list_max - selected_roles.len()) as u64,
                        list_length as u64,
                    ))
            })
            .clone()
    };

    //Role selection phase
    //Picking the roles that will be provided to a user as options once finished
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
            f.text(format!(
                "Page {}/{}",
                page_index + 1,
                (guild_roles.len() / list_max) + 1
            ))
        });
        embed
    };

    let role_selection_buttons_action_row = |page_index: &usize, selected_roles: &Vec<RoleId>| {
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
                b.style(ButtonStyle::Primary);
                if selected_roles.len() == 0 {
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

    let mut setup_message: Message = channel_id
        .send_message(&ctx, |m| {
            if let Some(msg) = edit_message {
                m.reference_message(&msg);
            }
            m.set_embed(role_selection_prompt(&page_index, &selected_roles));
            m.components(|c| {
                c.add_action_row(role_selection_buttons_action_row(
                    &page_index,
                    &selected_roles,
                ));
                c.add_action_row(role_selection_menu(
                    &guild_roles,
                    &page_index,
                    &list_max,
                    &selected_roles,
                ))
            })
        })
        .await?;

    loop {
        let interaction = setup_message
            .await_component_interaction(&ctx)
            .timeout(Duration::from_secs(60 * 10))
            .await;

        match interaction {
            Some(mc) => {
                mc.create_interaction_response(&ctx, |re| {
                    re.kind(InteractionResponseType::DeferredUpdateMessage)
                })
                .await?;

                if mc.data.custom_id.as_str() == "continue" {
                    break;
                }

                match mc.data.custom_id.as_str() {
                    "previous_page" => {
                        //retreat the selection list, everything else stays the same
                        page_index -= 1;
                    }
                    "next_page" => {
                        //advance the selection list, everything else stays the same
                        page_index += 1;
                    }
                    "cancel" => {
                        setup_message.delete(&ctx).await?;
                        return Err(anyhow!("cancel"));
                    }
                    "select_guild_roles" => {
                        //user has selected items from the list, update selected roles and update display
                        mc.data.values.iter().for_each(|r| {
                            println!(
                                "{}",
                                guild_roles
                                    .get(&RoleId(r.parse().unwrap()))
                                    .unwrap()
                                    .name
                                    .as_str()
                            );
                            if !selected_roles.contains(&RoleId(r.parse().unwrap())) {
                                selected_roles.push(RoleId(r.parse().unwrap()));
                            }
                        });

                        for (i, roleid) in guild_roles.keys().sorted().enumerate() {
                            if page_index * list_max <= i && i < (page_index * list_max) + list_max
                            {
                                //if roleid isn't in values, remove from selected_roles
                                if !mc.data.values.contains(&roleid.to_string()) {
                                    //remove from selected_roles
                                    if let Some(i) = selected_roles.iter().position(|p| p == roleid)
                                    {
                                        selected_roles.remove(i);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }

                setup_message
                    .edit(&ctx, |m| {
                        m.set_embed(role_selection_prompt(&page_index, &selected_roles));
                        m.components(|c| {
                            c.add_action_row(role_selection_buttons_action_row(
                                &page_index,
                                &selected_roles,
                            ));
                            c.add_action_row(role_selection_menu(
                                &guild_roles,
                                &page_index,
                                &list_max,
                                &selected_roles,
                            ))
                        })
                    })
                    .await?;
            }
            None => {}
        }
    }

    //Change ordering of selections
    setup_message
        .edit(&ctx, |m| {
            m.embed(|e| e.title("Choose an ordering for the list"));
            m.components(|c| {
                c.create_action_row(|ar| {
                    ar.create_button(|b| {
                        b.custom_id("alphabetical");
                        b.label("Alphabetical");
                        b.style(ButtonStyle::Primary)
                    });
                    ar.create_button(|b| {
                        b.custom_id("manual");
                        b.label("Manual");
                        b.style(ButtonStyle::Primary)
                    });
                    ar.create_button(|b| {
                        b.custom_id("cancel");
                        b.label("Cancel");
                        b.style(ButtonStyle::Danger)
                    })
                })
            })
        })
        .await?;

    {
        let interaction = setup_message
            .await_component_interaction(&ctx)
            .timeout(Duration::from_secs(60 * 10))
            .await
            .unwrap();
        interaction
            .create_interaction_response(&ctx, |re| {
                re.kind(InteractionResponseType::DeferredUpdateMessage)
            })
            .await?;

        match interaction.data.custom_id.as_str() {
            "alphabetical" => {
                selected_roles.sort_by(|a, b| {
                    guild_roles
                        .get(a)
                        .unwrap()
                        .name
                        .cmp(&guild_roles.get(b).unwrap().name)
                });
            }
            "manual" => {
                println!("{}", selected_roles.len());
                setup_message
                    .edit(&ctx, |m| {
                        m.embed(|e| e.title("Make selections in the desired order"));
                        m.components(|c| {
                            c.create_action_row(|ar| {
                                ar.create_select_menu(|sm| {
                                    sm.custom_id("manual_selection");
                                    sm.min_values(selected_roles.len() as u64);
                                    sm.max_values(selected_roles.len() as u64);
                                    sm.options(|ops| {
                                        let sm_options: Vec<CreateSelectMenuOption> =
                                            selected_roles
                                                .iter()
                                                .map(|role_id| {
                                                    CreateSelectMenuOption::default()
                                                        .value(role_id.as_u64())
                                                        .label(
                                                            guild_roles
                                                                .get(role_id)
                                                                .unwrap()
                                                                .name
                                                                .clone(),
                                                        )
                                                        .clone()
                                                })
                                                .collect();
                                        ops.set_options(sm_options)
                                    })
                                    // sm.min_values(selected_roles.len() as u64)
                                })
                            })
                        })
                    })
                    .await?;

                let interaction = setup_message
                    .await_component_interaction(&ctx)
                    .timeout(Duration::from_secs(60 * 10))
                    .await
                    .unwrap();
                interaction
                    .create_interaction_response(&ctx, |re| {
                        re.kind(InteractionResponseType::DeferredUpdateMessage)
                    })
                    .await?;

                selected_roles = interaction
                    .data
                    .values
                    .iter()
                    .map(|s| RoleId(s.parse().unwrap()))
                    .collect();
            }
            "cancel" => {
                setup_message.delete(&ctx).await?;
                return Err(anyhow!("cancel"));
            }
            _ => {}
        }
    }

    //Adding descriptions to each previously selected role phase
    setup_message
        .edit(&ctx, |m| {
            m.embed(|e| e.title("Add descriptions to selections?"));
            m.components(|c| {
                c.create_action_row(|ar| {
                    ar.create_button(|b| {
                        b.custom_id("yes");
                        b.label("Yes");
                        b.style(ButtonStyle::Primary)
                    });
                    ar.create_button(|b| {
                        b.custom_id("no");
                        b.label("No");
                        b.style(ButtonStyle::Primary)
                    });
                    ar.create_button(|b| {
                        b.custom_id("cancel");
                        b.label("Cancel");
                        b.style(ButtonStyle::Danger)
                    })
                })
            })
        })
        .await?;

    let mut role_descriptions: HashMap<RoleId, String> = HashMap::new();

    match setup_message
        .await_component_interaction(&ctx)
        .timeout(Duration::from_secs(60 * 10))
        .await
    {
        Some(mc) => {
            mc.create_interaction_response(&ctx, |re| {
                re.kind(InteractionResponseType::DeferredUpdateMessage)
            })
            .await?;

            match mc.data.custom_id.as_str() {
                "yes" => {
                    //for each role, ask for a reply with the description to set
                    for roleid in &selected_roles {
                        let role = guild_roles
                            .get(&roleid)
                            .expect("Unable to map RoleId to Role");
                        setup_message
                            .edit(&ctx, |m| {
                                m.embed(|e| {
                                    e.title(format!(
                                        "Reply with a description for the role: {}",
                                        role.name.as_str()
                                    ))
                                });
                                m.components(|c| c.set_action_rows(vec![]))
                            })
                            .await?;

                        let description = await_message_reply(&ctx, setup_message.clone())
                            .await
                            .expect("Unable to get a description response");

                        role_descriptions.insert(roleid.clone(), description);
                    }
                }
                "no" => {}
                "cancel" => {
                    setup_message.delete(&ctx).await?;
                    return Err(anyhow!("cancel"));
                }
                _ => {
                    //should not happen
                    println!(
                        "Unknown custom_id: {}, line: {}",
                        mc.data.custom_id.as_str(),
                        line!()
                    );
                }
            }
        }
        None => {
            //delete message and error out
            //TODO
        }
    }

    //Set the max number of selections a user can make
    setup_message
        .edit(&ctx, |m| {
            m.embed(|e| e.title("What is the maximum number of selections a user can make?"));
            m.components(|c| {
                c.create_action_row(|ar| {
                    ar.create_select_menu(|sm| {
                        sm.placeholder("Pick a number");
                        sm.custom_id("max_selections");
                        sm.max_values(1);
                        sm.options(|o| {
                            (1..(selected_roles.len() + 1)).for_each(|i| {
                                o.create_option(|op| {
                                    op.value(i);
                                    op.label(i)
                                });
                            });
                            o
                        })
                    })
                })
            })
        })
        .await?;

    let mut max_selection = 0;
    match setup_message
        .await_component_interaction(&ctx)
        .timeout(Duration::from_secs(10 * 60))
        .await
    {
        Some(mc) => {
            mc.create_interaction_response(ctx, |re| {
                re.kind(InteractionResponseType::DeferredUpdateMessage)
            })
            .await?;
            max_selection = mc.data.values[0].parse().unwrap();
        }
        None => {}
    }

    //add a message to role select message
    //display selected roles,
    //buttons for
    //set message, add embed, done, cancel

    let mut instructions_message = String::from("");
    let mut embeds: Vec<CreateEmbed> = Vec::new();

    loop {
        setup_message
            .edit(&ctx, |m| {
                m.embed(|e| {
                    e.title("Set a message or embeds?");
                    e.description("This embed will be deleted once setup is complete.")
                });
                m.components(|c| {
                    c.create_action_row(|ar| {
                        ar.create_button(|b| {
                            b.custom_id("set_message");
                            b.label("Set Message");
                            b.style(ButtonStyle::Primary)
                        });
                        ar.create_button(|b| {
                            b.custom_id("add_embed");
                            b.label("Add Embed");
                            b.style(ButtonStyle::Primary)
                        });
                        ar.create_button(|b| {
                            b.custom_id("done");
                            b.label("Done");
                            b.style(ButtonStyle::Primary)
                        });
                        ar.create_button(|b| {
                            b.custom_id("cancel");
                            b.label("Cancel");
                            b.style(ButtonStyle::Danger)
                        })
                    })
                })
            })
            .await?;

        match setup_message
            .await_component_interaction(&ctx)
            .timeout(Duration::from_secs(60 * 10))
            .await
        {
            Some(mc) => {
                mc.create_interaction_response(&ctx, |re| {
                    re.kind(InteractionResponseType::DeferredUpdateMessage)
                })
                .await
                .expect("Unable to send interaction response");

                match mc.data.custom_id.as_str() {
                    "set_message" => {
                        setup_message
                            .edit(&ctx, |m| {
                                m.content("");
                                m.embed(|e| e.title("Reply to me with the message"));
                                m.components(|c| c.set_action_rows(vec![]))
                            })
                            .await
                            .expect("Unable to edit message");

                        instructions_message = await_message_reply(&ctx, setup_message.clone())
                            .await
                            .expect("Unable to get a response");

                        setup_message
                            .edit(&ctx, |m| m.content(&instructions_message))
                            .await?
                    }
                    "add_embed" => {
                        setup_message
                            .edit(&ctx, |m| {
                                m.embed(|e| e.title("Reply with json representing the embed"));
                                m.components(|c| c.set_action_rows(vec![]))
                            })
                            .await
                            .expect("Unable to edit message");

                        let response = await_message_reply(&ctx, setup_message.clone())
                            .await
                            .expect("Unable to receive reply");
                        let json: serde_json::Result<HashMap<String, Value>> =
                            serde_json::from_str(&response);
                        match json {
                            Ok(mut json) => {
                                json.insert(String::from("type"), json!("rich"));
                                let embed: Embed = serde_json::from_str(
                                    serde_json::to_string(&json).unwrap().as_str(),
                                )
                                .expect("Unable to deserialize");
                                let embed = CreateEmbed::from(embed);

                                embeds.push(embed)
                            }
                            Err(why) => {
                                setup_message
                                    .channel_id
                                    .send_message(&ctx, |m| {
                                        m.reference_message(MessageReference::from(&setup_message));
                                        m.flags(channel::MessageFlags::EPHEMERAL);
                                        m.content("Invalid JSON")
                                    })
                                    .await?;
                                println!("{}", why);
                            }
                        }
                    }
                    "done" => {
                        break;
                    }
                    "cancel" => {
                        setup_message
                            .delete(&ctx)
                            .await
                            .expect("Unable to delete message");
                        return Err(anyhow!("cancel"));
                    }
                    _ => {}
                }
            }
            None => {}
        }
    }

    let select_menu = |selected_roles: Vec<RoleId>| {
        let mut sm = CreateSelectMenu::default();
        sm.custom_id("selectmenu");
        sm.min_values(0);
        sm.max_values(max_selection);
        sm.options(|ops| {
            let options: Vec<CreateSelectMenuOption> = selected_roles
                .iter()
                .map(|rid| {
                    let mut o = CreateSelectMenuOption::default();
                    let role_name = guild_roles.get(rid).unwrap().name.as_str();
                    // let emoji_id = roles_and_emojis.get(rid).unwrap();
                    o.label(role_name);
                    o.value(rid);
                    // o.emoji(emoji_id.clone().into());
                    if role_descriptions.len() == selected_roles.len() {
                        o.description(&role_descriptions.get(rid).unwrap());
                    }
                    o
                })
                .collect();
            ops.set_options(options)
        });
        sm
    };

    let mut action_rows: Vec<CreateActionRow> = Vec::new();
    let selection_menu = CreateActionRow::default()
        .add_select_menu(select_menu(selected_roles.clone()))
        .clone();

    action_rows.push(selection_menu);

    setup_message
        .edit(&ctx, |m| {
            m.content(&instructions_message);
            m.set_embeds(embeds.clone());
            m.components(|c| c.set_action_rows(action_rows.clone()))
        })
        .await
        .unwrap();

    let role_selector = RoleSelector {
        setup_message: setup_message,
        content: instructions_message,
        embeds: embeds,
        action_rows: action_rows,
    };
    Ok(role_selector)
}

async fn await_message_reply(ctx: &Context, parent_message: Message) -> anyhow::Result<String> {
    let timeout = 60 * 5; //In seconds

    let mut event_collector = EventCollectorBuilder::new(&ctx)
        .add_event_type(EventType::MessageCreate)
        .add_channel_id(parent_message.channel_id)
        .filter(move |e| {
            match e.as_ref() {
                Event::MessageCreate(mce) => {
                    if let Some(mid) = mce.message.referenced_message.clone() {
                        return mid.id == parent_message.id;
                    }
                }
                _ => {
                    return false;
                }
            }
            false
        })
        .timeout(Duration::from_secs(timeout))
        .build()?;

    let mut msg: String = String::from("");
    //Reply containing message

    if let Some(event) = event_collector.next().await {
        match event.as_ref() {
            Event::MessageCreate(m) => {
                msg = m.message.content.clone();
                //discord sometimes hangs and doesn't delete the message
                //hopefully this slows it down enough for discord
                std::thread::sleep(Duration::from_secs(1));
                if let Err(why) = m.message.delete(&ctx).await {
                    println!("{}", why);
                }
            }
            _ => {}
        }
    }

    Ok(msg)
}

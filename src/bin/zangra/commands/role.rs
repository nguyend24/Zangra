use std::borrow::Borrow;
use std::collections::HashSet;

use crate::utils::database::DatabasePool;
use anyhow::anyhow;
use anyhow::Result;

use serenity::builder::CreateEmbed;
use serenity::model::guild::Member;
use serenity::model::id::RoleId;
use serenity::utils::Color;
use serenity::{
    client::Context,
    model::application::interaction::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
};
use sqlx::query;

pub async fn mutex(ctx: &Context, command: &ApplicationCommandInteraction) -> Result<()> {
    let member = command
        .member
        .as_ref()
        .ok_or(anyhow!("No member associated with command call"))?;
    let member_permissions = member
        .permissions
        .ok_or(anyhow!("Can't retrieve member permissions"))?;

    if !member_permissions.administrator()
        && !(&member.user.id.as_u64() == &213709744261693442_u64.borrow())
    {
        command
            .create_interaction_response(&ctx, |response| {
                response.kind(InteractionResponseType::ChannelMessageWithSource);
                response.interaction_response_data(|data| {
                    data.content("This command is reserved for administrators only");
                    data.ephemeral(true);
                    data
                });

                response
            })
            .await?;

        return Err(anyhow!("Mutex command called by non admin"));
    }

    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().unwrap().clone();

    let aci_data = &command.data;

    for subcommand in &aci_data.options {
        match subcommand.name.as_str() {
            "add" => {
                let arg_role1: i64 = subcommand.options[0]
                    .value
                    .as_ref()
                    .ok_or(anyhow!("role1 not provided"))?
                    .as_str()
                    .unwrap()
                    .parse()?;
                let arg_role2: i64 = subcommand.options[1]
                    .value
                    .as_ref()
                    .ok_or(anyhow!("role2 not provided"))?
                    .as_str()
                    .unwrap()
                    .parse()?;

                if arg_role1 == arg_role2 {
                    command
                        .create_interaction_response(&ctx, |response| {
                            response.kind(InteractionResponseType::ChannelMessageWithSource);
                            response.interaction_response_data(|data| {
                                data.ephemeral(true);
                                data.embed(|e| {
                                    e.title("Roles cannot be mutex with themselves");
                                    e
                                });

                                data
                            });

                            response
                        })
                        .await?;

                    return Ok(());
                }

                let pairs = query!("SELECT * FROM MutuallyExclusiveRole")
                    .fetch_all(&pool)
                    .await?;

                for pair in pairs {
                    let r1 = pair
                        .role1
                        .ok_or(anyhow!("Error reading role1 from database"))?;
                    let r2 = pair
                        .role2
                        .ok_or(anyhow!("Error reading role2 from database"))?;

                    if (r1 == arg_role1 && r2 == arg_role2) || (r2 == arg_role1 && r1 == arg_role2)
                    {
                        command
                            .create_interaction_response(&ctx, |response| {
                                response.kind(InteractionResponseType::ChannelMessageWithSource);
                                response.interaction_response_data(|data| {
                                    data.ephemeral(true);
                                    print!("r1: {}, r2: {}", &r1, &r2);
                                    data.content(format!("Pairing already exists"));

                                    data
                                });

                                response
                            })
                            .await?;

                        return Ok(());
                    }
                }

                let guild_id = command.guild_id.ok_or(anyhow!("No guild id found"))?;
                let guild_id_i64 = *guild_id.as_u64() as i64;

                query!(
                    "INSERT INTO MutuallyExclusiveRole (GuildId, role1, role2) VALUES (?, ?, ?)",
                    guild_id_i64,
                    arg_role1,
                    arg_role2
                )
                .execute(&pool)
                .await?;

                let role1_name = RoleId(arg_role1 as u64)
                    .to_role_cached(&ctx)
                    .ok_or(anyhow!("Error getting role"))?
                    .name;
                let role2_name = RoleId(arg_role2 as u64)
                    .to_role_cached(&ctx)
                    .ok_or(anyhow!("Error getting role"))?
                    .name;

                command
                    .create_interaction_response(&ctx, |response| {
                        response.kind(InteractionResponseType::ChannelMessageWithSource);
                        response.interaction_response_data(|data| {
                            data.ephemeral(true);
                            data.embed(|e| {
                                e.title(format!(
                                    "{} and {} have been added as mutually exclusive roles",
                                    role1_name, role2_name
                                ));
                                e
                            });

                            data
                        });

                        response
                    })
                    .await?;
            }
            "remove" => {
                //remove pairing from databse
                let guild_id = &command.guild_id.ok_or(anyhow!("Can't get guild_id"))?;
                let guild_id_i64 = *guild_id.as_u64() as i64;

                let arg_role1: i64 = subcommand.options[0]
                    .value
                    .as_ref()
                    .ok_or(anyhow!("role1 not provided"))?
                    .as_str()
                    .unwrap()
                    .parse()?;
                let arg_role2: i64 = subcommand.options[1]
                    .value
                    .as_ref()
                    .ok_or(anyhow!("role2 not provided"))?
                    .as_str()
                    .unwrap()
                    .parse()?;

                let result1 = query!("DELETE FROM MutuallyExclusiveRole WHERE GuildId = ? AND role1 = ? AND role2 = ?", guild_id_i64, arg_role1, arg_role2).execute(&pool).await?;
                let result2 = query!("DELETE FROM MutuallyExclusiveRole WHERE GuildId = ? AND role1 = ? AND role2 = ?", guild_id_i64, arg_role2, arg_role1).execute(&pool).await?;

                let role1_name = RoleId(arg_role1 as u64)
                    .to_role_cached(&ctx)
                    .ok_or(anyhow!("Error getting role"))?
                    .name;
                let role2_name = RoleId(arg_role2 as u64)
                    .to_role_cached(&ctx)
                    .ok_or(anyhow!("Error getting role"))?
                    .name;

                let title = match result1.rows_affected() + result2.rows_affected() {
                    //if at least 1 row affected, then mutex pair exists
                    0 => {
                        format!("{} and {} are not mutex roles", role1_name, role2_name)
                    }
                    _ => {
                        format!(
                            "{} and {} have been removed as mutex roles",
                            role1_name, role2_name
                        )
                    }
                };

                command
                    .create_interaction_response(&ctx, |response| {
                        response.kind(InteractionResponseType::ChannelMessageWithSource);
                        response.interaction_response_data(|data| {
                            data.ephemeral(true);
                            data.embed(|e| {
                                e.title(title);
                                e.color(if result1.rows_affected() + result2.rows_affected() > 0 {
                                    Color::DARK_GREEN
                                } else {
                                    Color::RED
                                });

                                e
                            });

                            data
                        });

                        response
                    })
                    .await?;
            }
            "clear" => {
                //remove all pairs for this server
                command
                    .create_interaction_response(&ctx, |response| {
                        response.kind(InteractionResponseType::ChannelMessageWithSource);
                        response.interaction_response_data(|data| {
                            data.ephemeral(true);
                            data.embed(|embed| {
                                embed.title("All roles cleared");

                                embed
                            });

                            data
                        });

                        response
                    })
                    .await?;

                let guild_id = command
                    .guild_id
                    .ok_or(anyhow!("Can't get guild id"))?
                    .as_u64()
                    .clone() as i64;
                query!(
                    "DELETE FROM MutuallyExclusiveRole WHERE GuildId = ?",
                    guild_id
                )
                .execute(&pool)
                .await?;
            }
            "list" => {
                //list out all pairings for this server
                let guild_id = command
                    .guild_id
                    .ok_or(anyhow!("Can't get guild ID for mutex list"))?
                    .as_u64()
                    .clone() as i64;

                let mutex_pairs = query!(
                    "SELECT * FROM MutuallyExclusiveRole WHERE GuildId = ?",
                    guild_id
                )
                .fetch_all(&pool)
                .await?;

                let pairs: String = mutex_pairs //Convert role pairs into String
                    .iter()
                    .filter(|r| r.role1.is_some() && r.role2.is_some()) //filter out rows with null cells; shouldn't ever happen
                    .map(|r| {
                        //convert role ids into full roles with information
                        let role1 = RoleId(r.role1.unwrap() as u64)
                            .to_role_cached(&ctx)
                            .ok_or(anyhow!("unable to get data for role1 in mutex list"));
                        let role2 = RoleId(r.role2.unwrap() as u64)
                            .to_role_cached(&ctx)
                            .ok_or(anyhow!("unable to get information for role2 in mutex list"));

                        if role1.is_ok() && role2.is_ok() {
                            return format!("{} - {}", role1.unwrap().name, role2.unwrap().name);
                        } else {
                            return "".to_string();
                        }
                    })
                    .reduce(|r1, r2| r1 + "\n" + &r2)
                    .unwrap_or(String::from("No mutex roles set"));

                //build embed displaying the pairs

                let display_embed = {
                    let mut embed = CreateEmbed::default();
                    embed.title("Mutex Roles");
                    embed.description(pairs);

                    embed
                };

                command
                    .create_interaction_response(&ctx, |response| {
                        response.kind(InteractionResponseType::ChannelMessageWithSource);
                        response.interaction_response_data(|data| {
                            data.ephemeral(true);
                            data.add_embed(display_embed);

                            data
                        });

                        response
                    })
                    .await?;
            }
            _ => {}
        }
    }

    Ok(())
}

pub async fn check_mutex_roles(
    ctx: &Context,
    old_member_data: &Option<&Member>,
    new_member_data: &mut Member,
) -> Result<()> {
    if old_member_data.is_none() {
        return Ok(());
    }

    let data = ctx.data.read().await;
    let pool = data.get::<DatabasePool>().unwrap().clone();

    let old_member_data = old_member_data.unwrap();

    let old_roles: HashSet<RoleId> = old_member_data.roles.clone().into_iter().collect();
    let new_roles: HashSet<RoleId> = new_member_data.roles.clone().into_iter().collect();

    let role_diff = new_roles.difference(&old_roles);

    let guild_id_i64 = *new_member_data.guild_id.as_u64() as i64;
    // let mutex_pairs = query!("SELECT * FROM MutuallyExclusiveRole WHERE GuildId = ?", guild_id_i64).fetch_all(&pool).await?;

    let guild_name = new_member_data
        .guild_id
        .name(&ctx)
        .unwrap_or("Can't get guild name for mutex check".to_string());

    for role in role_diff {
        //check role1 column
        let role_id_i64 = *role.as_u64() as i64;
        let result: Vec<RoleId> = query!(
            "SELECT * FROM MutuallyExclusiveRole WHERE GuildId = ? AND role1 = ?",
            guild_id_i64,
            role_id_i64
        )
        .fetch_all(&pool)
        .await?
        .iter()
        .map(|r| r.role2)
        .filter(|r| r.is_some())
        .map(|r| RoleId(r.unwrap() as u64))
        .filter(|rid| new_roles.contains(rid))
        .collect();

        // new_member_data.remove_roles(&ctx, &result).await?;
        if let Err(why) = new_member_data.remove_roles(&ctx, &result).await {
            return Err(anyhow!(format!(
                "mutex check, guildname: {}\n\n {}",
                &guild_name, why
            )));
        };

        //check role2 column
        let result: Vec<RoleId> = query!(
            "SELECT * FROM MutuallyExclusiveRole WHERE GuildId = ? AND role2 = ?",
            guild_id_i64,
            role_id_i64
        )
        .fetch_all(&pool)
        .await?
        .iter()
        .map(|r| r.role1)
        .filter(|r| r.is_some())
        .map(|r| RoleId(r.unwrap() as u64))
        .collect();

        if let Err(why) = new_member_data.remove_roles(&ctx, &result).await {
            return Err(anyhow!(format!(
                "mutex check, guildname: {}\n\n {}",
                &guild_name, why
            )));
        };
    }

    Ok(())
}

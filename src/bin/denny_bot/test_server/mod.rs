use serenity::model::id::RoleId;
use serenity::prelude::Context;
use serenity::model::channel::Reaction;
use serenity::http::CacheHttp;
use std::convert::TryFrom;

pub async fn _reaction_add_test(ctx: &Context, reaction: &Reaction) {
    let guild_id = match reaction.guild_id {
        Some(id) => id,
        None => {
            println!("add_role_rules_verified - guild id not found");
            return
        }
    };

    if guild_id.as_u64() == &373993407741427713_u64{
        let user_id = match reaction.user_id {
            Some(id) => id,
            None => {
                println!("add_roles_rules_verified - user id not found");
                return
            }
        };
        let mut reaction_member = match ctx.cache.member(guild_id, user_id).await {
            Some(member) => member,
            None => {
                println!("add_role_rules_verified - member not found");
                return
            }
        };
        let role_to_add = RoleId(557328408913117225);
        if let Err(why) = reaction_member.add_role(&ctx.http, role_to_add).await {
            println!("Error adding role: {:?}", why)
        }
    }
}

pub async fn _reaction_remove_test(ctx: &Context, reaction: &Reaction) {
    let guild_id = match reaction.guild_id {
        Some(id) => id,
        None => {
            println!("add_role_rules_verified - guild id not found");
            return
        }
    };

    if guild_id.as_u64() == &373993407741427713_u64{
        let user_id = match reaction.user_id {
            Some(id) => id,
            None => {
                println!("add_roles_rules_verified - user id not found");
                return
            }
        };
        let mut reaction_member = match ctx.cache.member(guild_id, user_id).await {
            Some(member) => member,
            None => {
                println!("add_role_rules_verified - member not found");
                return
            }
        };
        let role_to_remove = RoleId(557328408913117225);
        if let Err(why) = reaction_member.remove_role(&ctx.http, role_to_remove).await {
            println!("Error adding role: {:?}", why)
        }
    }
}
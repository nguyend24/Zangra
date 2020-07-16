use serenity::model::id::RoleId;
use serenity::prelude::Context;
use serenity::model::channel::Reaction;
use serenity::http::CacheHttp;
use std::convert::TryFrom;

pub fn _reaction_add_test(ctx: &Context, reaction: &Reaction) {
    if reaction.guild_id.unwrap().as_u64() == &373993407741427713_u64{
        let mut reaction_member = ctx.cache.read().member(&reaction.guild_id.unwrap(), &reaction.user_id).unwrap();
        let role_to_add = RoleId(557328408913117225);
        if let Err(why) = reaction_member.add_role(&ctx.http, role_to_add) {
            println!("Error adding role: {:?}", why)
        }
    }
}

pub fn _reaction_remove_test(ctx: &Context, reaction: &Reaction) {
    if reaction.guild_id.unwrap().as_u64() == &373993407741427713_u64 {
        let mut reaction_member = ctx.cache.read().member(&reaction.guild_id.unwrap(), &reaction.user_id).unwrap();
        let role_to_add = RoleId(557328408913117225);
        if let Err(why) = reaction_member.remove_role(&ctx.http, role_to_add) {
            println!("Error adding role: {:?}", why)
        }
    }
}
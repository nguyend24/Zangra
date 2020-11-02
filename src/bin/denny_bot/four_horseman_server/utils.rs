use serenity::{
    client::Context,
    model::{
        channel::Reaction,
        id::{ChannelId, GuildId, RoleId},
        guild::Member
    },
    utils::Color,
};

static FOUR_HORSEMAN_MESSAGE_ID: u64 = 716047715208921110;
static FOUR_HORSEMAN_ROLE_ID: u64 = 715247696663019560;

//Hardcoded role verification for Preston's Community Server
pub fn add_role_rules_verified(ctx: &Context, add_reaction: &Reaction) {
    if add_reaction.message_id.as_u64() == &FOUR_HORSEMAN_MESSAGE_ID {
        let guild_id = match &add_reaction.guild_id {
            Some(id) => id,
            None => {
                println!("add_role_rules_verified - guild id not found");
                return
            }
        };
        let mut reaction_member = match ctx.cache.read().member(guild_id, &add_reaction.user_id) {
            Some(member) => member,
            None => {
                println!("add_role_rules_verified - member not found");
                return
            }
        };
        let role_to_add = RoleId(FOUR_HORSEMAN_ROLE_ID);
        if let Err(why) = reaction_member.add_role(ctx, role_to_add) {
            println!("Error adding role: {:?}", why)
        }
    }
}

pub fn _remove_role_rules_verified(ctx: &Context, remove_reaction: &Reaction) {
    if remove_reaction.message_id.as_u64() == &FOUR_HORSEMAN_MESSAGE_ID {
        let guild_id: &GuildId = match &remove_reaction.guild_id {
            Some(id) => id,
            None => {
                println!("remove_role_rules_verified - guild id not found");
                return
            }
        };
        let mut reaction_member: Member = match ctx.cache.read().member(&guild_id, &remove_reaction.user_id) {
            Some(member) => member,
            None => {
                println!("remove_role_rules_verified - member not found");
                return
            }
        };
        let role_to_remove: RoleId = RoleId(FOUR_HORSEMAN_ROLE_ID);
        if let Err(why) = reaction_member.remove_role(&ctx.http, role_to_remove) {
            println!("Error removing role: {:?}", why)
        }
    }
}

//Sends a set of rules to the rules channel
pub fn print_rules(ctx: &Context) {
    if let Err(why) =  ChannelId(714691912787034113).send_message(ctx, |m| {
        m.embed(|e| {
            e.title("Rules")
                .description("This is a place where you learn a little bit more of the base rules to follow while chilling in the community (^-^):")
                .color(Color::from_rgb(0, 81, 255))
                .field(
                    "❤ Be respectful",
                    "Please no hateful speech towards anyone for any reason. This is a place where we can all relax, so please don't start any arguments or fights. Any instigating of arguments or fights will lead to a timeout of the server.",
                    false,
                )
                .field(
                    "🔞 No NSFW content",
                    "Please no NSFW content. We like to keep things wholesome, so nothing NSFW of the sort around here. Swearing is alright, just try to keep things PG-13.",
                    false,
                )
                .field("💬 No spamming", "Try not to spam the chat, it just clutters other messages and sometimes just gets plain annoying. ", false)
                .field("🔊 Voice", "No ear rape, NSFW, or any sounds that could damage hearing.", false)
                .field(
                    "📜 Follow Discord Guidelines",
                    "Anything that goes against discord guidelines also goes against our guidelines. Understand the rules and follow them.",
                    false,
                )
                .field(
                    "😊 Community Heads & Staff",
                    "If you need any help or want to suggest anything for the server you can ask any of the community heads and/or staff.",
                    false,
                )
                .field("Heads & Staff", "<@&714698133057175633>", false)
        })
        .reactions(vec!["✅"])
    }) {
        println!("Error sending message: print_rules(): {}", why);
    }
}

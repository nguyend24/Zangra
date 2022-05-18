use serenity::{
    client::Context,
    model::{
        channel::{Reaction, ReactionType},
        id::{ChannelId, RoleId},
        guild::Member
    },
    utils::Color,
};

static FOUR_HORSEMAN_MESSAGE_ID: u64 = 716047715208921110;
static FOUR_HORSEMAN_ROLE_ID: u64 = 715247696663019560;

pub async fn add_member_join_role(ctx: &Context, new_member: &Member) {
    let role_to_add = RoleId(FOUR_HORSEMAN_ROLE_ID);
    let mut member = new_member.clone();
    if let Err(why) = member.add_role(ctx, role_to_add).await {
        println!("Error adding role: {:?}", why);
    }
}

pub async fn add_member_welcome_message(ctx: &Context, new_member: &Member) {
    let description = format!("Welcome <@{}> to Limited Budgetworks,\n\
        hope you enjoy your stay!", new_member.user.id);

    if let Err(why) = ChannelId(714692215577772085).send_message(ctx, |m| m.embed(|e|
        e.description(description)
            .image("https://media.discordapp.net/attachments/714340993129906306/729584384822476890/LBW_HI.gif"))).await {
        println!("Error sending LimitedBudgetworks welcome message. Why: {}", why);
    };
}

//Hardcoded role verification for Preston's Community Server
pub async fn add_role_rules_verified(ctx: &Context, add_reaction: &Reaction) {
    if add_reaction.message_id.as_u64() == &FOUR_HORSEMAN_MESSAGE_ID {
        let guild_id = match add_reaction.guild_id {
            Some(id) => id,
            None => {
                println!("add_role_rules_verified - guild id not found");
                return;
            }
        };
        let user_id = match add_reaction.user_id {
            Some(id) => id,
            None => {
                println!("add_roles_rules_verified - user id not found");
                return;
            }
        };
        let mut reaction_member = match ctx.http.get_member(*guild_id.as_u64(), *user_id.as_u64()).await {
            Ok(member) => member,
            Err(why) => {
                println!("add_role_rules_verified - member not found, Why: {}", why);
                return;
            }
        };
        let role_to_add = RoleId(FOUR_HORSEMAN_ROLE_ID);
        if let Err(why) = reaction_member.add_role(ctx, role_to_add).await {
            println!("Error adding role: {:?}", why);
        }
    }
}

// pub async fn _remove_role_rules_verified(ctx: &Context, remove_reaction: &Reaction) {
//     if remove_reaction.message_id.as_u64() == &FOUR_HORSEMAN_MESSAGE_ID {
//         let guild_id = match remove_reaction.guild_id {
//             Some(id) => id,
//             None => {
//                 println!("remove_role_rules_verified - guild id not found");
//                 return;
//             }
//         };
//         let user_id = match remove_reaction.user_id {
//             Some(id) => id,
//             None => {
//                 println!("add_roles_rules_verified - user id not found");
//                 return;
//             }
//         };
//         let mut reaction_member: Member = match ctx.cache.member(guild_id, user_id).await {
//             Some(member) => member,
//             None => {
//                 println!("remove_role_rules_verified - member not found");
//                 return;
//             }
//         };
//         let role_to_remove: RoleId = RoleId(FOUR_HORSEMAN_ROLE_ID);
//         if let Err(why) = reaction_member.remove_role(&ctx.http, role_to_remove).await {
//             println!("Error removing role: {:?}", why)
//         }
//     }
// }

//Sends a set of rules to the rules channel
pub async fn _print_rules(ctx: &Context) {
    if let Err(why) = ChannelId(714691912787034113).send_message(ctx, |m| {
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
            .reactions(vec![ReactionType::Unicode(String::from("✅"))])
    }).await {
        println!("Error sending message: print_rules(): {}", why);
    }
}

// use serenity::{
//     model::{channel::Message, id::ChannelId},
//     prelude::Context,
//     utils::Color,
// };

// pub fn edit_message(ctx: &Context) {
//     let mut message: Message = ChannelId(714691912787034113).message(&ctx.http, 716047715208921110).unwrap();
//     if let Err(why) = message.edit(&ctx.http, |m| {
//         m.embed(|e| {
//             e.title("Rules")
//                 .description("This is a place where you learn a little bit more of the base rules to follow while chilling in the community (^-^):")
//                 .color(Color::from_rgb(0, 81, 255))
//                 .field(
//                     "❤ Be respectful",
//                     "Please no hateful speech towards anyone for any reason. This is a place where we can all relax, so please don't start any arguments or fights. Any instigating of arguments or fights will lead to a timeout of the server.",
//                     false,
//                 )
//                 .field(
//                     "🔞 No NSFW content",
//                     "Please no NSFW content. We like to keep things wholesome, so nothing NSFW of the sort around here. Swearing is alright, just try to keep things PG-13.",
//                     false,
//                 )
//                 .field("💬 No spamming", "Try not to spam the chat, it just clutters other messages and sometimes just gets plain annoying. ", false)
//                 .field("🔊 Voice", "No ear rape, NSFW, or any sounds that could damage hearing.", false)
//                 .field(
//                     "📜 Follow Discord Guidelines",
//                     "Anything that goes against discord guidelines also goes against our guidelines. Understand the rules and follow them.",
//                     false,
//                 )
//                 .field(
//                     "😊 Community Heads & Staff",
//                     "If you need any help or want to suggest anything for the server you can ask any of the community heads and/or staff.",
//                     false,
//                 )
//                 .field("Heads & Staff", "<@&714698133057175633>\n<@&716137930644652072>", false)
//         })
//     }) {
//         println!("{}", why);
//     };
// }

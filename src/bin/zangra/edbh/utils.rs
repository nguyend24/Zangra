use serenity::client::Context;
use serenity::model::id::{GuildId, ChannelId};
use serenity::model::voice::VoiceState;
use crate::utils::voice::{VoiceStateChange, identify_state};
use chrono::Utc;
use chrono_tz::US::Eastern;
use serenity::utils::Color;

const LOG_CHANNEL_ID: u64 = 805186168647974964;

pub async fn voice_state_changed(ctx: &Context, guild_id: &GuildId, old: &Option<VoiceState>, new: &VoiceState) {
    if let Some(state_change) = identify_state(guild_id, &old, &new) {
        match state_change {
            VoiceStateChange::LeftVoiceChannel => {
                left_voice_channel(ctx, guild_id, old, new).await;
            }
            VoiceStateChange::JoinedVoiceChannel => {
                joined_voice_channel(ctx, guild_id, new).await;
            }
            VoiceStateChange::MovedVoiceChannel => {
                moved_voice_channel(ctx, guild_id, old, new).await;
            }
            VoiceStateChange::ServerDeafened => {}
            VoiceStateChange::ServerMuted => {}
            VoiceStateChange::SelfDeafened => {}
            VoiceStateChange::SelfMuted => {}
            VoiceStateChange::_SelfStream => {}
            VoiceStateChange::SelfVideo => {}
            VoiceStateChange::Suppress => {}
        }
    }
}

async fn left_voice_channel(ctx: &Context, _guild_id: &GuildId, _old: &Option<VoiceState>, new: &VoiceState) {
    let member = new.member.as_ref().unwrap();
    let name: String = member.user.name.clone();
    let icon_url: String = member.user.face();

    if let Err(why) = ChannelId(LOG_CHANNEL_ID).send_message(&ctx, |m| m
        .embed(|e| e
            .color(Color::RED)
            .title(format!("{} left Voice Channel", &name))
            .author(|a| a
                .name(&name)
                .icon_url(icon_url)
            )
            .field("Eastern Time", get_eastern_time(), false))).await {
        println!("Error sending BH left message. Why: {}", why);
    };
}

async fn joined_voice_channel(ctx: &Context, _guild_id: &GuildId, new: &VoiceState) {
    let member = new.member.as_ref().unwrap();
    let name: String = member.user.name.clone();
    let icon_url: String = member.user.face();

    if let Err(why) = ChannelId(LOG_CHANNEL_ID).send_message(&ctx, |m| m
        .embed(|e| e
            .color(Color::from_rgb(0, 255, 0)) //Green
            .title(format!("{} joined Voice Channel", &name))
            .author(|a| a
                .name(&name)
                .icon_url(icon_url)
            )
            .field("Eastern Time", get_eastern_time(), false))).await {
        println!("Error sending BH join message. Why: {}", why);
    };
}

async fn moved_voice_channel(ctx: &Context, _guild_id: &GuildId, old: &Option<VoiceState>, new: &VoiceState) {
    let member = new.member.as_ref().unwrap();
    let name: String = member.user.name.clone();
    let icon_url: String = member.user.face();

    let old_channel_id = old.as_ref().unwrap().channel_id.as_ref().unwrap();
    let new_channel_id = new.channel_id.as_ref().unwrap();
    let old_channel_name = match old_channel_id.name(ctx).await {
        Some(name) => name,
        None => "No channel name found".to_string()
    };
    let new_channel_name = match new_channel_id.name(ctx).await {
        Some(name) => name,
        None => "No channel name found".to_string()
    };

    if let Err(why) = ChannelId(LOG_CHANNEL_ID).send_message(ctx, |m| m
        .embed(|e| e
            .color(Color::GOLD)
            .title(format!("{} moved {} -> {}", &name, old_channel_name, new_channel_name))
            .author(|a| a
                .name(&name)
                .icon_url(icon_url))
            .field("Eastern Time", get_eastern_time(), false))).await {
        println!("Error sending BH move message. Why: {}", why);
    }
}

//Returns current in Eastern timezone as "YYYY-MM-DD HH:MM:SS AM/PM" 12 hour format
fn get_eastern_time() -> String {
    Utc::now().with_timezone(&Eastern).format("%F %r").to_string()
}
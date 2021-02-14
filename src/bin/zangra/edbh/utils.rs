use serenity::client::Context;
use serenity::model::id::{GuildId, ChannelId};
use serenity::model::voice::VoiceState;
use crate::utils::voice::{VoiceStateChange, identify_state};
use chrono::Utc;

const DEFAULT_AVATAR: &str = "https://www.denofgeek.com/wp-content/uploads/2020/06/Discord.png";

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
    let icon_url: String = match member.user.avatar_url() {
        Some(url) => url,
        None => String::from(DEFAULT_AVATAR)
    };

    if let Err(why) = ChannelId(805186168647974964).send_message(&ctx, |m| m
        .embed(|e| e
            .title(format!("{} left Voice Channel", &name))
            .author(|a| a
                .name(&name)
                .icon_url(icon_url)
            )
            .timestamp(Utc::now().to_rfc3339()))).await {
        println!("Error sending BH left message. Why: {}", why);
    };
}

async fn joined_voice_channel(ctx: &Context, _guild_id: &GuildId, new: &VoiceState) {
    let member = new.member.as_ref().unwrap();
    let name: String = member.user.name.clone();
    let icon_url: String = match member.user.avatar_url() {
        Some(url) => url,
        None => String::from(DEFAULT_AVATAR)
    };

    if let Err(why) = ChannelId(805186168647974964).send_message(&ctx, |m| m
        .embed(|e| e
            .title(format!("{} joined Voice Channel", &name))
            .author(|a| a
                .name(&name)
                .icon_url(icon_url)
            )
            .timestamp(Utc::now().to_rfc3339()))).await {
        println!("Error sending BH join message. Why: {}", why);
    };
}

async fn moved_voice_channel(ctx: &Context, _guild_id: &GuildId, old: &Option<VoiceState>, new: &VoiceState) {
    let member = new.member.as_ref().unwrap();
    let name: String = member.user.name.clone();
    let icon_url: String = match member.user.avatar_url() {
        Some(url) => url,
        None => String::from(DEFAULT_AVATAR)
    };

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

    if let Err(why) = ChannelId(805186168647974964).send_message(ctx, |m| m
        .embed(|e| e
            .title(format!("{} moved {} -> {}", &name, old_channel_name, new_channel_name))
            .author(|a| a
                .name(&name)
                .icon_url(icon_url))
            .timestamp(Utc::now().to_rfc3339()))).await {
        println!("Error sending BH move message. Why: {}", why);
    }
}
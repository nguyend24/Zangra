use serenity::model::id::GuildId;
use serenity::model::voice::VoiceState;
use std::fmt::{self, Display, Formatter};


pub enum VoiceStateChange {
    LeftVoiceChannel,
    JoinedVoiceChannel,
    MovedVoiceChannel,
    ServerDeafened,
    ServerMuted,
    SelfDeafened,
    SelfMuted,
    _SelfStream,
    SelfVideo,
    Suppress,
}

impl Display for VoiceStateChange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            VoiceStateChange::LeftVoiceChannel => f.write_str("Left voice channel"),
            VoiceStateChange::JoinedVoiceChannel => f.write_str("Joined voice channel"),
            VoiceStateChange::MovedVoiceChannel => f.write_str("Moved voice channel"),
            VoiceStateChange::ServerDeafened => f.write_str("Server deafened"),
            VoiceStateChange::ServerMuted => f.write_str("Server muted"),
            VoiceStateChange::SelfDeafened => f.write_str("Self deafened"),
            VoiceStateChange::SelfMuted => f.write_str("Self muted"),
            VoiceStateChange::_SelfStream => f.write_str("Self stream"),
            VoiceStateChange::SelfVideo => f.write_str("Self video"),
            VoiceStateChange::Suppress => f.write_str("Suppress"),
        }
    }
}

pub fn identify_state(_guild_id: &GuildId, old_state: &Option<VoiceState>, new: &VoiceState) -> Option<VoiceStateChange> {
    if let None = old_state {
        return Some(VoiceStateChange::JoinedVoiceChannel)
    }
    let old_state: &VoiceState = old_state.as_ref().unwrap();

    if let Some(old_id) = old_state.channel_id {
        if let Some(new_id) = new.channel_id {
            if old_id.as_u64() != new_id.as_u64() {
                return Some(VoiceStateChange::MovedVoiceChannel)
            }
        }
    }

    if let None = new.channel_id {
        return Some(VoiceStateChange::LeftVoiceChannel)
    }

    if old_state.deaf != new.deaf {
        return Some(VoiceStateChange::ServerDeafened)
    }

    if old_state.mute != new.mute {
        return Some(VoiceStateChange::ServerMuted)
    }

    if old_state.self_deaf != new.self_deaf {
        return Some(VoiceStateChange::SelfDeafened)
    }

    if old_state.self_mute != new.self_mute {
        return Some(VoiceStateChange::SelfMuted)
    }

    if old_state.self_video != new.self_video {
        return Some(VoiceStateChange::SelfVideo)
    }

    if old_state.suppress != new.suppress {
        return Some(VoiceStateChange::Suppress)
    }

    None
}
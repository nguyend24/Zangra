
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serenity::model::{
    id::{ChannelId, GuildId},
    prelude::{ChannelType, GuildInfo},
};
use tracing::error;

use crate::rest_api::entry::AppState;

/// /api/guilds/:guild_id/channels
pub async fn get_guild_channels(
    State(state): State<AppState>,
    Path(guild_id): Path<u64>,
) -> Result<Json<Vec<(ChannelId, String)>>, StatusCode> {
    let guild_id = GuildId(guild_id);
    let ctx = state.ctx;
    let mut channels: Vec<(ChannelId, String)> = Vec::new();

    match guild_id.channels(&ctx).await {
        Ok(c) => {
            c.iter()
                .filter(|(_k, v)| matches!(v.kind, ChannelType::Text))
                .for_each(|(k, v)| channels.push((k.to_owned(), v.name.to_owned())));
        }
        Err(why) => {
            error!("{}", why);
            return Err(StatusCode::BAD_GATEWAY);
        }
    };

    Ok(Json(channels))
}

pub async fn post_filter_guilds(State(state): State<AppState>, Json(guilds): Json<Vec<GuildInfo>>) -> Result<Json<Vec<GuildInfo>>, StatusCode> {
    let ctx = state.ctx;
    let mut in_guilds: Vec<GuildInfo> = Vec::new();
    let bot_user = match ctx.http.get_current_user().await {
        Ok(user) => user,
        Err(why) => {
            error!("{why}");
            return Err(StatusCode::BAD_GATEWAY);
        }
    };

    let bot_guild_ids: Vec<u64> = match bot_user.guilds(&ctx).await {
        Ok(guilds) => guilds.iter().map(|g| g.id.0).collect(),
        Err(why) => {
            error!("{why}");
            return Err(StatusCode::BAD_GATEWAY);
        }
    };

    
    for guild in guilds {
        if bot_guild_ids.contains(guild.id.as_u64()) {
            in_guilds.push(guild);
        }
    }

    Ok(Json(in_guilds))
}
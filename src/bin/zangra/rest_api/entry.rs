use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use serenity::{
    builder::CreateEmbed,
    client::Context,
    model::{
        channel::{Embed, Message},
        id::ChannelId,
        ModelError,
    },
    Error,
};
use sqlx::{query, Pool, Sqlite};
use tracing::{error, info};

use crate::{utils::database::DatabasePool, rest_api::routes::{guilds::{get_guild_channels, post_filter_guilds}, channels::post_channel_embed, embed_edit::post_edit_embed}};

#[derive(Clone)]
pub struct AppState {
    pub ctx: Arc<Context>,
    pub db_pool: Pool<Sqlite>,
}

pub async fn start_rest_api(ctx: &Context) -> Result<(), String> {
    let data = ctx.data.read().await;
    let db_pool = data.get::<DatabasePool>().unwrap().clone();

    let ctx = ctx.clone();

    tokio::spawn(async move {
        let ctx = Arc::new(ctx);
        let app_state: AppState = AppState { ctx, db_pool };

        let app = Router::new()
            .route("/", get(root))
            .route("/api/channels/:channel_id/embed", post(post_channel_embed))

            .route("/api/guilds/filter", post(post_filter_guilds))
            .route("/api/guilds/:guild_id/channels", get(get_guild_channels))

            .route("/api/embededit/:channel_id/:message_id", post(post_edit_embed))

            .route("/api/embed/:guild_id/:channel_id", post(post_embed))
            .route(
                "/api/embed/:guild_id/:channel_id/:message_id",
                get(get_embed).put(put_embed).delete(delete_embed),
            )
            .route("/api/embed/all/:guild_id", get(get_embed_all_guild))
            .route(
                "/api/embed/all/:guild_id/:channel_id",
                get(get_embed_all_channel),
            )
            
            .with_state(app_state);

        info!("Starting rest API");

        axum::Server::bind(&"0.0.0.0:4000".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    Ok(())
}

async fn root() {}

/// POST
///
/// Body contains name of action to perform and data need to perform that action within JSON
///
/// {
///     action:
///     data:{
///     channel:
///     embeddata:    
/// }
/// }
// async fn post_api(State(state): State<AppState>, body: String) -> impl IntoResponse {
//     let v: Value = match serde_json::from_str(&body) {
//         Ok(val) => val,
//         Err(why) => {
//             println!("API error: {}", why);
//             return Err(StatusCode::BAD_GATEWAY);
//         }
//     };

//     let action = match v["action"].as_str() {
//         Some(s) => s,
//         None => {
//             println!("API error: Can't read action field");
//             return Err(StatusCode::BAD_REQUEST);
//         }
//     };

//     let discord_action: DiscordAction = match action.parse() {
//         Ok(a) => a,
//         Err(_why) => return Err(StatusCode::BAD_REQUEST),
//     };

//     let action_runner = ActionRunner {
//         ctx: state.ctx.clone(),
//         db_pool: state.db_pool.clone(),
//         discord_action,
//         data: v["data"].clone(),
//     };

//     Ok(action_runner.run().await)
// }

/// Get a single embed
async fn get_embed(
    State(state): State<AppState>,
    Path((guild_id, channel_id, message_id)): Path<(u64, u64, u64)>,
) -> Result<Json<Value>, StatusCode> {
    let _guild_id = guild_id;

    match ChannelId(channel_id)
        .message(state.ctx.clone(), message_id)
        .await
    {
        Ok(msg) => {
            Ok(Json(json!(msg)))
        }
        Err(why) => {
            error!("{}", why);
            Ok(Json(json!({
                "status": "error",
                "error": "NoMessage",
            })))
        }
    }
}

/// create an embed
async fn post_embed(
    State(state): State<AppState>,
    Path((guild_id, channel_id)): Path<(u64, u64)>,
    Json(embed): Json<Embed>,
) -> Result<Json<Value>, StatusCode> {
    let message = match ChannelId(channel_id)
        .send_message(state.ctx.clone(), |m| {
            m.set_embed(CreateEmbed::from(embed));
            m
        })
        .await
    {
        Ok(msg) => msg,
        Err(Error::Model(ModelError::MessageTooLong(mtl))) => {
            return Ok(Json(json!(
                {
                    "status": "error",
                    "error": "MessageTooLong",
                    "mtl": mtl
                }
            )))
        }
        Err(Error::Http(http_err)) => {
            error!("{}", http_err);
            //missing permissions
            return Ok(Json(json!(
                {
                    "status": "error",
                    "error": "MissingPermissions"
                }
            )));
        }
        Err(why) => {
            //anything else
            //maybe channel doesn't exist or guild doesn't exist
            error!("{}", why);
            return Ok(Json(json!(
                {
                    "status": "error",
                    "error": "Unknown"
                }
            )));
        }
    };

    //add to db
    let message_id_i64 = message.id.0 as i64;
    let guild_id_i64 = guild_id as i64;
    let channel_id_i64 = channel_id as i64;
    match query!(
        "INSERT INTO Embed (EmbedId, GuildId, ChannelId) VALUES (?, ?, ?)",
        message_id_i64,
        guild_id_i64,
        channel_id_i64
    )
    .execute(&state.db_pool.clone())
    .await
    {
        Ok(_result) => {
            return Ok(Json(json!(message)));
        }
        Err(why) => {
            error!("{}", why);
            match message.delete(state.ctx.clone()).await {
                Ok(()) => {}
                Err(why) => {
                    error!("{}", why);
                }
            };
        }
    };

    Err(StatusCode::BAD_GATEWAY)
}

/// delete an embed
async fn delete_embed(
    State(state): State<AppState>,
    Path((guild_id, channel_id, message_id)): Path<(u64, u64, u64)>,
) -> Result<Json<Value>, StatusCode> {
    let _guild_id = guild_id;

    match ChannelId(channel_id)
        .message(state.ctx.clone(), message_id)
        .await
    {
        Ok(msg) => match msg.delete(state.ctx.clone()).await {
            Ok(()) => {}
            Err(why) => {
                error!("{}", why);
            }
        },
        Err(why) => {
            //can't get message
            //maybe message doesn't exist or channel doesn't exist
            error!("{}", why);
        }
    }

    Err(StatusCode::BAD_GATEWAY)
}

/// edit an embed
/// returns the message that was created if successful
async fn put_embed(
    State(state): State<AppState>,
    Path((guild_id, channel_id, message_id)): Path<(u64, u64, u64)>,
    Json(embed): Json<Embed>,
) -> Result<Json<Value>, StatusCode> {
    let _guild_id = guild_id;

    let mut message = match ChannelId(channel_id)
        .message(state.ctx.clone(), message_id)
        .await
    {
        Ok(msg) => msg,
        Err(why) => {
            //can't get message
            //maybe message doesn't exist or channel doesn't exist
            error!("{}", why);
            return Err(StatusCode::BAD_GATEWAY);
        }
    };

    match message
        .edit(state.ctx.clone(), |m| {
            m.set_embed(CreateEmbed::from(embed));
            m
        })
        .await
    {
        Ok(()) => {}
        Err(Error::Model(ModelError::MessageTooLong(mtl))) => {
            error!("MessageTooLong: {}", mtl);
            return Ok(Json(json!(
                {
                    "status": "error",
                    "error": "MessageTooLong",
                    "mtl": mtl
                }
            )));
        }
        Err(Error::Model(ModelError::InvalidUser)) => {
            return Ok(Json(json!(
                {
                    "status": "error",
                    "error": "InvalidUser",
                }
            )))
        }
        Err(why) => {
            error!("{}", why);
        }
    }

    Err(StatusCode::BAD_GATEWAY)
}

/// get all embeds in the entire server
async fn get_embed_all_guild(
    State(state): State<AppState>,
    Path(guild_id): Path<u64>,
) -> Result<Json<Value>, StatusCode> {
    let guild_id_i64 = guild_id as i64;
    let db_results = match query!(
        "SELECT EmbedId, GuildId, ChannelId FROM Embed 
        WHERE GuildId = ? 
        ORDER BY ChannelId",
        guild_id_i64
    )
    .fetch_all(&state.db_pool.clone())
    .await
    {
        Ok(rows) => rows,
        Err(why) => {
            error!("Unable to read from DB: {}", why);
            return Err(StatusCode::BAD_GATEWAY);
        }
    };

    let mut message_map: HashMap<ChannelId, Vec<Message>> = HashMap::new();

    for row in db_results {
        //group by channel
        let channel_id = ChannelId(row.ChannelId as u64);
        match channel_id
            .message(state.ctx.clone(), row.EmbedId as u64)
            .await
        {
            Ok(msg) => match message_map.get_mut(&channel_id) {
                Some(vec) => vec.push(msg),
                None => {
                    message_map.insert(channel_id, Vec::new());
                    message_map.get_mut(&channel_id).unwrap().push(msg);
                }
            },
            Err(Error::Http(http_err)) => {
                //missing read history permissions
                error!("{}", http_err);
                return Ok(Json(json!(
                    {
                        "status": "error",
                        "error": "MissingPermissions"
                    }
                )));
            }
            Err(why) => {
                //unable to retrieve message for some reason
                error!("{}", why);
            }
        };
    }

    Ok(Json(json!(
        {
            "status": "success",
            "messages": message_map
        }
    )))
}

/// get all embds in a single channel
async fn get_embed_all_channel(
    State(state): State<AppState>,
    Path((guild_id, channel_id)): Path<(u64, u64)>,
) -> Result<Json<Value>, StatusCode> {
    let _guild_id = guild_id;

    let channel_id_i64 = channel_id as i64;
    let db_results = match query!(
        "SELECT EmbedId, GuildId, ChannelId FROM Embed 
        WHERE ChannelId = ?",
        channel_id_i64
    )
    .fetch_all(&state.db_pool.clone())
    .await
    {
        Ok(rows) => rows,
        Err(why) => {
            error!("Unable to read from DB: {}", why);
            return Err(StatusCode::BAD_GATEWAY);
        }
    };

    let mut message_map: HashMap<ChannelId, Vec<Message>> = HashMap::new();

    for row in db_results {
        let channel_id = ChannelId(row.ChannelId as u64);
        match channel_id
            .message(state.ctx.clone(), row.EmbedId as u64)
            .await
        {
            Ok(msg) => match message_map.get_mut(&channel_id) {
                Some(vec) => vec.push(msg),
                None => {
                    message_map.insert(channel_id, Vec::new());
                    message_map.get_mut(&channel_id).unwrap().push(msg);
                }
            },
            Err(Error::Http(http_err)) => {
                //missing read history permissions
                error!("{}", http_err);
                return Ok(Json(json!(
                    {
                        "status": "error",
                        "error": "MissingPermissions"
                    }
                )));
            }
            Err(why) => {
                //unable to retrieve message for some reason
                error!("{}", why);
            }
        };
    }

    Ok(Json(json!(
        {
            "status": "success",
            "messages": message_map
        }
    )))
}

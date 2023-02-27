use axum::{extract::{State, Path}, http::StatusCode, Json};
use tracing::error;

use crate::rest_api::entry::AppState;

use super::channels::EmbedDetails;


pub async fn post_edit_embed(State(state): State<AppState>, Path((channel_id, message_id)): Path<(u64, u64)>, Json(embed_payload): Json<EmbedDetails>) -> StatusCode {
    let ctx = state.ctx.clone();
    let mut message = match ctx.http.get_message(channel_id, message_id).await {
        Ok(msg) => msg,
        Err(why) => {
            error!("{why}");
            return StatusCode::BAD_GATEWAY;
        }
    };

    match message.edit(&ctx, |e| {
        e.set_embed(embed_payload.into())
    }).await {
        Ok(()) => StatusCode::OK,
        Err(why) => {
            error!("{why}");
            StatusCode::BAD_GATEWAY
        }
    }

    // match ctx.http.get_message(channel_id, message_id).await.unwrap().id().send_message(&ctx, |m| {
    //     m.set_embed(embed_payload.into())
    // }).await {
    //     Ok(_msg) => {
    //         StatusCode::OK
    //     },
    //     Err(why) => {
    //         println!("{why}");
    //         StatusCode::BAD_GATEWAY
    //     }

    // }
    // StatusCode::OK
}
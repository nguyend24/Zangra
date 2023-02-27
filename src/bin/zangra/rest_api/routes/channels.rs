use axum::{extract::{State, Path}, http::StatusCode, Json};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use serenity::{builder::CreateEmbed, utils::Color};

use crate::rest_api::entry::AppState;
#[derive(Clone, Deserialize, Serialize)]
pub struct EmbedDetails {
    embed_title: Option<String>,
    embed_title_url: Option<String>,
    embed_description: Option<String>,
    embed_image_url: Option<String>,
    embed_thumbnail_url: Option<String>,
    embed_color: Option<String>
}

impl From<EmbedDetails> for CreateEmbed {
    fn from(ed: EmbedDetails) -> Self {
        let mut embed = CreateEmbed::default();

        if let Some(title) = ed.embed_title {
            embed.title(title);
        }

        if let Some(title_url) = ed.embed_title_url {
            embed.url(title_url);
        }

        if let Some(description) = ed.embed_description {
            embed.description(description);
        }

        if let Some(image_url) = ed.embed_image_url {
            embed.image(image_url);
        }

        if let Some(thumbnail_url) = ed.embed_thumbnail_url {
            embed.thumbnail(thumbnail_url);
        }

        if let Some(color) = ed.embed_color { 
            //hex to rgb conversion
            let r = u8::from_str_radix(&color[1..3], 16).unwrap();
            let g = u8::from_str_radix(&color[3..5], 16).unwrap();
            let b = u8::from_str_radix(&color[5..7], 16).unwrap();

            embed.color(Color::from_rgb(r, g, b));
        }

        embed
    }
}

#[debug_handler]
pub async fn post_channel_embed(State(state): State<AppState>, Path(channel_id): Path<u64>, Json(embed_payload): Json<EmbedDetails>) -> StatusCode {
    let ctx = state.ctx.clone();


    match ctx.http.get_channel(channel_id).await.unwrap().id().send_message(&ctx, |m| {
        m.set_embed(embed_payload.into())
    }).await {
        Ok(_msg) => {
            StatusCode::OK
        },
        Err(why) => {
            println!("{why}");
            StatusCode::BAD_GATEWAY
        }

    }
    // StatusCode::OK
}
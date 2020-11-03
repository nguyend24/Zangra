use std::env;

use crate::twitch::helix::webhooks::*;
use serenity::client::Context;
use serenity::model::id::ChannelId;

use serde_json::{Value};

use serenity::utils::Color;

use crate::twitch::utils;

use crate::twitch::TwitchClient;
use serenity::model::channel::Message;

use crate::twitch::helix::games::Game;
use std::sync::{Arc, RwLock};
use serenity::http::Http;
use serenity::cache::{Cache};
use crate::twitch::helix::users::User;

use rand::prelude::*;

pub(crate) struct WebhookHandler {
    http: Arc<Http>,
    cache: Arc<Cache>,
}

#[async_trait::async_trait]
impl WebhookEvents for WebhookHandler {
    async fn stream_live(&self, response: WebhookResponse) {
        send_twitch_live_message(&self.http, &response);
    }

    async fn stream_offline(&self) {
        if let Err(why) = ChannelId(557572781680754699).say(&self.http, "An unknown stream has gone offline").await {
            println!("Error sending stream offline message: {}", why);
        };
    }

    async fn ready(&self) {
        println!("Twitch webhooks listener ready!");
    }
}

pub fn set_up_twitch_webhooks(ctx: Context) {
    let client_id = env::var("TWITCH_CLIENT_ID").expect("Twitch Client ID not found in environment.");
    let bearer_token = env::var("TWITCH_BEARER_TOKEN").expect("Twitch bearer token not found in environment.");

    let webhook_handler = WebhookHandler {
        http: ctx.http,
        cache: ctx.cache,
    };
    let webhook_client = WebhookClient::new(webhook_handler, &client_id, &bearer_token);
    webhook_client.start();
}


const THUMBNAIL_WIDTH: u64 = 360;
const THUMBNAIL_HEIGHT: u64 = 180;
fn send_twitch_live_message(http: &Arc<Http>, response: &WebhookResponse) {
    const TWITCH_COLOR: Color = Color::from_rgb(100, 65, 164);

    let url: String = String::from("https://www.twitch.tv/") + &response.user_name;

    let user = match TwitchClient::get_users(&response.user_name) {
        Some(u) => u,
        None => User {
            id: 0,
            login: "".to_string(),
            display_name: "User not found".to_string(),
            user_type: "".to_string(),
            broadcaster_type: "".to_string(),
            description: "".to_string(),
            profile_image_url: "".to_string(),
            offline_image_url: "".to_string(),
            view_count: 0
        }
    };

    let game: Game = match TwitchClient::get_games(&response.game_id) {
        Some(g) => g, 
        None => Game {
            box_art_url: "http://www.noboxart.com".to_string(),
            id: "000000".to_string(),
            name: "Game not found".to_string()
        }
    };

    let thumbnail_url = response.thumbnail_url
        .replace("{width}", THUMBNAIL_WIDTH.to_string().as_str())
        .replace("{height}", THUMBNAIL_HEIGHT.to_string().as_str()) + thread_rng().gen_range(10000, 999999).to_string().as_str();

    if let Err(why) = ChannelId(557572781680754699).send_message(http, |m| {
        m.embed(|e| {
            e.author(|a| a.icon_url(&user.profile_image_url).name(&user.display_name))
                .title(&response.title)
                .url(&url)
                .thumbnail(&game.box_art_url.replace("{width}", "130").replace("{height}", "172"))
                .field("Game", &game.name, true)
                .field("Viewers", &response.viewer_count, true)
                .image(&thumbnail_url)
                .color(TWITCH_COLOR)
        })
    }) {
        println!("send_twitch_live_message(): Error sending message: {}", why);
    };
}
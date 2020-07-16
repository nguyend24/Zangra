pub mod helix;
pub mod utils;

use reqwest::blocking::Client;
use std::env;

use crate::twitch::helix::games::*;
use crate::twitch::helix::users::*;
use crate::twitch::utils::extract_data;

pub struct TwitchClient {

}

impl TwitchClient {
    pub fn get_users(user_login: &str) -> Option<User>{
        let client_id: String = env::var("TWITCH_CLIENT_ID").expect("Twitch Client ID not found in environment.");
        let bearer_token: String = env::var("TWITCH_BEARER_TOKEN").expect("Twitch bearer token not found in environment.");
        let mut query_url = String::from("https://api.twitch.tv/helix/users?login=");
        &mut query_url.push_str(user_login);

        let response = Client::new()
            .get(&query_url)
            .header("Client-Id", &client_id)
            .bearer_auth(&bearer_token)
            .send()
            .unwrap()
            .text()
            .unwrap();

        User::new(extract_data(&response).unwrap())
    }

    pub fn get_games(game_id: &str) -> Option<Game> {
        let client_id: String = env::var("TWITCH_CLIENT_ID").expect("Twitch Client ID not found in environment.");
        let bearer_token: String = env::var("TWITCH_BEARER_TOKEN").expect("Twitch bearer token not found in environment.");
        let mut query_url = String::from("https://api.twitch.tv/helix/games?id=");
        &mut query_url.push_str(game_id);

        let response = Client::new()
            .get(&query_url)
            .header("Client-Id", &client_id)
            .bearer_auth(&bearer_token)
            .send()
            .unwrap()
            .text()
            .unwrap();

        Game::new(extract_data(&response).unwrap())
    }
}

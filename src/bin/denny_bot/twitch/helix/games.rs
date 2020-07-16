use crate::twitch::utils;
use serde_json::{json, Value};

pub struct Game {
    pub box_art_url: String,
    pub id: String,
    pub name: String,
}

impl Game {
    pub fn new<S: Into<String>>(json_response: S) -> Option<Game> {
        let data = json_response.into();

        if let Ok(json_value) = serde_json::from_str(data.as_str()) {
            let json_value: Value = json_value;
            Some(Game {
                box_art_url: String::from(json_value.get("box_art_url")?.as_str()?),
                id: String::from(json_value.get("id")?.as_str()?),
                name: String::from(json_value.get("name")?.as_str()?),
            })
        } else {
            None
        }


    }
}

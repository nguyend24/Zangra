use serde_json::{json, Value};

use crate::twitch::utils;
use std::str::FromStr;

pub struct User {
    pub id: u64,
    pub login: String,
    pub display_name: String,
    pub user_type: String,
    pub broadcaster_type: String,
    pub description: String,
    pub profile_image_url: String,
    pub offline_image_url: String,
    pub view_count: u64,
}

impl User {
    pub fn new<S: Into<String>>(json_response: S) -> Option<User> {
        let data = json_response.into();

        if let Ok(json_value) = serde_json::from_str(data.as_str()){
            let json_value: Value = json_value;
            Some(User {
                id: u64::from_str(json_value.get("id")?.as_str()?).unwrap(),
                login: String::from(json_value.get("login")?.as_str()?),
                display_name: String::from(json_value.get("display_name")?.as_str()?),
                user_type: String::from(json_value.get("type")?.as_str().unwrap_or("")),
                broadcaster_type: String::from(json_value.get("broadcaster_type")?.as_str().unwrap_or("")),
                description: String::from(json_value.get("description")?.as_str()?),
                profile_image_url: String::from(json_value.get("profile_image_url")?.as_str().unwrap_or("")),
                offline_image_url: String::from(json_value.get("offline_image_url")?.as_str().unwrap_or("")),
                view_count: json_value.get("view_count")?.as_u64().unwrap_or(0),
            })
        } else {
            None
        }


    }
}

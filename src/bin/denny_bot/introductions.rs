use chrono::{Date, Local};
use std::collections::HashMap;


pub struct Introduction {
    name: String,
    birthday: String,
    twitch: String,
    instagram: String,
    hobbies: String,
    games: String,
    favorite_color: String,
    favorite_food: String,
    fun_fact: String,
}

pub fn parse<S: Into<String>>(msg: S) -> Option<Introduction>{
    let mut msg: String = msg.into();
    let mut temp: String = "".to_string();

    temp = msg[msg.rfind("Name:")?..].to_string();
    let name: String = temp[..temp.find("\n")?].to_string();

    temp = msg[msg.rfind("Birthday:")?..].to_string();
    let birthday: String = temp[..temp.find("\n")?].to_string();

    temp = msg[msg.rfind("Twitch:")?..].to_string();
    let twitch: String = temp[..temp.find("\n")?].to_string();

    temp = msg[msg.rfind("Instagram:")?..].to_string();
    let instagram: String = temp[..temp.find("\n")?].to_string();

    temp = msg[msg.rfind("Hobbies:")?..].to_string();
    let hobbies: String = temp[..temp.find("\n")?].to_string();

    temp = msg[msg.rfind("Games:")?..].to_string();
    let games: String = temp[..temp.find("\n")?].to_string();

    temp = msg[msg.rfind("Favorite Color:")?..].to_string();
    let favorite_color: String = temp[..temp.find("\n")?].to_string();

    temp = msg[msg.rfind("Favorite Food:")?..].to_string();
    let favorite_food: String = temp[..temp.find("\n")?].to_string();

    temp = msg[msg.rfind("Fun Fact:")?..].to_string();
    let fun_fact: String = temp[..temp.find("\n")?].to_string();

    Some(Introduction {
        name,
        birthday,
        twitch,
        instagram,
        hobbies,
        games,
        favorite_color,
        favorite_food,
        fun_fact
    })
}
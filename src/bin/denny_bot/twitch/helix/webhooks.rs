use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, mpsc};
use std::{thread, env};

use reqwest::blocking::RequestBuilder;

use crate::twitch::utils::*;
use std::time::Duration;
use std::str::FromStr;
use serde_json::Value;

pub struct WebhookClient {
    subscriptions: Arc<Vec<String>>,
    event_handler: Box<dyn WebhookEvents>,
    client_id: String,
    bearer_token: String
}

impl WebhookClient {
    pub fn new<T>(event_handler: T, client_id: impl AsRef<str>, bearer_token:  impl AsRef<str>) -> WebhookClient
    where
        T: WebhookEvents + 'static,
    {
        WebhookClient {
            subscriptions: Arc::new(vec![]),
            event_handler: Box::new(event_handler),
            client_id: String::from(client_id.as_ref()),
            bearer_token: String::from(bearer_token.as_ref())
        }
    }

    pub fn subscribe() {
        println!("subscribing");
        thread::spawn(|| {
            let client_id = env::var("TWITCH_CLIENT_ID").expect("Twitch Client ID not found in environment.");
            let bearer_token = env::var("TWITCH_BEARER_TOKEN").expect("Twitch bearer token not found in environment.");

            let mut subscriptions = Vec::new();

            subscriptions.push(r#"
            {"hub.callback": "http://duy-monroe.ddns.net:5000/stream_changed",
        "hub.mode": "subscribe",
        "hub.topic": "https://api.twitch.tv/helix/streams?user_id=532731305",
        "hub.lease_seconds": 10000
    }"#);

            subscriptions.push(r#"
            {"hub.callback": "http://duy-monroe.ddns.net:5000/stream_changed",
        "hub.mode": "subscribe",
        "hub.topic": "https://api.twitch.tv/helix/streams?user_id=87539788",
        "hub.lease_seconds": 10000
    }"#);

            subscriptions.push(r#"
            {"hub.callback": "http://duy-monroe.ddns.net:5000/stream_changed",
        "hub.mode": "subscribe",
        "hub.topic": "https://api.twitch.tv/helix/streams?user_id=519581041",
        "hub.lease_seconds": 10000
    }"#);

            loop {
                for sub in &subscriptions {
                    let request: Value = Value::from_str(sub).unwrap();
                    let response = reqwest::blocking::Client::new()
                        .post("https://api.twitch.tv/helix/webhooks/hub")
                        .json(&request)
                        .header("Client-ID", &client_id)
                        .bearer_auth(&bearer_token)
                        .send()
                        .unwrap()
                        .text();
                    println!("{}", response.unwrap());
                }

                thread::sleep(Duration::from_secs(9990));
            }
        });
    }

    pub fn start(self) {
        let arc_ref = self.subscriptions.clone();

        thread::spawn(move || {
            //Opens listener for events and challenge requests
            let listener = match TcpListener::bind("0.0.0.0:5000") {
                Ok(tcp_listener) => tcp_listener,
                Err(why) => panic!("Unable to open TCP Listener: {}", why),
            };

            WebhookClient::subscribe();
            let handler = self.event_handler;
            &handler.ready();
            for stream in listener.incoming() {
                handle_request(&handler,  stream.unwrap());
            }
        });
    }
}

pub struct WebhookResponse {
    pub game_id: String,
    pub language: String,
    pub thumbnail_url: String,
    pub title: String,
    pub stream_type: String,
    pub user_id: String,
    pub user_name: String,
    pub viewer_count: u64
}

static CHALLENGE_IDENTIFIER: &str = "hub.challenge=";
static STREAM_CHANGED_EVENT_IDENTIFIER: &str = "POST /stream_changed";

fn handle_request(event_handler: &Box<dyn WebhookEvents>, mut stream: TcpStream) {
    let stream_info = stream_to_str(&mut stream);

    if stream_info.contains(CHALLENGE_IDENTIFIER) { //Response to twitch Webhook challenge verification
        HttpResponse::respond_to_challenge(&mut stream, &stream_info);
        return (); //respond_to_challenge sends OK reponse with a body, so end handle_request here to prevent sending another OK
    } else if stream_info.contains(STREAM_CHANGED_EVENT_IDENTIFIER) { //Twitch stream has gone live or offline
        handle_stream_changed(event_handler, stream_info.as_str());
    }

    println!("{}", stream_info);
    HttpResponse::respond_http_ok(&mut stream); //Send OK response to stop retries
}

fn handle_stream_changed(event_handler: &Box<dyn WebhookEvents>, response: &str) -> Option<()>{
    if let Some(data) = extract_data(response) {
        let json = serde_json::from_str(data.as_str());
        if let Err(why) = json {
            println!("handle_stream_changed: {}", &data);
            return None;
        }
        let json: Value = json.unwrap();
        let webhook_response = WebhookResponse {
            game_id: json["game_id"].as_str()?.to_owned(),
            language: json["language"].as_str()?.to_owned(),
            thumbnail_url: json["thumbnail_url"].as_str()?.to_owned(),
            title: json["title"].as_str()?.to_owned(),
            stream_type: json["type"].as_str()?.to_owned(),
            user_id: json["user_id"].as_str()?.to_owned(),
            user_name: json["user_name"].as_str()?.to_owned(),
            viewer_count: json["viewer_count"].as_u64()?
        };
        event_handler.stream_live(webhook_response);
    } else {
        event_handler.stream_offline();
    }

    Some(())
}

pub trait WebhookEvents: Send {
    fn stream_live(&self, response: WebhookResponse) {}

    fn stream_offline(&self) {}

    fn ready(&self) {}
}

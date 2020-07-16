use std::io::{Read, Write};

use std::net::TcpStream;

pub struct HttpResponse;

impl HttpResponse {
    pub fn respond_to_challenge(stream: &mut TcpStream, stream_response: &str) {
        println!("Responding to twitch webhook challenge.");

        let challenge = match extract_challenge(&stream_response.to_string()) {
            Ok(c) => c,
            Err(_) => {
                println!("Challenge return unsuccessful. Challenge token not found.");
                String::from("")
            }
        };

        let mut response_to_challenge: String = String::from("HTTP/1.1 200 OK\r\n\r\n");
        response_to_challenge.push_str(challenge.as_str());

        if let Err(why) = stream.write(response_to_challenge.as_bytes()) {
            println!("{}", why);
        };
        if let Err(why) = stream.flush() {
            println!("{}", why);
        };
    }

    pub fn respond_http_ok(stream: &mut TcpStream) {
        let ok = "HTTP/1.1 200 OK\r\n\r\n";
        match stream.write(ok.as_bytes()) {
            Ok(bytes) => (),
            Err(why) => println!("HTTP 200 OK Response failed to write: {}", why),
        };
        match stream.flush() {
            Ok(()) => (),
            Err(why) => println!("HTTP 200 OK Response failed to flush: {}", why),
        };
    }
}

pub fn stream_to_str(stream: &mut TcpStream) -> String {
    let mut buffer: [u8; 512] = [0; 512];
    let mut bytes: Vec<u8> = Vec::new();

    loop {
        //Reads raw data from stream into @bytes
        let n = match stream.read(&mut buffer) {
            Ok(result) => result,
            Err(why) => 0,
        };

        for b in &buffer[0..] {
            if *b == 0 {
                break;
            }

            bytes.push(b.clone());
        }

        if n < buffer.len() {
            break;
        }
    }

    String::from_utf8_lossy(&bytes).to_string()
}

fn extract_challenge(response: &str) -> Result<String, String> {
    let start: usize = match response.find("hub.challenge=") {
        Some(num) => num + "hub.challenge=".len(),
        None => 0,
    };

    let end: usize = match response.find("&") {
        Some(num) => num,
        None => 0,
    };

    if start == 0 || end == 0 {
        return Err(String::from("Challenge token not found."));
    }

    let challenge = &response[start..end];
    return Ok(String::from(challenge));
}

pub fn extract_data<S: Into<String>>(response: S) -> Option<String> {
    let mut data = response.into();
    let data_start: usize = data.find("{\"data\":[").unwrap_or(0) + "{\"data\":[".len();
    data = data[data_start..].to_owned();
    let data_end: usize = data.find("]}").unwrap_or(0);

    if data_start == 0 {
        return None;
    }

    let value: &str = &data[..data_end];

    Some(String::from(value))
}
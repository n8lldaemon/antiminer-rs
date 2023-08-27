use std::env;
use std::collections::HashMap;
use reqwest;

pub fn send_webhook(message: &str) -> Result<(), Box<dyn std::error::Error>>{
    let client = reqwest::blocking::Client::new();
    let mut data = HashMap::new();

    data.insert("username", "antiminer-rs");
    data.insert("content", message);

    let resp = client.post(env::var("DISCORD_WEBHOOK").unwrap())
        .json(&data)
        .send()?;

    if !resp.status().is_success() {
        println!("Failed to send webhook!");
    }

    Ok(())
}

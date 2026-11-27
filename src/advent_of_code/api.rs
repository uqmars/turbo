use cached::proc_macro::cached;
use chrono::{Datelike, prelude::Utc};
use reqwest::Client;
use serde_json::Value;
use std::env;

use super::Leaderboard;
use crate::utilities::Error;

async fn retrieve_json(year: Option<u16>) -> Result<Value, Error> {
    let session_cookie = env::var("AOC_COOKIE")
        .expect("Expected an advent of code session cookie in the environment");
    let aoc_leaderboard = env::var("AOC_LEADERBOARD")
        .expect("Expected an advent of code leaderboard code in the environment");
    let leaderboard_url = aoc_leaderboard.split('-').next().unwrap();
    let url = format!(
        "https://adventofcode.com/{}/leaderboard/private/view/{}.json",
        year.unwrap_or(Utc::now().year_ce().1 as u16),
        leaderboard_url
    );

    let response = Client::new()
        .get(&url)
        .header("Cookie", format!("session={}", session_cookie))
        .send()
        .await?;

    let json;
    if response.status().is_success() {
        json = response.json().await?;
    } else {
        json = Value::Null;
        println!("Failed to fetch leaderboard: {}", response.status());
    }

    Ok(json)
}

fn parse_lb_json(lb: Value) -> Leaderboard {
    serde_json::from_value(lb).unwrap()
}

#[cached(max_size = 20, ttl_secs = 900)]
pub async fn retrieve_lb(year: Option<u16>) -> Leaderboard {
    let json: Value = retrieve_json(year).await.unwrap();
    parse_lb_json(json)
}

use reqwest::Response;
use serde::Deserialize;
use std::collections::HashMap;
use std::u8;

use crate::utilities::{Context, Error};
use pyo3::{
    ffi::printfunc,
    prelude::*,
    types::{IntoPyDict, PyModule},
};

mod api;
pub mod db;

#[derive(Clone, Debug, Deserialize)]
struct Star {
    star_index: u16,
    get_star_ts: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct Member {
    name: String,
    id: u64,
    local_score: u64,
    last_star_ts: u64,
    stars: u8,
    completion_day_level: HashMap<String, HashMap<u8, Star>>,
}

#[derive(Clone, Debug, Deserialize)]
struct Leaderboard {
    owner_id: u64,
    day1_ts: u64,
    event: String,
    num_days: u8,
    members: HashMap<String, Member>,
}

/// Actions the Advent of Code command
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("register", "unregister", "reg_check", "leaderboard")
)]
pub async fn aoc(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Actions the Advent of Code command
#[poise::command(slash_command, prefix_command, ephemeral)]
pub async fn register(
    ctx: Context<'_>,
    username: Option<String>,
    id: Option<u64>,
) -> Result<(), Error> {
    if username == None && id == None {
        println!("Registration attempted with no details");
        ctx.say("You must provide at least one of your Advent of Code username or id number")
            .await?;
        return Ok(());
    }
    let id_str = id.unwrap_or(0).to_string();
    let aoc_name = match username {
        Some(x) => x,
        None => api::retrieve_lb(None)
            .await
            .members
            .get(&id_str)
            .unwrap()
            .name
            .clone(),
    };
    let aoc_id = match id {
        Some(x) => x,
        None => {
            api::retrieve_lb(None)
                .await
                .members
                .iter()
                .find(|(_, v)| v.name.as_str() == &aoc_name)
                .unwrap()
                .1
                .id
        }
    };
    let _ = db::insert_registration_db(ctx.author().id.get(), &aoc_name, aoc_id);
    ctx.say(format!(
        "Your account has been registered with {} <{}>",
        aoc_name, aoc_id
    ))
    .await?;
    Ok(())
}

/// Actions the Advent of Code command
#[poise::command(slash_command, prefix_command, ephemeral)]
pub async fn unregister(ctx: Context<'_>) -> Result<(), Error> {
    let disc_id = ctx.author().id.get();
    let _ = db::remove_registration_db(disc_id);
    ctx.say(format!(
        "Your account (id {}) has been unregistered",
        disc_id
    ))
    .await?;
    Ok(())
}

/// Actions the Advent of Code command
#[poise::command(slash_command, prefix_command, ephemeral)]
pub async fn reg_check(ctx: Context<'_>) -> Result<(), Error> {
    let response = match db::get_aoc_from_discord(ctx.author().id.get()) {
        Ok(x) => match x {
            Some(y) => format!(
                "Your account is registered as being linked to the AOC user {}",
                y
            ),
            None => String::from("You don't have any aoc account on record"),
        },
        Err(_) => String::from("An error occurred"),
    };
    ctx.say(response).await?;
    Ok(())
}

/// Actions the Advent of Code command
#[poise::command(slash_command, prefix_command)]
pub async fn leaderboard(
    ctx: Context<'_>,
    day: Option<u8>,
    year: Option<u16>,
) -> Result<(), Error> {
    ctx.say("Leaderboard command run").await?;
    let raw_packet: Leaderboard = api::retrieve_lb(year).await;
    for (_, member) in raw_packet.members {
        println!("{} is in the leaderboard!", member.name);
    }
    Ok(())
}

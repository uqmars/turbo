use serde_json::Value;
use std::u8;

use crate::utilities::{Context, Error};
use pyo3::{
    ffi::printfunc,
    prelude::*,
    types::{IntoPyDict, PyModule},
};

mod api;

/// Advent of Code Leaderboard
///
/// Activates the Advent of Code Python Script

// const AOC_SCRIPT: &str = "/python-scripts/aoc/main.py";

// pub async fn advent_py(ctx: Context<'_>) -> String {
//     let msg: &str = "";
//
//     let aoc_help = PyModule::from_code(
//         py,
//         r#"
//         async def help(ctx, *args):
//         "#,
//         AOC_SCRIPT,
//         "help",
//     )?;
//
//     let aoc_help_message: String = aoc_help.getattr("help")?.call(msg.to_string())?.extract()?;
//
//     return aoc_help_message.to_string();
// }

/// Actions the Advent of Code command
#[poise::command(slash_command, prefix_command, subcommands("register", "leaderboard"))]
pub async fn aoc(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Actions the Advent of Code command
#[poise::command(slash_command, prefix_command, ephemeral)]
pub async fn register(ctx: Context<'_>, username: String) -> Result<(), Error> {
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
    let raw_packet: Value = api::retrieve_json(year).await?;
    println!("Raw packet retrieved!\r\n{:?}", raw_packet);
    Ok(())
}

use std::env;
use serde_json::Value;

use serenity::model::channel::*;
use poise::serenity_prelude as serenity;

pub struct Data {} // User data, which is stored and accessible in all command invocations
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Votey Thumbs
///
/// A completely original idea
pub async fn voty(ctx: &Context<'_>, msg: &serenity::Message) -> Result<(), Error> {
    let thumbsup = ReactionType::Unicode("👍".to_string());
    let thumbsdown = ReactionType::Unicode("👎".to_string());

    msg.react(ctx, thumbsup).await?;
    msg.react(ctx, thumbsdown).await?;
    Ok(())
}

/// Memberships command
///
/// Pulls current membership count and quorum, returning them as a string.
pub async fn memberships() -> Result<String, reqwest::Error> {
    let sheet_id = env::var("MEMBER_STATS_SHEET_ID").expect("Expected a google sheet ID in the environment");
    let cell_id = env::var("MEMBER_STATS_CELL").expect("Expected a google sheet cell ID in the environment");
    let sheets_api = env::var("GOOGLE_SHEETS_API_KEY").expect("Expected a google sheets API key in the environment");
    let membership_url: String = format!("https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}?key={}", sheet_id, cell_id, sheets_api);
    let resp = reqwest::get(membership_url)
        .await?
        .text()
        .await?;

    let parsed: Value = serde_json::from_str(&resp).unwrap();

    // Access fields using square brackets
    let val_a = &parsed["values"][0].get(0).unwrap();
    let members: i32 = val_a.as_str().unwrap().parse().unwrap();
    let val_b = &parsed["values"][1].get(0).unwrap();
    let quorum: i32 = val_b.as_str().unwrap().parse().unwrap();

    let msg: String = format!("There are currently {members} members of UQ MARS, making the current quorum {quorum}");
    Ok(msg)
}

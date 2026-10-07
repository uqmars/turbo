use std::env;

use std::sync::Arc;
use rusqlite::Connection;

use ::serenity::all::EditMessage;
use poise::serenity_prelude as serenity;

use serenity::{ChannelId, MessageId};

use crate::utilities;
use crate::utilities::{Error, Context};

const DB_FILE_NAME: &str = "rules.db3";

pub fn create_rules_db() -> Result<(), Box<dyn std::error::Error>> {

    let conn = Connection::open(DB_FILE_NAME)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS rules (
            unique_id   INTEGER PRIMARY KEY,
            id          INTEGER,
            rule_txt    TEXT NOT NULL,
            msg_id      INTEGER,
            channel_id  INTEGER
        ) STRICT;",
        ()
    )?;

    Ok(())
}

struct Rule {
    unique_id: u64,
    id: u64,
    rule: String,
    msg_id: u64,
    channel_id: u64
}

fn get_max_id() -> Result<u64, Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_FILE_NAME)?;
    
    let max_id = conn.query_row("SELECT MAX(id) FROM rules", (), |row| row.get(0)).unwrap_or(0);
    Ok(max_id)
}

fn get_uid_from_id(id: u64) -> Result<u64, Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_FILE_NAME)?;
    
    let uid = conn.query_row("SELECT unique_id FROM rules WHERE id = ?1", (id,), |row| row.get(0)).unwrap_or(0);
    Ok(uid)
}

fn get_rule_from_id(id: u64) -> Result<Rule, Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_FILE_NAME)?;
    
    let rule = conn.query_row("SELECT unique_id, id, rule_txt, msg_id, channel_id, guild_id FROM rules WHERE id = ?1", (id,), |row| {
        Ok(Rule {
            unique_id: row.get(0)?,
            id: row.get(1)?,
            rule: row.get(2)?,
            msg_id: row.get(3)?,
            channel_id: row.get(4)?,
        })}).unwrap();
    Ok(rule)
}

fn insert_rule_db(
    id: u64,
    rule: &str,
    msg_id: u64,
    channel_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {

    let conn = Connection::open(DB_FILE_NAME)?;
    
    conn.execute(
        "INSERT INTO rules (id, rule_txt, msg_id, channel_id)
            VALUES (?1, ?2, ?3, ?4);",
        (id, rule, msg_id, channel_id)
    )?;

    Ok(())
}

fn update_rule_db(
    id: u64,
    new_rule: &str,
) -> Result<(), Box<dyn std::error::Error>> {

    let conn = Connection::open(DB_FILE_NAME)?;
    
    conn.execute(
        "UPDATE rules SET rule_txt = ?1 WHERE id = ?2;",
        (new_rule, id)
    )?;

    Ok(())
}

fn update_id_db(
    uid: u64,
    new_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {

    let conn = Connection::open(DB_FILE_NAME)?;
    
    conn.execute(
        "UPDATE rules SET id = ?1 WHERE unique_id = ?2;",
        (new_id, uid)
    )?;

    Ok(())
}

fn remove_rule_db(
    id: u64,
    msg_id: u64
) -> Result<(), Box<dyn std::error::Error>> {
    
    let conn = Connection::open(DB_FILE_NAME)?;
    
    conn.execute(
        "DELETE FROM rules
            WHERE id=?1 AND msg_id=?2;",
        (id, msg_id)
    )?;

    Ok(())
}

fn get_all_rules_db() -> Result<Vec<Rule>, Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_FILE_NAME)?;

    let select_sql = "SELECT unique_id, id, rule_txt, msg_id, channel_id
        FROM    rules";
    let mut stmt = conn.prepare(select_sql)?;
    let mut rules: Vec<Rule> = Vec::new();
    let rule_iter = stmt.query_map((), |row| {
        Ok(Rule {
            unique_id: row.get(0)?,
            id: row.get(1)?,
            rule: row.get(2)?,
            msg_id: row.get(3)?,
            channel_id: row.get(4)?,
        })
    })?;

    for rule in rule_iter {
        rules.push(rule.unwrap());
    }
    
    Ok(rules)
}

async fn update_message(
    http: Arc<serenity::Http>,
    chann_id: ChannelId,
    msg_id: MessageId,
    new_txt: String
) {
    let mut msg = chann_id.message(&http, msg_id).await.unwrap();
    let _ = msg.edit(http, EditMessage::new().content(new_txt)).await;
}

async fn remove_message(
    http: Arc<serenity::Http>,
    chann_id: ChannelId,
    msg_id: MessageId
) {
    let msg = chann_id.message(&http, msg_id).await.unwrap();
    let _ = msg.delete(http).await;
}

fn format_rule_msg(id: u64, rule: &str) -> String {
    format!("Rule {}: {}", id, rule)
}

/// The parent rules command
#[poise::command(slash_command, prefix_command, subcommands("add", "list", "delete"))]
pub async fn rules(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Adds a new rule
#[poise::command(slash_command, prefix_command, ephemeral)]
pub async fn add(ctx: Context<'_>, txt: String) -> Result<(), Error> {
    // Get the rules channel
    let chann_id = ChannelId::from(env::var("RULES_CHANNEL_ID")
        .expect("Expected a rules channel ID in the environment").parse::<u64>().unwrap());

    // Sends message
    let id = get_max_id().unwrap() + 1;
    let msg_txt = format_rule_msg(id, &txt);
    let msg = chann_id.say(ctx.serenity_context().http.clone(), &msg_txt).await.unwrap();

    // Gets relevant message context
    let msg_id = msg.id;
    let _ = insert_rule_db(id, txt.as_str(), msg_id.get(), chann_id.get());
    ctx.say(format!("Added new rule:\r\n{}", msg_txt)).await?;
    Ok(())
}

/// Lists the current rules
#[poise::command(slash_command, prefix_command, ephemeral)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let rules = get_all_rules_db().unwrap();
    let mut msg: String = "See all of the current rules:".to_string();
    for rule in rules {
        msg = format!("{}\r\n{}: {}", msg, rule.id, rule.rule);
    }
    let _ = ctx.say(msg).await;
    Ok(())
}

/// Deletes the rule with the given ID
#[poise::command(slash_command, prefix_command, ephemeral)]
pub async fn delete(ctx: Context<'_>, id: u64) -> Result<(), Error> {
    let rules = get_all_rules_db().unwrap();
    let mut msg: String = String::new();
    for rule in rules {
        if rule.id == id {
            let _ = remove_rule_db(id, rule.msg_id);
            remove_message(ctx.serenity_context().http.clone(), ChannelId::from(rule.channel_id), MessageId::from(rule.msg_id)).await;
            msg = format!("Message {} has been removed:\r\n{}", rule.id, rule.rule);
        } else if rule.id > id {
            let _ = update_id_db(rule.unique_id, rule.id - 1);
            let new_msg_string = format_rule_msg(rule.id - 1, &rule.rule);
            update_message(ctx.serenity_context().http.clone(), ChannelId::from(rule.channel_id), MessageId::from(rule.msg_id), new_msg_string).await;
        }
    }
    let _ = ctx.say(msg).await;
    Ok(())
}

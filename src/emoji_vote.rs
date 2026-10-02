use std::time::{Duration, SystemTime};
use std::sync::Arc;
use rusqlite::Connection;

use serenity::model::channel::ReactionType;
use poise::serenity_prelude as serenity;

use serenity::{ChannelId, MessageId, GuildId};
use serenity::builder::{CreateMessage, CreateAttachment};

use crate::utilities;
use crate::utilities::{Error, Context};

const MAX_EMOJI_FILE_SIZE: u32 = 256_000;
const DB_FILE_NAME: &str = "emoji-vote.db3";
const DAY_IN_SEC: u64 = 86400;

pub fn create_emoji_db() -> Result<(), Box<dyn std::error::Error>> {

    let conn = Connection::open(DB_FILE_NAME)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS emoji_votes (
            id          INTEGER PRIMARY KEY,
            emoji_name  TEXT NOT NULL,
            msg_id      INTEGER,
            channel_id  INTEGER,
            guild_id    INTEGER,
            end_time    INTEGER
        ) STRICT;",
        ()
    )?;

    Ok(())
}

pub fn insert_emoji_db(
    emoji_name: &str,
    msg_id: u64,
    channel_id: u64,
    guild_id: u64,
    end_timestamp: u64
) -> Result<(), Box<dyn std::error::Error>> {

    let conn = Connection::open(DB_FILE_NAME)?;
    
    conn.execute(
        "INSERT INTO emoji_votes (emoji_name, msg_id, channel_id, guild_id, end_time)
            VALUES (?1, ?2, ?3, ?4, ?5);",
        (emoji_name, msg_id, channel_id, guild_id, end_timestamp)
    )?;

    Ok(())
}

pub fn remove_emoji_db(
    emoji_name: &str,
    msg_id: u64
) -> Result<(), Box<dyn std::error::Error>> {
    
    let conn = Connection::open(DB_FILE_NAME)?;
    
    conn.execute(
        "DELETE FROM emoji_votes
            WHERE emoji_name=?1 AND msg_id=?2;",
        (emoji_name, msg_id)
    )?;

    Ok(())
}

struct EmojiVote {
    emoji_name: String,
    msg_id: u64,
    channel_id: u64,
    guild_id: u64,
    end_time: u64
}

fn get_all_emoji_db() -> Result<Vec<EmojiVote>, Box<dyn std::error::Error>> {
    let conn = Connection::open(DB_FILE_NAME)?;

    let select_sql = "SELECT emoji_name, msg_id, channel_id, guild_id, end_time
        FROM    emoji_votes";
    let mut stmt = conn.prepare(select_sql)?;
    let mut emoji_votes: Vec<EmojiVote> = Vec::new();
    let vote_iter = stmt.query_map((), |row| {
        Ok(EmojiVote {
            emoji_name: row.get(0)?,
            msg_id: row.get(1)?,
            channel_id: row.get(2)?,
            guild_id: row.get(3)?,
            end_time: row.get(4)?,
        })
    })?;

    for vote in vote_iter {
        emoji_votes.push(vote.unwrap());
    }
    
    Ok(emoji_votes)
}

fn is_valid_emoji_name(name: &str) -> bool {
    for c in name.bytes() {
        if !(c.is_ascii_alphanumeric() || c == '_'.to_ascii_lowercase() as u8) {
            return false;
        }
    }
    true
}

fn check_attachment(img: &serenity::Attachment) -> Option<String> {
    if img.size >= MAX_EMOJI_FILE_SIZE {
        return Some("Image is too large".to_string());
    }
    match img.dimensions() {
        Some((x, y)) => {
            if x != y {
                return Some("Image is not square".to_string());
            }
        },
        None => {
            return Some("File is not an image".to_string());
        }
    }
    None
}

async fn vote_counting(
    http: Arc<serenity::Http>,
    guild: GuildId,
    chann_id: ChannelId,
    msg_id: MessageId,
    emoji_name: &str
) {
    let _ = remove_emoji_db(emoji_name, msg_id.get());
    
    let message = chann_id.message(&http, msg_id).await.unwrap();
    let img_attachment = message.attachments.first().unwrap();
    let img = CreateAttachment::url(&http, &img_attachment.url).await.unwrap();
    let mut yays: u64 = 0;
    let mut nays: u64 = 0;
    for reaction in &message.reactions {
        match reaction.reaction_type.to_string().chars().next().unwrap() {
            '👍' => yays = reaction.count,
            '👎' => nays = reaction.count,
            _ => ()
        }
    }

    let emoji: Option<serenity::Emoji>;
    let response = format!("The votes are in: {}.", if yays > nays {
        let img_bytes = img.to_base64();
        emoji = Some(guild.create_emoji(&http, emoji_name, &img_bytes).await.unwrap());
        "yay"
    } else {
        emoji = None;
        "nay"
    });

    let result_msg = chann_id.send_message(&http, CreateMessage::new()
            .reference_message((chann_id, msg_id))
            .content(response)
        ).await.unwrap();

    if let Some(x) = emoji{
        result_msg.react(&http, ReactionType::Custom {
            animated: x.animated,
            id: x.id,
            name: Some(x.name) }).await.unwrap();
    }
}

pub async fn revive_all_emoji_votes(http: Arc<serenity::Http>) {
    let votes = get_all_emoji_db().unwrap();
    let curr_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(); 

    for vote in votes {
        let http_dup = http.clone();
        if vote.end_time < SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() {
            tokio::spawn(async move {
                vote_counting(
                    http_dup, 
                    GuildId::from(vote.guild_id), 
                    ChannelId::from(vote.channel_id),
                    MessageId::from(vote.msg_id),
                    &vote.emoji_name
                ).await;
            });
        } else {
            let duration = vote.end_time - curr_time;
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(duration)).await;
                vote_counting(
                    http_dup, 
                    GuildId::from(vote.guild_id), 
                    ChannelId::from(vote.channel_id),
                    MessageId::from(vote.msg_id),
                    &vote.emoji_name
                ).await;
            });
        }
    }
}

pub async fn emoji_vote(
    ctx: Context<'_>,
    img: serenity::Attachment,
    emoji_name: String,
    time_days: u64
) -> Result<(), Error> {
    let img_url = &img.url;
    if !is_valid_emoji_name(&emoji_name) {
        ctx.say("Invalid emoji name").await?;
        return Ok(());
    }
    if let Some(resp) = check_attachment(&img) {
        ctx.say(resp).await?;
        return Ok(());
    }
    let msg_txt: String = format!("`:{}:`, yay or nay?", emoji_name);
    let img_attachment = serenity::CreateAttachment::url(&ctx, img_url).await.unwrap();
    let msg_reply = poise::CreateReply::default()
        .content(msg_txt)
        .attachment(img_attachment);
    let msg_sent = ctx.send(msg_reply).await?;
    let msg = msg_sent.message().await?;
    utilities::voty(&ctx, &msg).await?;

    let http = ctx.serenity_context().http.clone();
    let chann_id = ctx.channel_id();
    let msg_id = msg.id;
    let guild = ctx.guild_id().unwrap();

    let duration = time_days * DAY_IN_SEC;
    let end_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() + duration;
    let _ = insert_emoji_db(emoji_name.as_str(), msg_id.get(), chann_id.get(), guild.get(), end_time);

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(duration)).await;
        vote_counting(http, guild, chann_id, msg_id, &emoji_name).await;
    });

    Ok(())
}

use std::env;
 
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use poise::serenity_prelude as serenity;
use serde_json::Value;

mod text;

mod utilities;

struct Handler;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[async_trait]
impl serenity::EventHandler for Handler {

    // Thank you to Luna for helping us to get the message pattern matching to work! 
    async fn message(&self, ctx: serenity::Context, msg: Message) {

        // Ignore bot messages
        if msg.author.bot {
            return;
        }

        if msg.content.contains("codeword") {
            if let Err(why) = msg.reply_ping(&ctx.http, "You said the codeword!").await {
                println!("Error sending message: {:?}", why);
            }
        }

    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: serenity::Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

// Messages
const HELP_MESSAGE: &str = "
Our enemies may rest but rust never sleeps.

Hi! You're looking for help, so am I!
If you want a feature added or fixed, make a pull request or raise an issue.
Unsure how to constribute? Ask the friendly team! Check out my source code: https://github.com/uqmars/turbo

Games:
    pong        The game pong

Text:
    banter      Just a bit of banter!
    roll        Defaults 1d20.
                !roll [max] [min] [range]
";

const COMMAND_UNDER_REPAIR: &str = "This command is currently being fixed. Hold tight!";

/// Displays the help message
#[poise::command(slash_command, prefix_command)]
async fn help(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.say(HELP_MESSAGE).await?;
    Ok(())
}

/// Displays a banter message
#[poise::command(slash_command, prefix_command)]
async fn banter(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.say(text::banter().as_str()).await?;
    Ok(())
}

/// Displays a banter message
#[poise::command(slash_command, prefix_command)]
async fn roll(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.say(text::roll(None, None, None).as_str()).await?;
    Ok(())
}

/// Actions the voteythumbs command
#[poise::command(slash_command, prefix_command)]
async fn voteythumbs(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.say(COMMAND_UNDER_REPAIR).await?;
    Ok(())
}

/// Actions the Advent of Code command
#[poise::command(slash_command, prefix_command)]
async fn aoc(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.say(COMMAND_UNDER_REPAIR).await?;
    Ok(())
}

/// Actions the Members command
#[poise::command(slash_command, prefix_command)]
async fn members(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let exec_role = env::var("EXEC_ROLE_ID").expect("Expected an executive role ID in the environment");
    if !ctx.author().has_role(&ctx, ctx.guild_id().unwrap(), exec_role.parse::<u64>().unwrap()).await? {
        ctx.reply("You need to be an exec to run the `members` command.").await?;
        return Ok(());
    }

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
    let valA = &parsed["values"][0].get(0).unwrap();
    let members: i32 = valA.as_str().unwrap().parse().unwrap();
    let valB = &parsed["values"][1].get(0).unwrap();
    let quorum: i32 = valB.as_str().unwrap().parse().unwrap();

    let msg: String = format!("There are currently {members} members of UQ MARS, making the current quorum {quorum}");
    println!("{msg}");
    ctx.say(msg).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    // Pull in vars from '.env'
    dotenv::dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                help(),
                banter(),
                roll(),
                voteythumbs(),
                aoc(),
                members()
            ],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                if cfg!(debug_assertions) {
                    let guild_id = env::var("GUILD_ID")
                        .ok()
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(0);
                    poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    serenity::GuildId::new(guild_id),
                ).await?;
                } else {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }
                Ok(Data {})
            })
        })
        .build();

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await;

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.unwrap().start().await {
        println!("Client error: {:?}", why);
    }
}

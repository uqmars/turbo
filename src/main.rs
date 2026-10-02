use std::env;
 
use serenity::all::EmojiId;
use serenity::async_trait;
use serenity::model::channel::{Message, ReactionType};
use serenity::model::gateway::Ready;
use poise::serenity_prelude as serenity;
use serenity::builder::{CreateMessage, CreateAttachment};

mod text;
use text::{COMMAND_UNDER_REPAIR, HELP_MESSAGE};

mod utilities;
use utilities::{Data, Error, Context};

mod emoji_vote;

struct Handler;

#[async_trait]
impl serenity::EventHandler for Handler {

    // Thank you to Luna for helping us to get the message pattern matching to work! 
    async fn message(&self, ctx: serenity::Context, msg: Message) {

        // Ignore bot messages
        if msg.author.bot {
            return;
        }

        if msg.content.to_lowercase().split_whitespace().any(|word| word == "rust") {
            let rustacean_id = env::var("RUSTACEAN_EMOJI_ID")
                .expect("Expected an emoji ID in the environment")
                .parse::<u64>()
                .unwrap();
            let rustacean = ReactionType::Custom {
                animated: false,
                id: EmojiId::new(rustacean_id),
                name: Some("rustacean".to_string())
            };
            let _ = msg.react(&ctx, rustacean).await;
        }

        if msg.content.to_lowercase().split_whitespace().any(|word| word == "uqcsbot") {
            let image = CreateAttachment::path("res/DunkedOn.jpg").await.unwrap();
            let text = "What are you talking about that punk <@814086324709359648> for?";
            let msg_reply = CreateMessage::default()
                .content(text)
                .add_file(image)
                .reference_message(&msg);
            
            let _ = msg.channel_id.send_message(&ctx, msg_reply).await;
        }

    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, ctx: serenity::Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let _ = emoji_vote::create_emoji_db();
        emoji_vote::revive_all_emoji_votes(ctx.http.clone()).await;
    }
}

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
    max: Option<i32>,
    min: Option<i32>
) -> Result<(), Error> {
    ctx.say(text::roll(max, min).as_str()).await?;
    Ok(())
}

/// Actions the voteythumbs command
#[poise::command(slash_command, prefix_command)]
async fn voteythumbs(
    ctx: Context<'_>,
    msg: Option<String>
) -> Result<(), Error> {
    let msg_text: String = match msg {
        Some(x) => x,
        None => format!("{} has created a vote", ctx.author().display_name()),
    };
    let post = ctx.say(msg_text).await?;
    utilities::voty(&ctx, post.message().await?.as_ref()).await?;
    Ok(())
}

/// Actions the voteythumbs application command
#[poise::command(ephemeral = true,
    context_menu_command = "Votey Thumbs")]
async fn votey(
    ctx: Context<'_>,
    msg: serenity::Message,
) -> Result<(), Error> {
    utilities::voty(&ctx, &msg).await?;
    let response = format!("Reactions added to message from {}", msg.author.display_name());
    ctx.say(response).await?;
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
    // Give it time
    ctx.defer().await?;

    let exec_role = env::var("EXEC_ROLE_ID").expect("Expected an executive role ID in the environment");
    if !ctx.author().has_role(&ctx, ctx.guild_id().unwrap(), exec_role.parse::<u64>().unwrap()).await? {
        ctx.reply("You need to be an exec to run the `members` command.").await?;
        return Ok(());
    }
    
    let response: String = utilities::memberships().await.unwrap();
    ctx.say(response.as_str()).await?;
    Ok(())
}

async fn is_server_stakeholder_channel(ctx: Context<'_>) -> Result<bool, Error> {
    let stakeholder_channel_id = env::var("STAKEHOLDER_CHANNEL_ID")
        .expect("Expected a stakeholder channel ID in the environment").parse::<u64>().unwrap();
    let curr_channel_id = ctx.channel_id().get();
    Ok(stakeholder_channel_id == curr_channel_id)
}

/// Actions the Emoji Vote command
#[poise::command(slash_command, prefix_command,
    check = "is_server_stakeholder_channel")]
async fn emoji_vote(
    ctx: Context<'_>,
    img: serenity::Attachment,
    emoji_name: String,
    time_days: Option<u64>
) -> Result<(), Error> {
    emoji_vote::emoji_vote(ctx, img, emoji_name, time_days.unwrap_or(3)).await
}

#[tokio::main]
async fn main() {
    // Pull in vars from '.env'
    dotenv::dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_PRESENCES
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                help(),
                banter(),
                roll(),
                votey(),
                voteythumbs(),
                aoc(),
                members(),
                emoji_vote()
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

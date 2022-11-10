use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod text;
use text::*;

struct Handler;

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

const INVALID_COMMAND_MESSAGE: &str = "Invalid command. Type !help for 'working' commands.";

const HELP_COMMAND: &str = "!help";

const BANTER_COMMAND: &str = "!banter";

const ROLL_COMMAND: &str = "!roll";

#[async_trait]
impl EventHandler for Handler {

    // Thank you to Luna for helping us to get the message pattern matching to work! 
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_str() {
            HELP_COMMAND => {
                if let Err(why) = msg.channel_id.say(&ctx.http, HELP_MESSAGE).await {
                    println!("Error sending message: {:?}", why);
                }
            },
            BANTER_COMMAND => {
                if let Err(why) = msg.channel_id.say(&ctx.http, banter()).await {
                    println!("Error sending message: {:?}", why);
                }
            },
            ROLL_COMMAND => {
                // Need to be able to parse flags into each field
                if let Err(why) = msg.channel_id.say(&ctx.http, roll(None, None, None)).await {
                    println!("Error sending message: {:?}", why);
                }
            }
            _ => {
                if let Err(why) = msg.channel_id.say(&ctx.http, INVALID_COMMAND_MESSAGE).await {
                    println!("Error sending message: {:?}", why);
                }
            }
        };
    }


    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}



#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

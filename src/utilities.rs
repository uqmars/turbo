use serenity::async_trait;
use serenity::model::channel::*;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use rand::prelude::*;

/// Votey Thumbs
///
/// A completely original idea
pub async fn voty(ctx: Context, msg: Message) {
    let thumbsup = ReactionType::Unicode(r"\u0001f44d".to_string());
    let thumbsdown = ReactionType::Unicode(r"\U0001f44e".to_string());
    

    msg.react(&ctx, thumbsup).await;
    msg.react(&ctx, thumbsdown).await;
}

use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use rand::prelude::*;

mod text;
use text::*;

mod utilities;
use utilities::*;

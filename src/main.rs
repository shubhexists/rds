#![allow(deprecated)]
mod commands;
mod spotdl;
use crate::commands::{
    deafen::*, join::*, leave::*, mute::*, queue::*, skip::*, stop::*, undeafen::*, unmute::*,
};
use reqwest::Client as HttpClient;
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            macros::{command, group},
            CommandResult, Configuration,
        },
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, gateway::Ready, prelude::ChannelId},
    prelude::{GatewayIntents, TypeMapKey},
    Result as SerenityResult,
};
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler, SerenityInit};
use std::{env, sync::Arc};

struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(deafen, join, leave, mute, queue, skip, stop, ping, undeafen, unmute)]
struct General;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let token: String = env::var("RDS").expect("Expected a token in the environment");
    let framework: StandardFramework = StandardFramework::new().group(&GENERAL_GROUP);
    framework.configure(Configuration::new().prefix("!"));
    let intents: GatewayIntents =
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client: Client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await
        .expect("Err creating client");
    let _ = client
        .start()
        .await
        .map_err(|why: serenity::Error| println!("Client ended: {:?}", why));

    tokio::spawn(async move {
        let _ = client
            .start()
            .await
            .map_err(|why: serenity::Error| println!("Client ended: {:?}", why));
    });

    let _signal_err = tokio::signal::ctrl_c().await;
    println!("Received Ctrl-C, shutting down.");
}

async fn get_http_client(ctx: &Context) -> HttpClient {
    let data = ctx.data.read().await;
    data.get::<HttpKey>()
        .cloned()
        .expect("Guaranteed to exist in the typemap.")
}

struct TrackEndNotifier {
    chan_id: ChannelId,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            check_msg(
                self.chan_id
                    .say(&self.http, &format!("Tracks ended: {}.", track_list.len()))
                    .await,
            );
        }

        None
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, "Pong!").await);

    Ok(())
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

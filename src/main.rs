use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use songbird::SerenityInit;
use std::{collections::HashMap, env, sync::Arc};

#[group]
#[commands(ping, join)]
struct General;

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        for guild_id in ctx.cache.guilds() {
            match guild_id.channels(&ctx).await {
                Ok(channels) => {
                    println!("Channels for Guild {}: ", guild_id);
                    for (channel_id, channel) in channels {
                        println!(
                            "  - {} (ID: {}, Type: {:?})",
                            channel.name, channel_id, channel.kind
                        );
                    }
                }
                Err(why) => {
                    println!("Error fetching channels for guild {}: {:?}", guild_id, why);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let token = env::var("RDS").expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
#[only_in(guilds)]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "!pong").await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild: GuildId = msg.guild_id.unwrap();
    let channels: HashMap<ChannelId, GuildChannel> = guild.channels(ctx).await?;

    if let Some(channel) = channels.values().find(|&channel| {
        channel.kind == ChannelType::Voice && channel.name.to_lowercase().contains("general")
    }) {
        println!("Found voice channel: {} (ID: {})", channel.name, channel.id);

        let manager: Arc<songbird::Songbird> = songbird::get(ctx)
            .await
            .expect("Songbird client should be initialized");

        let (_, connection_result) = manager.join(guild, channel.id).await;

        match connection_result {
            Ok(_) => {
                msg.reply(ctx, format!("Joined voice channel #{}", channel.name))
                    .await?;
            }
            Err(why) => {
                msg.reply(ctx, format!("Failed to join voice channel: {}", why))
                    .await?;
            }
        }
    } else {
        let voice_channels: Vec<String> = channels
            .values()
            .filter(|channel| channel.kind == ChannelType::Voice)
            .map(|channel| channel.name.clone())
            .collect();

        msg.reply(
            ctx,
            format!(
                "Could not find a 'general' voice channel. Available voice channels: {}",
                voice_channels.join(", ")
            ),
        )
        .await?;
    }

    Ok(())
}

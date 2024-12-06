use crate::{check_msg, TrackEndNotifier};
use serenity::all::{
    standard::{macros::command, CommandResult},
    Context, Message,
};
use serenity::http::Http;
use serenity::model::prelude::ChannelId;
use serenity::prelude::Mentionable;
use songbird::{Event, TrackEvent};
use std::sync::Arc;

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let (guild_id, channel_id) = {
        let guild: serenity::all::CacheRef<
            '_,
            serenity::model::prelude::GuildId,
            serenity::model::prelude::Guild,
            std::convert::Infallible,
        > = msg.guild(&ctx.cache).unwrap();
        let channel_id: Option<ChannelId> = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state: &serenity::model::prelude::VoiceState| voice_state.channel_id);

        (guild.id, channel_id)
    };

    let connect_to: ChannelId = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager: Arc<songbird::Songbird> = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handle_lock) = manager.join(guild_id, connect_to).await {
        check_msg(
            msg.channel_id
                .say(&ctx.http, &format!("Joined {}", connect_to.mention()))
                .await,
        );
        let chan_id: ChannelId = msg.channel_id;
        let send_http: Arc<Http> = ctx.http.clone();
        let mut handle: tokio::sync::MutexGuard<'_, songbird::Call> = handle_lock.lock().await;
        handle.add_global_event(
            Event::Track(TrackEvent::End),
            TrackEndNotifier {
                chan_id,
                http: send_http,
            },
        );
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Error joining the channel")
                .await,
        );
    }

    Ok(())
}

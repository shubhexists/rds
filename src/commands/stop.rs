use crate::check_msg;
use serenity::all::{
    standard::{macros::command, Args, CommandResult},
    Context, Message,
};
use std::sync::Arc;
use tokio::sync::MutexGuard;

#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager: Arc<songbird::Songbird> = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler: MutexGuard<'_, songbird::Call> = handler_lock.lock().await;
        let queue: &songbird::tracks::TrackQueue = handler.queue();
        queue.stop();

        check_msg(msg.channel_id.say(&ctx.http, "Queue cleared.").await);
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

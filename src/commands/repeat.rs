use crate::check_msg;
use serenity::all::{
    standard::{macros::command, CommandResult},
    Context, Message,
};
use std::sync::Arc;
use tokio::sync::MutexGuard;
use tracing::error;

#[command]
#[only_in(guilds)]
async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id: serenity::model::prelude::GuildId = msg.guild(&ctx.cache).unwrap().id;

    let manager: Arc<songbird::Songbird> = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock: Arc<serenity::prelude::Mutex<songbird::Call>> = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let handler: MutexGuard<'_, songbird::Call> = handler_lock.lock().await;

    match handler.queue().current() {
        Some(current) => match current.enable_loop() {
            Ok(_) => {
                check_msg(msg.reply(ctx, "Added song on repeat").await);
            }
            Err(e) => {
                error!("Error repeating song: {}", e);
                check_msg(
                    msg.reply(ctx, "Some error occurred, unable to add song on repeat")
                        .await,
                );
            }
        },
        None => {
            check_msg(msg.reply(ctx, "No current active tracks").await);
        }
    }

    Ok(())
}

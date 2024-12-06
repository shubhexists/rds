use crate::check_msg;
use serenity::all::{
    standard::{macros::command, CommandResult},
    Context, Message,
};
use std::sync::Arc;

#[command]
async fn deafen(ctx: &Context, msg: &Message) -> CommandResult {
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

    let mut handler = handler_lock.lock().await;

    if handler.is_deaf() {
        check_msg(msg.channel_id.say(&ctx.http, "Already deafened").await);
    } else {
        if let Err(e) = handler.deafen(true).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        }

        check_msg(msg.channel_id.say(&ctx.http, "Deafened").await);
    }

    Ok(())
}

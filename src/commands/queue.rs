use crate::{check_msg, get_http_client};
use serenity::all::{
    standard::{macros::command, Args, CommandResult},
    Context, Message,
};
use songbird::input::YoutubeDl;

#[command]
#[only_in(guilds)]
async fn queue(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Must provide a URL to a video or audio")
                    .await,
            );

            return Ok(());
        }
    };

    if !url.starts_with("http") {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Must provide a valid URL")
                .await,
        );

        return Ok(());
    }

    let guild_id: serenity::model::prelude::GuildId = msg.guild_id.unwrap();
    let http_client: reqwest::Client = get_http_client(ctx).await;

    let manager: std::sync::Arc<songbird::Songbird> = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler: tokio::sync::MutexGuard<'_, songbird::Call> = handler_lock.lock().await;

        let src: YoutubeDl = YoutubeDl::new(http_client, url);
        handler.enqueue_input(src.into()).await;

        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Added song to queue: position {}", handler.queue().len()),
                )
                .await,
        );
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

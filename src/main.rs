use anyhow::{bail, Context, Result};
use env_logger::Env;

use crate::{
    message_details::MessageDetails, stream_details::StreamDetails, utils::chat_messages_do,
    youtube_service::YouTubeClient,
};

mod message_details;
mod stream_details;
mod token_store;
mod utils;
mod youtube_service;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let yt = YouTubeClient::create("ytmod".into(), "tokens".into(), "client_secret.json".into())
        .await
        .context("Failed to create YouTube client")?;

    let mut streams = yt
        .get_own_broadcasts()
        .await
        .context("Failed to get broadcasts")?;

    if streams.is_empty() {
        bail!("No streams found!");
    }

    // TODO: Maybe show a prompt select a stream if there are multiple
    let stream =
        StreamDetails::from_broadcast(streams.remove(0)).context("Failed to get stream details")?;

    println!("Selecting stream: ");
    println!("Title: {}", stream.title);
    println!("Description: {}", stream.description);
    println!("Link: https://www.youtube.com/watch?v={}", stream.id);

    chat_messages_do(&yt, &stream, Some(5000), |message| async {
        let message =
            MessageDetails::from_message(message).context("Failed to get message details")?;

        if message.contents.starts_with("DELETEME") {
            if let Err(err) = yt
                .delete_message_with_reason(&stream.live_chat_id, &message.id, "VOLUNTARY EXILE")
                .await
            {
                log::error!("Failed to delete message {:?}: {err}", message.contents);
            }
        }

        println!(
            "===== Message by {} at {}: {}",
            message.author_name, message.time_sent, message.contents
        );
        Ok(())
    })
    .await?;

    Ok(())
}

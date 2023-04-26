use crate::{
    message_details::MessageDetails, stream_details::StreamDetails, utils::chat_messages_do,
    youtube_service::YouTubeClient,
};
use anyhow::{bail, Context, Result};
use env_logger::Env;
use toxic_service::ToxicService;

mod message_details;
mod stream_details;
mod token_store;
mod toxic_service;
mod utils;
mod youtube_service;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let toxic_service = ToxicService::new("127.0.0.1".into(), 8888)
        .with_context(|| "Failed to create Toxic Service")?;

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

    println!("{:=<32}", "");
    println!("Stream Title: {}", stream.title);
    println!("Stream Description: {}", stream.description);
    println!("Link: https://www.youtube.com/watch?v={}", stream.id);
    println!("{:=<32}", "");

    chat_messages_do(&yt, &stream, Some(5000), |message| async {
        let message =
            MessageDetails::from_message(message).context("Failed to get message details")?;

        println!("\n{:-<16}", "");
        println!("Message time: {}", message.time_sent);
        println!("Message from: {}", message.author_name);
        println!("Message text: {}", message.contents);

        if message.author_name != "Tuhin Tarafder" {
            match toxic_service.is_toxic(&message.contents).await {
                Ok(is_toxic) => {
                    println!("Is Toxic: {is_toxic}");
                    if is_toxic {
                    match yt
                        .delete_message_with_reason(
                            &stream.live_chat_id,
                            &message.id,
                            "Trash taken out",
                        )
                        .await
                    {
                        Ok(_) => {
                            log::info!("Deleted comment!");
                        }
                        Err(err) => {
                            log::error!("Failed to delete message: {err}",);
                        }
                    }
                }
                }
                Err(err) => {
                    log::error!("Failed to get toxicity report: {err}",);
                }
            }
        }
        Ok(())
    })
    .await?;

    Ok(())
}

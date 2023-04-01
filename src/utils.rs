use anyhow::{Context, Result};
use google_youtube3::api::LiveChatMessage;
use std::{future::Future, time::Duration};

use crate::{stream_details::StreamDetails, youtube_service::YouTubeClient};

pub async fn chat_messages_do<F, O>(
    yt: &YouTubeClient,
    stream: &StreamDetails,
    poll_interval: Option<u64>,
    pred: F,
) -> Result<()>
where
    F: Fn(LiveChatMessage) -> O,
    O: Future<Output = Result<()>>,
{
    let mut skip_n = 0;
    loop {
        let (messages, defaul_poll_interval) = yt
            .get_messages(stream.live_chat_id.as_str(), &mut skip_n)
            .await
            .context("Failed to get messages")?;

        for message in messages {
            let message_id = message.id.clone();
            if let Err(err) = pred(message).await {
                log::error!("Error while processing message {:?}: {err}", message_id);
            };
        }

        std::thread::sleep(Duration::from_millis(
            poll_interval.unwrap_or(defaul_poll_interval),
        ));
    }
}

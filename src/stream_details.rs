use anyhow::{Context, Ok, Result};
use google_youtube3::api::LiveBroadcast;

pub struct StreamDetails {
    pub id: String,
    pub title: String,
    pub description: String,
    pub live_chat_id: String,
}

impl StreamDetails {
    pub fn from_broadcast(broadcast: LiveBroadcast) -> Result<Self> {
        let id = broadcast.id.with_context(|| "Failed to get broadcast ID")?;

        let broadcast_snippet = broadcast
            .snippet
            .with_context(|| "Failed to get broadcast snippet")?;
        let live_chat_id = broadcast_snippet
            .live_chat_id
            .with_context(|| "Failed to get broadcast live chat ID")?;
        let title = broadcast_snippet
            .title
            .with_context(|| "Failed to get broadcast title")?;
        let description = broadcast_snippet
            .description
            .with_context(|| "Failed to get broadcast description")?;

        Ok(Self {
            id,
            title,
            description,
            live_chat_id,
        })
    }
}

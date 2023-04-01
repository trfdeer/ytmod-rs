use anyhow::{Context, Ok, Result};
use google_youtube3::{
    api::LiveChatMessage,
    chrono::{DateTime, Utc},
};

pub struct MessageDetails {
    pub id: String,
    pub author_id: String,
    pub author_name: String,
    pub contents: String,
    pub time_sent: DateTime<Utc>,
}

impl MessageDetails {
    pub fn from_message(message: LiveChatMessage) -> Result<Self> {
        let message_author = message
            .author_details
            .with_context(|| "Failed to get message author")?;
        let message_snippet = message
            .snippet
            .with_context(|| "Failed to get message snippet")?;

        let id = message.id.with_context(|| "Failed to get message id")?;
        let author_id = message_author
            .channel_id
            .with_context(|| "Failed to get message author id")?;
        let author_name = message_author
            .display_name
            .with_context(|| "Failed to get message author name")?;
        let contents = message_snippet
            .display_message
            .with_context(|| "Failed to get message contents")?;
        let time_sent = message_snippet
            .published_at
            .with_context(|| "Failed to get message time")?;

        Ok(Self {
            id,
            author_id,
            author_name,
            contents,
            time_sent,
        })
    }
}

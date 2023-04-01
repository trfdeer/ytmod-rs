use std::{fs::File, io::BufReader, path::PathBuf, str::FromStr};

use anyhow::{anyhow, bail, Context, Result};
use google_youtube3::{
    api::{LiveBroadcast, LiveChatMessage, LiveChatMessageSnippet, LiveChatTextMessageDetails},
    oauth2::{self, ApplicationSecret},
    YouTube,
};
use hyper::{client::HttpConnector, StatusCode};
use hyper_rustls::HttpsConnector;

use crate::token_store::JsonTokenStore;

#[derive(Clone)]
pub struct YouTubeClient(YouTube<HttpsConnector<HttpConnector>>);

impl YouTubeClient {
    pub async fn create(
        app_name: String,
        tokens_dir: String,
        secrets_file_path: PathBuf,
    ) -> Result<Self> {
        if !PathBuf::from_str(tokens_dir.as_str())?.exists() {
            std::fs::create_dir_all(tokens_dir.as_str())
                .with_context(|| "Failed to create tokens directory.")?;
        }

        let auth = {
            let secrets_file = File::open(&secrets_file_path)
                .with_context(|| format!("Failed to open {secrets_file_path:?}"))?;
            let secrets_reader = BufReader::new(secrets_file);
            let secret: ApplicationSecret = serde_json::from_reader(secrets_reader)?;

            oauth2::InstalledFlowAuthenticator::builder(
                secret,
                oauth2::InstalledFlowReturnMethod::HTTPRedirect,
            )
            .with_storage(Box::new(JsonTokenStore::new(app_name, tokens_dir)))
            .build()
            .await?
        };

        let client = hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        );

        let client = YouTube::new(client, auth);

        Ok(Self(client))
    }

    pub async fn get_own_broadcasts(&self) -> Result<Vec<LiveBroadcast>> {
        let (_, streams) = self
            .0
            .live_broadcasts()
            .list(&vec![
                "id".into(),
                "snippet".into(),
                "contentDetails".into(),
            ])
            .mine(true)
            .doit()
            .await
            .with_context(|| "Broadcasts request failed")?;

        if streams.items.is_none() {
            bail!("Failed to get streams !!! THIS SOULDN'T HAVE HAPPENED !!!");
        }

        let streams = streams
            .items
            .with_context(|| "Failed to get broadcast items")?;

        Ok(streams)
    }

    /// Pagination is not implemented. `get_messages(..)` will always return only th first 2000 messages, which is good enough for now
    pub async fn get_messages(
        &self,
        chat_id: &str,
        skip_n: &mut usize,
    ) -> Result<(Vec<LiveChatMessage>, u64)> {
        let (_, messages) = self
            .0
            .live_chat_messages()
            .list(
                chat_id,
                &vec!["id".into(), "snippet".into(), "authorDetails".into()],
            )
            .max_results(2000)
            .doit()
            .await
            .with_context(|| "Live chat messages request failed")?;

        let message_items = messages
            .items
            .with_context(|| "Failed to get message items")?;

        let message_items = message_items
            .iter()
            .skip(*skip_n)
            .cloned()
            .collect::<Vec<_>>();

        *skip_n += message_items.len();

        Ok((
            message_items,
            messages.polling_interval_millis.unwrap_or(2000) as u64,
        ))
    }

    pub async fn post_message(&self, live_chat_id: &str, message: &str) -> Result<LiveChatMessage> {
        let message_req_data = LiveChatMessage {
            snippet: Some(LiveChatMessageSnippet {
                live_chat_id: Some(live_chat_id.to_owned()),
                type_: Some("textMessageEvent".to_owned()),
                text_message_details: Some(LiveChatTextMessageDetails {
                    message_text: Some(message.to_owned()),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let (_, inserted_message) = self
            .0
            .live_chat_messages()
            .insert(message_req_data)
            .add_part("snippet")
            .doit()
            .await
            .with_context(|| "Add message request failed")?;

        Ok(inserted_message)
    }

    pub async fn delete_message(&self, message_id: &str) -> Result<()> {
        let resp = self
            .0
            .live_chat_messages()
            .delete(message_id)
            .doit()
            .await
            .with_context(|| "Message delete request failed.")?;

        match resp.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::UNAUTHORIZED => Err(anyhow!("Access not authorized")),
            StatusCode::NOT_FOUND => Err(anyhow!("Message not found")),
            _ => unreachable!(),
        }
    }

    pub async fn delete_message_with_reason(
        &self,
        live_chat_id: &str,
        message_id: &str,
        reason: &str,
    ) -> Result<()> {
        self.delete_message(message_id).await?;
        self.post_message(live_chat_id, reason).await?;

        Ok(())
    }
}

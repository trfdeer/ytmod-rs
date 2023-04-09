use reqwest::Client;

use anyhow::Result;
use serde_json::Value;
use urlencoding::encode;

pub struct ToxicService {
    client: Client,
    host: String,
    port: u16,
}

impl ToxicService {
    pub fn new(host: String, port: u16) -> Result<Self> {
        let client = Client::new();
        Ok(Self { client, host, port })
    }

    fn get_uri(&self, text: &str) -> String {
        format!("http://{}:{}/text?q={}", self.host, self.port, encode(text))
    }

    pub async fn is_toxic(&self, text: &str) -> Result<bool> {
        let res = self.client.get(self.get_uri(text)).send().await?;
        let data = res.json::<Value>().await?;

        let is_toxic = data
            .as_object()
            .unwrap()
            .get("toxic")
            .unwrap()
            .as_u64()
            .unwrap();

        Ok(is_toxic == 1)
    }
}

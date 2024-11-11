use crate::{errors::QstashError, rate_limited_client::RateLimitedClient};
use reqwest::Url;

pub struct QstashClient {
    pub(crate) client: RateLimitedClient,
    pub(crate) base_url: Url,
}

impl QstashClient {
    pub fn default() -> Result<Self, QstashError> {
        let base_url = Url::parse("https://qstash.upstash.io")
            .map_err(|e| QstashError::InvalidBaseUrl(e.to_string()))?;

        Ok(QstashClient {
            client: RateLimitedClient::new("".to_string()),
            base_url,
        })
    }

    pub fn new(api_key: String) -> Result<Self, QstashError> {
        let mut qstash_client = QstashClient::default()?;
        qstash_client.client = RateLimitedClient::new(api_key);

        Ok(qstash_client)
    }

    pub fn builder() -> QstashClientBuilder {
        QstashClientBuilder::default()
    }
}

#[derive(Default)]
pub struct QstashClientBuilder {
    base_url: Option<Url>,
    api_key: Option<String>,
}

impl QstashClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base_url(mut self, url: Url) -> Result<Self, QstashError> {
        self.base_url = Some(url);
        Ok(self)
    }

    pub fn api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    pub fn build(self) -> Result<QstashClient, QstashError> {
        let base_url = self.base_url;
        let api_key = self.api_key.unwrap_or_default();

        let mut qstash_client = QstashClient::default()?;
        qstash_client.client = RateLimitedClient::new(api_key);

        if let Some(base_url) = base_url {
            qstash_client.base_url = base_url;
        }

        Ok(qstash_client)
    }
}


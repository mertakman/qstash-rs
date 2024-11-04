use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::QstashClient;
use crate::errors::QstashError;

impl QstashClient {
    pub async fn get_signing_keys(&self) -> Result<Signature, QstashError> {
        let url = self
            .base_url
            .join("/v2/keys")
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self.client.get_request_builder(Method::GET, url);

        let response_body = self
            .client
            .send_request(request)
            .await?
            .bytes()
            .await
            .map_err(QstashError::RequestFailed)?;

        let response: Signature =
            serde_json::from_slice(&response_body).map_err(QstashError::ResponseBodyParseError)?;
        Ok(response)
    }

    pub async fn rotate_signing_keys(&self) -> Result<Signature, QstashError> {
        let url = self
            .base_url
            .join("/v2/keys/rotate")
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self.client.get_request_builder(Method::POST, url);

        let response_body = self
            .client
            .send_request(request)
            .await?
            .bytes()
            .await
            .map_err(QstashError::RequestFailed)?;

        let response: Signature =
            serde_json::from_slice(&response_body).map_err(QstashError::ResponseBodyParseError)?;
        Ok(response)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Signature {
    current: String,
    next: String,
}

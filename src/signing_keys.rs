use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::QstashClient;
use crate::errors::QstashError;

impl QstashClient {
    pub async fn get_signing_keys(&self) -> Result<Signature, QstashError> {
        let request = self.client.get_request_builder(
            Method::GET,
            self.base_url
                .join("/v2/keys")
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<Signature>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn rotate_signing_keys(&self) -> Result<Signature, QstashError> {
        let request = self.client.get_request_builder(
            Method::POST,
            self.base_url
                .join("/v2/keys/rotate")
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<Signature>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Signature {
    current: String,
    next: String,
}

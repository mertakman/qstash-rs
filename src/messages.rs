use reqwest::Method;

use crate::client::QstashClient;
use crate::errors::QstashError;
use crate::message_types::{BatchEntry, Message, MessageResponseResult};
use reqwest::header::HeaderMap;
use urlencoding::encode;

impl QstashClient {
    pub async fn publish_message(
        &self,
        destination: &str,
        headers: HeaderMap,
        body: Vec<u8>,
    ) -> Result<MessageResponseResult, QstashError> {
        let request = self
            .client
            .get_request_builder(
                Method::POST,
                self.base_url
                    .join(&format!("/v2/publish/{}", destination))
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .headers(headers)
            .body(body);

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<MessageResponseResult>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn enqueue_message(
        &self,
        destination: &str,
        queue_name: &str,
        headers: HeaderMap,
        body: Vec<u8>,
    ) -> Result<MessageResponseResult, QstashError> {
        let request = self
            .client
            .get_request_builder(
                Method::POST,
                self.base_url
                    .join(&format!(
                        "/v2/enqueue/{}/{}",
                        encode(queue_name),
                        encode(destination)
                    ))
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .headers(headers)
            .body(body);

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<MessageResponseResult>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn batch_messages(
        &self,
        batch_entries: Vec<BatchEntry>,
    ) -> Result<Vec<MessageResponseResult>, QstashError> {
        let request = self
            .client
            .get_request_builder(
                Method::POST,
                self.base_url
                    .join("/v2/batch")
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .json(&batch_entries);

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<Vec<MessageResponseResult>>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn get_message(&self, message_id: &str) -> Result<Message, QstashError> {
        let request = self.client.get_request_builder(
            Method::GET,
            self.base_url
                .join(&format!("/v2/messages/{}", encode(message_id)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<Message>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn cancel_message(&self, message_id: &str) -> Result<(), QstashError> {
        let request = self.client.get_request_builder(
            Method::DELETE,
            self.base_url
                .join(&format!("/v2/messages/{}", encode(message_id)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn bulk_cancel_messages(&self, message_ids: &[&str]) -> Result<(), QstashError> {
        let body = json!({
            "messageIds": message_ids,
        });

        let request = self
            .client
            .get_request_builder(
                Method::DELETE,
                self.base_url
                    .join("/v2/messages")
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .json(&body);

        self.client.send_request(request).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}

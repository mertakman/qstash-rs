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
        let url = self
            .base_url
            .join(&format!("/v2/publish/{}", (destination)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self
            .client
            .get_request_builder(Method::POST, url)
            .headers(headers)
            .body(body);

        let response_body = self
            .client
            .send_request(request)
            .await?
            .bytes()
            .await
            .map_err(QstashError::RequestFailed)?;

        println!("{:?}", response_body.to_vec().to_ascii_lowercase());
        let response: MessageResponseResult =
            serde_json::from_slice(&response_body).map_err(QstashError::ResponseBodyParseError)?;

        Ok(response)
    }

    pub async fn enqueue_message(
        &self,
        destination: &str,
        queue_name: &str,
        headers: HeaderMap,
        body: Vec<u8>,
    ) -> Result<MessageResponseResult, QstashError> {
        let url = self
            .base_url
            .join(&format!(
                "/v2/enqueue/{}/{}",
                encode(queue_name),
                encode(destination)
            ))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self
            .client
            .get_request_builder(Method::POST, url)
            .headers(headers)
            .body(body);

        let response_body = self
            .client
            .send_request(request)
            .await?
            .bytes()
            .await
            .map_err(QstashError::RequestFailed)?;

        let response: MessageResponseResult =
            serde_json::from_slice(&response_body).map_err(QstashError::ResponseBodyParseError)?;

        Ok(response)
    }

    pub async fn batch_messages(
        &self,
        batch_entries: Vec<BatchEntry>,
    ) -> Result<Vec<MessageResponseResult>, QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/batch"))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self
            .client
            .get_request_builder(Method::POST, url)
            .json(&batch_entries);

        let response_body = self
            .client
            .send_request(request)
            .await?
            .bytes()
            .await
            .map_err(QstashError::RequestFailed)?;

        let response: Vec<MessageResponseResult> =
            serde_json::from_slice(&response_body).map_err(QstashError::ResponseBodyParseError)?;

        Ok(response)
    }

    pub async fn get_message(&self, message_id: &str) -> Result<Message, QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/messages/{}", encode(message_id)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self.client.get_request_builder(Method::GET, url);
        let response = self.client.send_request(request).await?;

        let message = response
            .json::<Message>()
            .await
            .map_err(QstashError::RequestFailed)?;

        Ok(message)
    }

    pub async fn cancel_message(&self, message_id: &str) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/messages/{}", encode(message_id)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;
        let request = self.client.get_request_builder(Method::DELETE, url);
        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn bulk_cancel_messages(&self, message_ids: &[&str]) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/messages"))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let body = json!({
            "messageIds": message_ids,
        });

        let request = self
            .client
            .get_request_builder(Method::DELETE, url)
            .json(&body);

        self.client.send_request(request).await?;
        Ok(())
    }
}

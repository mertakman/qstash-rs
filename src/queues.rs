use crate::client::QstashClient;
use crate::errors::QstashError;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

impl QstashClient {
    pub async fn upsert_queue(
        &self,
        upsert_request: UpsertQueueRequest,
    ) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/"))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self
            .client
            .get_request_builder(Method::POST, url)
            .json(&upsert_request);

        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn remove_queue(&self, queue_name: &str) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/{}", encode(queue_name)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self.client.get_request_builder(Method::DELETE, url);
        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn list_queues(&self) -> Result<Vec<Queue>, QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/"))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self.client.get_request_builder(Method::GET, url);

        let response_body = self
            .client
            .send_request(request)
            .await?
            .bytes()
            .await
            .map_err(QstashError::RequestFailed)?;

        let response: Vec<Queue> =
            serde_json::from_slice(&response_body).map_err(QstashError::ResponseBodyParseError)?;

        Ok(response)
    }

    pub async fn get_queue(&self, queue_name: &str) -> Result<Queue, QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/{}/", encode(queue_name)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self.client.get_request_builder(Method::GET, url);

        let response_body = self
            .client
            .send_request(request)
            .await?
            .bytes()
            .await
            .map_err(QstashError::RequestFailed)?;

        let response: Queue =
            serde_json::from_slice(&response_body).map_err(QstashError::ResponseBodyParseError)?;

        Ok(response)
    }

    pub async fn pause_queue(&self, queue_name: &str) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/{}/pause", encode(queue_name)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self.client.get_request_builder(Method::POST, url);
        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn resume_queue(&self, queue_name: &str) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/{}/resume", encode(queue_name)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self.client.get_request_builder(Method::POST, url);
        self.client.send_request(request).await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpsertQueueRequest {
    #[serde(rename = "queueName")]
    pub queue_name: String,
    pub parallelism: i32,
}

/// Represents the metadata of a queue with creation, update, and processing details.
#[derive(Serialize, Deserialize, Debug)]
pub struct Queue {
    /// The creation time of the queue in Unix milliseconds.
    #[serde(rename = "createdAt")]
    pub created_at: i64,

    /// The update time of the queue in Unix milliseconds.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,

    /// The name of the queue.
    pub name: String,

    /// The number of parallel consumers consuming from the queue.
    pub parallelism: i32,

    /// The number of unprocessed messages that exist in the queue.
    pub lag: i32,
}

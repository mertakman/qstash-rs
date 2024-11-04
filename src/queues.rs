use crate::client::QstashClient;
use crate::errors::QstashError;
use reqwest::Method;
use urlencoding::encode;

impl QstashClient {
    pub async fn upsert_queue(&self) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/"))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        todo!()
    }

    pub async fn remove_queue(&self, queue_name: &str) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/{}/", encode(queue_name)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        todo!()
    }

    pub async fn list_queues(&self) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/"))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        todo!()
    }

    pub async fn get_queue(&self, queue_name: &str) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/{}/", encode(queue_name)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        todo!()
    }

    pub async fn pause_queue(&self, queue_name: &str) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/{}/pause", encode(queue_name)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        todo!()
    }

    pub async fn resume_queue(&self, queue_name: &str) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/{}/resume", encode(queue_name)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self.client.get_request_builder(Method::DELETE, url);
        self.client.send_request(request).await?;
        Ok(())
    }
}

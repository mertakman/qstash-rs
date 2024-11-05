use crate::{client::QstashClient, errors::QstashError};


impl QstashClient {
    pub async fn dlq_list_messages(&self) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/dlq/"))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        todo!()
    }

    pub async fn dlq_get_message(&self) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/"))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        todo!()
    }

    pub async fn dlq_delete_message(&self) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/"))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        todo!()
    }

    pub async fn dlq_delete_messages(&self) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/queues/"))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        todo!()
    }
}
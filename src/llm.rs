use reqwest::Method;

use crate::client::QstashClient;
use crate::errors::QstashError;

impl QstashClient {
    pub async fn create_chat_completion(&self) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join("/llm/v1/chat/completions")
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self.client.get_request_builder(Method::GET, url);
        let response = self.client.send_request(request).await?;

        match response.error_for_status() {
            Ok(_) => Ok(()),
            Err(err) => Err(QstashError::RequestFailed(err)),
        }
    }
}

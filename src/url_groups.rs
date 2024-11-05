use reqwest::Method;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::client::QstashClient;
use crate::errors::QstashError;

impl QstashClient {
    pub async fn upsert_url_group_endpoint(
        &self,
        url_group_name: &str,
        endpoints: Vec<Endpoint>,
    ) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/topics/{}/endpoints", encode(url_group_name)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let body = json!({
            "endpoints": endpoints,
        })
        .to_string();

        let request = self
            .client
            .get_request_builder(Method::POST, url)
            .body(body);
        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn get_url_group(&self, url_group_name: &str) -> Result<UrlGroup, QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/topics/{}", encode(url_group_name)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self.client.get_request_builder(Method::GET, url);

        let response_body = self
            .client
            .send_request(request)
            .await?
            .bytes()
            .await
            .map_err(QstashError::RequestFailed)?;

        let response: UrlGroup =
            serde_json::from_slice(&response_body).map_err(QstashError::ResponseBodyParseError)?;

        Ok(response)
    }
    pub async fn list_url_groups(&self) -> Result<Vec<UrlGroup>, QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/topics"))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self.client.get_request_builder(Method::GET, url);

        let response_body = self
            .client
            .send_request(request)
            .await?
            .bytes()
            .await
            .map_err(QstashError::RequestFailed)?;

        let response: Vec<UrlGroup> =
            serde_json::from_slice(&response_body).map_err(QstashError::ResponseBodyParseError)?;

        Ok(response)
    }

    pub async fn remove_endpoints(
        &self,
        url_group_name: &str,
        endpoints: Vec<Endpoint>,
    ) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/topics/{}/endpoints", encode(url_group_name)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let body = json!({
            "endpoints": endpoints,
        })
        .to_string();

        let request = self
            .client
            .get_request_builder(Method::DELETE, url)
            .body(body);
        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn remove_url_group(&self, url_group_name: &str) -> Result<(), QstashError> {
        let url = self
            .base_url
            .join(&format!("/v2/topics/{}", encode(url_group_name)))
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self.client.get_request_builder(Method::DELETE, url);
        self.client.send_request(request).await?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UrlGroup {
    created_at: u64,
    updated_at: u64,
    name: String,
    endpoints: Vec<Endpoint>, // Assuming Endpoint is your existing type
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Endpoint {
    #[serde(skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    url: String,
}

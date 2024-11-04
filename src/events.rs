use reqwest::Method;

use crate::client::QstashClient;
use crate::errors::QstashError;
use crate::events_types::{EventsRequest, EventsResponse};

impl QstashClient {
    pub async fn list_events(&self, request: EventsRequest) -> Result<EventsResponse, QstashError> {
        let url = self
            .base_url
            .join("/v2/events")
            .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?;

        let request = self
            .client
            .get_request_builder(Method::GET, url)
            .query(&request.to_query_params());

        let response_body = self
            .client
            .send_request(request)
            .await?
            .bytes()
            .await
            .map_err(QstashError::RequestFailed)?;

        let response: EventsResponse =
            serde_json::from_slice(&response_body).map_err(QstashError::ResponseBodyParseError)?;
        Ok(response)
    }
}

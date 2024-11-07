use reqwest::Method;

use crate::client::QstashClient;
use crate::errors::QstashError;
use crate::events_types::{EventsRequest, EventsResponse};

impl QstashClient {
    pub async fn list_events(&self, request: EventsRequest) -> Result<EventsResponse, QstashError> {
        let request = self
            .client
            .get_request_builder(
                Method::GET,
                self.base_url
                    .join("/v2/events")
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .query(&request.to_query_params());

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<EventsResponse>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}

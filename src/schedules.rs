use std::collections::HashMap;

use reqwest::header::HeaderMap;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::client::QstashClient;
use crate::errors::QstashError;

impl QstashClient {
    pub async fn create_schedule(
        &self,
        destination: &str,
        headers: HeaderMap,
        body: Vec<u8>,
    ) -> Result<CreateScheduleResponse, QstashError> {
        let request = self
            .client
            .get_request_builder(
                Method::POST,
                self.base_url
                    .join(&format!("/v2/schedules/{}", destination))
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .headers(headers)
            .body(body);

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<CreateScheduleResponse>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn get_schedule(&self, schedule_id: &str) -> Result<Schedule, QstashError> {
        let request = self.client.get_request_builder(
            Method::GET,
            self.base_url
                .join(&format!("/v2/schedules/{}", encode(schedule_id)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<Schedule>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn list_schedules(&self) -> Result<Vec<Schedule>, QstashError> {
        let request = self.client.get_request_builder(
            Method::GET,
            self.base_url
                .join("/v2/schedules")
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<Vec<Schedule>>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn remove_schedule(&self, schedule_id: &str) -> Result<(), QstashError> {
        let request = self.client.get_request_builder(
            Method::DELETE,
            self.base_url
                .join(&format!("/v2/schedules/{}", encode(schedule_id)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        self.client.send_request(request).await?;

        Ok(())
    }

    pub async fn pause_schedule(&self, schedule_id: &str) -> Result<(), QstashError> {
        let request = self.client.get_request_builder(
            Method::POST,
            self.base_url
                .join(&format!("/v2/schedules/{}/pause", encode(schedule_id)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        self.client.send_request(request).await?;

        Ok(())
    }

    pub async fn resume_schedule(&self, schedule_id: &str) -> Result<(), QstashError> {
        let request = self.client.get_request_builder(
            Method::POST,
            self.base_url
                .join(&format!("/v2/schedules/{}/resume", encode(schedule_id)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        self.client.send_request(request).await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateScheduleResponse {
    #[serde(rename = "scheduleId")]
    pub schedule_id: String,
}
/// Represents a single schedule object within the Response array.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    /// The creation time of the object. Unix timestamp in milliseconds.
    pub created_at: i64,

    /// The ID of the schedule.
    pub id: String,

    /// The cron expression used to schedule the message.
    pub cron: String,

    /// URL or URL Group (topic) name.
    pub destination: String,

    /// The HTTP method to use for the message.
    pub method: String,

    /// The headers of the message.
    pub header: HashMap<String, Vec<String>>,

    /// The body of the message.
    pub body: String,

    /// The number of retries that should be attempted in case of delivery failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<i32>,

    /// The delay in seconds before the message is delivered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<i32>,

    /// The URL where a callback is sent after the message is delivered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback: Option<String>,
}

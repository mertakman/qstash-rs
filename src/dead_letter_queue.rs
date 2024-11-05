use std::collections::HashMap;

use reqwest::Method;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{client::QstashClient, errors::QstashError};

impl QstashClient {
    pub async fn dlq_list_messages(&self) -> Result<DLQMessagesList, QstashError> {
        let request = self.client.get_request_builder(
            Method::GET,
            self.base_url
                .join("/v2/dlq/")
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<DLQMessagesList>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn dlq_get_message(&self, dlq_id: &str) -> Result<DLQMessage, QstashError> {
        let request = self.client.get_request_builder(
            Method::GET,
            self.base_url
                .join(&format!("/v2/dlq/{}", encode(dlq_id)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<DLQMessage>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn dlq_delete_message(&self, dlq_id: &str) -> Result<(), QstashError> {
        let request = self.client.get_request_builder(
            Method::DELETE,
            self.base_url
                .join(&format!("/v2/dlq/{}", encode(dlq_id)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn dlq_delete_messages(
        &self,
        dlq_ids: Vec<String>,
    ) -> Result<DLQDeleteMessagesResponse, QstashError> {
        let body = json!({
            "dlqIds": dlq_ids,
        })
        .to_string();

        let request = self
            .client
            .get_request_builder(
                Method::DELETE,
                self.base_url
                    .join("/v2/queues/")
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .body(body);

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<DLQDeleteMessagesResponse>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }
}

/// Represents a paginated response containing a list of messages.
#[derive(Serialize, Deserialize, Debug)]
pub struct DLQMessagesList {
    /// A cursor which you can use in subsequent requests to paginate through all events.
    /// If no cursor is returned, you have reached the end of the events.
    pub cursor: Option<String>,

    /// Array of messages.
    pub messages: Vec<DLQMessage>,
}

/// Represents an individual message with delivery and metadata details.
#[derive(Serialize, Deserialize, Debug)]
pub struct DLQMessage {
    /// A unique identifier for this message.
    #[serde(rename = "messageId")]
    pub message_id: String,

    /// The URL to which the message should be delivered.
    #[serde(rename = "url")]
    pub url: String,

    /// The unix timestamp in milliseconds when the message was created.
    #[serde(rename = "createdAt")]
    pub created_at: i64,

    /// IP address of the publisher of this message.
    #[serde(rename = "callerIP")]
    pub caller_ip: String,

    /// The unique id within the DLQ. Use this to remove the message from the DLQ DELETE /v2/dlq/{dlqId}.
    #[serde(rename = "dlqId")]
    pub dlq_id: String,

    /// The URL Group (topic) name if this message was sent to a URL Group.
    #[serde(rename = "topicName")]
    pub topic_name: Option<String>,

    /// The endpoint name of the message if the endpoint is given a name within the URL Group.
    #[serde(rename = "endpointName")]
    pub endpoint_name: Option<String>,

    /// The HTTP method to use for the message.
    #[serde(rename = "method")]
    pub method: Option<String>,

    /// The HTTP headers sent to your API.
    #[serde(rename = "header")]
    pub header: Option<HashMap<String, Vec<String>>>,

    /// The body of the message if it is composed of UTF-8 characters only, empty otherwise.
    #[serde(rename = "body")]
    pub body: Option<String>,

    /// The base64 encoded body if the body contains a non-UTF-8 character only, empty otherwise.
    #[serde(rename = "bodyBase64")]
    pub body_base64: Option<String>,

    /// The number of retries that should be attempted in case of delivery failure.
    #[serde(rename = "maxRetries")]
    pub max_retries: Option<i32>,

    /// The unix timestamp in milliseconds before which the message should not be delivered.
    #[serde(rename = "notBefore")]
    pub not_before: Option<i64>,

    /// The URL where we send a callback each time the message is attempted to be delivered.
    #[serde(rename = "callback")]
    pub callback: Option<String>,

    /// The URL where we send a callback after the message fails.
    #[serde(rename = "failureCallback")]
    pub failure_callback: Option<String>,

    /// The schedule ID of the message if the message is triggered by a schedule.
    #[serde(rename = "scheduleId")]
    pub schedule_id: Option<String>,

    /// The name of the queue if this message is enqueued on a queue.
    #[serde(rename = "queueName")]
    pub queue_name: Option<String>,

    /// The HTTP status code of the last failed delivery attempt.
    #[serde(rename = "responseStatus")]
    pub response_status: Option<i32>,

    /// The response header of the last failed delivery attempt.
    #[serde(rename = "responseHeader")]
    pub response_header: Option<String>,

    /// The response body of the last failed delivery attempt if it is composed of UTF-8 characters only, empty otherwise.
    #[serde(rename = "responseBody")]
    pub response_body: Option<String>,

    /// The base64 encoded response body of the last failed delivery attempt if the response body contains a non-UTF-8 character only, empty otherwise.
    #[serde(rename = "responseBodyBase64")]
    pub response_body_base64: Option<String>,
}

#[derive(Deserialize)]
pub struct DLQDeleteMessagesResponse {
    pub deleted: u32,
}

use std::collections::HashMap;

use reqwest::Method;
use serde::{Deserialize, Serialize};

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
                .join(&format!("/v2/dlq/{}", dlq_id))
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
                .join(&format!("/v2/dlq/{}", dlq_id))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn dlq_delete_messages(
        &self,
        dlq_ids: Vec<String>,
    ) -> Result<DLQDeleteMessagesResponse, QstashError> {
        let request = self
            .client
            .get_request_builder(
                Method::DELETE,
                self.base_url
                    .join("/v2/queues/")
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .json(&json!({
                "dlqIds": dlq_ids,
            }));

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

#[derive(Serialize, Deserialize)]
pub struct DLQDeleteMessagesResponse {
    pub deleted: u32,
}

#[cfg(test)]
mod tests {

    use crate::client::QstashClient;
    use crate::dead_letter_queue::{DLQDeleteMessagesResponse, DLQMessage, DLQMessagesList};
    use crate::errors::QstashError;
    use httpmock::Method::{DELETE, GET};
    use httpmock::MockServer;
    use reqwest::StatusCode;
    use reqwest::Url;
    use serde_json::json;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_dlq_list_messages_success() {
        let server = MockServer::start();
        let expected_response = DLQMessagesList {
            cursor: Some("next_cursor".to_string()),
            messages: vec![DLQMessage {
                message_id: "msg123".to_string(),
                url: "https://example.com/endpoint".to_string(),
                created_at: 1625097600000,
                caller_ip: "127.0.0.1".to_string(),
                dlq_id: "dlq123".to_string(),
                topic_name: Some("topic1".to_string()),
                endpoint_name: Some("endpoint1".to_string()),
                method: Some("POST".to_string()),
                header: Some(HashMap::from([(
                    "Content-Type".to_string(),
                    vec!["application/json".to_string()],
                )])),
                body: Some("{\"key\":\"value\"}".to_string()),
                body_base64: None,
                max_retries: Some(3),
                not_before: Some(1625097600000),
                callback: Some("https://example.com/callback".to_string()),
                failure_callback: Some("https://example.com/failure_callback".to_string()),
                schedule_id: Some("sched123".to_string()),
                queue_name: Some("queue1".to_string()),
                response_status: Some(500),
                response_header: Some("Header".to_string()),
                response_body: Some("Internal Server Error".to_string()),
                response_body_base64: None,
            }],
        };
        let list_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/dlq/")
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_response);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.dlq_list_messages().await;
        list_mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.cursor, expected_response.cursor);
        assert_eq!(response.messages.len(), expected_response.messages.len());
        // Further assertions can be added to check the contents of the messages
    }

    #[tokio::test]
    async fn test_dlq_list_messages_rate_limit_error() {
        let server = MockServer::start();
        let list_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/dlq/")
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600000")
                .body("Rate limit exceeded");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.dlq_list_messages().await;
        list_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded {
                reset: 1625097600000
            })
        ));
    }

    #[tokio::test]
    async fn test_dlq_list_messages_invalid_response() {
        let server = MockServer::start();
        let list_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/dlq/")
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .body("Invalid JSON");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.dlq_list_messages().await;
        list_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_dlq_get_message_success() {
        let server = MockServer::start();
        let dlq_id = "dlq123";
        let expected_message = DLQMessage {
            message_id: "msg123".to_string(),
            url: "https://example.com/endpoint".to_string(),
            created_at: 1625097600000,
            caller_ip: "127.0.0.1".to_string(),
            dlq_id: dlq_id.to_string(),
            topic_name: Some("topic1".to_string()),
            endpoint_name: Some("endpoint1".to_string()),
            method: Some("POST".to_string()),
            header: Some(HashMap::from([(
                "Content-Type".to_string(),
                vec!["application/json".to_string()],
            )])),
            body: Some("{\"key\":\"value\"}".to_string()),
            body_base64: None,
            max_retries: Some(3),
            not_before: Some(1625097600000),
            callback: Some("https://example.com/callback".to_string()),
            failure_callback: Some("https://example.com/failure_callback".to_string()),
            schedule_id: Some("sched123".to_string()),
            queue_name: Some("queue1".to_string()),
            response_status: Some(500),
            response_header: Some("Header".to_string()),
            response_body: Some("Internal Server Error".to_string()),
            response_body_base64: None,
        };
        let get_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/dlq/{}", dlq_id))
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_message);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.dlq_get_message(dlq_id).await;
        get_mock.assert();
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message.dlq_id, expected_message.dlq_id);
        // Further assertions can be added to check the contents of the message
    }

    #[tokio::test]
    async fn test_dlq_get_message_rate_limit_error() {
        let server = MockServer::start();
        let dlq_id = "dlq123";
        let get_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/dlq/{}", dlq_id))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600000")
                .body("Rate limit exceeded");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.dlq_get_message(dlq_id).await;
        get_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded {
                reset: 1625097600000
            })
        ));
    }

    #[tokio::test]
    async fn test_dlq_get_message_invalid_response() {
        let server = MockServer::start();
        let dlq_id = "dlq123";
        let get_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/dlq/{}", dlq_id))
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .body("Invalid JSON");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.dlq_get_message(dlq_id).await;
        get_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_dlq_delete_message_success() {
        let server = MockServer::start();
        let dlq_id = "dlq123";
        let delete_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/dlq/{}", dlq_id))
                .header("Authorization", "Bearer test_api_key");
            then.status(204);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.dlq_delete_message(dlq_id).await;
        delete_mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dlq_delete_message_rate_limit_error() {
        let server = MockServer::start();
        let dlq_id = "dlq123";
        let delete_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/dlq/{}", dlq_id))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600000")
                .body("Rate limit exceeded");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.dlq_delete_message(dlq_id).await;
        delete_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded {
                reset: 1625097600000
            })
        ));
    }

    #[tokio::test]
    async fn test_dlq_delete_messages_success() {
        let server = MockServer::start();
        let dlq_ids = vec!["dlq123".to_string(), "dlq124".to_string()];
        let expected_response = DLQDeleteMessagesResponse { deleted: 2 };
        let delete_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path("/v2/queues/")
                .header("Authorization", "Bearer test_api_key")
                .json_body(json!({
                    "dlqIds": ["dlq123", "dlq124"]
                }));
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_response);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.dlq_delete_messages(dlq_ids.clone()).await;
        delete_mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.deleted, expected_response.deleted);
    }

    #[tokio::test]
    async fn test_dlq_delete_messages_rate_limit_error() {
        let server = MockServer::start();
        let dlq_ids = vec!["dlq123".to_string(), "dlq124".to_string()];
        let delete_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path("/v2/queues/")
                .header("Authorization", "Bearer test_api_key")
                .json_body(json!({
                    "dlqIds": ["dlq123", "dlq124"]
                }));
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600000")
                .body("Rate limit exceeded");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.dlq_delete_messages(dlq_ids.clone()).await;
        delete_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded {
                reset: 1625097600000
            })
        ));
    }

    #[tokio::test]
    async fn test_dlq_delete_messages_invalid_response() {
        let server = MockServer::start();
        let dlq_ids = vec!["dlq123".to_string(), "dlq124".to_string()];
        let delete_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path("/v2/queues/")
                .header("Authorization", "Bearer test_api_key")
                .json_body(json!({
                    "dlqIds": ["dlq123", "dlq124"]
                }));
            then.status(200)
                .header("Content-Type", "application/json")
                .body("Invalid Response");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.dlq_delete_messages(dlq_ids.clone()).await;
        delete_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }
}

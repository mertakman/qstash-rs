use reqwest::Method;

use crate::client::QstashClient;
use crate::errors::QstashError;
use crate::message_types::{BatchEntry, Message, MessageResponseResult};
use reqwest::header::HeaderMap;

impl QstashClient {
    pub async fn publish_message(
        &self,
        destination: &str,
        headers: HeaderMap,
        body: Vec<u8>,
    ) -> Result<MessageResponseResult, QstashError> {
        let request = self
            .client
            .get_request_builder(
                Method::POST,
                self.base_url
                    .join(&format!("/v2/publish/{}", destination))
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .headers(headers)
            .body(body);

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<MessageResponseResult>()
            .await
            .map_err(QstashError::ResponseBodyParseError)?;

        Ok(response)
    }

    pub async fn enqueue_message(
        &self,
        destination: &str,
        queue_name: &str,
        headers: HeaderMap,
        body: Vec<u8>,
    ) -> Result<MessageResponseResult, QstashError> {
        let request = self
            .client
            .get_request_builder(
                Method::POST,
                self.base_url
                    .join(&format!("/v2/enqueue/{}/{}", queue_name, destination))
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .headers(headers)
            .body(body);

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<MessageResponseResult>()
            .await
            .map_err(QstashError::ResponseBodyParseError)?;

        Ok(response)
    }

    pub async fn batch_messages(
        &self,
        batch_entries: Vec<BatchEntry>,
    ) -> Result<Vec<MessageResponseResult>, QstashError> {
        let request = self
            .client
            .get_request_builder(
                Method::POST,
                self.base_url
                    .join("/v2/batch")
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .json(&batch_entries);

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<Vec<MessageResponseResult>>()
            .await
            .map_err(QstashError::ResponseBodyParseError)?;

        Ok(response)
    }

    pub async fn get_message(&self, message_id: &str) -> Result<Message, QstashError> {
        let request = self.client.get_request_builder(
            Method::GET,
            self.base_url
                .join(&format!("/v2/messages/{}", message_id))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<Message>()
            .await
            .map_err(QstashError::ResponseBodyParseError)?;

        Ok(response)
    }

    pub async fn cancel_message(&self, message_id: &str) -> Result<(), QstashError> {
        let request = self.client.get_request_builder(
            Method::DELETE,
            self.base_url
                .join(&format!("/v2/messages/{}", message_id))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn bulk_cancel_messages(&self, message_ids: Vec<String>) -> Result<(), QstashError> {
        println!(
            "{}",
            &json!({
                "messageIds": message_ids,
            })
            .to_string()
        );
        let request = self
            .client
            .get_request_builder(
                Method::DELETE,
                self.base_url
                    .join("/v2/messages")
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .json(&json!({
                "messageIds": message_ids,
            }));

        self.client.send_request(request).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // tests/qstash_client_message_tests.rs

    use crate::client::QstashClient;
    use crate::errors::QstashError;
    use crate::message_types::{BatchEntry, Message, MessageResponse, MessageResponseResult};
    use httpmock::Method::{DELETE, GET, POST};
    use httpmock::MockServer;
    use reqwest::header::{HeaderMap, HeaderValue};
    use reqwest::StatusCode;
    use reqwest::Url;
    use serde_json::json;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_publish_message_success_single_response() {
        let server = MockServer::start();
        let destination = "https://example.com/publish";
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let body = b"{\"key\":\"value\"}".to_vec();
        let expected_response = MessageResponseResult::URLResponse(MessageResponse {
            message_id: "msg123".to_string(),
            url: Some("https://example.com/publish".to_string()),
            deduplicated: Some(false),
        });
        let publish_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/publish/https://example.com/publish")
                .header("Authorization", "Bearer test_api_key")
                .header("content-type", "application/json")
                .body("{\"key\":\"value\"}");
            then.status(StatusCode::OK.as_u16())
                .header("content-type", "application/json")
                .json_body_obj(&expected_response);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.publish_message(destination, headers, body).await;
        publish_mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response, expected_response);
    }

    #[tokio::test]
    async fn test_publish_message_success_multiple_responses() {
        let server = MockServer::start();
        let destination = "https://example.com/publish";
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let body = b"{\"key\":\"value\"}".to_vec();
        let expected_response = MessageResponseResult::URLGroupResponse(vec![
            MessageResponse {
                message_id: "msg123".to_string(),
                url: Some("https://example.com/publish".to_string()),
                deduplicated: Some(false),
            },
            MessageResponse {
                message_id: "msg124".to_string(),
                url: Some("https://example.com/publish".to_string()),
                deduplicated: Some(true),
            },
        ]);
        let publish_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/publish/https://example.com/publish")
                .header("Authorization", "Bearer test_api_key")
                .header("content-type", "application/json")
                .body("{\"key\":\"value\"}");
            then.status(StatusCode::OK.as_u16())
                .header("content-type", "application/json")
                .json_body_obj(&expected_response);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.publish_message(destination, headers, body).await;
        publish_mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response, expected_response);
    }

    #[tokio::test]
    async fn test_publish_message_rate_limit_error() {
        let server = MockServer::start();
        let destination = "https://example.com/publish";
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let body = b"{\"key\":\"value\"}".to_vec();
        let publish_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/publish/https://example.com/publish")
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600")
                .body("Rate limit exceeded");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.publish_message(destination, headers, body).await;
        publish_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_publish_message_invalid_response() {
        let server = MockServer::start();
        let destination = "https://example.com/publish";
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let body = b"{\"key\":\"value\"}".to_vec();
        let publish_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/publish/https://example.com/publish")
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::OK.as_u16())
                .header("content-type", "application/json")
                .body("Invalid JSON");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.publish_message(destination, headers, body).await;
        publish_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_enqueue_message_success_single_response() {
        let server = MockServer::start();
        let destination = "https://example.com/enqueue";
        let queue_name = "queue1";
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let body = b"{\"key\":\"value\"}".to_vec();
        let expected_response = MessageResponseResult::URLResponse(MessageResponse {
            message_id: "msg125".to_string(),
            url: Some("https://example.com/enqueue".to_string()),
            deduplicated: Some(false),
        });
        let enqueue_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!(
                    "/v2/enqueue/{}/https://example.com/enqueue",
                    queue_name
                ))
                .header("Authorization", "Bearer test_api_key")
                .header("content-type", "application/json")
                .body("{\"key\":\"value\"}");
            then.status(StatusCode::OK.as_u16())
                .header("content-type", "application/json")
                .json_body_obj(&expected_response);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client
            .enqueue_message(destination, queue_name, headers, body)
            .await;
        enqueue_mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response, expected_response);
    }

    #[tokio::test]
    async fn test_enqueue_message_rate_limit_error() {
        let server = MockServer::start();
        let destination = "https://example.com/enqueue";
        let queue_name = "queue1";
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let body = b"{\"key\":\"value\"}".to_vec();
        let enqueue_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!(
                    "/v2/enqueue/{}/https://example.com/enqueue",
                    queue_name
                ))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600")
                .body("Rate limit exceeded");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client
            .enqueue_message(destination, queue_name, headers, body)
            .await;
        enqueue_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_enqueue_message_invalid_response() {
        let server = MockServer::start();
        let destination = "https://example.com/enqueue";
        let queue_name = "queue1";
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        let body = b"{\"key\":\"value\"}".to_vec();
        let enqueue_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!(
                    "/v2/enqueue/{}/https://example.com/enqueue",
                    queue_name
                ))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::OK.as_u16())
                .header("content-type", "application/json")
                .body("Invalid JSON");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client
            .enqueue_message(destination, queue_name, headers, body)
            .await;
        enqueue_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_batch_messages_success() {
        let server = MockServer::start();
        let batch_entries = vec![
            BatchEntry {
                destination: "https://example.com/publish1".to_string(),
                queue: Some("queue1".to_string()),
                headers: {
                    let mut headers = HeaderMap::new();
                    headers.insert("content-type", HeaderValue::from_static("application/json"));
                    headers
                },
                body: Some("Message 1".to_string()),
            },
            BatchEntry {
                destination: "https://example.com/publish2".to_string(),
                queue: Some("queue2".to_string()),
                headers: {
                    let mut headers = HeaderMap::new();
                    headers.insert("content-type", HeaderValue::from_static("text/plain"));
                    headers
                },
                body: Some("Message 2".to_string()),
            },
        ];
        let expected_response = vec![
            MessageResponseResult::URLResponse(MessageResponse {
                message_id: "msg126".to_string(),
                url: Some("https://example.com/publish1".to_string()),
                deduplicated: Some(false),
            }),
            MessageResponseResult::URLGroupResponse(vec![
                MessageResponse {
                    message_id: "msg127".to_string(),
                    url: Some("https://example.com/publish2".to_string()),
                    deduplicated: Some(true),
                },
                MessageResponse {
                    message_id: "msg128".to_string(),
                    url: Some("https://example.com/publish2".to_string()),
                    deduplicated: Some(false),
                },
            ]),
        ];
        let batch_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/batch")
                .header("Authorization", "Bearer test_api_key")
                .json_body(json!([
                    {
                        "destination": "https://example.com/publish1",
                        "queue": "queue1",
                        "headers": {
                            "content-type": "application/json"
                        },
                        "body": "Message 1"
                    },
                    {
                        "destination": "https://example.com/publish2",
                        "queue": "queue2",
                        "headers": {
                            "content-type": "text/plain"
                        },
                        "body": "Message 2"
                    }
                ]));
            then.status(StatusCode::OK.as_u16())
                .header("content-type", "application/json")
                .json_body(json!([
                    {
                        "messageId": "msg126",
                        "url": "https://example.com/publish1",
                        "deduplicated": false
                    },
                    [
                        {
                            "messageId": "msg127",
                            "url": "https://example.com/publish2",
                            "deduplicated": true
                        },
                        {
                            "messageId": "msg128",
                            "url": "https://example.com/publish2",
                            "deduplicated": false
                        }
                    ]
                ]));
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.batch_messages(batch_entries).await;
        batch_mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response, expected_response);
    }

    #[tokio::test]
    async fn test_batch_messages_rate_limit_error() {
        let server = MockServer::start();
        let batch_entries = vec![BatchEntry {
            destination: "https://example.com/publish1".to_string(),
            queue: Some("queue1".to_string()),
            headers: {
                let mut headers = HeaderMap::new();
                headers.insert("content-type", HeaderValue::from_static("application/json"));
                headers
            },
            body: Some("Message 1".to_string()),
        }];
        let batch_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/batch")
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600")
                .body("Rate limit exceeded");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.batch_messages(batch_entries).await;
        batch_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_batch_messages_invalid_response() {
        let server = MockServer::start();
        let batch_entries = vec![BatchEntry {
            destination: "https://example.com/publish1".to_string(),
            queue: Some("queue1".to_string()),
            headers: {
                let mut headers = HeaderMap::new();
                headers.insert("content-type", HeaderValue::from_static("application/json"));
                headers
            },
            body: Some("Message 1".to_string()),
        }];
        let batch_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/batch")
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::OK.as_u16())
                .header("content-type", "application/json")
                .body("Invalid JSON");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.batch_messages(batch_entries).await;
        batch_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_get_message_success() {
        let server = MockServer::start();
        let message_id = "msg123";
        let expected_message = Message {
            message_id: "msg123".to_string(),
            topic_name: "topic1".to_string(),
            url: "https://example.com/publish".to_string(),
            method: "POST".to_string(),
            header: HashMap::from([
                (
                    "content-type".to_string(),
                    vec!["application/json".to_string()],
                ),
                ("X-Custom".to_string(), vec!["value1".to_string()]),
            ]),
            body: "{\"key\":\"value\"}".to_string(),
            created_at: 1625097600,
        };
        let get_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/messages/{}", message_id))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::OK.as_u16())
                .header("content-type", "application/json")
                .json_body_obj(&expected_message);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.get_message(message_id).await;
        get_mock.assert();
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message, expected_message);
    }

    #[tokio::test]
    async fn test_get_message_rate_limit_error() {
        let server = MockServer::start();
        let message_id = "msg123";
        let get_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/messages/{}", message_id))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600")
                .body("Rate limit exceeded");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.get_message(message_id).await;
        get_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_get_message_invalid_response() {
        let server = MockServer::start();
        let message_id = "msg123";
        let get_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/messages/{}", message_id))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::OK.as_u16())
                .header("content-type", "application/json")
                .body("Invalid JSON");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.get_message(message_id).await;
        get_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_cancel_message_success() {
        let server = MockServer::start();
        let message_id = "msg123";
        let cancel_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/messages/{}", message_id))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::NO_CONTENT.as_u16());
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.cancel_message(message_id).await;
        cancel_mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cancel_message_rate_limit_error() {
        let server = MockServer::start();
        let message_id = "msg123";
        let cancel_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/messages/{}", message_id))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600")
                .body("Rate limit exceeded");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.cancel_message(message_id).await;
        cancel_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_bulk_cancel_messages_success() {
        let server = MockServer::start();
        let message_ids = [
            "msg123".to_string(),
            "msg124".to_string(),
            "msg125".to_string(),
        ];
        let bulk_cancel_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path("/v2/messages")
                .header("Authorization", "Bearer test_api_key")
                .json_body(json!({
                    "messageIds": ["msg123", "msg124", "msg125"]
                }));
            then.status(StatusCode::NO_CONTENT.as_u16());
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.bulk_cancel_messages(message_ids.to_vec()).await;
        bulk_cancel_mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_bulk_cancel_messages_rate_limit_error() {
        let server = MockServer::start();
        let message_ids = [
            "msg123".to_string(),
            "msg124".to_string(),
            "msg125".to_string(),
        ];
        let bulk_cancel_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path("/v2/messages")
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600")
                .body("Rate limit exceeded");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.bulk_cancel_messages(message_ids.to_vec()).await;
        bulk_cancel_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_publish_message_header_serialization() {
        let server = MockServer::start();
        let destination = "https://example.com/publish";
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        headers.insert("X-Custom-Header", HeaderValue::from_static("CustomValue"));
        let body = b"{\"key\":\"value\"}".to_vec();
        let expected_response = MessageResponseResult::URLResponse(MessageResponse {
            message_id: "msg129".to_string(),
            url: Some("https://example.com/publish".to_string()),
            deduplicated: Some(false),
        });
        let publish_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/publish/https://example.com/publish")
                .header("Authorization", "Bearer test_api_key")
                .header("content-type", "application/json")
                .header("X-Custom-Header", "CustomValue")
                .body("{\"key\":\"value\"}");
            then.status(StatusCode::OK.as_u16())
                .header("content-type", "application/json")
                .json_body_obj(&expected_response);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.publish_message(destination, headers, body).await;
        publish_mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response, expected_response);
    }

    #[tokio::test]
    async fn test_enqueue_message_header_serialization() {
        let server = MockServer::start();
        let destination = "https://example.com/enqueue";
        let queue_name = "queue1";
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("text/plain"));
        headers.insert("X-Another-Header", HeaderValue::from_static("AnotherValue"));
        let body = b"Enqueue message".to_vec();
        let expected_response = MessageResponseResult::URLResponse(MessageResponse {
            message_id: "msg130".to_string(),
            url: Some("https://example.com/enqueue".to_string()),
            deduplicated: Some(false),
        });
        let enqueue_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!(
                    "/v2/enqueue/{}/https://example.com/enqueue",
                    queue_name
                ))
                .header("Authorization", "Bearer test_api_key")
                .header("content-type", "text/plain")
                .header("X-Another-Header", "AnotherValue")
                .body("Enqueue message");
            then.status(StatusCode::OK.as_u16())
                .header("content-type", "application/json")
                .json_body_obj(&expected_response);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client
            .enqueue_message(destination, queue_name, headers, body)
            .await;
        enqueue_mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response, expected_response);
    }
}

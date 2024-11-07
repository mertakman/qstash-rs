use crate::client::QstashClient;
use crate::errors::QstashError;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

impl QstashClient {
    pub async fn upsert_queue(
        &self,
        upsert_request: UpsertQueueRequest,
    ) -> Result<(), QstashError> {
        let request = self
            .client
            .get_request_builder(
                Method::POST,
                self.base_url
                    .join("/v2/queues/")
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .json(&upsert_request);

        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn remove_queue(&self, queue_name: &str) -> Result<(), QstashError> {
        let request = self.client.get_request_builder(
            Method::DELETE,
            self.base_url
                .join(&format!("/v2/queues/{}", encode(queue_name)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn list_queues(&self) -> Result<Vec<Queue>, QstashError> {
        let request = self.client.get_request_builder(
            Method::GET,
            self.base_url
                .join("/v2/queues/")
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<Vec<Queue>>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn get_queue(&self, queue_name: &str) -> Result<Queue, QstashError> {
        let request = self.client.get_request_builder(
            Method::GET,
            self.base_url
                .join(&format!("/v2/queues/{}/", encode(queue_name)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<Queue>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn pause_queue(&self, queue_name: &str) -> Result<(), QstashError> {
        let request = self.client.get_request_builder(
            Method::POST,
            self.base_url
                .join(&format!("/v2/queues/{}/pause", encode(queue_name)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn resume_queue(&self, queue_name: &str) -> Result<(), QstashError> {
        let request = self.client.get_request_builder(
            Method::POST,
            self.base_url
                .join(&format!("/v2/queues/{}/resume", encode(queue_name)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        self.client.send_request(request).await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpsertQueueRequest {
    #[serde(rename = "queueName")]
    pub queue_name: String,
    pub parallelism: i32,
}

/// Represents the metadata of a queue with creation, update, and processing details.
#[derive(Serialize, Deserialize, Debug)]
pub struct Queue {
    /// The creation time of the queue in Unix milliseconds.
    #[serde(rename = "createdAt")]
    pub created_at: i64,

    /// The update time of the queue in Unix milliseconds.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,

    /// The name of the queue.
    pub name: String,

    /// The number of parallel consumers consuming from the queue.
    pub parallelism: i32,

    /// The number of unprocessed messages that exist in the queue.
    pub lag: i32,
}

#[cfg(test)]
mod tests {
    use crate::errors::QstashError;
    use crate::*;
    use client::QstashClient;
    use httpmock::Method::{DELETE, GET, POST};
    use httpmock::MockServer;
    use queues::{Queue, UpsertQueueRequest};
    use reqwest::StatusCode;
    use reqwest::Url;
    use urlencoding::encode;

    #[tokio::test]
    async fn test_upsert_queue_success() {
        let server = MockServer::start();
        let upsert_request = UpsertQueueRequest {
            queue_name: "test-queue".to_string(),
            parallelism: 5,
        };
        let upsert_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/queues/")
                .header("Authorization", "Bearer test_api_key")
                .header("Content-Type", "application/json")
                .json_body_obj(&upsert_request);
            then.status(200);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.upsert_queue(upsert_request).await;
        upsert_mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upsert_queue_rate_limit_error() {
        let server = MockServer::start();
        let upsert_request = UpsertQueueRequest {
            queue_name: "test-queue".to_string(),
            parallelism: 5,
        };
        let rate_limit_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/queues/")
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
        let result = client.upsert_queue(upsert_request).await;
        rate_limit_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_upsert_queue_invalid_response() {
        let server = MockServer::start();
        let upsert_request = UpsertQueueRequest {
            queue_name: "test-queue".to_string(),
            parallelism: 5,
        };
        let invalid_response_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/queues/")
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
        let result = client.upsert_queue(upsert_request).await;
        invalid_response_mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_queue_success() {
        let server = MockServer::start();
        let queue_name = "test-queue";
        let remove_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/queues/{}", encode(queue_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.remove_queue(queue_name).await;
        remove_mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_queue_rate_limit_error() {
        let server = MockServer::start();
        let queue_name = "test-queue";
        let rate_limit_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/queues/{}", encode(queue_name)))
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
        let result = client.remove_queue(queue_name).await;
        rate_limit_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_remove_queue_invalid_response() {
        let server = MockServer::start();
        let queue_name = "test-queue";
        let invalid_response_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/queues/{}", encode(queue_name)))
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
        let result = client.remove_queue(queue_name).await;
        invalid_response_mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_queues_success() {
        let server = MockServer::start();
        let expected_queues = vec![
            Queue {
                created_at: 1625097600,
                updated_at: 1625097600,
                name: "queue1".to_string(),
                parallelism: 3,
                lag: 10,
            },
            Queue {
                created_at: 1625097700,
                updated_at: 1625097700,
                name: "queue2".to_string(),
                parallelism: 5,
                lag: 0,
            },
        ];
        let list_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/queues/")
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_queues);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.list_queues().await;
        list_mock.assert();
        assert!(result.is_ok());
        let queues = result.unwrap();
        assert_eq!(queues.len(), expected_queues.len());
        for (a, e) in queues.iter().zip(expected_queues.iter()) {
            assert_eq!(a.name, e.name);
            assert_eq!(a.parallelism, e.parallelism);
            assert_eq!(a.lag, e.lag);
        }
    }

    #[tokio::test]
    async fn test_list_queues_rate_limit_error() {
        let server = MockServer::start();
        let rate_limit_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/queues/")
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
        let result = client.list_queues().await;
        rate_limit_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_list_queues_invalid_response() {
        let server = MockServer::start();
        let invalid_response_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/queues/")
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
        let result = client.list_queues().await;
        invalid_response_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_get_queue_success() {
        let server = MockServer::start();
        let queue_name = "test-queue";
        let expected_queue = Queue {
            created_at: 1625097600,
            updated_at: 1625097600,
            name: queue_name.to_string(),
            parallelism: 4,
            lag: 20,
        };
        let get_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/queues/{}/", encode(queue_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_queue);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.get_queue(queue_name).await;
        get_mock.assert();
        assert!(result.is_ok());
        let queue = result.unwrap();
        assert_eq!(queue.name, expected_queue.name);
        assert_eq!(queue.parallelism, expected_queue.parallelism);
        assert_eq!(queue.lag, expected_queue.lag);
    }

    #[tokio::test]
    async fn test_get_queue_rate_limit_error() {
        let server = MockServer::start();
        let queue_name = "test-queue";
        let rate_limit_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/queues/{}/", encode(queue_name)))
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
        let result = client.get_queue(queue_name).await;
        rate_limit_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_get_queue_invalid_response() {
        let server = MockServer::start();
        let queue_name = "test-queue";
        let invalid_response_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/queues/{}/", encode(queue_name)))
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
        let result = client.get_queue(queue_name).await;
        invalid_response_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_pause_queue_success() {
        let server = MockServer::start();
        let queue_name = "test-queue";
        let pause_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/queues/{}/pause", encode(queue_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.pause_queue(queue_name).await;
        pause_mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pause_queue_rate_limit_error() {
        let server = MockServer::start();
        let queue_name = "test-queue";
        let rate_limit_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/queues/{}/pause", encode(queue_name)))
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
        let result = client.pause_queue(queue_name).await;
        rate_limit_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_pause_queue_invalid_response() {
        let server = MockServer::start();
        let queue_name = "test-queue";
        let invalid_response_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/queues/{}/pause", encode(queue_name)))
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
        let result = client.pause_queue(queue_name).await;
        invalid_response_mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resume_queue_success() {
        let server = MockServer::start();
        let queue_name = "test-queue";
        let resume_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/queues/{}/resume", encode(queue_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.resume_queue(queue_name).await;
        resume_mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resume_queue_rate_limit_error() {
        let server = MockServer::start();
        let queue_name = "test-queue";
        let rate_limit_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/queues/{}/resume", encode(queue_name)))
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
        let result = client.resume_queue(queue_name).await;
        rate_limit_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_resume_queue_invalid_response() {
        let server = MockServer::start();
        let queue_name = "test-queue";
        let invalid_response_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/queues/{}/resume", encode(queue_name)))
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
        let result = client.resume_queue(queue_name).await;
        invalid_response_mock.assert();
        assert!(result.is_ok());
    }
}

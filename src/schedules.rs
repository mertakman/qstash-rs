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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::errors::QstashError;
    use crate::*;
    use client::QstashClient;
    use httpmock::Method::{DELETE, GET, POST};
    use httpmock::MockServer;
    use reqwest::header::HeaderMap;
    use reqwest::StatusCode;
    use reqwest::Url;
    use schedules::{CreateScheduleResponse, Schedule};
    use urlencoding::encode;

    #[tokio::test]
    async fn test_create_schedule_success() {
        let server = MockServer::start();
        let destination = "https://example.com/destination";
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        let body = b"{\"key\":\"value\"}".to_vec();
        let expected_response = CreateScheduleResponse {
            schedule_id: "schedule123".to_string(),
        };
        let create_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/schedules/https://example.com/destination")
                .header("Authorization", "Bearer test_api_key")
                .header("Content-Type", "application/json")
                .body("{\"key\":\"value\"}");
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
        let upsert_body = b"{\"key\":\"value\"}".to_vec();
        let result = client
            .create_schedule(destination, headers, upsert_body)
            .await;
        create_mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.schedule_id, expected_response.schedule_id);
    }

    #[tokio::test]
    async fn test_create_schedule_rate_limit_error() {
        let server = MockServer::start();
        let destination = "https://example.com/destination";
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        let body = b"{\"key\":\"value\"}".to_vec();
        let create_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/schedules/https://example.com/destination")
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
        let result = client.create_schedule(destination, headers, body).await;
        create_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_create_schedule_invalid_response() {
        let server = MockServer::start();
        let destination = "https://example.com/destination";
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        let body = b"{\"key\":\"value\"}".to_vec();
        let create_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/v2/schedules/https://example.com/destination")
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
        let result = client.create_schedule(destination, headers, body).await;
        create_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_get_schedule_success() {
        let server = MockServer::start();
        let schedule_id = "schedule123";
        let expected_schedule = Schedule {
            created_at: 1625097600000,
            id: schedule_id.to_string(),
            cron: "0 0 * * *".to_string(),
            destination: "https://example.com/destination".to_string(),
            method: "POST".to_string(),
            header: HashMap::from([(
                "Content-Type".to_string(),
                vec!["application/json".to_string()],
            )]),
            body: "{\"key\":\"value\"}".to_string(),
            retries: Some(3),
            delay: Some(60),
            callback: Some("https://example.com/callback".to_string()),
        };
        let get_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/schedules/{}", encode(schedule_id)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_schedule);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.get_schedule(schedule_id).await;
        get_mock.assert();
        assert!(result.is_ok());
        let schedule = result.unwrap();
        assert_eq!(schedule.id, expected_schedule.id);
        assert_eq!(schedule.cron, expected_schedule.cron);
        assert_eq!(schedule.destination, expected_schedule.destination);
        assert_eq!(schedule.method, expected_schedule.method);
        assert_eq!(schedule.header, expected_schedule.header);
        assert_eq!(schedule.body, expected_schedule.body);
        assert_eq!(schedule.retries, expected_schedule.retries);
        assert_eq!(schedule.delay, expected_schedule.delay);
        assert_eq!(schedule.callback, expected_schedule.callback);
    }

    #[tokio::test]
    async fn test_get_schedule_rate_limit_error() {
        let server = MockServer::start();
        let schedule_id = "schedule123";
        let get_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/schedules/{}", encode(schedule_id)))
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
        let result = client.get_schedule(schedule_id).await;
        get_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_get_schedule_invalid_response() {
        let server = MockServer::start();
        let schedule_id = "schedule123";
        let get_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/schedules/{}", encode(schedule_id)))
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
        let result = client.get_schedule(schedule_id).await;
        get_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_list_schedules_success() {
        let server = MockServer::start();
        let expected_schedules = vec![
            Schedule {
                created_at: 1625097600000,
                id: "schedule123".to_string(),
                cron: "0 0 * * *".to_string(),
                destination: "https://example.com/destination1".to_string(),
                method: "POST".to_string(),
                header: HashMap::from([(
                    "Content-Type".to_string(),
                    vec!["application/json".to_string()],
                )]),
                body: "{\"key\":\"value1\"}".to_string(),
                retries: Some(3),
                delay: Some(60),
                callback: Some("https://example.com/callback1".to_string()),
            },
            Schedule {
                created_at: 1625097700000,
                id: "schedule456".to_string(),
                cron: "30 1 * * *".to_string(),
                destination: "https://example.com/destination2".to_string(),
                method: "GET".to_string(),
                header: HashMap::from([(
                    "Accept".to_string(),
                    vec!["application/json".to_string()],
                )]),
                body: "".to_string(),
                retries: None,
                delay: Some(120),
                callback: None,
            },
        ];
        let list_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/schedules")
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_schedules);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.list_schedules().await;
        list_mock.assert();
        assert!(result.is_ok());
        let schedules = result.unwrap();
        assert_eq!(schedules.len(), expected_schedules.len());
        for (a, e) in schedules.iter().zip(expected_schedules.iter()) {
            assert_eq!(a.id, e.id);
            assert_eq!(a.cron, e.cron);
            assert_eq!(a.destination, e.destination);
            assert_eq!(a.method, e.method);
            assert_eq!(a.header, e.header);
            assert_eq!(a.body, e.body);
            assert_eq!(a.retries, e.retries);
            assert_eq!(a.delay, e.delay);
            assert_eq!(a.callback, e.callback);
        }
    }

    #[tokio::test]
    async fn test_list_schedules_rate_limit_error() {
        let server = MockServer::start();
        let list_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/schedules")
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
        let result = client.list_schedules().await;
        list_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_list_schedules_invalid_response() {
        let server = MockServer::start();
        let list_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/schedules")
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
        let result = client.list_schedules().await;
        list_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_remove_schedule_success() {
        let server = MockServer::start();
        let schedule_id = "schedule123";
        let remove_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/schedules/{}", encode(schedule_id)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.remove_schedule(schedule_id).await;
        remove_mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_schedule_rate_limit_error() {
        let server = MockServer::start();
        let schedule_id = "schedule123";
        let rate_limit_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/schedules/{}", encode(schedule_id)))
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
        let result = client.remove_schedule(schedule_id).await;
        rate_limit_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_pause_schedule_success() {
        let server = MockServer::start();
        let schedule_id = "schedule123";
        let pause_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/schedules/{}/pause", encode(schedule_id)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.pause_schedule(schedule_id).await;
        pause_mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pause_schedule_rate_limit_error() {
        let server = MockServer::start();
        let schedule_id = "schedule123";
        let rate_limit_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/schedules/{}/pause", encode(schedule_id)))
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
        let result = client.pause_schedule(schedule_id).await;
        rate_limit_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_resume_schedule_success() {
        let server = MockServer::start();
        let schedule_id = "schedule123";
        let resume_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/schedules/{}/resume", encode(schedule_id)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.resume_schedule(schedule_id).await;
        resume_mock.assert();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_resume_schedule_rate_limit_error() {
        let server = MockServer::start();
        let schedule_id = "schedule123";
        let rate_limit_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/schedules/{}/resume", encode(schedule_id)))
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
        let result = client.resume_schedule(schedule_id).await;
        rate_limit_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }
}

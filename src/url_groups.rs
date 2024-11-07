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
        let request = self
            .client
            .get_request_builder(
                Method::POST,
                self.base_url
                    .join(&format!("/v2/topics/{}/endpoints", encode(url_group_name)))
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .json(&json!({
                "endpoints": endpoints,
            }));

        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn get_url_group(&self, url_group_name: &str) -> Result<UrlGroup, QstashError> {
        let request = self.client.get_request_builder(
            Method::GET,
            self.base_url
                .join(&format!("/v2/topics/{}", encode(url_group_name)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<UrlGroup>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }
    pub async fn list_url_groups(&self) -> Result<Vec<UrlGroup>, QstashError> {
        let request = self.client.get_request_builder(
            Method::GET,
            self.base_url
                .join("/v2/topics")
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<Vec<UrlGroup>>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn remove_endpoints(
        &self,
        url_group_name: &str,
        endpoints: Vec<Endpoint>,
    ) -> Result<(), QstashError> {
        let request = self
            .client
            .get_request_builder(
                Method::DELETE,
                self.base_url
                    .join(&format!("/v2/topics/{}/endpoints", encode(url_group_name)))
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .json(&json!({
                "endpoints": endpoints,
            }));

        self.client.send_request(request).await?;
        Ok(())
    }

    pub async fn remove_url_group(&self, url_group_name: &str) -> Result<(), QstashError> {
        let request = self.client.get_request_builder(
            Method::DELETE,
            self.base_url
                .join(&format!("/v2/topics/{}", encode(url_group_name)))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

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

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Endpoint {
    #[serde(skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(skip_serializing_if = "String::is_empty", default)]
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::{DELETE, GET, POST};
    use httpmock::MockServer;
    use reqwest::StatusCode;
    use reqwest::Url;
    use serde_json::json;

    /// Helper function to URL-encode a string.
    /// You can replace this with the actual `encode` function used in your implementation.
    fn encode(input: &str) -> String {
        urlencoding::encode(input).into_owned()
    }

    #[tokio::test]
    async fn test_upsert_url_group_endpoint_success() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Define the input parameters
        let url_group_name = "test-group";
        let endpoints = vec![
            Endpoint {
                name: "endpoint1".to_string(),
                url: "https://example.com/1".to_string(),
            },
            Endpoint {
                name: "endpoint2".to_string(),
                url: "https://example.com/2".to_string(),
            },
        ];

        // Create a mock for the POST /v2/topics/{url_group_name}/endpoints endpoint
        let upsert_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/topics/{}/endpoints", encode(url_group_name)))
                .header("Authorization", "Bearer test_api_key")
                .header("Content-Type", "application/json")
                .json_body_obj(&json!({ "endpoints": endpoints }));
            then.status(200);
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the upsert_url_group_endpoint method
        let result = client
            .upsert_url_group_endpoint(url_group_name, endpoints.clone())
            .await;

        // Assert that the mock was called exactly once
        upsert_mock.assert();

        // Assert that the result is Ok
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upsert_url_group_endpoint_rate_limit_error() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Define the input parameters
        let url_group_name = "test-group";
        let endpoints = vec![Endpoint {
            name: "endpoint1".to_string(),
            url: "https://example.com/1".to_string(),
        }];

        // Create a mock for the POST /v2/topics/{url_group_name}/endpoints endpoint that simulates a rate limit error
        let rate_limit_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/topics/{}/endpoints", encode(url_group_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600")
                .body("Rate limit exceeded");
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the upsert_url_group_endpoint method
        let result = client
            .upsert_url_group_endpoint(url_group_name, endpoints)
            .await;

        // Assert that the mock was called exactly once
        rate_limit_mock.assert();

        // Assert that the result is a rate limit error
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_upsert_url_group_endpoint_invalid_response() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Define the input parameters
        let url_group_name = "test-group";
        let endpoints = vec![Endpoint {
            name: "endpoint1".to_string(),
            url: "https://example.com/1".to_string(),
        }];

        // Create a mock for the POST /v2/topics/{url_group_name}/endpoints endpoint that returns invalid JSON
        let invalid_response_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/topics/{}/endpoints", encode(url_group_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .body("Invalid JSON");
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the upsert_url_group_endpoint method
        let result = client
            .upsert_url_group_endpoint(url_group_name, endpoints)
            .await;

        // Assert that the mock was called exactly once
        invalid_response_mock.assert();

        // Assert that the result is Ok since upsert_url_group_endpoint returns () on success
        // If the response body is invalid, send_request would fail before returning Ok(())
        assert!(result.is_err());
        if let Err(QstashError::RequestFailed(_)) = result {
            // Expected error due to invalid response handling in send_request
        } else {
            panic!("Expected a RequestFailed error due to invalid response");
        }
    }

    #[tokio::test]
    async fn test_get_url_group_success() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Define the input parameters and expected response
        let url_group_name = "test-group";
        let expected_url_group = UrlGroup {
            created_at: 1625097600,
            updated_at: 1625097700,
            name: url_group_name.to_string(),
            endpoints: vec![
                Endpoint {
                    name: "endpoint1".to_string(),
                    url: "https://example.com/1".to_string(),
                },
                Endpoint {
                    name: "endpoint2".to_string(),
                    url: "https://example.com/2".to_string(),
                },
            ],
        };

        // Create a mock for the GET /v2/topics/{url_group_name} endpoint
        let get_url_group_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/topics/{}", encode(url_group_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_url_group);
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the get_url_group method
        let result = client.get_url_group(url_group_name).await;

        // Assert that the mock was called exactly once
        get_url_group_mock.assert();

        // Assert that the result matches the expected UrlGroup
        assert!(result.is_ok());
        let url_group = result.unwrap();
        assert_eq!(url_group.name, expected_url_group.name);
        assert_eq!(
            url_group.endpoints.len(),
            expected_url_group.endpoints.len()
        );
        for (actual, expected) in url_group
            .endpoints
            .iter()
            .zip(expected_url_group.endpoints.iter())
        {
            assert_eq!(actual.name, expected.name);
            assert_eq!(actual.url, expected.url);
        }
    }

    #[tokio::test]
    async fn test_get_url_group_rate_limit_error() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Define the input parameters
        let url_group_name = "test-group";

        // Create a mock for the GET /v2/topics/{url_group_name} endpoint that simulates a rate limit error
        let rate_limit_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/topics/{}", encode(url_group_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600")
                .body("Rate limit exceeded");
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the get_url_group method
        let result = client.get_url_group(url_group_name).await;

        // Assert that the mock was called exactly once
        rate_limit_mock.assert();

        // Assert that the result is a rate limit error
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_get_url_group_invalid_response() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Define the input parameters
        let url_group_name = "test-group";

        // Create a mock for the GET /v2/topics/{url_group_name} endpoint that returns invalid JSON
        let invalid_response_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/topics/{}", encode(url_group_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .body("Invalid JSON");
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the get_url_group method
        let result = client.get_url_group(url_group_name).await;

        // Assert that the mock was called exactly once
        invalid_response_mock.assert();

        // Assert that the result is a response body parse error
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_list_url_groups_success() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Define the expected response
        let expected_url_groups = vec![
            UrlGroup {
                created_at: 1625097600,
                updated_at: 1625097600,
                name: "group1".to_string(),
                endpoints: vec![Endpoint {
                    name: "endpoint1".to_string(),
                    url: "https://example.com/1".to_string(),
                }],
            },
            UrlGroup {
                created_at: 1625097700,
                updated_at: 1625097700,
                name: "group2".to_string(),
                endpoints: vec![Endpoint {
                    name: "endpoint2".to_string(),
                    url: "https://example.com/2".to_string(),
                }],
            },
        ];

        // Create a mock for the GET /v2/topics endpoint
        let list_url_groups_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/topics")
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_url_groups);
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the list_url_groups method
        let result = client.list_url_groups().await;

        // Assert that the mock was called exactly once
        list_url_groups_mock.assert();

        // Assert that the result matches the expected UrlGroups
        assert!(result.is_ok());
        let url_groups = result.unwrap();
        assert_eq!(url_groups.len(), expected_url_groups.len());
        for (actual, expected) in url_groups.iter().zip(expected_url_groups.iter()) {
            assert_eq!(actual.name, expected.name);
            assert_eq!(actual.endpoints.len(), expected.endpoints.len());
            for (a, e) in actual.endpoints.iter().zip(expected.endpoints.iter()) {
                assert_eq!(a.name, e.name);
                assert_eq!(a.url, e.url);
            }
        }
    }

    #[tokio::test]
    async fn test_list_url_groups_rate_limit_error() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Create a mock for the GET /v2/topics endpoint that simulates a rate limit error
        let rate_limit_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/topics")
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600")
                .body("Rate limit exceeded");
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the list_url_groups method
        let result = client.list_url_groups().await;

        // Assert that the mock was called exactly once
        rate_limit_mock.assert();

        // Assert that the result is a rate limit error
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_list_url_groups_invalid_response() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Create a mock for the GET /v2/topics endpoint that returns invalid JSON
        let invalid_response_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/topics")
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .body("Invalid JSON");
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the list_url_groups method
        let result = client.list_url_groups().await;

        // Assert that the mock was called exactly once
        invalid_response_mock.assert();

        // Assert that the result is a response body parse error
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_remove_endpoints_success() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Define the input parameters
        let url_group_name = "test-group";
        let endpoints = vec![Endpoint {
            name: "endpoint1".to_string(),
            url: "https://example.com/1".to_string(),
        }];

        // Create a mock for the DELETE /v2/topics/{url_group_name}/endpoints endpoint
        let remove_endpoints_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/topics/{}/endpoints", encode(url_group_name)))
                .header("Authorization", "Bearer test_api_key")
                .header("Content-Type", "application/json")
                .json_body_obj(&json!({ "endpoints": endpoints }));
            then.status(200);
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the remove_endpoints method
        let result = client
            .remove_endpoints(url_group_name, endpoints.clone())
            .await;

        // Assert that the mock was called exactly once
        remove_endpoints_mock.assert();

        // Assert that the result is Ok
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_endpoints_rate_limit_error() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Define the input parameters
        let url_group_name = "test-group";
        let endpoints = vec![Endpoint {
            name: "endpoint1".to_string(),
            url: "https://example.com/1".to_string(),
        }];

        // Create a mock for the DELETE /v2/topics/{url_group_name}/endpoints endpoint that simulates a rate limit error
        let rate_limit_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/topics/{}/endpoints", encode(url_group_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600")
                .body("Rate limit exceeded");
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the remove_endpoints method
        let result = client.remove_endpoints(url_group_name, endpoints).await;

        // Assert that the mock was called exactly once
        rate_limit_mock.assert();

        // Assert that the result is a rate limit error
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_remove_endpoints_invalid_response() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Define the input parameters
        let url_group_name = "test-group";
        let endpoints = vec![Endpoint {
            name: "endpoint1".to_string(),
            url: "https://example.com/1".to_string(),
        }];

        // Create a mock for the DELETE /v2/topics/{url_group_name}/endpoints endpoint that returns invalid JSON
        let invalid_response_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/topics/{}/endpoints", encode(url_group_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .body("Invalid JSON");
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the remove_endpoints method
        let result = client.remove_endpoints(url_group_name, endpoints).await;

        // Assert that the mock was called exactly once
        invalid_response_mock.assert();

        // Assert that the result is Ok since remove_endpoints returns () on success
        // If the response body is invalid, send_request would fail before returning Ok(())
        assert!(result.is_err());
        if let Err(QstashError::RequestFailed(_)) = result {
            // Expected error due to invalid response handling in send_request
        } else {
            panic!("Expected a RequestFailed error due to invalid response");
        }
    }

    #[tokio::test]
    async fn test_remove_url_group_success() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Define the input parameters
        let url_group_name = "test-group";

        // Create a mock for the DELETE /v2/topics/{url_group_name} endpoint
        let remove_url_group_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/topics/{}", encode(url_group_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200);
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the remove_url_group method
        let result = client.remove_url_group(url_group_name).await;

        // Assert that the mock was called exactly once
        remove_url_group_mock.assert();

        // Assert that the result is Ok
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_url_group_rate_limit_error() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Define the input parameters
        let url_group_name = "test-group";

        // Create a mock for the DELETE /v2/topics/{url_group_name} endpoint that simulates a rate limit error
        let rate_limit_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/topics/{}", encode(url_group_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600")
                .body("Rate limit exceeded");
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the remove_url_group method
        let result = client.remove_url_group(url_group_name).await;

        // Assert that the mock was called exactly once
        rate_limit_mock.assert();

        // Assert that the result is a rate limit error
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_remove_url_group_invalid_response() {
        // Start a lightweight mock server
        let server = MockServer::start();

        // Define the input parameters
        let url_group_name = "test-group";

        // Create a mock for the DELETE /v2/topics/{url_group_name} endpoint that returns invalid JSON
        let invalid_response_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/topics/{}", encode(url_group_name)))
                .header("Authorization", "Bearer test_api_key");
            then.status(200)
                .header("Content-Type", "application/json")
                .body("Invalid JSON");
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the remove_url_group method
        let result = client.remove_url_group(url_group_name).await;

        // Assert that the mock was called exactly once
        invalid_response_mock.assert();

        // Assert that the result is Ok since remove_url_group returns () on success
        // If the response body is invalid, send_request would fail before returning Ok(())
        assert!(result.is_err());
        if let Err(QstashError::RequestFailed(_)) = result {
            // Expected error due to invalid response handling in send_request
        } else {
            panic!("Expected a RequestFailed error due to invalid response");
        }
    }
}

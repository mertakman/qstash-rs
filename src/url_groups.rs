use reqwest::Method;
use serde::{Deserialize, Serialize};

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
                    .join(&format!("/v2/topics/{}/endpoints", url_group_name))
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
                .join(&format!("/v2/topics/{}", url_group_name))
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
                    .join(&format!("/v2/topics/{}/endpoints", url_group_name))
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
                .join(&format!("/v2/topics/{}", url_group_name))
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        self.client.send_request(request).await?;

        Ok(())
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct UrlGroup {
    created_at: u64,
    updated_at: u64,
    name: String,
    endpoints: Vec<Endpoint>,
}

#[derive(Default, Serialize, Clone, Deserialize, Debug)]
#[serde(default)]
pub struct Endpoint {
    #[serde(skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
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

    #[tokio::test]
    async fn test_upsert_url_group_endpoint_success() {
        let server = MockServer::start();

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

        let upsert_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/topics/{}/endpoints", url_group_name))
                .header("Authorization", "Bearer test_api_key")
                .header("Content-Type", "application/json")
                .json_body_obj(&json!({ "endpoints": endpoints }));
            then.status(StatusCode::OK.as_u16());
        });

        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        let result = client
            .upsert_url_group_endpoint(url_group_name, endpoints.clone())
            .await;

        upsert_mock.assert();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upsert_url_group_endpoint_rate_limit_error() {
        let server = MockServer::start();

        let url_group_name = "test-group";
        let endpoints = vec![Endpoint {
            name: "endpoint1".to_string(),
            url: "https://example.com/1".to_string(),
        }];

        let rate_limit_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/v2/topics/{}/endpoints", url_group_name))
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
            .upsert_url_group_endpoint(url_group_name, endpoints)
            .await;

        rate_limit_mock.assert();

        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_get_url_group_success() {
        let server = MockServer::start();

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

        let get_url_group_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/topics/{}", url_group_name))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_url_group);
        });

        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        let result = client.get_url_group(url_group_name).await;

        get_url_group_mock.assert();

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
        let server = MockServer::start();

        let url_group_name = "test-group";

        let rate_limit_mock = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/v2/topics/{}", url_group_name))
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

        let result = client.get_url_group(url_group_name).await;

        rate_limit_mock.assert();

        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_list_url_groups_success() {
        let server = MockServer::start();

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

        let list_url_groups_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/topics")
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_url_groups);
        });

        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        let result = client.list_url_groups().await;

        list_url_groups_mock.assert();

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
        let server = MockServer::start();

        let rate_limit_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/topics")
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

        let result = client.list_url_groups().await;

        rate_limit_mock.assert();

        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_list_url_groups_invalid_response() {
        let server = MockServer::start();

        let invalid_response_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/topics")
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .body("Invalid JSON");
        });

        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        let result = client.list_url_groups().await;

        invalid_response_mock.assert();

        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_remove_endpoints_success() {
        let server = MockServer::start();

        let url_group_name = "test-group";
        let endpoints = vec![Endpoint {
            name: "endpoint1".to_string(),
            url: "https://example.com/1".to_string(),
        }];

        let remove_endpoints_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/topics/{}/endpoints", url_group_name))
                .header("Authorization", "Bearer test_api_key")
                .header("Content-Type", "application/json")
                .json_body_obj(&json!({ "endpoints": endpoints }));
            then.status(StatusCode::OK.as_u16());
        });

        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        let result = client
            .remove_endpoints(url_group_name, endpoints.clone())
            .await;

        remove_endpoints_mock.assert();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_endpoints_rate_limit_error() {
        let server = MockServer::start();

        let url_group_name = "test-group";
        let endpoints = vec![Endpoint {
            name: "endpoint1".to_string(),
            url: "https://example.com/1".to_string(),
        }];

        let rate_limit_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/topics/{}/endpoints", url_group_name))
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

        let result = client.remove_endpoints(url_group_name, endpoints).await;

        rate_limit_mock.assert();

        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_remove_url_group_success() {
        let server = MockServer::start();

        let url_group_name = "test-group";

        let remove_url_group_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/topics/{}", url_group_name))
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::OK.as_u16());
        });

        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        let result = client.remove_url_group(url_group_name).await;

        remove_url_group_mock.assert();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_url_group_rate_limit_error() {
        let server = MockServer::start();

        let url_group_name = "test-group";

        let rate_limit_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/v2/topics/{}", url_group_name))
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

        let result = client.remove_url_group(url_group_name).await;

        rate_limit_mock.assert();

        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }
}

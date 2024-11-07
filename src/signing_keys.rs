use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::client::QstashClient;
use crate::errors::QstashError;

impl QstashClient {
    pub async fn get_signing_keys(&self) -> Result<Signature, QstashError> {
        let request = self.client.get_request_builder(
            Method::GET,
            self.base_url
                .join("/v2/keys")
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<Signature>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }

    pub async fn rotate_signing_keys(&self) -> Result<Signature, QstashError> {
        let request = self.client.get_request_builder(
            Method::POST,
            self.base_url
                .join("/v2/keys/rotate")
                .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
        );

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<Signature>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Signature {
    current: String,
    next: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::QstashError;
    use httpmock::Method::GET;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use reqwest::StatusCode;
    use reqwest::Url;

    #[tokio::test]
    async fn test_get_signing_keys_success() {
        let server = MockServer::start();

        // Define the expected response
        let expected_signature = Signature {
            current: "current_key".to_string(),
            next: "next_key".to_string(),
        };

        // Create a mock for the GET /v2/keys endpoint
        let get_keys_mock = server.mock(|when, then| {
            when.method(GET).path("/v2/keys");
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_signature);
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the get_signing_keys method
        let result = client.get_signing_keys().await;

        // Assert that the mock was called exactly once
        get_keys_mock.assert();

        // Assert that the result matches the expected signature
        assert!(result.is_ok());
        let signature = result.unwrap();
        assert_eq!(signature.current, expected_signature.current);
        assert_eq!(signature.next, expected_signature.next);
    }

    #[tokio::test]
    async fn test_rotate_signing_keys_success() {
        let server = MockServer::start();

        // Define the expected response
        let expected_signature = Signature {
            current: "new_current_key".to_string(),
            next: "new_next_key".to_string(),
        };

        // Create a mock for the POST /v2/keys/rotate endpoint
        let rotate_keys_mock = server.mock(|when, then| {
            when.method(POST).path("/v2/keys/rotate");
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_signature);
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the rotate_signing_keys method
        let result = client.rotate_signing_keys().await;

        // Assert that the mock was called exactly once
        rotate_keys_mock.assert();

        // Assert that the result matches the expected signature
        assert!(result.is_ok());
        let signature = result.unwrap();
        assert_eq!(signature.current, expected_signature.current);
        assert_eq!(signature.next, expected_signature.next);
    }

    #[tokio::test]
    async fn test_get_signing_keys_rate_limit_error() {
        let server = MockServer::start();
        // Create a mock for the GET /v2/keys endpoint that simulates a rate limit error
        let rate_limit_mock = server.mock(|when, then| {
            when.method(GET).path("/v2/keys");
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

        // Call the get_signing_keys method
        let result = client.get_signing_keys().await;

        // Assert that the mock was called exactly once
        rate_limit_mock.assert();

        // Assert that the result is a rate limit error
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { .. })
        ));
    }

    #[tokio::test]
    async fn test_rotate_signing_keys_invalid_response() {
        let server = MockServer::start();

        // Create a mock for the POST /v2/keys/rotate endpoint that returns invalid JSON
        let invalid_response_mock = server.mock(|when, then| {
            when.method(POST).path("/v2/keys/rotate");
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .body("{ invalid json }");
        });

        // Build the QstashClient with the mock server's base URL
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        // Call the rotate_signing_keys method
        let result = client.rotate_signing_keys().await;

        // Assert that the mock was called exactly once
        invalid_response_mock.assert();

        // Assert that the result is a response body parse error
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }
}

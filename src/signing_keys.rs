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

        let expected_signature = Signature {
            current: "current_key".to_string(),
            next: "next_key".to_string(),
        };

        let get_keys_mock = server.mock(|when, then| {
            when.method(GET).path("/v2/keys");
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_signature);
        });

        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        let result = client.get_signing_keys().await;

        get_keys_mock.assert();

        assert!(result.is_ok());
        let signature = result.unwrap();
        assert_eq!(signature.current, expected_signature.current);
        assert_eq!(signature.next, expected_signature.next);
    }

    #[tokio::test]
    async fn test_rotate_signing_keys_success() {
        let server = MockServer::start();

        let expected_signature = Signature {
            current: "new_current_key".to_string(),
            next: "new_next_key".to_string(),
        };

        let rotate_keys_mock = server.mock(|when, then| {
            when.method(POST).path("/v2/keys/rotate");
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_signature);
        });

        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        let result = client.rotate_signing_keys().await;

        rotate_keys_mock.assert();

        assert!(result.is_ok());
        let signature = result.unwrap();
        assert_eq!(signature.current, expected_signature.current);
        assert_eq!(signature.next, expected_signature.next);
    }

    #[tokio::test]
    async fn test_get_signing_keys_rate_limit_error() {
        let server = MockServer::start();
        let rate_limit_mock = server.mock(|when, then| {
            when.method(GET).path("/v2/keys");
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

        let result = client.get_signing_keys().await;

        rate_limit_mock.assert();

        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { .. })
        ));
    }

    #[tokio::test]
    async fn test_rotate_signing_keys_invalid_response() {
        let server = MockServer::start();

        let invalid_response_mock = server.mock(|when, then| {
            when.method(POST).path("/v2/keys/rotate");
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .body("{ invalid json }");
        });

        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");

        let result = client.rotate_signing_keys().await;

        invalid_response_mock.assert();

        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }
}

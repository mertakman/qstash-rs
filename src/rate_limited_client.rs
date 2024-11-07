use reqwest::{header::HeaderMap, Client, Method, RequestBuilder, Response, StatusCode, Url};

use crate::errors::QstashError;

/// Struct for handling rate-limited requests.
pub struct RateLimitedClient {
    http_client: Client,
    api_key: String,
}

impl RateLimitedClient {
    pub fn new(api_key: String) -> Self {
        RateLimitedClient {
            http_client: Client::new(),
            api_key,
        }
    }

    pub fn get_request_builder(&self, method: Method, url: Url) -> RequestBuilder {
        self.http_client.request(method, url)
    }

    /// Sends a request and returns immediately on any rate limit or error without retrying.
    pub async fn send_request(&self, request: RequestBuilder) -> Result<Response, QstashError> {
        let response = request
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(QstashError::RequestFailed)?;

        // Check if the response has an error status and handle rate limits.
        if let Err(err) = response.error_for_status_ref() {
            if let Some(status) = err.status() {
                if status == StatusCode::TOO_MANY_REQUESTS {
                    // Return the appropriate rate limit error based on headers.
                    return Err(handle_rate_limit_error(&response));
                }
            }
            return Err(QstashError::RequestFailed(err));
        }

        Ok(response)
    }
}

/// Parses the response headers to determine which rate limit was exceeded.
pub fn handle_rate_limit_error(response: &Response) -> QstashError {
    let headers = response.headers();

    if headers.contains_key("RateLimit-Limit") {
        // Daily Rate Limit Exceeded
        let reset = parse_reset_time(headers, "RateLimit-Reset");
        return QstashError::DailyRateLimitExceeded { reset };
    } else if headers.contains_key("Burst-RateLimit-Limit") {
        // Burst Rate Limit Exceeded
        let reset = parse_reset_time(headers, "Burst-RateLimit-Reset");
        return QstashError::BurstRateLimitExceeded { reset };
    } else if headers.contains_key("x-ratelimit-limit-requests") {
        // Chat-based Rate Limit Exceeded
        let reset_requests = parse_reset_time(headers, "x-ratelimit-reset-requests");
        let reset_tokens = parse_reset_time(headers, "x-ratelimit-reset-tokens");
        return QstashError::ChatRateLimitExceeded {
            reset_requests,
            reset_tokens,
        };
    }
    QstashError::UnspecifiedRateLimitExceeded
}

fn parse_reset_time(headers: &HeaderMap, header_name: &str) -> u64 {
    headers
        .get(header_name)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use reqwest::Method;

    #[tokio::test]
    async fn test_send_request_success() {
        // Arrange
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200);
        });

        let client = RateLimitedClient::new("test_api_key".to_string());
        let url = Url::parse(&format!("{}/test", &server.base_url())).unwrap();
        let request_builder = client.get_request_builder(Method::GET, url);

        // Act
        let result = client.send_request(request_builder).await;

        // Assert
        assert!(result.is_ok());
        mock.assert();
    }

    #[tokio::test]
    async fn test_send_request_daily_rate_limit_exceeded() {
        // Arrange
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(429)
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "3600");
        });

        let client = RateLimitedClient::new("test_api_key".to_string());
        let url = Url::parse(&format!("{}/test", &server.base_url())).unwrap();
        let request_builder = client.get_request_builder(Method::GET, url);

        // Act
        let result = client.send_request(request_builder).await;

        // Assert
        match result {
            Err(QstashError::DailyRateLimitExceeded { reset }) => assert_eq!(reset, 3600),
            _ => panic!("Expected DailyRateLimitExceeded error"),
        }
        mock.assert();
    }

    #[tokio::test]
    async fn test_send_request_burst_rate_limit_exceeded() {
        // Arrange
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(429)
                .header("Burst-RateLimit-Limit", "100")
                .header("Burst-RateLimit-Reset", "60");
        });

        let client = RateLimitedClient::new("test_api_key".to_string());
        let url = Url::parse(&format!("{}/test", &server.base_url())).unwrap();
        let request_builder = client.get_request_builder(Method::GET, url);

        // Act
        let result = client.send_request(request_builder).await;

        // Assert
        match result {
            Err(QstashError::BurstRateLimitExceeded { reset }) => assert_eq!(reset, 60),
            _ => panic!("Expected BurstRateLimitExceeded error"),
        }
        mock.assert();
    }

    #[tokio::test]
    async fn test_send_request_chat_rate_limit_exceeded() {
        // Arrange
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(429)
                .header("x-ratelimit-limit-requests", "100")
                .header("x-ratelimit-reset-requests", "30")
                .header("x-ratelimit-reset-tokens", "45");
        });

        let client = RateLimitedClient::new("test_api_key".to_string());
        let url = Url::parse(&format!("{}/test", &server.base_url())).unwrap();
        let request_builder = client.get_request_builder(Method::GET, url);

        // Act
        let result = client.send_request(request_builder).await;

        // Assert
        match result {
            Err(QstashError::ChatRateLimitExceeded {
                reset_requests,
                reset_tokens,
            }) => {
                assert_eq!(reset_requests, 30);
                assert_eq!(reset_tokens, 45);
            }
            _ => panic!("Expected ChatRateLimitExceeded error"),
        }
        mock.assert();
    }

    #[tokio::test]
    async fn test_send_request_unspecified_rate_limit_exceeded() {
        // Arrange
        let server = MockServer::start_async().await;
        let mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(429);
        });

        let client = RateLimitedClient::new("test_api_key".to_string());
        let url = Url::parse(&format!("{}/test", &server.base_url())).unwrap();
        let request_builder = client.get_request_builder(Method::GET, url);

        // Act
        let result = client.send_request(request_builder).await;

        // Assert
        match result {
            Err(QstashError::UnspecifiedRateLimitExceeded) => (),
            _ => panic!("Expected UnspecifiedRateLimitExceeded error"),
        }
        mock.assert();
    }
}

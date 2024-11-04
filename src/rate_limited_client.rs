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

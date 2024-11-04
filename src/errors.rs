use std::error;
use std::fmt;

#[derive(Debug)]
pub enum QstashError {
    InvalidApiKey,
    InvalidBaseUrl(String),
    InvalidRequestUrl(String),
    RequestFailed(reqwest::Error),
    ResponseBodyParseError(serde_json::Error),
    DailyRateLimitExceeded {
        reset: u64,
    },
    BurstRateLimitExceeded {
        reset: u64,
    },
    ChatRateLimitExceeded {
        reset_requests: u64,
        reset_tokens: u64,
    },
    UnspecifiedRateLimitExceeded,
}

impl fmt::Display for QstashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QstashError::InvalidApiKey => write!(f, "Invalid API key"),
            QstashError::InvalidBaseUrl(url) => write!(f, "Invalid base URL: {}", url),
            QstashError::InvalidRequestUrl(url) => write!(f, "Invalid request URL: {}", url),
            QstashError::RequestFailed(err) => write!(f, "Request failed: {}", err),
            QstashError::ResponseBodyParseError(err) => {
                write!(f, "Failed to parse response body: {}", err)
            }
            QstashError::DailyRateLimitExceeded { reset } => {
                write!(f, "Daily rate limit exceeded. Retry after: {}", reset)
            }
            QstashError::BurstRateLimitExceeded { reset } => {
                write!(f, "Burst rate limit exceeded. Retry after: {}", reset)
            }
            QstashError::ChatRateLimitExceeded {
                reset_requests,
                reset_tokens,
            } => write!(
                f,
                "Chat rate limit exceeded. Retry after requests reset: {}, tokens reset: {}",
                reset_requests, reset_tokens
            ),
            QstashError::UnspecifiedRateLimitExceeded => {
                write!(f, "Rate limit exceeded, but no details provided")
            }
        }
    }
}

impl error::Error for QstashError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            QstashError::InvalidApiKey => None,
            QstashError::InvalidBaseUrl(_) => None,
            QstashError::InvalidRequestUrl(_) => None,
            QstashError::RequestFailed(err) => Some(err),
            QstashError::ResponseBodyParseError(err) => Some(err),
            QstashError::DailyRateLimitExceeded { .. } => None,
            QstashError::BurstRateLimitExceeded { .. } => None,
            QstashError::ChatRateLimitExceeded { .. } => None,
            QstashError::UnspecifiedRateLimitExceeded => None,
        }
    }
}

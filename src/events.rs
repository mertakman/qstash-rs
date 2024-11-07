use reqwest::Method;

use crate::client::QstashClient;
use crate::errors::QstashError;
use crate::events_types::{EventsRequest, EventsResponse};

impl QstashClient {
    pub async fn list_events(&self, request: EventsRequest) -> Result<EventsResponse, QstashError> {
        let request = self
            .client
            .get_request_builder(
                Method::GET,
                self.base_url
                    .join("/v2/events")
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .query(&request.to_query_params());

        let response = self
            .client
            .send_request(request)
            .await?
            .json::<EventsResponse>()
            .await
            .map_err(|e| QstashError::ResponseBodyParseError(e))?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use crate::client::QstashClient;
    use crate::errors::QstashError;
    use crate::events_types::Event;
    use crate::events_types::EventState;
    use crate::events_types::EventsRequest;
    use crate::events_types::EventsResponse;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use reqwest::StatusCode;
    use reqwest::Url;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_list_events_success() {
        let server = MockServer::start();
        let events_request = EventsRequest {
            cursor: Some("next_page".to_string()),
            message_id: Some("msg123".to_string()),
            state: Some("active".to_string()),
            url: Some("http://example.com".to_string()),
            topic_name: Some("topic1".to_string()),
            schedule_id: Some("sched1".to_string()),
            queue_name: Some("queue1".to_string()),
            from_date: Some(1234567890),
            to_date: Some(1234567899),
            count: Some(100),
            order: Some("desc".to_string()),
        };
        let expected_response = EventsResponse {
            cursor: Some("next_page_cursor".to_string()),
            events: vec![Event {
                time: 1645564800000,
                message_id: "msg_123".to_string(),
                header: HashMap::from([
                    (
                        "Content-Type".to_string(),
                        vec!["application/json".to_string()],
                    ),
                    (
                        "X-Custom".to_string(),
                        vec!["value1".to_string(), "value2".to_string()],
                    ),
                ]),
                body: b"Hello World".to_vec(),
                state: EventState::Delivered,
                error: None,
                next_delivery_time: Some(1645564900000),
                url: Some("https://example.com".to_string()),
                topic_name: Some("notifications".to_string()),
                endpoint_name: Some(1),
                schedule_id: Some("sched1".to_string()),
                queue_name: Some("queue1".to_string()),
            }],
        };
        let list_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/events")
                .query_param("cursor", "next_page")
                .query_param("messageId", "msg123")
                .query_param("state", "active")
                .query_param("url", "http://example.com")
                .query_param("topicName", "topic1")
                .query_param("scheduleId", "sched1")
                .query_param("queueName", "queue1")
                .query_param("fromDate", "1234567890")
                .query_param("toDate", "1234567899")
                .query_param("count", "100")
                .query_param("order", "desc")
                .header("Authorization", "Bearer test_api_key");
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_response);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.list_events(events_request).await;
        list_mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response, expected_response);
    }

    #[tokio::test]
    async fn test_list_events_rate_limit_error() {
        let server = MockServer::start();
        let events_request = EventsRequest {
            cursor: Some("next_page".to_string()),
            message_id: Some("msg123".to_string()),
            state: Some("active".to_string()),
            url: Some("http://example.com".to_string()),
            topic_name: Some("topic1".to_string()),
            schedule_id: Some("sched1".to_string()),
            queue_name: Some("queue1".to_string()),
            from_date: Some(1234567890),
            to_date: Some(1234567899),
            count: Some(100),
            order: Some("desc".to_string()),
        };
        let list_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/events")
                .query_param("cursor", "next_page")
                .query_param("messageId", "msg123")
                .query_param("state", "active")
                .query_param("url", "http://example.com")
                .query_param("topicName", "topic1")
                .query_param("scheduleId", "sched1")
                .query_param("queueName", "queue1")
                .query_param("fromDate", "1234567890")
                .query_param("toDate", "1234567899")
                .query_param("count", "100")
                .query_param("order", "desc")
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
        let result = client.list_events(events_request).await;
        list_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_list_events_invalid_response() {
        let server = MockServer::start();
        let events_request = EventsRequest {
            cursor: Some("next_page".to_string()),
            message_id: Some("msg123".to_string()),
            state: Some("active".to_string()),
            url: Some("http://example.com".to_string()),
            topic_name: Some("topic1".to_string()),
            schedule_id: Some("sched1".to_string()),
            queue_name: Some("queue1".to_string()),
            from_date: Some(1234567890),
            to_date: Some(1234567899),
            count: Some(100),
            order: Some("desc".to_string()),
        };
        let list_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/v2/events")
                .query_param("cursor", "next_page")
                .query_param("messageId", "msg123")
                .query_param("state", "active")
                .query_param("url", "http://example.com")
                .query_param("topicName", "topic1")
                .query_param("scheduleId", "sched1")
                .query_param("queueName", "queue1")
                .query_param("fromDate", "1234567890")
                .query_param("toDate", "1234567899")
                .query_param("count", "100")
                .query_param("order", "desc")
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
        let result = client.list_events(events_request).await;
        list_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }
}

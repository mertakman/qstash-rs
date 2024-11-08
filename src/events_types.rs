use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct EventsRequest {
    /// By providing a cursor you can paginate through all of the events.
    pub cursor: Option<String>,
    /// Filter events by message id.
    pub message_id: Option<String>,
    /// Filter events by state.
    pub state: Option<String>,
    /// Filter events by URL.
    pub url: Option<String>,
    /// Filter events by topic name.
    pub topic_name: Option<String>,
    /// Filter events by schedule id.
    pub schedule_id: Option<String>,
    /// Filter events by queue name.
    pub queue_name: Option<String>,
    /// Filter events by starting date, in milliseconds (Unix timestamp). This is inclusive.
    pub from_date: Option<i64>,
    /// Filter events by ending date, in milliseconds (Unix timestamp). This is inclusive.
    pub to_date: Option<i64>,
    /// The number of events to return. Default and max is 1000.
    pub count: Option<i32>,
    /// The sorting order of events by timestamp. Valid values are “earliestFirst” and “latestFirst”. The default is “latestFirst”.
    pub order: Option<String>,
}

impl EventsRequest {
    pub fn new() -> Self {
        EventsRequest::default()
    }

    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params: Vec<(String, String)> = Vec::new();

        // Only add parameters that are Some
        if let Some(ref cursor) = self.cursor {
            params.push(("cursor".to_string(), cursor.to_string()));
        }
        if let Some(ref message_id) = self.message_id {
            params.push(("messageId".to_string(), message_id.to_string()));
        }
        if let Some(ref state) = self.state {
            params.push(("state".to_string(), state.to_string()));
        }
        if let Some(ref url) = self.url {
            params.push(("url".to_string(), url.to_string()));
        }
        if let Some(ref topic_name) = self.topic_name {
            params.push(("topicName".to_string(), topic_name.to_string()));
        }
        if let Some(ref schedule_id) = self.schedule_id {
            params.push(("scheduleId".to_string(), schedule_id.to_string()));
        }
        if let Some(ref queue_name) = self.queue_name {
            params.push(("queueName".to_string(), queue_name.to_string()));
        }
        if let Some(from_date) = self.from_date {
            params.push(("fromDate".to_string(), from_date.to_string()));
        }
        if let Some(to_date) = self.to_date {
            params.push(("toDate".to_string(), to_date.to_string()));
        }
        if let Some(count) = self.count {
            params.push(("count".to_string(), count.to_string()));
        }
        if let Some(ref order) = self.order {
            params.push(("order".to_string(), order.to_string()));
        }

        params
    }
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct EventsResponse {
    /// A cursor which you can use in subsequent requests to paginate through all events. If no cursor is returned, you have reached the end of the events.
    pub cursor: Option<String>,
    pub events: Vec<Event>,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Event {
    /// Timestamp of this log entry, in milliseconds
    pub time: i64,
    /// The associated message id
    pub message_id: String,
    /// The headers of the message.
    pub header: HashMap<String, Vec<String>>,
    #[serde(
        serialize_with = "serialize_body",
        deserialize_with = "deserialize_body"
    )]
    /// Body of the message.
    pub body: Vec<u8>,
    /// The current state of the message at this point in time.
    pub state: EventState,

    // Optional fields
    /// An explanation what went wrong
    pub error: Option<String>,
    /// The next scheduled time of the message. (Unix timestamp in milliseconds)
    pub next_delivery_time: Option<i64>,
    /// The destination url
    pub url: Option<String>,
    /// The name of the URL Group (topic) if this message was sent through a topic
    pub topic_name: Option<String>,
    /// The name of the endpoint if this message was sent through a URL Group
    pub endpoint_name: Option<i32>,
    /// The scheduleId of the message if the message is triggered by a schedule
    pub schedule_id: Option<String>,
    /// The name of the queue if this message is enqueued on a queue
    pub queue_name: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventState {
    #[default]
    None,
    /// The message has been accepted and stored in QStash
    Created,
    /// The task is currently being processed by a worker.
    Active,
    /// The task has been scheduled to retry.
    Retry,
    /// The execution threw an error and the task is waiting to be retried or failed.
    Error,
    /// The message was successfully delivered.
    Delivered,
    /// The task has errored too many times or encountered an error that it cannot recover from.
    Failed,
    /// The cancel request from the user is recorded.
    CancelRequested,
    /// The cancel request from the user is honored.
    Cancelled,
}

fn serialize_body<S>(body: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&base64::encode(body))
}

fn deserialize_body<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| base64::decode(string.as_bytes()).map_err(Error::custom))
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_request_is_empty() {
        let request = EventsRequest::new();
        let params = request.to_query_params();
        assert!(params.is_empty());
    }

    #[test]
    fn test_single_parameter() {
        let mut request = EventsRequest::new();
        request.cursor = Some("next_page".to_string());

        let params = request.to_query_params();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], ("cursor".to_string(), "next_page".to_string()));
    }

    #[test]
    fn test_multiple_parameters() {
        let mut request = EventsRequest::new();
        request.cursor = Some("next_page".to_string());
        request.message_id = Some("msg123".to_string());
        request.state = Some("active".to_string());

        let params = request.to_query_params();
        assert_eq!(params.len(), 3);
        assert!(params.contains(&("cursor".to_string(), "next_page".to_string())));
        assert!(params.contains(&("messageId".to_string(), "msg123".to_string())));
        assert!(params.contains(&("state".to_string(), "active".to_string())));
    }

    #[test]
    fn test_numeric_parameters() {
        let mut request = EventsRequest::new();
        request.from_date = Some(1234567890);
        request.to_date = Some(1234567899);
        request.count = Some(100);

        let params = request.to_query_params();
        assert_eq!(params.len(), 3);
        assert!(params.contains(&("fromDate".to_string(), "1234567890".to_string())));
        assert!(params.contains(&("toDate".to_string(), "1234567899".to_string())));
        assert!(params.contains(&("count".to_string(), "100".to_string())));
    }

    #[test]
    fn test_all_parameters() {
        let request = EventsRequest {
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

        let params = request.to_query_params();
        assert_eq!(params.len(), 11);
        assert!(params.contains(&("cursor".to_string(), "next_page".to_string())));
        assert!(params.contains(&("messageId".to_string(), "msg123".to_string())));
        assert!(params.contains(&("state".to_string(), "active".to_string())));
        assert!(params.contains(&("url".to_string(), "http://example.com".to_string())));
        assert!(params.contains(&("topicName".to_string(), "topic1".to_string())));
        assert!(params.contains(&("scheduleId".to_string(), "sched1".to_string())));
        assert!(params.contains(&("queueName".to_string(), "queue1".to_string())));
        assert!(params.contains(&("fromDate".to_string(), "1234567890".to_string())));
        assert!(params.contains(&("toDate".to_string(), "1234567899".to_string())));
        assert!(params.contains(&("count".to_string(), "100".to_string())));
        assert!(params.contains(&("order".to_string(), "desc".to_string())));
    }

    #[test]
    fn test_partial_parameters() {
        let mut request = EventsRequest::new();
        request.topic_name = Some("topic1".to_string());
        request.count = Some(50);
        request.order = Some("asc".to_string());

        let params = request.to_query_params();
        assert_eq!(params.len(), 3);
        assert!(params.contains(&("topicName".to_string(), "topic1".to_string())));
        assert!(params.contains(&("count".to_string(), "50".to_string())));
        assert!(params.contains(&("order".to_string(), "asc".to_string())));
    }

    #[test]
    fn test_default_implementation() {
        let request = EventsRequest::default();
        let params = request.to_query_params();
        assert!(params.is_empty());
    }

    #[test]
    fn test_deserialize_response() {
        let json_str = r#"{
            "cursor": "next_page",
            "events": [{
                "time": 1645564800000,
                "messageId": "msg_123",
                "header": {
                    "content-type": ["application/json"],
                    "x-custom": ["value1", "value2"]
                },
                "body": "SGVsbG8gV29ybGQ=",
                "state": "DELIVERED",
                "url": "https://example.com",
                "topicName": "notifications",
                "nextDeliveryTime": 1645564900000
            }]
        }"#;

        let response: EventsResponse = serde_json::from_str(json_str).unwrap();
        assert_eq!(response.cursor, Some("next_page".to_string()));
        assert_eq!(response.events.len(), 1);

        let event = &response.events[0];
        assert_eq!(event.message_id, "msg_123");
        assert_eq!(event.state, EventState::Delivered);
        assert_eq!(event.body, b"Hello World");
        assert!(matches!(event.url, Some(ref url) if url == "https://example.com"));
    }

    #[test]
    fn test_deserialize_minimal_event() {
        let json_str = json!({
            "time": 1645564800000 as i64,
            "messageId": "msg_123",
            "header": {},
            "body": "SGVsbG8=",
            "state": "CREATED"
        })
        .to_string();

        let event: Event = serde_json::from_str(&json_str).unwrap();
        assert_eq!(event.message_id, "msg_123");
        assert_eq!(event.state, EventState::Created);
        assert_eq!(event.body, b"Hello");
        assert!(event.url.is_none());
        assert!(event.topic_name.is_none());
    }

    #[test]
    fn test_serialize_deserialize() {
        let event = Event {
            time: 1645564800000,
            message_id: "msg_123".to_string(),
            header: HashMap::new(),
            body: b"Hello World".to_vec(),
            state: EventState::Created,
            error: None,
            next_delivery_time: None,
            url: None,
            topic_name: None,
            endpoint_name: None,
            schedule_id: None,
            queue_name: None,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.body, b"Hello World");
    }

    #[test]
    fn test_binary_data() {
        // Test with non-UTF8 binary data
        let binary_data = vec![0x00, 0xFF, 0x42, 0x13, 0x37];
        let event = Event {
            time: 1645564800000,
            message_id: "msg_123".to_string(),
            header: HashMap::new(),
            body: binary_data.clone(),
            state: EventState::Created,
            error: None,
            next_delivery_time: None,
            url: None,
            topic_name: None,
            endpoint_name: None,
            schedule_id: None,
            queue_name: None,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.body, binary_data);
    }
}

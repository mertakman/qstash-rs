use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::de::{self};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub message_id: String,
    pub topic_name: String,
    pub url: String,
    pub method: String,
    pub header: HashMap<String, Vec<String>>,
    pub body: String,
    pub created_at: i64,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    pub message_id: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub deduplicated: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageResponseResult {
    SingleResponse(MessageResponse),
    MultipleResponses(Vec<MessageResponse>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchEntry {
    pub destination: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue: Option<String>,
    #[serde(
        serialize_with = "serialize_headers",
        deserialize_with = "deserialize_headers"
    )]
    pub headers: HeaderMap,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

// Custom serializer for HeaderMap
fn serialize_headers<S>(headers: &HeaderMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let headers_map: std::collections::HashMap<String, String> = headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
        .collect();
    headers_map.serialize(serializer)
}

// Custom deserializer for HeaderMap
fn deserialize_headers<'de, D>(deserializer: D) -> Result<HeaderMap, D::Error>
where
    D: Deserializer<'de>,
{
    let headers_map: std::collections::HashMap<String, String> =
        Deserialize::deserialize(deserializer)?;
    let mut headers = HeaderMap::new();

    for (k, v) in headers_map {
        let name = HeaderName::from_bytes(k.as_bytes()).map_err(de::Error::custom)?;
        let value = HeaderValue::from_str(&v).map_err(de::Error::custom)?;
        headers.insert(name, value);
    }
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_message() {
        let single_json = r#"
            {
                "messageId": "msd_1234",
                "url": "https://www.example.com"
            }
        "#;

        let messages: MessageResponseResult = serde_json::from_str(single_json).unwrap();
        match messages {
            MessageResponseResult::SingleResponse(message) => {
                assert_eq!(message.message_id, "msd_1234");
                assert_eq!(message.url, Some("https://www.example.com".into()));
            }
            _ => panic!("Expected a single message"),
        }
    }
    #[test]
    fn test_multiple_messages() {
        let multiple_json = r#"
            [
                {
                    "messageId": "msd_1234",
                    "url": "https://www.example.com"
                },
                {
                    "messageId": "msd_5678",
                    "url": "https://www.somewhere-else.com",
                    "deduplicated": true
                }
            ]
        "#;

        let messages: MessageResponseResult = serde_json::from_str(multiple_json).unwrap();
        match messages {
            MessageResponseResult::MultipleResponses(messages) => {
                assert_eq!(messages.len(), 2);
                assert_eq!(messages[0].message_id, "msd_1234");
                assert_eq!(messages[0].url, Some("https://www.example.com".into()));
                assert_eq!(messages[1].message_id, "msd_5678");
                assert_eq!(messages[1].url, Some("https://www.somewhere-else.com".into()));
                assert_eq!(messages[1].deduplicated, Some(true));
            }
            _ => panic!("Expected multiple messages"),
        }
    }

    #[test]
    fn test_batch_entry_serialization() {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("X-Custom-Header", HeaderValue::from_static("custom_value"));

        let batch_entry = BatchEntry {
            destination: "some_destination".to_string(),
            queue: Some("some_queue".to_string()),
            headers,
            body: Some("This is a body".to_string()),
        };

        let serialized =
            serde_json::to_string(&batch_entry).expect("Failed to serialize BatchEntry");

        let deserialized: BatchEntry =
            serde_json::from_str(&serialized).expect("Failed to deserialize BatchEntry");

        assert_eq!(batch_entry.destination, deserialized.destination);
        assert_eq!(batch_entry.queue, deserialized.queue);
        assert_eq!(batch_entry.body, deserialized.body);

        for (key, value) in batch_entry.headers.iter() {
            let deserialized_value = deserialized
                .headers
                .get(key)
                .expect("Key not found in deserialized headers");
            assert_eq!(value, deserialized_value);
        }
    }
}

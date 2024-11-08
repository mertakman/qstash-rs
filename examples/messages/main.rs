use http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    HeaderValue,
};
use qstash_rs::{client::QstashClient, message_types::MessageResponseResult};
use reqwest::header::HeaderMap;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("QSTASH_API_KEY").expect("QSTASH_API_KEY not set");

    let destination = "https://www.example.com".to_string();
    let body = "{\"message\": \"Hello, world!\"}".to_string();

    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("Upstash-Method", HeaderValue::from_static("POST"));
    headers.insert("Upstash-Delay", HeaderValue::from_static("0s"));
    headers.insert("Upstash-Retries", HeaderValue::from_static("3"));
    headers.insert(
        "Upstash-Forward-Custom-Header",
        HeaderValue::from_static("custom-value"),
    );

    let client: QstashClient = QstashClient::builder().api_key(&api_key).build()?;
    println!(
        "Starting to publish message to the destination: {}",
        destination
    );

    let publish_message_resp = client
        .publish_message(&destination, headers, body.clone().into())
        .await?;
    println!("Message published successfully to {}!", destination);
    println!(
        "Response from publishing message: {:#?}",
        publish_message_resp
    );

    let message_id = match publish_message_resp {
        MessageResponseResult::URLResponse(url_response) => url_response.message_id,
        MessageResponseResult::URLGroupResponse(_) => {
            panic!("Response is not of type URLResponse");
        }
    };

    println!("Retrieving message with id: {}", message_id);
    let get_message_resp = client.get_message(&message_id).await?;
    println!("Successfully retrieved message with id: {}.", message_id);
    println!("Retrieved message details: {:#?}", get_message_resp);

    println!("Initiating cancellation of message with id: {}", message_id);
    client.cancel_message(&message_id).await?;
    println!(
        "Message with id: {} has been cancelled successfully.",
        message_id
    );

    Ok(())
}

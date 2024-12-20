use std::env;

use qstash_rs::{client::QstashClient, events_types::EventsRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("QSTASH_API_KEY").expect("QSTASH_API_KEY not set");
    let client = QstashClient::builder().api_key(&api_key).build()?;

    println!("Starting the process to retrieve the event list.");
    let resp = client.list_events(EventsRequest::default()).await?;
    println!("Successfully retrieved the events list.");
    println!("Retrieved Events: {:#?}", resp);

    Ok(())
}

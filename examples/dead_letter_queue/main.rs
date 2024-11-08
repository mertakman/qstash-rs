use std::env;

use qstash_rs::{client::QstashClient, dead_letter_queue::DlqQueryParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("QSTASH_API_KEY").expect("QSTASH_API_KEY not set");

    let client = QstashClient::builder().api_key(&api_key).build()?;

    let dlq_messages_list = client.dlq_list_messages(DlqQueryParams::default()).await?;
    println!("{:#?}", dlq_messages_list);

    let message_list = vec![];
    let deleted_messages_list = client.dlq_delete_messages(message_list).await?;
    println!("{:#?}", deleted_messages_list);

    let dlq_message_id = "";
    client.dlq_delete_message(dlq_message_id).await?;

    Ok(())
}

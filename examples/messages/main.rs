use qstash_rs::client::QstashClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("QSTASH_API_KEY").expect("QSTASH_API_KEY not set");

    let client = QstashClient::builder().api_key(&api_key).build().unwrap();

    let message_ids = [
        "msg_id_0".to_string(),
        "msg_id_1".to_string(),
        "msg_id_2".to_string(),
        "msg_id_3".to_string(),
    ];

    let client = QstashClient::builder().api_key(" ").build().unwrap();

    // client.publish_message(destination, headers, body).await?;
    // client.enqueue_message(destination, queue_name, headers, body)
    // client.batch_messages(destination, queue_name, headers, body)
    // client.get_message(destination, queue_name, headers, body)
    // client.cancel_message(&"msg_id_0".to_string()).await?;
    // client.bulk_cancel_messages(message_ids[1..].to_vec()).await?;
    //
    Ok(())
}

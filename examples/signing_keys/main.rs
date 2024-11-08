use std::env;

use qstash_rs::client::QstashClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("QSTASH_API_KEY").expect("QSTASH_API_KEY not set");

    let client = QstashClient::builder().api_key(&api_key).build().unwrap();

    let signing_keys = client.get_signing_keys().await?;
    println!("Signing keys retrieved successfully");
    println!("{:#?}", signing_keys);

    println!("Rotating signing keys");
    let new_signing_keys = client.rotate_signing_keys().await?;
    println!("Signing keys rotated successfully");
    println!("{:#?}", new_signing_keys);

    Ok(())
}

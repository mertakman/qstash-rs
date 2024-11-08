use std::env;

use qstash_rs::client::QstashClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let api_key = env::var("QSTASH_API_KEY").expect("QSTASH_API_KEY not set");
    let api_key = "eyJVc2VySUQiOiJiMmIxZWIyYS1iNDUzLTQ3NzQtYTllMC0xOTYzMmJhMmE2YmQiLCJQYXNzd29yZCI6ImJhYTNmZmEyODNlYjQzMTdhOGJjY2ViYmIzYTliMGI2In0=".to_string();
    let client = QstashClient::builder().api_key(&api_key).build().unwrap();

    Ok(())
}

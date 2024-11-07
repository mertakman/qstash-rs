use qstash_rs::client::QstashClient;
use reqwest::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = QstashClient::builder()
        .base_url(Url::parse("https://api.qstash.com").unwrap())?
        .api_key("my-api-key")
        .build()
        .unwrap();

    let _ = client;
    
    Ok(())
}

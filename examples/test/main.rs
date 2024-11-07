use qstash_rs::client::QstashClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = QstashClient::builder()
        .api_key("=")
        .build()
        .unwrap();

    client.get_message("http://asd.com").await?;

    Ok(())
}

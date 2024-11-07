#[tokio::main]
async fn main() {
    let client = QstashClient::builder()
        .base_url(Url::parse("https://api.qstash.com").unwrap())
        .api_key("my-api-key")
        .build()
        .unwrap();
}

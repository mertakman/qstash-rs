use std::io::ErrorKind;

use aws_config::BehaviorVersion;
use aws_sdk_secretsmanager::Client as SecretsManagerClient;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use qstash_rs::{client::QstashClient, errors::QstashError};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
struct Request {
    pub message_id: String,
}

#[derive(Serialize)]
struct Response {
    pub message: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let secret_name = "QSTASH_APIKEY"; // Replace with your secret name
    let region_provider =
        aws_config::meta::region::RegionProviderChain::default_provider().or_else("eu-west-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
    let client = SecretsManagerClient::new(&config);

    let secret_value = client
        .get_secret_value()
        .secret_id(secret_name)
        .send()
        .await?
        .secret_string
        .unwrap_or_default();

    let qstash_client = QstashClient::new(secret_value).map_err(|e| Error::from(e.to_string()))?;

    let app = App::new(qstash_client)?;
    let func = service_fn(|event: LambdaEvent<Request>| app.func_handler(event));
    lambda_runtime::run(func).await
}

struct App {
    qstash_client: QstashClient,
}

impl App {
    fn new(qstash_client: QstashClient) -> Result<Self, Error> {
        Ok(App { qstash_client })
    }

    async fn func_handler(&self, event: LambdaEvent<Request>) -> Result<Response, Error> {
        match self
            .qstash_client
            .get_message(&event.payload.message_id)
            .await
        {
            Ok(message) => {
                return Ok(Response {
                    message: json!({ "message": message }).to_string(),
                })
            }
            Err(e) => match e {
                QstashError::RequestFailed(err) => match err.status() {
                    Some(StatusCode::BAD_REQUEST) => Err(Box::new(std::io::Error::new(
                        ErrorKind::Other,
                        "Bad request",
                    ))),
                    Some(StatusCode::NOT_FOUND) => Err(Box::new(std::io::Error::new(
                        ErrorKind::NotFound,
                        "Message not found",
                    ))),
                    Some(StatusCode::INTERNAL_SERVER_ERROR) => Err(Box::new(std::io::Error::new(
                        ErrorKind::Other,
                        "Internal server error",
                    ))),
                    _ => Err(Box::new(std::io::Error::new(
                        ErrorKind::Other,
                        format!("Error getting message: {}", err),
                    ))),
                },
                _ => Err(Box::new(std::io::Error::new(
                    ErrorKind::Other,
                    format!("Error getting message: {}", e),
                ))),
            },
        }
    }
}

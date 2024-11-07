use reqwest::StatusCode;
use qstash_rs::{client::QstashClient, errors::QstashError};
use serde_json::json;
use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let api_key = match env.secret("QSTASH_APIKEY") {
        Ok(secret) => secret.to_string(),
        Err(e) => return Response::error(&format!("Error getting API key: {}", e), 500),
    };
    let qstash_client = match QstashClient::new(api_key) {
        Ok(client) => client,
        Err(e) => return Response::error(&format!("Error creating Qstash client: {}", e), 500),
    };

    if !matches!(req.method(), Method::Get) {
      return Response::error("Method Not Allowed", 405);
    }

    let url = &req.url()?;
    let message_id = match url.query_pairs().find(|(key, _)| key == "message_id") {
        Some((_, value)) => value.to_string(),
        None => return Response::error("Query parameter 'message_id' is missing", 400),
    };

    match qstash_client.get_message(&message_id).await {
        Ok(message) => {
            let json_message = json!({ "message": message });
            Response::from_json(&json_message)
        }
        Err(e) => match e {
            QstashError::RequestFailed(err) => match err.status() {
                Some(StatusCode::BAD_REQUEST) => return Response::error("Bad request", 400),
                Some(StatusCode::NOT_FOUND) => return Response::error("Message not found", 404),
                Some(StatusCode::INTERNAL_SERVER_ERROR) => {
                    return Response::error("Internal server error", 500)
                }
                _ => return Response::error(&format!("Error getting message: {}", err), 500),
            },
            _ => return Response::error(&format!("Error getting message: {}", e), 500),
        },
    }
}
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    pub name: String,
}

#[derive(Serialize)]
struct Response {
    pub message: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func_handler);
    lambda_runtime::run(func).await
}

async fn func_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let (request, _context) = event.into_parts();
    let message = format!("Hello, {}!", request.name);
    Ok(Response { message })
}

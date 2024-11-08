use std::env;

use qstash_rs::{
    client::QstashClient,
    llm_types::{ChatCompletionRequest, Message},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("QSTASH_API_KEY").expect("QSTASH_API_KEY not set");
    let client = QstashClient::builder().api_key(&api_key).build()?;

    let mut chat_completion_request = ChatCompletionRequest::default();
    chat_completion_request.model = "meta-llama/Meta-Llama-3-8B-Instruct".to_string();
    chat_completion_request.messages = vec![Message {
        role: "user".to_string(),
        content: "What is the capital of Türkiye?".to_string(),
        name: None,
    }];

    println!("Starting the process to create a chat completion.");
    let resp = client
        .create_chat_completion(chat_completion_request)
        .await?;
    println!("Retrieved response succesfully");
    println!("{:#?}", resp);
    Ok(())
}

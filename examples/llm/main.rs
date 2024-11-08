use std::env;
use tokio::time::{sleep, Duration};

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

    println!("Now lets get response as stream of tokens");

    let mut chat_completion_request = ChatCompletionRequest::default();
    chat_completion_request.model = "meta-llama/Meta-Llama-3-8B-Instruct".to_string();
    chat_completion_request.max_tokens = Some(40);
    chat_completion_request.messages = vec![Message {
        role: "user".to_string(),
        content: "If you were a comedian, what joke would you tell that would completely flop?"
            .to_string(),
        name: None,
    }];
    chat_completion_request.stream = Some(true);

    let resp = client
        .create_chat_completion(chat_completion_request)
        .await?;

    let mut streamed_response = match resp {
        qstash_rs::llm_types::ChatCompletionResponse::Stream(streamed_response) => streamed_response,
        qstash_rs::llm_types::ChatCompletionResponse::Direct(_) => {
            panic!("Response is not of type StreamedResponse");
        }
    };

    let mut message = String::new();
    while let Some(a) = streamed_response.get_next_stream_message().await? {
        if a.choices[0].delta.content.is_some() {
            message.push_str(&a.choices[0].delta.content.as_ref().unwrap());
            print!("\r{}", &message);
            sleep(Duration::from_millis(250)).await;
        }
    }
    
    Ok(())
}

use reqwest::Method;

use crate::client::QstashClient;
use crate::errors::QstashError;
use crate::llm_types::{
    ChatCompletionRequest, ChatCompletionResponse, DirectResponse, StreamResponse,
};

impl QstashClient {
    pub async fn create_chat_completion(
        &self,
        chat_completion_request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, QstashError> {
        let request = self
            .client
            .get_request_builder(
                Method::GET,
                self.base_url
                    .join("/llm/v1/chat/completions")
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .json(&chat_completion_request);

        if Some(true) == chat_completion_request.stream {
            println!("Request: {:?}", request);
        }
        let response = self.client.send_request(request).await?;

        match chat_completion_request.stream {
            Some(true) => {
                return Ok(ChatCompletionResponse::Stream(StreamResponse::new(
                    response,
                )));
            }
            _ => {
                let response = response
                    .json::<DirectResponse>()
                    .await
                    .map_err(|e| QstashError::ResponseBodyParseError(e))?;
                Ok(ChatCompletionResponse::Direct(response))
            }
        }
    }
}

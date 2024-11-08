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
                Method::POST,
                self.base_url
                    .join("/llm/v1/chat/completions")
                    .map_err(|e| QstashError::InvalidRequestUrl(e.to_string()))?,
            )
            .json(&chat_completion_request);

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

#[cfg(test)]
mod tests {
    use crate::client::QstashClient;
    use crate::errors::QstashError;
    use crate::llm_types::*;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use reqwest::StatusCode;
    use reqwest::Url;

    #[tokio::test]
    async fn test_chat_completion_direct_success() {
        let server = MockServer::start();
        let chat_request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            }],
            frequency_penalty: Some(0.5),
            logit_bias: None,
            logprobs: Some(true),
            top_logprobs: Some(5),
            max_tokens: Some(150),
            n: Some(1),
            presence_penalty: Some(0.3),
            response_format: Some(ResponseFormat {
                format_type: FormatType::Text,
            }),
            seed: Some(42),
            stop: Some(vec!["\n".to_string()]),
            stream: Some(false),
            temperature: Some(0.7),
            top_p: Some(0.9),
        };
        let expected_response = DirectResponse {
            id: "chatcmpl-123".to_string(),
            choices: vec![Choice {
                message: Message {
                    role: "assistant".to_string(),
                    content: "Hello! How can I assist you today?".to_string(),
                    name: None,
                },
                finish_reason: Some("stop".to_string()),
                stop_reason: Some("\n".to_string()),
                index: 0,
                logprobs: Some(LogProbs {
                    content: vec![TokenInfo {
                        token: "Hello".to_string(),
                        logprob: -0.5,
                        bytes: Some(vec![72, 101, 108, 108, 111]),
                        top_logprobs: vec![TopLogProb {
                            token: "Hello".to_string(),
                            logprob: -0.5,
                            bytes: Some(vec![72, 101, 108, 108, 111]),
                        }],
                    }],
                }),
            }],
            created: 1625097600,
            model: "gpt-4".to_string(),
            system_fingerprint: "fingerprint123".to_string(),
            object: "chat.completion".to_string(),
            usage: Usage {
                completion_tokens: 10,
                prompt_tokens: 5,
                total_tokens: 15,
            },
        };
        let direct_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/llm/v1/chat/completions")
                .header("Authorization", "Bearer test_api_key")
                .json_body_obj(&chat_request);
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .json_body_obj(&expected_response);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.create_chat_completion(chat_request).await;
        direct_mock.assert();
        assert!(result.is_ok());
        let response = result.unwrap();
        match response {
            ChatCompletionResponse::Direct(response) => {
                assert_eq!(response, expected_response);
            }
            _ => panic!("Expected DirectResponse"),
        }
    }

    #[tokio::test]
    async fn test_chat_completion_direct_rate_limit_error() {
        let server = MockServer::start();
        let chat_request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            }],
            frequency_penalty: Some(0.5),
            logit_bias: None,
            logprobs: Some(true),
            top_logprobs: Some(5),
            max_tokens: Some(150),
            n: Some(1),
            presence_penalty: Some(0.3),
            response_format: Some(ResponseFormat {
                format_type: FormatType::Text,
            }),
            seed: Some(42),
            stop: Some(vec!["\n".to_string()]),
            stream: Some(false),
            temperature: Some(0.7),
            top_p: Some(0.9),
        };
        let direct_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/llm/v1/chat/completions")
                .header("Authorization", "Bearer test_api_key")
                .json_body_obj(&chat_request);
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600")
                .body("Rate limit exceeded");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.create_chat_completion(chat_request).await;
        direct_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_chat_completion_direct_invalid_response() {
        let server = MockServer::start();
        let chat_request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            }],
            frequency_penalty: Some(0.5),
            logit_bias: None,
            logprobs: Some(true),
            top_logprobs: Some(5),
            max_tokens: Some(150),
            n: Some(1),
            presence_penalty: Some(0.3),
            response_format: Some(ResponseFormat {
                format_type: FormatType::Text,
            }),
            seed: Some(42),
            stop: Some(vec!["\n".to_string()]),
            stream: Some(false),
            temperature: Some(0.7),
            top_p: Some(0.9),
        };
        let direct_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/llm/v1/chat/completions")
                .header("Authorization", "Bearer test_api_key")
                .json_body_obj(&chat_request);
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .body("Invalid JSON");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.create_chat_completion(chat_request).await;
        direct_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::ResponseBodyParseError(_))
        ));
    }

    #[tokio::test]
    async fn test_chat_completion_stream_success() {
        let server = MockServer::start();
        let chat_request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            }],
            frequency_penalty: Some(0.5),
            logit_bias: None,
            logprobs: Some(true),
            top_logprobs: Some(5),
            max_tokens: Some(150),
            n: Some(1),
            presence_penalty: Some(0.3),
            response_format: Some(ResponseFormat {
                format_type: FormatType::Text,
            }),
            seed: Some(42),
            stop: Some(vec!["\n".to_string()]),
            stream: Some(true),
            temperature: Some(0.7),
            top_p: Some(0.9),
        };
        let stream_response = "\
        {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1625097600,\"model\":\"gpt-4\",\"choices\":[{\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null,\"index\":0,\"logprobs\":null}]}\n\n\
        {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1625097600,\"model\":\"gpt-4\",\"choices\":[{\"delta\":{\"content\":\" World\"},\"finish_reason\":null,\"index\":0,\"logprobs\":null}]}\n\n\
        [DONE]";
        let stream_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/llm/v1/chat/completions")
                .header("Authorization", "Bearer test_api_key")
                .json_body_obj(&chat_request);
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .body(stream_response);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let response = client.create_chat_completion(chat_request).await.unwrap();
        let mut stream = match response {
            ChatCompletionResponse::Stream(stream_response) => stream_response,
            _ => panic!("Expected StreamResponse"),
        };
        let mut messages = Vec::new();
        while let Some(message) = stream.get_next_stream_message().await.unwrap() {
            messages.push(message);
        }
        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[0].choices[0].delta.content,
            Some("Hello".to_string())
        );
        assert_eq!(
            messages[1].choices[0].delta.content,
            Some(" World".to_string())
        );

        drop(stream_mock);
    }

    #[tokio::test]
    async fn test_chat_completion_stream_rate_limit_error() {
        let server = MockServer::start();
        let chat_request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            }],
            frequency_penalty: Some(0.5),
            logit_bias: None,
            logprobs: Some(true),
            top_logprobs: Some(5),
            max_tokens: Some(150),
            n: Some(1),
            presence_penalty: Some(0.3),
            response_format: Some(ResponseFormat {
                format_type: FormatType::Text,
            }),
            seed: Some(42),
            stop: Some(vec!["\n".to_string()]),
            stream: Some(true),
            temperature: Some(0.7),
            top_p: Some(0.9),
        };
        let stream_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/llm/v1/chat/completions")
                .header("Authorization", "Bearer test_api_key")
                .json_body_obj(&chat_request);
            then.status(StatusCode::TOO_MANY_REQUESTS.as_u16())
                .header("RateLimit-Limit", "1000")
                .header("RateLimit-Reset", "1625097600")
                .body("Rate limit exceeded");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.create_chat_completion(chat_request).await;
        stream_mock.assert();
        assert!(matches!(
            result,
            Err(QstashError::DailyRateLimitExceeded { reset: 1625097600 })
        ));
    }

    #[tokio::test]
    async fn test_chat_completion_stream_invalid_response() {
        let server = MockServer::start();
        let chat_request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            }],
            frequency_penalty: Some(0.5),
            logit_bias: None,
            logprobs: Some(true),
            top_logprobs: Some(5),
            max_tokens: Some(150),
            n: Some(1),
            presence_penalty: Some(0.3),
            response_format: Some(ResponseFormat {
                format_type: FormatType::Text,
            }),
            seed: Some(42),
            stop: Some(vec!["\n".to_string()]),
            stream: Some(true),
            temperature: Some(0.7),
            top_p: Some(0.9),
        };
        let stream_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/llm/v1/chat/completions")
                .header("Authorization", "Bearer test_api_key")
                .json_body_obj(&chat_request);
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .body("Invalid JSON\n\n");
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let result = client.create_chat_completion(chat_request).await;
        stream_mock.assert();
        match result {
            Ok(ChatCompletionResponse::Stream(mut e)) => match e.get_next_stream_message().await {
                Ok(_) => panic!("Expected ResponseStreamParseError"),
                Err(QstashError::ResponseStreamParseError(_)) => {}
                _ => panic!("Expected ResponseStreamParseError"),
            },
            _ => panic!("Expected ResponseStreamParseError"),
        }
    }

    #[tokio::test]
    async fn test_stream_response_multiple_messages() {
        let server = MockServer::start();
        let chat_request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            }],
            frequency_penalty: Some(0.5),
            logit_bias: None,
            logprobs: Some(true),
            top_logprobs: Some(5),
            max_tokens: Some(150),
            n: Some(1),
            presence_penalty: Some(0.3),
            response_format: Some(ResponseFormat {
                format_type: FormatType::Text,
            }),
            seed: Some(42),
            stop: Some(vec!["\n".to_string()]),
            stream: Some(true),
            temperature: Some(0.7),
            top_p: Some(0.9),
        };
        let stream_response = "\
        {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1625097600,\"model\":\"gpt-4\",\"choices\":[{\"delta\":{\"content\":\"Hello\"},\"finish_reason\":null,\"index\":0,\"logprobs\":null}]}\n\n\
        {\"id\":\"chatcmpl-123\",\"object\":\"chat.completion.chunk\",\"created\":1625097600,\"model\":\"gpt-4\",\"choices\":[{\"delta\":{\"content\":\" World\"},\"finish_reason\":null,\"index\":0,\"logprobs\":null}]}\n\n\
        [DONE]";
        let stream_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/llm/v1/chat/completions")
                .header("Authorization", "Bearer test_api_key")
                .json_body_obj(&chat_request);
            then.status(StatusCode::OK.as_u16())
                .header("Content-Type", "application/json")
                .body(stream_response);
        });
        let client = QstashClient::builder()
            .base_url(Url::parse(&server.base_url()).unwrap())
            .unwrap()
            .api_key("test_api_key")
            .build()
            .expect("Failed to build QstashClient");
        let response = client.create_chat_completion(chat_request).await.unwrap();
        let mut stream = match response {
            ChatCompletionResponse::Stream(stream_response) => stream_response,
            _ => panic!("Expected StreamResponse"),
        };
        let mut messages = Vec::new();
        while let Some(message) = stream.get_next_stream_message().await.unwrap() {
            messages.push(message);
        }
        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages[0].choices[0].delta.content,
            Some("Hello".to_string())
        );
        assert_eq!(
            messages[1].choices[0].delta.content,
            Some(" World".to_string())
        );

        drop(stream_mock);
    }
}

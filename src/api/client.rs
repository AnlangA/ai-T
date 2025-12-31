use crate::error::{Result, TranslationError};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    pub thinking: Option<ThinkingConfig>,
}

#[derive(Debug, Serialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type")]
    pub thinking_type: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct StreamChunk {
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    pub object: String,
    #[allow(dead_code)]
    pub created: u64,
    #[allow(dead_code)]
    pub model: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
pub struct StreamChoice {
    #[allow(dead_code)]
    pub index: u32,
    pub delta: Delta,
    #[allow(dead_code)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Delta {
    #[allow(dead_code)]
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl ApiClient {
    pub fn new(api_key: String) -> Self {
        tracing::debug!("Creating new API client");
        if api_key.is_empty() {
            tracing::warn!("API key is empty");
        }
        
        ApiClient {
            client: Client::new(),
            api_key,
            base_url: "https://api.z.ai/api/coding/paas/v4".to_string(),
        }
    }

    pub async fn stream_chat(
        &self,
        messages: Vec<ChatMessage>,
    ) -> tokio::sync::mpsc::UnboundedReceiver<Result<String>> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let request = ChatRequest {
            model: "glm-4.7".to_string(),
            messages,
            stream: true,
            thinking: Some(ThinkingConfig {
                thinking_type: "enabled".to_string(),
            }),
        };

        let url = format!("{}/chat/completions", self.base_url);
        let api_key = self.api_key.clone();

        tracing::info!("Starting streaming chat request to: {}", url);

        tokio::spawn(async move {
            let client = Client::new();
            match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    tracing::debug!("Received response with status: {}", status);
                    
                    if !status.is_success() {
                        tracing::error!("API returned error status: {}", status);
                        let _ = tx.send(Err(TranslationError::ApiError(format!("API error: {}", status))));
                        return;
                    }

                    let mut stream = response.bytes_stream();
                    let mut buffer = Vec::new();
                    let mut chunk_count = 0;

                    use futures_util::StreamExt;

                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                buffer.extend_from_slice(&chunk);
                                let data = String::from_utf8_lossy(&buffer);

                                let lines: Vec<&str> = data.lines().collect();

                                for (i, line) in lines.iter().enumerate() {
                                    if i == lines.len() - 1 && !line.starts_with("data: ") {
                                        continue;
                                    }

                                    if let Some(json_str) = line.strip_prefix("data: ") {
                                        if json_str.trim() == "[DONE]" {
                                            tracing::debug!("Stream completed with {} chunks", chunk_count);
                                            let _ = tx.send(Ok(String::new()));
                                            return;
                                        }

                                        match serde_json::from_str::<StreamChunk>(json_str) {
                                            Ok(chunk) => {
                                                if let Some(choice) = chunk.choices.first()
                                                    && let Some(content) = &choice.delta.content {
                                                        chunk_count += 1;
                                                        tracing::trace!("Received chunk {}: {} bytes", chunk_count, content.len());
                                                        let _ = tx.send(Ok(content.clone()));
                                                    }
                                            }
                                            Err(e) => {
                                                tracing::warn!("Failed to parse stream chunk: {}", e);
                                            }
                                        }
                                    }
                                }

                                buffer.clear();
                            }
                            Err(e) => {
                                tracing::error!("Stream error: {}", e);
                                let _ = tx.send(Err(TranslationError::StreamError(format!("Stream error: {}", e))));
                                return;
                            }
                        }
                    }
                    tracing::debug!("Stream ended naturally");
                    let _ = tx.send(Ok(String::new()));
                }
                Err(e) => {
                    tracing::error!("Request error: {}", e);
                    let _ = tx.send(Err(TranslationError::NetworkError(e)));
                }
            }
        });

        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_creation() {
        let client = ApiClient::new("test_key".to_string());
        assert_eq!(client.api_key, "test_key");
        assert!(client.base_url.contains("api.z.ai"));
    }

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: ChatMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(msg.role, deserialized.role);
        assert_eq!(msg.content, deserialized.content);
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: "glm-4.7".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "test".to_string(),
            }],
            stream: true,
            thinking: Some(ThinkingConfig {
                thinking_type: "enabled".to_string(),
            }),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("glm-4.7"));
        assert!(json.contains("user"));
        assert!(json.contains("test"));
    }

    #[test]
    fn test_stream_chunk_deserialization() {
        let json = r#"{
            "id": "test",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "glm-4.7",
            "choices": [{
                "index": 0,
                "delta": {
                    "content": "Hello"
                },
                "finish_reason": null
            }]
        }"#;

        let chunk: StreamChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.id, "test");
        assert_eq!(chunk.choices.len(), 1);
        assert_eq!(chunk.choices[0].delta.content, Some("Hello".to_string()));
    }
}

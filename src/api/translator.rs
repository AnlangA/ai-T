use crate::api::client::{ApiClient, ChatMessage};
use crate::error::Result;

pub struct Translator {
    client: ApiClient,
}

impl Translator {
    pub fn new(api_key: String) -> Self {
        tracing::info!("Creating translator with API key");
        Translator {
            client: ApiClient::new(api_key),
        }
    }

    pub fn translate(
        &self,
        text: String,
        target_language: String,
    ) -> tokio::sync::mpsc::UnboundedReceiver<Result<String>> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tracing::info!(
            target_language = %target_language,
            text_length = text.len(),
            "Starting translation"
        );

        let prompt = format!(
            "Translate the following text to {}. Only output the translation, nothing else:\n\n{}",
            target_language, text
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        let client = self.client.clone();

        tokio::spawn(async move {
            let mut stream_rx = client.stream_chat(messages).await;
            
            while let Some(result) = stream_rx.recv().await {
                let _ = tx.send(result);
            }
            
            tracing::debug!("Translation stream completed");
        });

        rx
    }
}

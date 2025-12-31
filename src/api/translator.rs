//! Translation service using the Z.AI API.
//!
//! This module provides high-level translation functionality,
//! wrapping the API client with translation-specific logic.

use crate::api::client::{ApiClient, ChatMessage};
use crate::error::Result;

/// Translator service for handling translation requests.
pub struct Translator {
    client: ApiClient,
}

impl Translator {
    /// Creates a new translator with the given API key.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The Z.AI API key for authentication
    pub fn new(api_key: String) -> Self {
        tracing::info!("Creating translator with API key");
        Translator {
            client: ApiClient::new(api_key),
        }
    }

    /// Translates text to the target language using streaming.
    ///
    /// # Arguments
    ///
    /// * `text` - The source text to translate
    /// * `target_language` - The target language name
    ///
    /// # Returns
    ///
    /// A receiver channel that yields streaming chunks of the translation
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

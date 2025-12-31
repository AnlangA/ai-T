//! Translation service using the Z.AI API.
//!
//! This module provides high-level translation functionality,
//! wrapping the API client with translation-specific logic.

use crate::api::client::{ApiClient, ChatMessage};
use crate::error::Result;
use crate::utils::cache::TranslationCache;
use std::sync::Arc;

/// Translator service for handling translation requests.
pub struct Translator {
    client: ApiClient,
    cache: Arc<TranslationCache>,
}

impl Translator {
    /// Creates a new translator with the given API key and cache.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The Z.AI API key for authentication
    /// * `cache` - Translation cache for storing/retrieving translations
    pub fn new(api_key: String, cache: Arc<TranslationCache>) -> Self {
        tracing::info!("Creating translator with API key");
        Translator {
            client: ApiClient::new(api_key),
            cache,
        }
    }

    /// Translates text to the target language using streaming.
    /// Checks cache first before making API call.
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

        // Check cache first
        let cache = self.cache.clone();
        if let Some(cached_translation) = cache.get(&text, &target_language) {
            tracing::info!("Using cached translation");
            // Send cached result in chunks to simulate streaming
            let _ = tx.send(Ok(cached_translation));
            let _ = tx.send(Ok(String::new())); // Signal completion
            return rx;
        }

        let prompt = format!(
            "Translate the following text to {}. Only output the translation, nothing else:\n\n{}",
            target_language, text
        );

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }];

        let client = self.client.clone();
        let cache_for_storage = cache.clone();
        let text_for_cache = text.clone();
        let lang_for_cache = target_language.clone();

        tokio::spawn(async move {
            let mut stream_rx = client.stream_chat(messages).await;
            let mut full_translation = String::new();

            while let Some(result) = stream_rx.recv().await {
                match &result {
                    Ok(chunk) if !chunk.is_empty() => {
                        full_translation.push_str(chunk);
                    }
                    _ => {}
                }
                let _ = tx.send(result);
            }

            // Store in cache after successful translation
            if !full_translation.is_empty() {
                cache_for_storage.set(&text_for_cache, &lang_for_cache, full_translation);
            }

            tracing::debug!("Translation stream completed");
        });

        rx
    }
}

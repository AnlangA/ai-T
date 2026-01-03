//! Translation service using the Z.AI API.
//!
//! This module provides high-level translation functionality,
//! wrapping the API client with translation-specific logic.

use crate::api::client::{ApiClient, ChatMessage};
use crate::error::Result;
use crate::utils::cache::TranslationCache;
use std::sync::Arc;

/// Parses translation response to extract translation and optional keyword analysis
fn parse_translation_and_keywords(
    response: &str,
    enable_keyword_analysis: bool,
) -> (String, Option<String>) {
    if !enable_keyword_analysis {
        return (response.to_string(), None);
    }

    // Try to extract [Translation] and [Terminology] sections
    if response.contains("[Translation]") && response.contains("[Terminology]") {
        if let Some(translation_start) = response.find("[Translation]") {
            if let Some(terminology_start) = response.find("[Terminology]") {
                let translation = response
                    [translation_start + "[Translation]".len()..terminology_start]
                    .trim()
                    .to_string();
                let terminology = &response[terminology_start + "[Terminology]".len()..];
                let terminology = terminology.trim().to_string();

                return if terminology.is_empty() {
                    (translation, None)
                } else {
                    (translation, Some(terminology))
                };
            }
        }
    }

    // If keyword analysis was enabled but sections not found, treat entire response as translation
    (response.to_string(), None)
}

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
    /// * `enable_keyword_analysis` - Whether to enable keyword analysis
    ///
    /// # Returns
    ///
    /// A receiver channel that yields streaming chunks of the translation
    pub fn translate(
        &self,
        text: String,
        target_language: String,
        enable_keyword_analysis: bool,
    ) -> tokio::sync::mpsc::UnboundedReceiver<Result<String>> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tracing::info!(
            target_language = %target_language,
            text_length = text.len(),
            enable_keyword_analysis = %enable_keyword_analysis,
            "Starting translation"
        );

        // Check cache based on current keyword analysis setting
        // Cache key includes source text, target language, and keyword analysis bool
        let cache = self.cache.clone();
        if let Some((cached_translation, cached_keyword_analysis)) =
            cache.get(&text, &target_language, enable_keyword_analysis)
        {
            tracing::info!("Using cached translation");
            // Send cached result in chunks to simulate streaming
            let _ = tx.send(Ok(cached_translation));
            if let Some(keyword_analysis) = cached_keyword_analysis {
                let _ = tx.send(Ok(keyword_analysis));
            }
            let _ = tx.send(Ok(String::new())); // Signal completion
            return rx;
        }

        // Build messages with system prompt
        let mut messages = Vec::new();

        // Always use a system prompt for better translation quality
        let system_prompt = if enable_keyword_analysis {
            "You are a senior professional translator with deep expertise across multiple domains including technology, science, business, and academia.

## Core Task
Translate the provided text to the target language while maintaining accuracy, fluency, and contextual appropriateness.

## Translation Principles
1. Preserve the original tone, style, and emotional nuance
2. Ensure natural, idiomatic expressions in the target language
3. Maintain cultural sensitivity and contextual relevance
4. Use precise terminology appropriate for the domain
5. Handle ambiguous expressions based on context

## Term Analysis Requirements
After completing the translation, identify and explain technical terms, industry-specific vocabulary, or specialized concepts that may require clarification.

### What constitutes a term needing explanation:
- Technical jargon specific to a field (e.g., \"API\", \"microservices\", \"neural network\")
- Domain-specific acronyms and abbreviations
- Professional terminology not commonly understood by general audience
- Concepts with multiple meanings where clarification is needed
- Industry standards or protocols

### What to exclude:
- Commonly understood words and phrases
- Everyday language and general vocabulary
- Terms that have direct, unambiguous equivalents
- Cultural references that are globally recognized

### Output Format
[Translation]
<Your translation here>

[Terminology]
- [Term]: [Concise explanation in the target language, 1-2 sentences]

### Example Output
[Translation]
The API endpoint uses asynchronous processing to handle high-throughput requests.

[Terminology]
- API: Application Programming Interface, a software interface that allows applications to communicate with each other
- Asynchronous: A programming pattern where operations can execute independently without blocking the main thread
- Throughput: The rate at which a system processes requests or data

### Quality Standards
- Explanations must be clear, concise, and accurate
- Avoid unnecessary complexity in definitions
- List terms alphabetically
- Maximum 5-7 terms per text (most important ones only)"
        } else {
            "You are a professional translator with native-level proficiency in both source and target languages.

## Core Task
Translate the provided text to the target language with the highest possible accuracy and naturalness.

## Translation Guidelines

### Accuracy & Fidelity
- Capture the exact meaning without adding or omitting information
- Preserve technical accuracy, numbers, dates, and proper nouns
- Maintain the original informational content

### Fluency & Naturalness
- Use idiomatic expressions natural to the target language
- Ensure grammatical correctness and proper syntax
- Avoid literal translations that sound unnatural
- Adapt sentence structures when necessary for better flow

### Context & Tone
- Preserve the author's intended tone (formal, casual, technical, literary, etc.)
- Maintain emotional nuances and rhetorical devices
- Adapt cultural references appropriately for the target audience
- Consider the purpose of the text (instructional, conversational, academic, etc.)

### Quality Standards
- Read the entire text before translating for context understanding
- Ensure consistency in terminology throughout the translation
- Use appropriate register for the intended audience
- Review and refine for natural flow and clarity

## Output Format
Provide ONLY the translation with no additional commentary, explanations, or formatting markers."
        };

        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        });

        let user_prompt = format!(
            "Translate the following text to {}:\n\n{}",
            target_language, text
        );

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: user_prompt,
        });

        let client = self.client.clone();
        let cache_for_storage = cache.clone();
        let text_for_cache = text.clone();
        let lang_for_cache = target_language.clone();
        let enable_keyword_analysis_for_cache = enable_keyword_analysis;

        tokio::spawn(async move {
            let mut stream_rx = client.stream_chat(messages).await;
            let mut full_response = String::new();

            while let Some(result) = stream_rx.recv().await {
                match &result {
                    Ok(chunk) if !chunk.is_empty() => {
                        full_response.push_str(chunk);
                    }
                    _ => {}
                }
                let _ = tx.send(result);
            }

            // Store in cache after successful translation
            if !full_response.is_empty() {
                let (translation, keyword_analysis) = parse_translation_and_keywords(
                    &full_response,
                    enable_keyword_analysis_for_cache,
                );
                cache_for_storage.set(
                    &text_for_cache,
                    &lang_for_cache,
                    enable_keyword_analysis_for_cache,
                    translation,
                    keyword_analysis,
                );
            }

            tracing::debug!("Translation stream completed");
        });

        rx
    }
}

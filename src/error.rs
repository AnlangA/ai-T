//! Error types for the translation application.
//!
//! This module defines all error types that can occur during translation operations,
//! using the `thiserror` crate for automatic trait implementations.

use thiserror::Error;

/// Main error type for translation operations.
#[derive(Error, Debug)]
pub enum TranslationError {
    /// Error returned by the API service
    #[error("API error: {0}")]
    ApiError(String),

    /// Network-related errors from reqwest
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// JSON serialization/deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Streaming-related errors
    #[error("Stream error: {0}")]
    StreamError(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Invalid or missing API key
    #[error("Invalid API key")]
    InvalidApiKey,

    /// General translation failure
    #[error("Translation failed: {0}")]
    TranslationFailed(String),
}

/// Type alias for Results using `TranslationError`.
pub type Result<T> = std::result::Result<T, TranslationError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = TranslationError::ApiError("Test error".to_string());
        assert_eq!(err.to_string(), "API error: Test error");

        let err = TranslationError::InvalidApiKey;
        assert_eq!(err.to_string(), "Invalid API key");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let trans_err = TranslationError::from(io_err);
        assert!(matches!(trans_err, TranslationError::IoError(_)));
    }
}

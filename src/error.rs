use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranslationError {
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Stream error: {0}")]
    StreamError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid API key")]
    InvalidApiKey,
    
    #[error("Translation failed: {0}")]
    TranslationFailed(String),
}

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

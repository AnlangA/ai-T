//! Text-to-Speech (TTS) service module.
//!
//! This module provides text-to-speech functionality using the text2audio crate.
//! It handles conversion of text to audio files with configurable voice, speed, and volume.

use crate::lock_mutex;
use std::sync::{Arc, Mutex};
use text2audio::{Model, Text2Audio, Voice};

/// TTS configuration parameters
#[derive(Debug, Clone)]
pub struct TtsConfig {
    /// Voice selection for TTS
    pub voice: Voice,
    /// Speech speed multiplier (0.5 - 2.0)
    pub speed: f32,
    /// Audio volume level (0.0 - 10.0)
    pub volume: f32,
    /// Maximum segment length for text splitting
    pub max_segment_length: usize,
    /// Number of parallel conversions
    pub parallel: usize,
    /// Enable coding plan mode
    pub coding_plan: bool,
    /// Enable thinking mode
    pub enable_thinking: bool,
}

impl Default for TtsConfig {
    fn default() -> Self {
        TtsConfig {
            voice: Voice::Tongtong,
            speed: 1.0,
            volume: 1.0,
            max_segment_length: 800,
            parallel: 5,
            coding_plan: true,
            enable_thinking: true,
        }
    }
}

impl TtsConfig {
    /// Creates a new TtsConfig with custom parameters
    pub fn new(
        voice: Voice,
        speed: f32,
        volume: f32,
        coding_plan: bool,
        enable_thinking: bool,
    ) -> Self {
        TtsConfig {
            voice,
            speed: speed.clamp(0.5, 2.0),
            volume: volume.clamp(0.0, 10.0),
            coding_plan,
            enable_thinking,
            ..Default::default()
        }
    }
}

/// TTS conversion task status
#[derive(Debug, Clone, PartialEq)]
pub enum TtsStatus {
    #[allow(dead_code)]
    /// Task is idle, not started
    Idle,
    #[allow(dead_code)]
    /// Conversion in progress
    Converting,
    /// Conversion completed, audio ready
    Completed(String),
    /// Conversion failed
    Failed(String),
}

/// Text-to-Speech service
pub struct TtsService {
    api_key: String,
    config: Arc<Mutex<TtsConfig>>,
    runtime_handle: tokio::runtime::Handle,
}

impl TtsService {
    /// Creates a new TTS service with the given runtime handle
    pub fn new(api_key: String, runtime_handle: tokio::runtime::Handle) -> Self {
        TtsService {
            api_key,
            config: Arc::new(Mutex::new(TtsConfig::default())),
            runtime_handle,
        }
    }

    /// Updates the TTS configuration
    pub fn update_config(&self, config: TtsConfig) {
        *lock_mutex!(self.config) = config;
    }

    /// Gets current TTS configuration
    pub fn get_config(&self) -> TtsConfig {
        lock_mutex!(self.config).clone()
    }

    pub fn convert_async<F>(&self, text: &str, output_path: &str, callback: F)
    where
        F: FnOnce(TtsStatus) + Send + 'static,
    {
        if text.trim().is_empty() {
            callback(TtsStatus::Failed("Text is empty".to_string()));
            return;
        }

        let config = self.get_config();
        let api_key = self.api_key.clone();
        let text_owned = text.to_string();
        let output_path_owned = output_path.to_string();
        let runtime_handle = self.runtime_handle.clone();

        // Create the converter
        let converter = Text2Audio::new(&api_key)
            .with_model(Model::GLM4_7)
            .with_thinking(config.enable_thinking)
            .with_coding_plan(config.coding_plan)
            .with_voice(config.voice)
            .with_speed(config.speed)
            .with_volume(config.volume)
            .with_max_segment_length(config.max_segment_length)
            .with_parallel(config.parallel);

        // Use spawn_blocking to run blocking operation without creating new runtime
        runtime_handle.spawn_blocking(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            let status = match rt.block_on(converter.convert(&text_owned, &output_path_owned)) {
                Ok(()) => TtsStatus::Completed(output_path_owned),
                Err(e) => TtsStatus::Failed(format!("Conversion error: {}", e)),
            };

            callback(status);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_config_default() {
        let config = TtsConfig::default();
        assert_eq!(config.speed, 1.0);
        assert_eq!(config.volume, 1.0);
        assert!(config.coding_plan);
        assert!(config.enable_thinking);
    }

    #[test]
    fn test_tts_config_clamping() {
        let config = TtsConfig::new(Voice::Tongtong, 3.0, 15.0, true, false);
        assert_eq!(config.speed, 2.0); // Clamped to max
        assert_eq!(config.volume, 10.0); // Clamped to max
        assert!(!config.enable_thinking);
    }

    #[test]
    fn test_tts_status_variants() {
        let status_idle = TtsStatus::Idle;
        let status_converting = TtsStatus::Converting;
        let status_completed = TtsStatus::Completed("test.wav".to_string());
        let status_failed = TtsStatus::Failed("error".to_string());

        assert_eq!(status_idle, TtsStatus::Idle);
        assert_eq!(status_converting, TtsStatus::Converting);
        assert!(matches!(status_completed, TtsStatus::Completed(_)));
        assert!(matches!(status_failed, TtsStatus::Failed(_)));
    }
}

//! Application configuration management.
//!
//! This module handles loading, saving, and managing application configuration
//! including API keys, language preferences, and UI settings.

use egui::Id;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use text2audio::Voice;

/// Application configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Z.AI API key for authentication
    pub api_key: String,
    /// Target language for translation
    pub target_language: String,
    /// UI font size in pixels
    pub font_size: f32,
    /// Whether to use dark theme
    pub dark_theme: bool,
    /// TTS voice selection
    #[serde(default = "default_voice")]
    pub tts_voice: String,
    /// TTS speed multiplier
    #[serde(default = "default_speed")]
    pub tts_speed: f32,
    /// TTS volume level
    #[serde(default = "default_volume")]
    pub tts_volume: f32,
    /// Enable keyword analysis during translation
    #[serde(default = "default_keyword_analysis")]
    pub enable_keyword_analysis: bool,
}

/// Default keyword analysis setting
fn default_keyword_analysis() -> bool {
    false
}

/// Default voice name for TTS
fn default_voice() -> String {
    "Tongtong".to_string()
}

/// Default TTS speed
fn default_speed() -> f32 {
    1.0
}

/// Default TTS volume
fn default_volume() -> f32 {
    1.0
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            api_key: String::new(),
            target_language: "English".to_string(),
            font_size: 16.0,
            dark_theme: true,
            tts_voice: default_voice(),
            tts_speed: default_speed(),
            tts_volume: default_volume(),
            enable_keyword_analysis: default_keyword_analysis(),
        }
    }
}

impl AppConfig {
    /// Returns the path to the configuration file.
    pub fn config_path() -> PathBuf {
        PathBuf::from(".ai-translate-config.json")
    }

    /// Loads configuration from file, or returns default if file doesn't exist.
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists()
            && let Ok(content) = fs::read_to_string(&path)
            && let Ok(config) = serde_json::from_str::<AppConfig>(&content)
        {
            return config;
        }
        AppConfig::default()
    }

    /// Returns a list of supported target languages.
    pub fn get_supported_languages() -> Vec<&'static str> {
        vec![
            "English",
            "中文",
            "日本語",
            "한국어",
            "Français",
            "Deutsch",
            "Español",
            "Português",
            "Русский",
            "Italiano",
        ]
    }

    /// Returns a list of supported TTS voices.
    pub fn get_supported_voices() -> Vec<&'static str> {
        vec![
            "Tongtong", "Chuichui", "Xiaochen", "Jam", "Kazi", "Douji", "Luodo",
        ]
    }

    /// Converts a voice name string to Voice enum.
    pub fn parse_voice(voice_name: &str) -> Voice {
        match voice_name {
            "Tongtong" => Voice::Tongtong,
            "Chuichui" => Voice::Chuichui,
            "Xiaochen" => Voice::Xiaochen,
            "Jam" => Voice::Jam,
            "Kazi" => Voice::Kazi,
            "Douji" => Voice::Douji,
            "Luodo" => Voice::Luodo,
            _ => Voice::Tongtong,
        }
    }

    /// Returns the egui memory ID for this configuration.
    pub fn config_id() -> Id {
        Id::new("app_config")
    }

    /// Saves the configuration to egui's memory.
    pub fn save_to_memory(&self, ctx: &egui::Context) {
        ctx.memory_mut(|mem| {
            mem.data.insert_persisted(Self::config_id(), self.clone());
        });
    }

    /// Loads the configuration from egui's memory.
    pub fn load_from_memory(ctx: &egui::Context) -> Option<Self> {
        ctx.memory_mut(|mem| mem.data.get_persisted::<Self>(Self::config_id()))
    }

    /// Loads the configuration from memory or file, or returns default.
    pub fn load_or_default(ctx: &egui::Context) -> Self {
        Self::load_from_memory(ctx).unwrap_or_else(|| {
            let config = Self::load();
            config.save_to_memory(ctx);
            config
        })
    }

    /// Loads the configuration from eframe storage.
    pub fn from_storage(storage: &dyn eframe::Storage) -> Self {
        if let Some(json) = storage.get_string("app_config") {
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Saves the configuration to eframe storage.
    pub fn save_to_storage(&self, storage: &mut dyn eframe::Storage) {
        if let Ok(json) = serde_json::to_string(self) {
            storage.set_string("app_config", json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.api_key, "");
        assert_eq!(config.target_language, "English");
        assert_eq!(config.font_size, 16.0);
        assert!(config.dark_theme);
        assert!(!config.enable_keyword_analysis);
    }

    #[test]
    fn test_supported_languages() {
        let languages = AppConfig::get_supported_languages();
        assert!(languages.len() >= 10);
        assert!(languages.contains(&"English"));
        assert!(languages.contains(&"中文"));
        assert!(languages.contains(&"日本語"));
    }

    #[test]
    fn test_serialization() {
        let config = AppConfig {
            api_key: "test_key".to_string(),
            target_language: "中文".to_string(),
            font_size: 18.0,
            dark_theme: false,
            tts_voice: "Tongtong".to_string(),
            tts_speed: 1.0,
            tts_volume: 1.0,
            enable_keyword_analysis: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.api_key, deserialized.api_key);
        assert_eq!(config.target_language, deserialized.target_language);
        assert_eq!(config.font_size, deserialized.font_size);
        assert_eq!(config.dark_theme, deserialized.dark_theme);
        assert_eq!(config.tts_voice, deserialized.tts_voice);
        assert_eq!(config.tts_speed, deserialized.tts_speed);
        assert_eq!(config.tts_volume, deserialized.tts_volume);
        assert_eq!(config.enable_keyword_analysis, deserialized.enable_keyword_analysis);
    }
}

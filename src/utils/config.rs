use egui::Id;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api_key: String,
    pub target_language: String,
    pub font_size: f32,
    pub dark_theme: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            api_key: String::new(),
            target_language: "English".to_string(),
            font_size: 16.0,
            dark_theme: true,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        PathBuf::from(".ai-translate-config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists()
            && let Ok(content) = fs::read_to_string(&path)
                && let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    return config;
                }
        AppConfig::default()
    }

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

    pub fn config_id() -> Id {
        Id::new("app_config")
    }

    pub fn save_to_memory(&self, ctx: &egui::Context) {
        ctx.memory_mut(|mem| {
            mem.data.insert_persisted(Self::config_id(), self.clone());
        });
    }

    pub fn load_from_memory(ctx: &egui::Context) -> Option<Self> {
        ctx.memory_mut(|mem| {
            mem.data.get_persisted::<Self>(Self::config_id())
        })
    }

    pub fn load_or_default(ctx: &egui::Context) -> Self {
        Self::load_from_memory(ctx).unwrap_or_else(|| {
            let config = Self::load();
            config.save_to_memory(ctx);
            config
        })
    }
}

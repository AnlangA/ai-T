//! Translation cache for storing and retrieving previous translations.
//!
//! This module provides in-memory and persistent caching of translations
//! to avoid redundant API calls for previously translated text.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// A cache entry containing the translated text
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    translation: String,
    timestamp: i64,
}

/// Translation cache for storing translations in memory and on disk
pub struct TranslationCache {
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    cache_file: PathBuf,
}

impl TranslationCache {
    /// Creates a new translation cache
    ///
    /// # Arguments
    ///
    /// * `cache_file` - Path to the cache file for persistence
    pub fn new(cache_file: PathBuf) -> Self {
        tracing::info!("Initializing translation cache at: {:?}", cache_file);
        
        let cache = if cache_file.exists() {
            Self::load_from_file(&cache_file).unwrap_or_default()
        } else {
            HashMap::new()
        };

        TranslationCache {
            cache: Arc::new(Mutex::new(cache)),
            cache_file,
        }
    }

    /// Generates a cache key from source text and target language
    fn generate_key(source_text: &str, target_language: &str) -> String {
        format!("{}::{}", target_language, source_text)
    }

    /// Retrieves a translation from the cache
    ///
    /// # Arguments
    ///
    /// * `source_text` - The source text that was translated
    /// * `target_language` - The target language
    ///
    /// # Returns
    ///
    /// Some(translation) if found in cache, None otherwise
    pub fn get(&self, source_text: &str, target_language: &str) -> Option<String> {
        let key = Self::generate_key(source_text, target_language);
        let cache = self.cache.lock().unwrap();
        
        if let Some(entry) = cache.get(&key) {
            tracing::info!("Cache hit for key: {}", key.chars().take(50).collect::<String>());
            Some(entry.translation.clone())
        } else {
            tracing::debug!("Cache miss for key: {}", key.chars().take(50).collect::<String>());
            None
        }
    }

    /// Stores a translation in the cache
    ///
    /// # Arguments
    ///
    /// * `source_text` - The source text that was translated
    /// * `target_language` - The target language
    /// * `translation` - The translation result
    pub fn set(&self, source_text: &str, target_language: &str, translation: String) {
        let key = Self::generate_key(source_text, target_language);
        let entry = CacheEntry {
            translation,
            timestamp: chrono::Local::now().timestamp(),
        };

        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(key.clone(), entry);
            tracing::info!("Cached translation for key: {}", key.chars().take(50).collect::<String>());
        }

        // Save to disk asynchronously (best effort)
        if let Err(e) = self.save_to_file() {
            tracing::warn!("Failed to save cache to disk: {}", e);
        }
    }

    /// Loads cache from file
    fn load_from_file(path: &PathBuf) -> Result<HashMap<String, CacheEntry>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let cache: HashMap<String, CacheEntry> = serde_json::from_str(&content)?;
        tracing::info!("Loaded {} entries from cache file", cache.len());
        Ok(cache)
    }

    /// Saves cache to file
    fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cache = self.cache.lock().unwrap();
        let content = serde_json::to_string(&*cache)?;
        fs::write(&self.cache_file, content)?;
        tracing::debug!("Saved {} entries to cache file", cache.len());
        Ok(())
    }

    /// Clears all entries from the cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        tracing::info!("Cache cleared");
        
        // Remove cache file
        if self.cache_file.exists() {
            let _ = fs::remove_file(&self.cache_file);
        }
    }
}

impl Default for TranslationCache {
    fn default() -> Self {
        let cache_file = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ai-translate")
            .join("translation_cache.json");
        
        if let Some(parent) = cache_file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        Self::new(cache_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_cache_key_generation() {
        let key1 = TranslationCache::generate_key("hello", "Chinese");
        let key2 = TranslationCache::generate_key("hello", "Japanese");
        let key3 = TranslationCache::generate_key("world", "Chinese");
        
        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
        assert_eq!(key1, TranslationCache::generate_key("hello", "Chinese"));
    }

    #[test]
    fn test_cache_set_and_get() {
        let temp_dir = env::temp_dir();
        let cache_file = temp_dir.join("test_cache.json");
        let cache = TranslationCache::new(cache_file.clone());

        cache.set("hello", "Chinese", "你好".to_string());
        
        let result = cache.get("hello", "Chinese");
        assert_eq!(result, Some("你好".to_string()));
        
        let result = cache.get("hello", "Japanese");
        assert_eq!(result, None);

        // Cleanup
        let _ = fs::remove_file(cache_file);
    }

    #[test]
    fn test_cache_persistence() {
        let temp_dir = env::temp_dir();
        let cache_file = temp_dir.join("test_cache_persist.json");
        
        {
            let cache = TranslationCache::new(cache_file.clone());
            cache.set("test", "English", "test result".to_string());
        }
        
        {
            let cache = TranslationCache::new(cache_file.clone());
            let result = cache.get("test", "English");
            assert_eq!(result, Some("test result".to_string()));
        }

        // Cleanup
        let _ = fs::remove_file(cache_file);
    }

    #[test]
    fn test_cache_clear() {
        let temp_dir = env::temp_dir();
        let cache_file = temp_dir.join("test_cache_clear.json");
        let cache = TranslationCache::new(cache_file.clone());

        cache.set("test", "Chinese", "测试".to_string());
        assert!(cache.get("test", "Chinese").is_some());
        
        cache.clear();
        assert!(cache.get("test", "Chinese").is_none());

        // Cleanup
        let _ = fs::remove_file(cache_file);
    }
}

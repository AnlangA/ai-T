//! Translation cache for storing and retrieving previous translations.
//!
//! This module provides in-memory and persistent caching of translations
//! to avoid redundant API calls for previously translated text.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Helper macro to lock mutex with consistent error handling
macro_rules! lock_mutex {
    ($mutex:expr) => {
        $mutex.lock().expect("Mutex poisoned")
    };
}

/// A cache entry containing translated text and optional keyword analysis the translated text
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    translation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    keyword_analysis: Option<String>,
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

    /// Generates a cache key from source text, target language, and keyword analysis setting
    fn generate_key(
        source_text: &str,
        target_language: &str,
        enable_keyword_analysis: bool,
    ) -> String {
        format!(
            "{}::{}::{}",
            target_language, enable_keyword_analysis, source_text
        )
    }

    /// Retrieves a translation from the cache
    ///
    /// # Arguments
    ///
    /// * `source_text` - The source text that was translated
    /// * `target_language` - The target language
    /// * `enable_keyword_analysis` - Whether keyword analysis was enabled
    ///
    /// # Returns
    ///
    /// Some((translation, keyword_analysis)) if found in cache, None otherwise
    pub fn get(
        &self,
        source_text: &str,
        target_language: &str,
        enable_keyword_analysis: bool,
    ) -> Option<(String, Option<String>)> {
        let key = Self::generate_key(source_text, target_language, enable_keyword_analysis);
        let cache = lock_mutex!(self.cache);

        if let Some(entry) = cache.get(&key) {
            tracing::info!(
                "Cache hit for key: {}",
                key.chars().take(50).collect::<String>()
            );
            Some((entry.translation.clone(), entry.keyword_analysis.clone()))
        } else {
            tracing::debug!(
                "Cache miss for key: {}",
                key.chars().take(50).collect::<String>()
            );
            None
        }
    }

    /// Stores a translation in the cache
    ///
    /// # Arguments
    ///
    /// * `source_text` - The source text that was translated
    /// * `target_language` - The target language
    /// * `enable_keyword_analysis` - Whether keyword analysis was enabled
    /// * `translation` - The translation result
    /// * `keyword_analysis` - Optional keyword analysis result
    pub fn set(
        &self,
        source_text: &str,
        target_language: &str,
        enable_keyword_analysis: bool,
        translation: String,
        keyword_analysis: Option<String>,
    ) {
        const MAX_CACHE_SIZE: usize = 1000;
        const CLEANUP_SIZE: usize = 100;

        let key = Self::generate_key(source_text, target_language, enable_keyword_analysis);
        let entry = CacheEntry {
            translation,
            keyword_analysis,
            timestamp: chrono::Utc::now().timestamp(),
        };

        {
            let mut cache = lock_mutex!(self.cache);
            cache.insert(key.clone(), entry);
            tracing::info!(
                "Cached translation for key: {}",
                key.chars().take(50).collect::<String>()
            );

            // Check if cache size exceeds limit
            if cache.len() > MAX_CACHE_SIZE {
                tracing::info!(
                    "Cache size {} exceeds limit {}, removing oldest {} entries",
                    cache.len(),
                    MAX_CACHE_SIZE,
                    CLEANUP_SIZE
                );

                // Collect all entries with their keys and timestamps
                let mut entries: Vec<(String, i64)> = cache
                    .iter()
                    .map(|(k, v)| (k.clone(), v.timestamp))
                    .collect();

                // Sort by timestamp (oldest first)
                entries.sort_by(|a, b| a.1.cmp(&b.1));

                // Remove oldest CLEANUP_SIZE entries
                for (key_to_remove, _) in entries.iter().take(CLEANUP_SIZE) {
                    cache.remove(key_to_remove);
                }

                tracing::info!("Cache cleanup completed, new size: {}", cache.len());
            }
        }

        // Save to disk asynchronously (best effort)
        if let Err(e) = self.save_to_file() {
            tracing::warn!("Failed to save cache to disk: {}", e);
        }
    }

    /// Loads cache from file
    fn load_from_file(
        path: &std::path::Path,
    ) -> Result<HashMap<String, CacheEntry>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let cache: HashMap<String, CacheEntry> = serde_json::from_str(&content)?;
        tracing::info!("Loaded {} entries from cache file", cache.len());
        Ok(cache)
    }

    /// Saves cache to file
    fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cache = self.cache.lock().expect("Cache mutex poisoned");
        let content = serde_json::to_string(&*cache)?;
        fs::write(&self.cache_file, content)?;
        tracing::debug!("Saved {} entries to cache file", cache.len());
        Ok(())
    }

    /// Clears all entries from the cache
    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut cache = self.cache.lock().expect("Cache mutex poisoned");
        cache.clear();
        tracing::info!("Cache cleared");

        // Remove cache file
        if self.cache_file.exists() {
            let _ = fs::remove_file(&self.cache_file);
        }
    }

    /// Returns the number of entries in the cache
    pub fn len(&self) -> usize {
        lock_mutex!(self.cache).len()
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
        let key1 = TranslationCache::generate_key("hello", "Chinese", true);
        let key2 = TranslationCache::generate_key("hello", "Japanese", false);
        let key3 = TranslationCache::generate_key("world", "Chinese", true);

        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
        assert_eq!(
            key1,
            TranslationCache::generate_key("hello", "Chinese", true)
        );
    }

    #[test]
    fn test_cache_set_and_get() {
        let temp_dir = env::temp_dir();
        let cache_file = temp_dir.join("test_cache.json");
        let cache = TranslationCache::new(cache_file.clone());

        cache.set("hello", "Chinese", false, "你好".to_string(), None);

        let result = cache.get("hello", "Chinese", false);
        assert_eq!(result, Some(("你好".to_string(), None)));

        let result = cache.get("hello", "Japanese", false);
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
            cache.set(
                "test",
                "English",
                true,
                "test result".to_string(),
                Some("keyword: test".to_string()),
            );
        }

        {
            let cache = TranslationCache::new(cache_file.clone());
            let result = cache.get("test", "English", true);
            assert_eq!(
                result,
                Some(("test result".to_string(), Some("keyword: test".to_string())))
            );
        }

        // Cleanup
        let _ = fs::remove_file(cache_file);
    }

    #[test]
    fn test_cache_clear() {
        let temp_dir = env::temp_dir();
        let cache_file = temp_dir.join("test_cache_clear.json");
        let cache = TranslationCache::new(cache_file.clone());

        cache.set("test", "Chinese", false, "测试".to_string(), None);
        assert!(cache.get("test", "Chinese", false).is_some());

        cache.clear();
        assert!(cache.get("test", "Chinese", false).is_none());

        // Cleanup
        let _ = fs::remove_file(cache_file);
    }

    #[test]
    fn test_cache_limit() {
        let temp_dir = env::temp_dir();
        let cache_file = temp_dir.join("test_cache_limit.json");
        let cache = TranslationCache::new(cache_file.clone());

        // Add more entries than the limit
        for i in 0..1500 {
            cache.set(
                &format!("test_{}", i),
                "English",
                i % 2 == 0,
                format!("translation_{}", i),
                if i % 2 == 0 {
                    Some(format!("keyword_{}", i))
                } else {
                    None
                },
            );
        }

        // Verify cache size is within limit
        let cache_size = {
            let cache_inner = cache.cache.lock().expect("Cache mutex poisoned");
            cache_inner.len()
        };
        assert!(
            cache_size <= 1000,
            "Cache size {} should be <= 1000",
            cache_size
        );

        // Verify newer entries exist
        assert!(cache.get("test_1499", "English", false).is_some());
        assert!(cache.get("test_1400", "English", true).is_some());

        // Cleanup
        let _ = fs::remove_file(cache_file);
    }
}

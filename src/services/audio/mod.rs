//! Audio cache and playback management module.
//!
//! This module provides functionality for managing audio cache
//! and controlling audio playback.

mod player;

pub use player::{AudioPlayer, PlaybackState};

use crate::lock_mutex;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Cache index entry for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheIndexEntry {
    /// Path to audio file
    audio_path: PathBuf,
    /// Timestamp when audio was generated
    timestamp: i64,
    /// Hash of the text that was converted to audio
    text_hash: String,
}

/// Cache entry for audio files
#[derive(Debug, Clone)]
struct AudioCacheEntry {
    /// Path to the audio file
    audio_path: PathBuf,
    /// Timestamp when the audio was generated
    timestamp: i64,
}

/// Audio cache manager with 100-entry limit
pub struct AudioCache {
    cache: Arc<Mutex<HashMap<String, AudioCacheEntry>>>,
    cache_dir: PathBuf,
    index_file: PathBuf,
    max_entries: usize,
}

impl AudioCache {
    /// Creates a new audio cache
    ///
    /// # Arguments
    ///
    /// * `cache_dir` - Directory to store audio cache files
    pub fn new(cache_dir: PathBuf) -> Self {
        tracing::info!("Initializing audio cache at: {:?}", cache_dir);

        let index_file = cache_dir.join("cache_index.json");

        // Create cache directory if it doesn't exist
        let _ = fs::create_dir_all(&cache_dir);

        // Load cache from index file
        let cache = Self::load_cache_from_index(&index_file, &cache_dir);

        AudioCache {
            cache: Arc::new(Mutex::new(cache)),
            cache_dir,
            index_file,
            max_entries: 100,
        }
    }

    /// Generates a cache key from text
    fn generate_key(text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Retrieves an audio file path from the cache
    ///
    /// # Arguments
    ///
    /// * `text` - The text that was converted to audio
    ///
    /// # Returns
    ///
    /// Some(audio_path) if found in cache, None otherwise
    pub fn get(&self, text: &str) -> Option<PathBuf> {
        let key = Self::generate_key(text);
        let cache = lock_mutex!(self.cache);

        if let Some(entry) = cache.get(&key) {
            // Check if the audio file still exists
            if entry.audio_path.exists() {
                tracing::info!("Audio cache hit for text hash: {}", key);
                return Some(entry.audio_path.clone());
            } else {
                tracing::warn!("Cached audio file not found: {:?}", entry.audio_path);
            }
        }

        None
    }

    /// Stores an audio file in the cache
    ///
    /// # Arguments
    ///
    /// * `text` - The text that was converted to audio
    /// * `audio_path` - Path to the generated audio file
    pub fn set(&self, text: &str, audio_path: PathBuf) {
        if !audio_path.exists() {
            tracing::warn!("Audio file does not exist: {:?}", audio_path);
            return;
        }

        let key = Self::generate_key(text);
        let entry = AudioCacheEntry {
            audio_path: audio_path.clone(),
            timestamp: Utc::now().timestamp(),
        };

        {
            let mut cache = lock_mutex!(self.cache);

            // Remove old entry if exists
            if let Some(old_entry) = cache.get(&key) {
                // Delete old audio file if different
                if old_entry.audio_path != audio_path {
                    let _ = fs::remove_file(&old_entry.audio_path);
                }
            }

            cache.insert(key.clone(), entry);

            // Check if cache size exceeds limit
            if cache.len() > self.max_entries {
                self.cleanup_oldest_entries(&mut cache);
            }

            tracing::info!("Cached audio for text hash: {}", key);
        }

        // Save index to file
        self.save_cache_index();
    }

    /// Clears all entries from the cache
    pub fn clear(&self) {
        let cache = lock_mutex!(self.cache);

        tracing::info!("Clearing audio cache ({} entries)", cache.len());

        for entry in cache.values() {
            if entry.audio_path.exists()
                && let Err(e) = fs::remove_file(&entry.audio_path)
            {
                tracing::warn!("Failed to remove audio file {:?}: {}", entry.audio_path, e);
            }
        }

        drop(cache);
        lock_mutex!(self.cache).clear();

        // Delete index file
        if let Err(e) = fs::remove_file(&self.index_file) {
            tracing::warn!("Failed to remove cache index file: {}", e);
        }
    }

    /// Gets the number of entries in the cache
    pub fn len(&self) -> usize {
        lock_mutex!(self.cache).len()
    }

    /// Loads cache from index file
    fn load_cache_from_index(
        index_file: &Path,
        _cache_dir: &Path,
    ) -> HashMap<String, AudioCacheEntry> {
        let mut cache = HashMap::new();

        if !index_file.exists() {
            tracing::info!("Cache index file not found, starting with empty cache");
            return cache;
        }

        match fs::read_to_string(index_file) {
            Ok(index_json) => {
                match serde_json::from_str::<Vec<CacheIndexEntry>>(&index_json) {
                    Ok(entries) => {
                        tracing::info!("Loading {} entries from cache index", entries.len());

                        for entry in entries {
                            // Check if audio file exists
                            if entry.audio_path.exists() {
                                cache.insert(
                                    entry.text_hash.clone(),
                                    AudioCacheEntry {
                                        audio_path: entry.audio_path.clone(),
                                        timestamp: entry.timestamp,
                                    },
                                );
                            } else {
                                tracing::warn!(
                                    "Audio file not found for cache entry, skipping: {:?}",
                                    entry.audio_path
                                );
                            }
                        }

                        tracing::info!("Successfully loaded {} valid cache entries", cache.len());
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse cache index file: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to read cache index file: {}", e);
            }
        }

        cache
    }

    /// Saves cache index to file
    fn save_cache_index(&self) {
        let cache = lock_mutex!(self.cache);

        // Convert cache entries to index entries
        let index_entries: Vec<CacheIndexEntry> = cache
            .iter()
            .map(|(text_hash, entry)| CacheIndexEntry {
                audio_path: entry.audio_path.clone(),
                timestamp: entry.timestamp,
                text_hash: text_hash.clone(),
            })
            .collect();

        drop(cache);

        // Serialize and write to file
        match serde_json::to_string_pretty(&index_entries) {
            Ok(json) => {
                if let Err(e) = fs::write(&self.index_file, json) {
                    tracing::error!("Failed to write cache index file: {}", e);
                } else {
                    tracing::debug!("Saved cache index with {} entries", index_entries.len());
                }
            }
            Err(e) => {
                tracing::error!("Failed to serialize cache index: {}", e);
            }
        }
    }

    /// Cleans up oldest entries when cache size exceeds limit
    fn cleanup_oldest_entries(&self, cache: &mut HashMap<String, AudioCacheEntry>) {
        const CLEANUP_SIZE: usize = 20;

        tracing::info!(
            "Cache size {} exceeds limit {}, removing oldest {} entries",
            cache.len(),
            self.max_entries,
            CLEANUP_SIZE
        );

        // Collect all entries with their keys and timestamps
        let mut entries: Vec<(String, i64, PathBuf)> = cache
            .iter()
            .map(|(k, v)| (k.clone(), v.timestamp, v.audio_path.clone()))
            .collect();

        // Sort by timestamp (oldest first)
        entries.sort_by(|a, b| a.1.cmp(&b.1));

        // Remove oldest CLEANUP_SIZE entries
        for (key_to_remove, _, path) in entries.iter().take(CLEANUP_SIZE) {
            // Delete audio file
            if path.exists() {
                let _ = fs::remove_file(path);
            }
            cache.remove(key_to_remove);
        }

        tracing::info!("Audio cache cleanup completed, new size: {}", cache.len());

        // Save updated index
        self.save_cache_index();
    }

    /// Gets a path for a new cached audio file
    pub fn get_new_audio_path(&self, text: &str) -> PathBuf {
        let key = Self::generate_key(text);
        self.cache_dir.join(format!("{}.wav", key))
    }
}

impl Default for AudioCache {
    fn default() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ai-translate")
            .join("audio");

        Self::new(cache_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let key1 = AudioCache::generate_key("hello");
        let key2 = AudioCache::generate_key("world");
        let key3 = AudioCache::generate_key("hello");

        assert_ne!(key1, key2);
        assert_eq!(key1, key3);
    }

    #[test]
    fn test_audio_player_creation() {
        let player = AudioPlayer::new();
        assert_eq!(player.get_state(), PlaybackState::Idle);
        assert!(!player.is_playing());
    }

    #[test]
    fn test_audio_player_stop() {
        let player = AudioPlayer::new();
        // Stopping when idle should not fail
        assert!(player.stop().is_ok());
    }
}

//! Logging functionality for translation history.
//!
//! This module provides file-based logging for translation operations,
//! recording timestamps, languages, and translation content.

use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

/// Logger for recording translation history to a file.
pub struct Logger {
    file: Mutex<std::fs::File>,
}

impl Logger {
    /// Creates a new logger that writes to the specified file path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the log file
    ///
    /// # Returns
    ///
    /// Result containing the logger or an IO error
    pub fn new(path: &str) -> std::io::Result<Self> {
        tracing::info!("Initializing translation logger at: {}", path);
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Logger {
            file: Mutex::new(file),
        })
    }

    /// Logs a translation operation with metadata.
    ///
    /// # Arguments
    ///
    /// * `source_lang` - Source language name
    /// * `target_lang` - Target language name
    /// * `source_text` - Original text
    /// * `translated` - Translated text
    pub fn log(&self, source_lang: &str, target_lang: &str, source_text: &str, translated: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

        // Log to tracing as well
        tracing::info!(
            source_language = source_lang,
            target_language = target_lang,
            source_length = source_text.len(),
            translated_length = translated.len(),
            "Translation completed"
        );

        // Log to file
        let log_entry = format!(
            "[{}]\nSource Language: {}\nTarget Language: {}\nSource Text: {}\nTranslation: {}\n{}\n",
            timestamp,
            source_lang,
            target_lang,
            source_text,
            translated,
            "-".repeat(80)
        );

        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(log_entry.as_bytes());
            let _ = file.flush();
        }
    }
}

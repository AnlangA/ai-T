use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

pub struct Logger {
    file: Mutex<std::fs::File>,
}

impl Logger {
    pub fn new(path: &str) -> std::io::Result<Self> {
        tracing::info!("Initializing translation logger at: {}", path);
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Logger {
            file: Mutex::new(file),
        })
    }

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

unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}

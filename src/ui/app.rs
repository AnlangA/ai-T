use crate::api::translator::Translator;
use crate::channel::channel::UiMessage;
use crate::lock_mutex;
use crate::services::audio::{AudioCache, AudioPlayer};
use crate::services::tts::{TtsConfig, TtsService};
use crate::ui::display::DisplayPanel;
use crate::ui::settings::{SettingsChange, SettingsConfig, SettingsPanel, ThemePreference};
use crate::ui::sidebar::Sidebar;
use crate::ui::theme::Theme;
use crate::utils::cache::TranslationCache;
use crate::utils::config::AppConfig;
use crate::utils::logger::Logger;
use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

/// Enum representing the type of TTS (source or translation)
enum TtsType {
    Source,
    Translation,
}

pub struct TranslateApp {
    config: AppConfig,
    sidebar: Sidebar,
    display: DisplayPanel,
    theme: Theme,
    settings: SettingsPanel,
    logger: Option<Arc<Logger>>,
    cache: Arc<TranslationCache>,
    translator: Option<Arc<Translator>>,
    is_translating: bool,
    cancel_requested: Arc<Mutex<bool>>,
    ui_tx: UnboundedSender<UiMessage>,
    ui_rx: Arc<Mutex<Option<UnboundedReceiver<UiMessage>>>>,
    _runtime: tokio::runtime::Runtime, // Prefixed with _ to silence unused warning
    runtime_handle: tokio::runtime::Handle,

    // TTS components
    tts_service: Arc<TtsService>,
    audio_cache: Arc<AudioCache>,
    audio_player: Arc<AudioPlayer>,
    // Independent TTS cancellation flags
    source_tts_cancel_requested: Arc<Mutex<bool>>,
    translation_tts_cancel_requested: Arc<Mutex<bool>>,
}

impl TranslateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = cc
            .storage
            .map(AppConfig::from_storage)
            .unwrap_or_else(|| AppConfig::load_or_default(&cc.egui_ctx));

        let theme = Theme {
            dark: config.dark_theme,
            font_size: config.font_size,
        };

        theme.setup_fonts(&cc.egui_ctx);
        theme.apply_style(&cc.egui_ctx);
        theme.set_visuals(&cc.egui_ctx);

        let mut sidebar = Sidebar::default();
        sidebar.set_api_key(config.api_key.clone());
        sidebar.set_target_language(config.target_language.clone());

        let settings = SettingsPanel::new(SettingsConfig {
            font_size: config.font_size,
            dark_theme: config.dark_theme,
            tts_voice: config.tts_voice.clone(),
            tts_speed: config.tts_speed,
            tts_volume: config.tts_volume,
            enable_keyword_analysis: config.enable_keyword_analysis,
            think_enable: config.think_enable,
            coding_plan: config.coding_plan,
        });

        let logger = Logger::new("translations.log").ok().map(Arc::new);
        let cache = Arc::new(TranslationCache::default());
        let audio_cache = Arc::new(AudioCache::default());
        let audio_player = Arc::new(AudioPlayer::new());

        let (ui_tx, ui_rx) = mpsc::unbounded_channel();

        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        let runtime_handle = rt.handle().clone();

        // Initialize TTS service with API key and runtime handle
        let tts_service = Arc::new(TtsService::new(config.api_key.clone(), runtime_handle.clone()));

        // Configure TTS service
        let tts_config = TtsConfig::new(
            AppConfig::parse_voice(&config.tts_voice),
            config.tts_speed,
            config.tts_volume,
            config.coding_plan,
            config.think_enable,
        );
        tts_service.update_config(tts_config);

        TranslateApp {
            _runtime: rt,
            config,
            sidebar,
            display: DisplayPanel::default(),
            theme,
            settings,
            logger,
            cache,
            translator: None,
            is_translating: false,
            cancel_requested: Arc::new(Mutex::new(false)),
            ui_tx,
            ui_rx: Arc::new(Mutex::new(Some(ui_rx))),
            runtime_handle,
            tts_service,
            audio_cache,
            audio_player,
            source_tts_cancel_requested: Arc::new(Mutex::new(false)),
            translation_tts_cancel_requested: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start_translation(&mut self, api_key: String) {
        if self.is_translating {
            tracing::warn!("Translation already in progress, ignoring request");
            return;
        }

        tracing::info!("Starting new translation");

        // Stop all audio activities when starting new translation
        tracing::info!("Stopping audio playback...");
        self.stop_audio();

        tracing::info!("Cancelling source TTS conversion...");
        self.cancel_source_tts();

        tracing::info!("Cancelling translation TTS conversion...");
        self.cancel_translation_tts();

        tracing::info!("All audio activities stopped for new translation");

        // Reset cancel flag
        *lock_mutex!(self.cancel_requested) = false;

        let translator = Arc::new(Translator::new(api_key, self.cache.clone()));
        self.translator = Some(translator.clone());

        let source_text = self.sidebar.get_source_text();
        let target_language = self.sidebar.get_target_language();

        tracing::debug!(
            source_length = source_text.len(),
            target_language = %target_language,
            "Translation parameters"
        );

        self.display.clear_translation();
        self.is_translating = true;
        self.display.set_translating(true);
        self.display.set_input(source_text.clone());

        let ui_tx = self.ui_tx.clone();
        let handle = self.runtime_handle.clone();
        let cancel_flag = self.cancel_requested.clone();

        let enable_keyword_analysis = self.config.enable_keyword_analysis;
        handle.spawn(async move {
            let mut stream_rx =
                translator.translate(source_text, target_language, enable_keyword_analysis);

            loop {
                tokio::select! {
                    // Check cancel flag continuously
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)), if {
                        *cancel_flag.lock().expect("Cancel flag mutex poisoned")
                    } => {
                        tracing::info!("Translation cancelled by user");
                        let _ = ui_tx.send(UiMessage::TranslationCancelled);
                        break;
                    }
                    // Receive stream data
                    result = stream_rx.recv() => {
                        match result {
                            Some(Ok(chunk)) => {
                                if chunk.is_empty() {
                                    let _ = ui_tx.send(UiMessage::TranslationComplete);
                                    break;
                                }
                                let _ = ui_tx.send(UiMessage::UpdateTranslation(chunk));
                            }
                            Some(Err(e)) => {
                                tracing::error!("Translation error: {}", e);
                                let _ = ui_tx.send(UiMessage::Error(e.to_string()));
                                break;
                            }
                            None => {
                                // Stream closed
                                tracing::info!("Translation stream ended");
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn cancel_translation(&mut self) {
        if self.is_translating {
            tracing::info!("Cancelling translation");
            *self
                .cancel_requested
                .lock()
                .expect("Cancel flag mutex poisoned") = true;
        }
    }

    /// Starts TTS conversion for source text
    pub fn start_source_tts(&mut self, text: String) {
        self.start_tts(text, TtsType::Source);
    }

    /// Starts TTS conversion
    fn start_tts(&mut self, text: String, tts_type: TtsType) {
        if text.trim().is_empty() {
            tracing::warn!("Cannot start TTS for empty text");
            return;
        }

        let (cancel_flag, tts_type_name) = match tts_type {
            TtsType::Source => (&self.source_tts_cancel_requested, "Source"),
            TtsType::Translation => (&self.translation_tts_cancel_requested, "Translation"),
        };

        // Check if audio is already cached
        if let Some(audio_path) = self.audio_cache.get(&text) {
            tracing::info!("{} audio already cached: {:?}", tts_type_name, audio_path);
            match tts_type {
                TtsType::Source => {
                    self.display
                        .set_source_audio_path(Some(audio_path.display().to_string()));
                }
                TtsType::Translation => {
                    self.display
                        .set_translation_audio_path(Some(audio_path.display().to_string()));
                }
            }
            return;
        }

        tracing::info!(
            "Starting TTS conversion for {} (length: {})",
            tts_type_name,
            text.len()
        );

        // Reset cancel flag and get new audio file path
        *lock_mutex!(cancel_flag) = false;
        let audio_path = self.audio_cache.get_new_audio_path(&text);

        // Update UI state and send start message
        match tts_type {
            TtsType::Source => {
                self.display.set_source_tts_converting(true);
                let _ = self.ui_tx.send(UiMessage::SourceTtsStarted);
            }
            TtsType::Translation => {
                self.display.set_translation_tts_converting(true);
                let _ = self.ui_tx.send(UiMessage::TranslationTtsStarted);
            }
        }

        // Start async conversion
        let tts_service = self.tts_service.clone();
        let audio_cache = self.audio_cache.clone();
        let text_clone = text.clone();
        let ui_tx = self.ui_tx.clone();
        let cancel_flag = cancel_flag.clone();
        let tts_type_clone = tts_type;

        let handle = self.runtime_handle.clone();

        handle.spawn(async move {
            // Check if cancellation was requested before starting
            if *lock_mutex!(cancel_flag) {
                tracing::info!("{} TTS cancelled before start", tts_type_name);
                return;
            }

            let audio_path_str = audio_path.to_string_lossy().to_string();

            // Perform conversion
            let text_for_cache = text_clone.clone();
            let audio_path_for_cache = audio_path.clone();
            let ui_tx_clone = ui_tx.clone();
            let cancel_flag_clone = cancel_flag.clone();
            tts_service.convert_async(&text_clone, &audio_path_str, move |status| {
                // Check if cancellation was requested
                if *lock_mutex!(cancel_flag_clone) {
                    tracing::info!("{} TTS cancelled", tts_type_name);
                    return;
                }

                match status {
                    crate::services::tts::TtsStatus::Completed(path) => {
                        // Store in cache
                        audio_cache.set(&text_for_cache, audio_path_for_cache);
                        tracing::info!("{} TTS completed: {}", tts_type_name, path);
                        let msg = match tts_type_clone {
                            TtsType::Source => UiMessage::SourceTtsCompleted(path),
                            TtsType::Translation => UiMessage::TranslationTtsCompleted(path),
                        };
                        let _ = ui_tx_clone.send(msg);
                    }
                    crate::services::tts::TtsStatus::Failed(err) => {
                        tracing::error!("{} TTS failed: {}", tts_type_name, err);
                        let _ = ui_tx_clone.send(UiMessage::TtsFailed(err));
                    }
                    _ => {}
                }
            });
        });
    }

    /// Starts TTS conversion for translation text
    pub fn start_translation_tts(&mut self, text: String) {
        self.start_tts(text, TtsType::Translation);
    }

    /// Plays audio file
    pub fn play_audio(&mut self, audio_path: String) {
        tracing::info!("Playing audio: {}", audio_path);

        // Check if this audio is currently playing (Stop button clicked)
        if matches!(self.audio_player.get_state(), crate::services::audio::PlaybackState::Playing(ref p) if p == &audio_path)
        {
            tracing::info!("Stopping audio playback: {}", audio_path);
            if let Err(e) = self.audio_player.stop() {
                tracing::warn!("Failed to stop playback: {}", e);
            }
            self.display
                .set_playback_state(crate::services::audio::PlaybackState::Idle);
            return;
        }

        // Stop any currently playing audio
        if self.audio_player.is_playing()
            && let Err(e) = self.audio_player.stop()
        {
            tracing::warn!("Failed to stop current playback: {}", e);
        }

        // Start new playback
        match self.audio_player.play(&audio_path) {
            Ok(_) => {
                tracing::info!("Audio playback started");
                self.display
                    .set_playback_state(crate::services::audio::PlaybackState::Playing(
                        audio_path.clone(),
                    ));
            }
            Err(e) => {
                tracing::error!("Failed to play audio: {}", e);
                self.display
                    .set_playback_state(crate::services::audio::PlaybackState::Failed(format!(
                        "Playback error: {}",
                        e
                    )));
            }
        }
    }

    /// Stops audio playback
    pub fn stop_audio(&mut self) {
        if self.audio_player.is_playing() {
            tracing::info!("Stopping audio playback");
            if let Err(e) = self.audio_player.stop() {
                tracing::warn!("Failed to stop playback: {}", e);
            }
            self.display
                .set_playback_state(crate::services::audio::PlaybackState::Idle);
        }
    }

    /// Cancels TTS conversion
    fn cancel_tts(&mut self, tts_type: TtsType) {
        let (is_converting, cancel_flag, tts_type_name) = match tts_type {
            TtsType::Source => (
                self.display.is_source_converting(),
                &self.source_tts_cancel_requested,
                "Source",
            ),
            TtsType::Translation => (
                self.display.is_translation_converting(),
                &self.translation_tts_cancel_requested,
                "Translation",
            ),
        };

        if is_converting {
            tracing::info!("Cancelling {} TTS", tts_type_name);
            *lock_mutex!(cancel_flag) = true;
            // Clear converting state immediately
            match tts_type {
                TtsType::Source => {
                    self.display.set_source_tts_converting(false);
                }
                TtsType::Translation => {
                    self.display.set_translation_tts_converting(false);
                }
            }
        }
    }

    /// Cancels source TTS conversion
    pub fn cancel_source_tts(&mut self) {
        self.cancel_tts(TtsType::Source);
    }

    /// Cancels translation TTS conversion
    pub fn cancel_translation_tts(&mut self) {
        self.cancel_tts(TtsType::Translation);
    }

    /// Clears audio cache
    pub fn clear_audio_cache(&mut self) {
        tracing::info!("Clearing audio cache");
        self.audio_cache.clear();
        self.display.set_source_audio_path(None);
        self.display.set_translation_audio_path(None);
    }

    /// Clears translation cache
    pub fn clear_translation_cache(&mut self) {
        tracing::info!("Clearing translation cache");
        self.cache.clear();
    }

    fn process_messages(&mut self, ctx: &egui::Context) {
        // Collect all messages first to avoid borrowing issues
        let messages: Vec<UiMessage> = {
            let mut rx_opt = self.ui_rx.lock().unwrap();
            if let Some(rx) = rx_opt.as_mut() {
                let mut msgs = Vec::new();
                while let Ok(msg) = rx.try_recv() {
                    msgs.push(msg);
                }
                msgs
            } else {
                Vec::new()
            }
        };

        // Process collected messages
        for msg in messages {
            match msg {
                UiMessage::UpdateTranslation(chunk) => {
                    self.display.update_translation(chunk);
                    ctx.request_repaint();
                }
                UiMessage::Error(err) => {
                    tracing::error!("UI received translation error: {}", err);
                    self.is_translating = false;
                    self.display.set_translating(false);
                    self.display.set_error(err);
                    ctx.request_repaint();
                }
                UiMessage::TranslationComplete => {
                    tracing::info!("Translation completed successfully");
                    self.is_translating = false;
                    self.display.set_translating(false);

                    if let Some(logger) = &self.logger {
                        logger.log(
                            "Auto-detected",
                            &self.config.target_language,
                            &self.sidebar.get_source_text(),
                            &self.display.translation,
                        );
                    }
                }
                UiMessage::TranslationCancelled => {
                    tracing::info!("Translation cancelled");
                    self.is_translating = false;
                    self.display.set_translating(false);
                    ctx.request_repaint();
                }
                UiMessage::RequestSourceTts(text) => {
                    tracing::info!("Source TTS requested");
                    self.start_source_tts(text);
                }
                UiMessage::RequestTranslationTts(text) => {
                    tracing::info!("Translation TTS requested");
                    self.start_translation_tts(text);
                }
                UiMessage::StopPlayback => {
                    tracing::info!("Stop playback requested");
                    self.stop_audio();
                }
                UiMessage::SourceTtsStarted => {
                    tracing::info!("Source TTS started");
                    ctx.request_repaint();
                }
                UiMessage::TranslationTtsStarted => {
                    tracing::info!("Translation TTS started");
                    ctx.request_repaint();
                }
                UiMessage::SourceTtsCompleted(path) => {
                    tracing::info!("Source TTS completed: {}", path);
                    self.display.set_source_tts_converting(false);
                    self.display.set_source_audio_path(Some(path));
                    ctx.request_repaint();
                }
                UiMessage::TranslationTtsCompleted(path) => {
                    tracing::info!("Translation TTS completed: {}", path);
                    self.display.set_translation_tts_converting(false);
                    self.display.set_translation_audio_path(Some(path));
                    ctx.request_repaint();
                }
                UiMessage::TtsFailed(err) => {
                    tracing::error!("TTS failed: {}", err);
                    self.display.set_source_tts_converting(false);
                    self.display.set_translation_tts_converting(false);
                    ctx.request_repaint();
                }
                UiMessage::PlaybackStateChanged(state) => {
                    self.display.set_playback_state(state);
                    ctx.request_repaint();
                }
            }
        }
    }
}

impl eframe::App for TranslateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_messages(ctx);
        self.theme.set_visuals(ctx);

        // Check if audio playback has finished
        self.audio_player.update_state_if_finished();
        if self.audio_player.get_state() == crate::services::audio::PlaybackState::Idle {
            self.display
                .set_playback_state(crate::services::audio::PlaybackState::Idle);
        }

        egui::TopBottomPanel::top("top_bar")
            .exact_height(40.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("⚙ Settings").clicked() {
                            self.settings.toggle_panel();
                        }
                    });
                });
            });

        let (translate_requested, cancel_requested, api_key_to_save) =
            self.sidebar.ui(ctx, self.is_translating);

        if let Some(api_key) = api_key_to_save {
            self.config.api_key = api_key;
        }
        self.config.target_language = self.sidebar.get_target_language();

        if translate_requested {
            let api_key = self.sidebar.get_api_key();
            if !api_key.is_empty() {
                self.start_translation(api_key);
            }
        }

        if cancel_requested {
            self.cancel_translation();
            ctx.request_repaint(); // Force immediate UI update to show cancel
        }

        let (_show_settings, settings_changes) =
            self.settings
                .ui(ctx, Some(self.cache.clone()), self.audio_cache.len());

        if let Some(change) = settings_changes {
            match change {
                SettingsChange::FontSize(new_font_size) => {
                    self.config.font_size = new_font_size;
                    self.theme.font_size = new_font_size;
                    self.theme.apply_style(ctx);
                    tracing::info!("Font size changed to: {}", new_font_size);
                }
                SettingsChange::Theme(theme_preference) => {
                    let dark_theme = matches!(
                        theme_preference,
                        ThemePreference::Dark | ThemePreference::System
                    );
                    self.config.dark_theme = dark_theme;
                    self.theme.dark = dark_theme;
                    self.theme.set_visuals(ctx);
                    tracing::info!("Theme changed to: {:?}", theme_preference);
                }
                SettingsChange::TtsVoice(voice) => {
                    self.config.tts_voice = voice.clone();
                    let tts_config = TtsConfig::new(
                        AppConfig::parse_voice(&voice),
                        self.config.tts_speed,
                        self.config.tts_volume,
                        self.config.coding_plan,
                        self.config.think_enable,
                    );
                    self.tts_service.update_config(tts_config);
                    tracing::info!("TTS voice changed to: {}", voice);
                }
                SettingsChange::TtsSpeed(speed) => {
                    self.config.tts_speed = speed;
                    let tts_config = TtsConfig::new(
                        AppConfig::parse_voice(&self.config.tts_voice),
                        speed,
                        self.config.tts_volume,
                        self.config.coding_plan,
                        self.config.think_enable,
                    );
                    self.tts_service.update_config(tts_config);
                    tracing::info!("TTS speed changed to: {}", speed);
                }
                SettingsChange::TtsVolume(volume) => {
                    self.config.tts_volume = volume;
                    let tts_config = TtsConfig::new(
                        AppConfig::parse_voice(&self.config.tts_voice),
                        self.config.tts_speed,
                        volume,
                        self.config.coding_plan,
                        self.config.think_enable,
                    );
                    self.tts_service.update_config(tts_config);
                    tracing::info!("TTS volume changed to: {}", volume);
                }
                SettingsChange::KeywordAnalysis(enabled) => {
                    self.config.enable_keyword_analysis = enabled;
                    tracing::info!(
                        "Keyword analysis {}",
                        if enabled { "enabled" } else { "disabled" }
                    );
                }
                SettingsChange::ThinkEnable(enabled) => {
                    self.config.think_enable = enabled;
                    let tts_config = TtsConfig::new(
                        AppConfig::parse_voice(&self.config.tts_voice),
                        self.config.tts_speed,
                        self.config.tts_volume,
                        self.config.coding_plan,
                        self.config.think_enable,
                    );
                    self.tts_service.update_config(tts_config);
                    tracing::info!(
                        "Thinking mode {}",
                        if enabled { "enabled" } else { "disabled" }
                    );
                }
                SettingsChange::CodingPlan(enabled) => {
                    self.config.coding_plan = enabled;
                    let tts_config = TtsConfig::new(
                        AppConfig::parse_voice(&self.config.tts_voice),
                        self.config.tts_speed,
                        self.config.tts_volume,
                        self.config.coding_plan,
                        self.config.think_enable,
                    );
                    self.tts_service.update_config(tts_config);
                    tracing::info!(
                        "Coding plan mode {}",
                        if enabled { "enabled" } else { "disabled" }
                    );
                }
                SettingsChange::ClearTranslationCache => {
                    self.clear_translation_cache();
                }
                SettingsChange::ClearAudioCache => {
                    self.clear_audio_cache();
                }
            }
        }

        let (
            play_source_clicked,
            source_audio_to_play,
            play_translation_clicked,
            translation_audio_to_play,
            start_source_tts,
            start_translation_tts,
            cancel_source_tts,
            cancel_translation_tts,
        ) = self.display.ui(ctx, self.theme.font_size);

        // Handle source TTS start
        if start_source_tts {
            let source_text = self.display.input_text().to_string();
            if !source_text.trim().is_empty() {
                self.start_source_tts(source_text);
            }
        }

        // Handle translation TTS start
        if start_translation_tts {
            let translation_text = self.display.translation.clone();
            if !translation_text.trim().is_empty() {
                self.start_translation_tts(translation_text);
            }
        }

        // Handle source audio button click
        if play_source_clicked && let Some(audio_path) = source_audio_to_play {
            self.play_audio(audio_path);
        }

        // Handle translation audio button click
        if play_translation_clicked && let Some(audio_path) = translation_audio_to_play {
            self.play_audio(audio_path);
        }

        // Handle source TTS cancel
        if cancel_source_tts {
            self.cancel_source_tts();
            ctx.request_repaint(); // Force UI repaint to show cancel immediately
        }

        // Handle translation TTS cancel
        if cancel_translation_tts {
            self.cancel_translation_tts();
            ctx.request_repaint(); // Force UI repaint to show cancel immediately
        }

        // Note: TTS is now manually triggered by user buttons
        // Removed auto-start TTS logic to give users more control

        // Request repaint to ensure continuous UI updates
        // This enables smooth animations (e.g., spinner) without requiring mouse movement
        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.config.save_to_storage(storage);
    }
}

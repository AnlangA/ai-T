use crate::api::translator::Translator;
use crate::channel::channel::UiMessage;
use crate::services::audio::{AudioCache, AudioPlayer};
use crate::services::tts::{TtsConfig, TtsService};
use crate::ui::display::DisplayPanel;
use crate::ui::settings::{SettingsChange, SettingsPanel};
use crate::ui::sidebar::Sidebar;
use crate::ui::theme::Theme;
use crate::utils::cache::TranslationCache;
use crate::utils::config::AppConfig;
use crate::utils::logger::Logger;
use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

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
    runtime_handle: tokio::runtime::Handle,

    // TTS components
    tts_service: Arc<TtsService>,
    audio_cache: Arc<AudioCache>,
    audio_player: Arc<AudioPlayer>,
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

        let settings = SettingsPanel::new(
            config.font_size,
            config.dark_theme,
            config.tts_voice.clone(),
            config.tts_speed,
            config.tts_volume,
        );

        let logger = Logger::new("translations.log").ok().map(Arc::new);
        let cache = Arc::new(TranslationCache::default());
        let audio_cache = Arc::new(AudioCache::default());
        let audio_player = Arc::new(AudioPlayer::new());

        // Initialize TTS service with API key
        let tts_service = Arc::new(TtsService::new(config.api_key.clone()));

        // Configure TTS service
        let tts_config = TtsConfig::new(
            AppConfig::parse_voice(&config.tts_voice),
            config.tts_speed,
            config.tts_volume,
        );
        tts_service.update_config(tts_config);

        let (ui_tx, ui_rx) = mpsc::unbounded_channel();

        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        let runtime_handle = rt.handle().clone();
        std::mem::forget(rt);

        TranslateApp {
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
        }
    }

    pub fn start_translation(&mut self, api_key: String) {
        if self.is_translating {
            tracing::warn!("Translation already in progress, ignoring request");
            return;
        }

        tracing::info!("Starting new translation");

        // Reset cancel flag
        *self.cancel_requested.lock().expect("Cancel flag mutex poisoned") = false;

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

        handle.spawn(async move {
            let mut stream_rx = translator.translate(source_text, target_language);

            while let Some(result) = stream_rx.recv().await {
                // Check if cancellation was requested
                if *cancel_flag.lock().expect("Cancel flag mutex poisoned") {
                    tracing::info!("Translation cancelled by user");
                    let _ = ui_tx.send(UiMessage::TranslationCancelled);
                    break;
                }

                match result {
                    Ok(chunk) => {
                        if chunk.is_empty() {
                            let _ = ui_tx.send(UiMessage::TranslationComplete);
                            break;
                        }
                        let _ = ui_tx.send(UiMessage::UpdateTranslation(chunk));
                    }
                    Err(e) => {
                        tracing::error!("Translation error: {}", e);
                        let _ = ui_tx.send(UiMessage::Error(e.to_string()));
                        break;
                    }
                }
            }
        });
    }

    pub fn cancel_translation(&mut self) {
        if self.is_translating {
            tracing::info!("Cancelling translation");
            *self.cancel_requested.lock().expect("Cancel flag mutex poisoned") = true;
        }
    }

    /// Starts TTS conversion for source text
    pub fn start_source_tts(&mut self, text: String) {
        if text.trim().is_empty() {
            tracing::warn!("Cannot start TTS for empty text");
            return;
        }

        // Check if audio is already cached
        if let Some(audio_path) = self.audio_cache.get(&text) {
            tracing::info!("Source audio already cached: {:?}", audio_path);
            self.display.set_source_audio_path(Some(audio_path.display().to_string()));
            return;
        }

        tracing::info!("Starting TTS conversion for source text (length: {})", text.len());

        // Get new audio file path
        let audio_path = self.audio_cache.get_new_audio_path(&text);

        // Update UI state
        self.display.set_source_tts_converting(true);

        // Send message to UI
        let _ = self.ui_tx.send(UiMessage::SourceTtsStarted);

        // Start async conversion
        let tts_service = self.tts_service.clone();
        let audio_cache = self.audio_cache.clone();
        let text_clone = text.clone();
        let ui_tx = self.ui_tx.clone();

        let handle = self.runtime_handle.clone();

        handle.spawn(async move {
            let audio_path_str = audio_path.to_string_lossy().to_string();

            // Perform conversion
            let text_for_cache = text_clone.clone();
            let audio_path_for_cache = audio_path.clone();
            let ui_tx_clone = ui_tx.clone();
            tts_service.convert_async(&text_clone, &audio_path_str, move |status| {
                match status {
                    crate::services::tts::TtsStatus::Completed(path) => {
                        // Store in cache
                        audio_cache.set(&text_for_cache, audio_path_for_cache);
                        tracing::info!("Source TTS completed: {}", path);
                        let _ = ui_tx_clone.send(UiMessage::SourceTtsCompleted(path));
                    }
                    crate::services::tts::TtsStatus::Failed(err) => {
                        tracing::error!("Source TTS failed: {}", err);
                        let _ = ui_tx_clone.send(UiMessage::TtsFailed(err));
                    }
                    _ => {}
                }
            });
        });
    }

    /// Starts TTS conversion for translation text
    pub fn start_translation_tts(&mut self, text: String) {
        if text.trim().is_empty() {
            tracing::warn!("Cannot start TTS for empty translation");
            return;
        }

        // Check if audio is already cached
        if let Some(audio_path) = self.audio_cache.get(&text) {
            tracing::info!("Translation audio already cached: {:?}", audio_path);
            self.display.set_translation_audio_path(Some(audio_path.display().to_string()));
            return;
        }

        tracing::info!("Starting TTS conversion for translation (length: {})", text.len());

        // Get new audio file path
        let audio_path = self.audio_cache.get_new_audio_path(&text);

        // Update UI state
        self.display.set_translation_tts_converting(true);

        // Send message to UI
        let _ = self.ui_tx.send(UiMessage::TranslationTtsStarted);

        // Start async conversion
        let tts_service = self.tts_service.clone();
        let audio_cache = self.audio_cache.clone();
        let text_clone = text.clone();
        let ui_tx = self.ui_tx.clone();

        let handle = self.runtime_handle.clone();

        handle.spawn(async move {
            let audio_path_str = audio_path.to_string_lossy().to_string();

            // Perform conversion
            let text_for_cache = text_clone.clone();
            let audio_path_for_cache = audio_path.clone();
            let ui_tx_clone = ui_tx.clone();
            tts_service.convert_async(&text_clone, &audio_path_str, move |status| {
                match status {
                    crate::services::tts::TtsStatus::Completed(path) => {
                        // Store in cache
                        audio_cache.set(&text_for_cache, audio_path_for_cache);
                        tracing::info!("Translation TTS completed: {}", path);
                        let _ = ui_tx_clone.send(UiMessage::TranslationTtsCompleted(path));
                    }
                    crate::services::tts::TtsStatus::Failed(err) => {
                        tracing::error!("Translation TTS failed: {}", err);
                        let _ = ui_tx_clone.send(UiMessage::TtsFailed(err));
                    }
                    _ => {}
                }
            });
        });
    }

    /// Plays audio file
    pub fn play_audio(&mut self, audio_path: String) {
        tracing::info!("Playing audio: {}", audio_path);

        // Stop any currently playing audio
        if self.audio_player.is_playing() {
            if let Err(e) = self.audio_player.stop() {
                tracing::warn!("Failed to stop current playback: {}", e);
            }
        }

        // Start new playback
        match self.audio_player.play(&audio_path) {
            Ok(_) => {
                tracing::info!("Audio playback started");
                self.display.set_playback_state(
                    crate::services::audio::PlaybackState::Playing(audio_path.clone()),
                );
            }
            Err(e) => {
                tracing::error!("Failed to play audio: {}", e);
                self.display.set_playback_state(
                    crate::services::audio::PlaybackState::Failed(format!(
                        "Playback error: {}",
                        e
                    ))
                );
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
            self.display.set_playback_state(
                crate::services::audio::PlaybackState::Idle,
            );
        }
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
        let mut rx_opt = self.ui_rx.lock().unwrap();
        if let Some(rx) = rx_opt.as_mut() {
            while let Ok(msg) = rx.try_recv() {
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
}

impl eframe::App for TranslateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_messages(ctx);
        self.theme.set_visuals(ctx);

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

        let (translate_requested, cancel_requested, api_key_to_save) = self.sidebar.ui(ctx, self.is_translating);

        if let Some(api_key) = api_key_to_save {
            self.config.api_key = api_key.clone();
        }
        self.config.target_language = self.sidebar.get_target_language().clone();

        if translate_requested {
            let api_key = self.sidebar.get_api_key();
            if !api_key.is_empty() {
                self.start_translation(api_key);
            }
        }

        if cancel_requested {
            self.cancel_translation();
        }

        let (_show_settings, settings_changes) = self.settings.ui(
            ctx,
            Some(self.cache.clone()),
            self.audio_cache.len(),
        );

        if let Some(change) = settings_changes {
            match change {
                SettingsChange::Theme(new_font_size, new_dark_theme) => {
                    self.config.font_size = new_font_size;
                    self.config.dark_theme = new_dark_theme;
                    self.theme.font_size = new_font_size;
                    self.theme.dark = new_dark_theme;
                    self.theme.apply_style(ctx);
                    self.theme.set_visuals(ctx);
                }
                SettingsChange::Tts(voice, speed, volume) => {
                    self.config.tts_voice = voice.clone();
                    self.config.tts_speed = speed;
                    self.config.tts_volume = volume;

                    // Update TTS service config
                    let tts_config = TtsConfig::new(
                        AppConfig::parse_voice(&voice),
                        speed,
                        volume,
                    );
                    self.tts_service.update_config(tts_config);

                    tracing::info!(
                        "TTS settings updated: voice={}, speed={}, volume={}",
                        voice,
                        speed,
                        volume
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

        let (play_source_clicked, source_audio_to_play, play_translation_clicked, translation_audio_to_play) =
            self.display.ui(ctx, self.theme.font_size);

        // Handle source audio button click
        if play_source_clicked {
            if let Some(audio_path) = source_audio_to_play {
                self.play_audio(audio_path);
            }
        }

        // Handle translation audio button click
        if play_translation_clicked {
            if let Some(audio_path) = translation_audio_to_play {
                self.play_audio(audio_path);
            }
        }

        // Auto-start TTS for source text when translation completes
        if !self.is_translating && !self.display.translation.is_empty() {
            let source_text = self.sidebar.get_source_text();
            if !source_text.is_empty()
                && self.display.get_source_audio_path().is_none()
                && !self.display.is_source_converting()
            {
                self.start_source_tts(source_text);
            }

            let translation_text = self.display.translation.clone();
            if !translation_text.is_empty()
                && self.display.get_translation_audio_path().is_none()
                && !self.display.is_translation_converting()
            {
                self.start_translation_tts(translation_text);
            }
        }

        // Request repaint to ensure continuous UI updates
        // This enables smooth animations (e.g., spinner) without requiring mouse movement
        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.config.save_to_storage(storage);
    }
}

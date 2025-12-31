use crate::api::translator::Translator;
use crate::channel::channel::UiMessage;
use crate::ui::display::DisplayPanel;
use crate::ui::settings::SettingsPanel;
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
    ui_tx: UnboundedSender<UiMessage>,
    ui_rx: Arc<Mutex<Option<UnboundedReceiver<UiMessage>>>>,
    runtime_handle: tokio::runtime::Handle,
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

        let settings = SettingsPanel::new(config.font_size, config.dark_theme);

        let logger = Logger::new("translations.log").ok().map(Arc::new);
        let cache = Arc::new(TranslationCache::default());

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
            ui_tx,
            ui_rx: Arc::new(Mutex::new(Some(ui_rx))),
            runtime_handle,
        }
    }

    pub fn start_translation(&mut self, api_key: String) {
        if self.is_translating {
            tracing::warn!("Translation already in progress, ignoring request");
            return;
        }

        tracing::info!("Starting new translation");

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

        handle.spawn(async move {
            let mut stream_rx = translator.translate(source_text, target_language);

            while let Some(result) = stream_rx.recv().await {
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

        let (translate_requested, api_key_to_save) = self.sidebar.ui(ctx, self.is_translating);

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

        let (_, theme_changes) = self.settings.ui(ctx);

        if let Some((new_font_size, new_dark_theme)) = theme_changes {
            self.config.font_size = new_font_size;
            self.config.dark_theme = new_dark_theme;
            self.theme.font_size = new_font_size;
            self.theme.dark = new_dark_theme;
            self.theme.apply_style(ctx);
            self.theme.set_visuals(ctx);
        }

        self.display.ui(ctx, self.theme.font_size);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.config.save_to_storage(storage);
    }
}

use crate::utils::cache::TranslationCache;
use crate::utils::config::AppConfig;
use egui::{self, *};
use std::sync::Arc;

pub struct SettingsPanel {
    pub font_size: f32,
    pub dark_theme: bool,
    pub tts_voice: String,
    pub tts_speed: f32,
    pub tts_volume: f32,
    pub enable_keyword_analysis: bool,
    show_panel: bool,
    #[allow(dead_code)]
    clear_translation_cache: bool,
    #[allow(dead_code)]
    clear_audio_cache: bool,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        SettingsPanel {
            font_size: 16.0,
            dark_theme: true,
            tts_voice: "Tongtong".to_string(),
            tts_speed: 1.0,
            tts_volume: 1.0,
            enable_keyword_analysis: false,
            show_panel: false,
            clear_translation_cache: false,
            clear_audio_cache: false,
        }
    }
}

impl SettingsPanel {
    pub fn new(
        font_size: f32,
        dark_theme: bool,
        tts_voice: String,
        tts_speed: f32,
        tts_volume: f32,
        enable_keyword_analysis: bool,
    ) -> Self {
        SettingsPanel {
            font_size,
            dark_theme,
            tts_voice,
            tts_speed,
            tts_volume,
            enable_keyword_analysis,
            show_panel: false,
            clear_translation_cache: false,
            clear_audio_cache: false,
        }
    }

    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        translation_cache: Option<Arc<TranslationCache>>,
        audio_cache_len: usize,
    ) -> (bool, Option<SettingsChange>) {
        let mut settings_changed = None;
        let mut close_requested = false;

        Window::new("Settings")
            .collapsible(true)
            .resizable(false)
            .open(&mut self.show_panel)
            .default_size([350.0, 500.0])
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui| {
                        // Appearance Section
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🎨Appearance Settings").strong().size(18.0));
                        });
                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(12.0);

                        // Font Size
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("📏Font Size:").size(14.0));
                            ui.add_space(10.0);
                            ui.add(
                                Slider::new(&mut self.font_size, 12.0..=24.0)
                                    .step_by(1.0)
                                    .suffix(" px")
                                    .show_value(true),
                            );
                        });
                        ui.add_space(15.0);

                        // Theme
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🌗Theme:").size(14.0));
                            ui.add_space(10.0);

                            let dark_btn = egui::Button::new(RichText::new("🌑 Dark").size(13.0))
                                .corner_radius(6.0);
                            if ui.add(dark_btn).clicked() {
                                self.dark_theme = true;
                            }

                            ui.add_space(8.0);

                            let light_btn = egui::Button::new(RichText::new("☀️ Light").size(13.0))
                                .corner_radius(6.0);
                            if ui.add(light_btn).clicked() {
                                self.dark_theme = false;
                            }
                        });

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(12.0);

                        // Translation Features Section
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("📝Translation Features").strong().size(18.0));
                        });
                        ui.add_space(12.0);

                        // Keyword Analysis Toggle
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🔍Keyword Analysis:").size(14.0));
                            ui.add_space(10.0);
                            if ui
                                .checkbox(&mut self.enable_keyword_analysis, "")
                                .changed()
                            {
                                // Checkbox state changed
                            }
                        });
                        ui.label(
                            RichText::new(
                                "When enabled, analyze professional terms and provide explanations during translation. Each term on a separate line.",
                            )
                            .size(12.0)
                            .weak()
                            .color(Color32::GRAY),
                        );

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(12.0);

                        // TTS Section
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🔊TTS Settings").strong().size(18.0));
                        });
                        ui.add_space(12.0);

                        // Voice Selection
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🎤Voice:").size(14.0));
                            ui.add_space(10.0);

                            let voices = AppConfig::get_supported_voices();
                            egui::ComboBox::from_id_salt("voice_selector")
                                .selected_text(RichText::new(&self.tts_voice).size(14.0))
                                .width(150.0)
                                .show_ui(ui, |ui| {
                                    for voice in voices {
                                        ui.selectable_value(
                                            &mut self.tts_voice,
                                            voice.to_string(),
                                            voice,
                                        );
                                    }
                                });
                        });
                        ui.add_space(15.0);

                        // Speed Slider
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚡Speed:").size(14.0));
                            ui.add_space(10.0);
                            ui.add(
                                Slider::new(&mut self.tts_speed, 0.5..=2.0)
                                    .step_by(0.1)
                                    .suffix("x")
                                    .show_value(true),
                            );
                        });
                        ui.add_space(15.0);

                        // Volume Slider
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🔈Volume:").size(14.0));
                            ui.add_space(10.0);
                            ui.add(
                                Slider::new(&mut self.tts_volume, 0.0..=10.0)
                                    .step_by(0.5)
                                    .show_value(true),
                            );
                        });

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(12.0);

                        // Cache Management Section
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("💾Cache Management").strong().size(18.0));
                        });
                        ui.add_space(12.0);

                        // Translation Cache
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "Translation cache: {} entries",
                                    translation_cache.as_ref().map_or(0, |c| c.len())
                                ))
                                .size(14.0),
                            );
                        });
                        ui.add_space(8.0);

                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("🗑️Clear Translation Cache").size(13.0),
                                )
                                .corner_radius(6.0),
                            )
                            .clicked()
                        {
                            settings_changed = Some(SettingsChange::ClearTranslationCache);
                        }

                        ui.add_space(15.0);

                        // Audio Cache
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("Audio cache: {} files", audio_cache_len))
                                    .size(14.0),
                            );
                        });
                        ui.add_space(8.0);

                        if ui
                            .add(
                                egui::Button::new(RichText::new("🗑️Clear Audio Cache").size(13.0))
                                    .corner_radius(6.0),
                            )
                            .clicked()
                        {
                            settings_changed = Some(SettingsChange::ClearAudioCache);
                        }

                        ui.add_space(25.0);
                        ui.separator();
                        ui.add_space(15.0);

                        // Action Buttons
                        ui.vertical_centered(|ui| {
                            ui.horizontal(|ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("✓Apply Settings").size(14.0),
                                        )
                                        .corner_radius(8.0),
                                    )
                                    .clicked()
                                {
                                    settings_changed = Some(SettingsChange::All(
                                        self.font_size,
                                        self.dark_theme,
                                        self.tts_voice.clone(),
                                        self.tts_speed,
                                        self.tts_volume,
                                        self.enable_keyword_analysis,
                                    ));
                                }

                                ui.add_space(15.0);

                                if ui
                                    .add(
                                        egui::Button::new(RichText::new("❌Close").size(14.0))
                                            .corner_radius(8.0),
                                    )
                                    .clicked()
                                {
                                    close_requested = true;
                                }
                            });
                        });
                    });
                });
            });

        if close_requested {
            self.show_panel = false;
        }

        (self.show_panel, settings_changed)
    }

    pub fn toggle_panel(&mut self) {
        self.show_panel = !self.show_panel;
    }
}

#[derive(Debug, Clone)]
pub enum SettingsChange {
    All(f32, bool, String, f32, f32, bool),
    ClearTranslationCache,
    ClearAudioCache,
}

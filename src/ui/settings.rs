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
    show_panel: bool,
    clear_translation_cache: bool,
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
            show_panel: false,
            clear_translation_cache: false,
            clear_audio_cache: false,
        }
    }
}

impl SettingsPanel {
    pub fn new(font_size: f32, dark_theme: bool, tts_voice: String, tts_speed: f32, tts_volume: f32) -> Self {
        SettingsPanel {
            font_size,
            dark_theme,
            tts_voice,
            tts_speed,
            tts_volume,
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
                        ui.heading("Appearance Settings");
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.label("Font Size:");
                        ui.add_space(5.0);

                        ui.add(
                            Slider::new(&mut self.font_size, 12.0..=24.0)
                                .step_by(1.0)
                                .suffix(" px")
                                .show_value(true),
                        );
                        ui.add_space(10.0);

                        ui.label("Theme:");
                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.dark_theme, true, "Dark");
                            ui.radio_value(&mut self.dark_theme, false, "Light");
                        });

                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.heading("TTS Settings");
                        ui.add_space(10.0);

                        ui.label("Voice:");
                        ui.add_space(5.0);

                        let voices = AppConfig::get_supported_voices();
                        egui::ComboBox::from_id_salt("voice_selector")
                            .selected_text(&self.tts_voice)
                            .show_ui(ui, |ui| {
                                for voice in voices {
                                    ui.selectable_value(&mut self.tts_voice, voice.to_string(), voice);
                                }
                            });

                        ui.add_space(10.0);

                        ui.label("Speed:");
                        ui.add_space(5.0);

                        ui.add(
                            Slider::new(&mut self.tts_speed, 0.5..=2.0)
                                .step_by(0.1)
                                .suffix("x")
                                .show_value(true),
                        );

                        ui.add_space(10.0);

                        ui.label("Volume:");
                        ui.add_space(5.0);

                        ui.add(
                            Slider::new(&mut self.tts_volume, 0.0..=10.0)
                                .step_by(0.5)
                                .show_value(true),
                        );

                        ui.add_space(15.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.heading("Cache Management");
                        ui.add_space(10.0);

                        ui.label(format!("Translation cache: {} entries", translation_cache.as_ref().map_or(0, |c| c.len())));
                        ui.add_space(5.0);

                        if ui
                            .button("Clear Translation Cache")
                            .clicked()
                        {
                            settings_changed = Some(SettingsChange::ClearTranslationCache);
                        }

                        ui.add_space(10.0);

                        ui.label(format!("Audio cache: {} files", audio_cache_len));
                        ui.add_space(5.0);

                        if ui.button("Clear Audio Cache").clicked() {
                            settings_changed = Some(SettingsChange::ClearAudioCache);
                        }

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.vertical_centered(|ui| {
                            ui.horizontal(|ui| {
                                if ui.button("Apply Changes").clicked() {
                                    settings_changed = Some(SettingsChange::Theme(
                                        self.font_size,
                                        self.dark_theme,
                                    ));
                                }

                                if ui.button("Apply TTS Settings").clicked() {
                                    settings_changed = Some(SettingsChange::Tts(
                                        self.tts_voice.clone(),
                                        self.tts_speed,
                                        self.tts_volume,
                                    ));
                                }

                                if ui.button("Close").clicked() {
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
    Theme(f32, bool),
    Tts(String, f32, f32),
    ClearTranslationCache,
    ClearAudioCache,
}


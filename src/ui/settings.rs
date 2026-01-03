use crate::utils::cache::TranslationCache;
use crate::utils::config::AppConfig;
use egui::{self, *};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

pub struct SettingsPanel {
    pub font_size: f32,
    pub theme_preference: ThemePreference,
    pub tts_voice: String,
    pub tts_speed: f32,
    pub tts_volume: f32,
    pub enable_keyword_analysis: bool,
    pub think_enable: bool,
    pub coding_plan: bool,
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
            theme_preference: ThemePreference::Dark,
            tts_voice: "Tongtong".to_string(),
            tts_speed: 1.0,
            tts_volume: 1.0,
            enable_keyword_analysis: false,
            think_enable: true,
            coding_plan: true,
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
        think_enable: bool,
        coding_plan: bool,
    ) -> Self {
        SettingsPanel {
            font_size,
            theme_preference: if dark_theme {
                ThemePreference::Dark
            } else {
                ThemePreference::Light
            },
            tts_voice,
            tts_speed,
            tts_volume,
            enable_keyword_analysis,
            think_enable,
            coding_plan,
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

        // Track old values to detect changes
        let old_font_size = self.font_size;
        let old_theme_preference = self.theme_preference;
        let old_tts_voice = self.tts_voice.clone();
        let old_tts_speed = self.tts_speed;
        let old_tts_volume = self.tts_volume;
        let old_enable_keyword_analysis = self.enable_keyword_analysis;
        let old_think_enable = self.think_enable;
        let old_coding_plan = self.coding_plan;

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
                            ui.radio_value(&mut self.theme_preference, ThemePreference::Light, "🌞Light");
                            ui.add_space(8.0);
                            ui.radio_value(&mut self.theme_preference, ThemePreference::Dark, "🌑Dark");
                            ui.add_space(8.0);
                            ui.radio_value(&mut self.theme_preference, ThemePreference::System, "💻 System");
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
                            ui.checkbox(&mut self.enable_keyword_analysis, "");
                        });
                        ui.label(
                            RichText::new(
                                "When enabled, analyze professional terms and provide explanations during translation. Each term on a separate line.",
                            )
                            .size(12.0)
                            .weak()
                            .color(Color32::GRAY),
                        );
                        ui.add_space(12.0);

                        // Think Enable Toggle
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🌟Thinking Mode:").size(14.0));
                            ui.add_space(10.0);
                            ui.checkbox(&mut self.think_enable, "");
                        });
                        ui.label(
                            RichText::new(
                                "When enabled, TTS will use thinking mode for better speech generation quality.",
                            )
                            .size(12.0)
                            .weak()
                            .color(Color32::GRAY),
                        );
                        ui.add_space(12.0);

                        // Coding Plan Toggle
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("💻Coding Plan Mode:").size(14.0));
                            ui.add_space(10.0);
                            ui.checkbox(&mut self.coding_plan, "");
                        });
                        ui.label(
                            RichText::new(
                                "When enabled, TTS will use coding plan mode for optimized speech generation.",
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
                                    RichText::new("Clear Translation Cache").size(13.0),
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
                                egui::Button::new(RichText::new("Clear Audio Cache").size(13.0))
                                    .corner_radius(6.0),
                            )
                            .clicked()
                        {
                            settings_changed = Some(SettingsChange::ClearAudioCache);
                        }

                        ui.add_space(25.0);
                        ui.separator();
                        ui.add_space(15.0);
                    });
                });
            });

        // Detect changes and apply immediately
        if self.font_size != old_font_size {
            settings_changed = Some(SettingsChange::FontSize(self.font_size));
        } else if self.theme_preference != old_theme_preference {
            settings_changed = Some(SettingsChange::Theme(self.theme_preference));
        } else if self.tts_voice != old_tts_voice {
            settings_changed = Some(SettingsChange::TtsVoice(self.tts_voice.clone()));
        } else if self.tts_speed != old_tts_speed {
            settings_changed = Some(SettingsChange::TtsSpeed(self.tts_speed));
        } else if self.tts_volume != old_tts_volume {
            settings_changed = Some(SettingsChange::TtsVolume(self.tts_volume));
        } else if self.enable_keyword_analysis != old_enable_keyword_analysis {
            settings_changed = Some(SettingsChange::KeywordAnalysis(
                self.enable_keyword_analysis,
            ));
        } else if self.think_enable != old_think_enable {
            settings_changed = Some(SettingsChange::ThinkEnable(self.think_enable));
        } else if self.coding_plan != old_coding_plan {
            settings_changed = Some(SettingsChange::CodingPlan(self.coding_plan));
        }

        (self.show_panel, settings_changed)
    }

    pub fn toggle_panel(&mut self) {
        self.show_panel = !self.show_panel;
    }
}

#[derive(Debug, Clone)]
pub enum SettingsChange {
    FontSize(f32),
    Theme(ThemePreference),
    TtsVoice(String),
    TtsSpeed(f32),
    TtsVolume(f32),
    KeywordAnalysis(bool),
    ThinkEnable(bool),
    CodingPlan(bool),
    ClearTranslationCache,
    ClearAudioCache,
}

//! Main display panel for showing source text and translations.
//!
//! This module provides the central UI component that displays
//! the input text and streaming translation results.

use crate::services::audio::PlaybackState;
use egui::*;

/// Display panel showing source text and translation results.
#[derive(Default)]
pub struct DisplayPanel {
    input_text: String,
    pub translation: String,
    is_translating: bool,
    error_message: Option<String>,

    // TTS and playback state
    source_tts_converting: bool,
    source_audio_path: Option<String>,
    translation_tts_converting: bool,
    translation_audio_path: Option<String>,
    playback_state: PlaybackState,
}

impl DisplayPanel {
    /// Sets the input text to display.
    pub fn set_input(&mut self, text: String) {
        self.input_text = text;
        self.error_message = None;
    }

    /// Gets input text
    pub fn input_text(&self) -> &str {
        &self.input_text
    }

    /// Appends a chunk of translation text (for streaming).
    pub fn update_translation(&mut self, chunk: String) {
        self.translation.push_str(&chunk);
    }

    /// Clears the translation text.
    pub fn clear_translation(&mut self) {
        self.translation.clear();
        self.error_message = None;
        // Clear audio paths when starting new translation
        self.source_audio_path = None;
        self.translation_audio_path = None;
    }

    /// Sets whether a translation is in progress.
    pub fn set_translating(&mut self, translating: bool) {
        self.is_translating = translating;
    }

    /// Sets an error message to display.
    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
    }

    /// Sets the source TTS conversion state
    pub fn set_source_tts_converting(&mut self, converting: bool) {
        self.source_tts_converting = converting;
        if converting {
            self.source_audio_path = None;
        }
    }

    /// Sets the translation TTS conversion state
    pub fn set_translation_tts_converting(&mut self, converting: bool) {
        self.translation_tts_converting = converting;
        if converting {
            self.translation_audio_path = None;
        }
    }

    /// Sets the source audio path
    pub fn set_source_audio_path(&mut self, path: Option<String>) {
        self.source_audio_path = path;
    }

    /// Sets the translation audio path
    pub fn set_translation_audio_path(&mut self, path: Option<String>) {
        self.translation_audio_path = path;
    }

    /// Updates the playback state
    pub fn set_playback_state(&mut self, state: PlaybackState) {
        self.playback_state = state;
    }

    /// Gets whether source audio is converting
    pub fn is_source_converting(&self) -> bool {
        self.source_tts_converting
    }

    /// Gets whether translation audio is converting
    pub fn is_translation_converting(&self) -> bool {
        self.translation_tts_converting
    }

    /// Creates a styled button for audio playback
    fn create_audio_button(
        &self,
        ui: &mut egui::Ui,
        converting: bool,
        audio_path: Option<&str>,
        _is_source: bool,
        enabled: bool,
    ) -> egui::Response {
        let button_text = if converting {
            format!("⏳ Converting")
        } else if let Some(path) = audio_path {
            if matches!(self.playback_state, PlaybackState::Playing(ref p) if p == path) {
                "⏸ Stop".to_string()
            } else {
                format!("▶ Play")
            }
        } else {
            "🔇 No Audio".to_string()
        };

        let button = egui::Button::new(RichText::new(button_text).size(12.0)).corner_radius(8.0);

        ui.add_enabled(enabled && !converting && audio_path.is_some(), button)
    }

    /// Creates a styled frame for text display.
    fn create_text_frame(&self, ui: &Ui) -> Frame {
        Frame::NONE
            .stroke(Stroke::new(
                1.5,
                ui.visuals()
                    .widgets
                    .noninteractive
                    .bg_stroke
                    .color
                    .gamma_multiply(0.8),
            ))
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(Margin::symmetric(16, 12))
            .corner_radius(8.0)
    }

    /// Renders the display panel UI.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context
    /// * `font_size` - Font size for text display
    ///
    /// # Returns
    ///
    /// (play_source_clicked, source_audio_to_play, play_translation_clicked, translation_audio_to_play,
    ///  start_source_tts, start_translation_tts, cancel_source_tts, cancel_translation_tts)
    pub fn ui(
        &mut self,
        ctx: &Context,
        font_size: f32,
    ) -> (
        bool,
        Option<String>,
        bool,
        Option<String>,
        bool,
        bool,
        bool,
        bool,
    ) {
        let mut play_source_clicked = false;
        let mut source_audio_to_play = None;
        let mut play_translation_clicked = false;
        let mut translation_audio_to_play = None;
        let mut start_source_tts = false;
        let mut start_translation_tts = false;
        let mut cancel_source_tts = false;
        let mut cancel_translation_tts = false;

        CentralPanel::default().show(ctx, |ui| {
            ui.add_space(16.0);

            // Calculate responsive heights based on available space
            let available_height = ui.available_height() - 20.0;
            let panel_height = (available_height / 2.0).max(150.0) - 16.0; // Ensure minimum height

            ui.vertical(|ui| {
                // Source Text section with audio controls
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("📄 Source Text")
                            .strong()
                            .size(font_size * 1.1),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);

                        // TTS Convert button (always enabled)
                        if !self.source_tts_converting && !self.input_text.trim().is_empty() {
                            let btn = egui::Button::new(RichText::new("🔊 Convert").size(12.0))
                                .corner_radius(6.0);
                            if ui.add(btn).on_hover_text("Convert text to audio").clicked() {
                                start_source_tts = true;
                            }
                        }

                        ui.add_space(8.0);

                        // Cancel TTS button (only shown during conversion)
                        if self.source_tts_converting {
                            let btn = egui::Button::new(RichText::new("❌ Cancel").size(12.0))
                                .corner_radius(6.0);
                            if ui.add(btn).on_hover_text("Cancel TTS conversion").clicked() {
                                cancel_source_tts = true;
                            }
                        }

                        ui.add_space(8.0);

                        // Play/Stop audio button
                        if self
                            .create_audio_button(
                                ui,
                                self.source_tts_converting,
                                self.source_audio_path.as_deref(),
                                true,
                                true,
                            )
                            .clicked()
                        {
                            play_source_clicked = true;
                            if let Some(path) = self.source_audio_path.clone() {
                                source_audio_to_play = Some(path);
                            }
                        }
                    });
                });
                ui.add_space(10.0);

                self.create_text_frame(ui).show(ui, |ui| {
                    ScrollArea::vertical()
                        .max_height(panel_height)
                        .id_salt("source_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let mut source_edit = self.input_text.clone();
                            TextEdit::multiline(&mut source_edit)
                                .font(FontId::new(font_size, FontFamily::Proportional))
                                .desired_width(f32::INFINITY)
                                .desired_rows(5)
                                .frame(false)
                                .lock_focus(true)
                                .show(ui);
                        });
                });

                ui.add_space(16.0);

                // Translation section with audio controls
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("🌐 Translation")
                            .strong()
                            .size(font_size * 1.1),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);

                        // TTS Convert button (only enabled after translation completes)
                        let translation_tts_enabled =
                            !self.is_translating && !self.translation.is_empty();
                        if !self.translation_tts_converting && translation_tts_enabled {
                            let btn = egui::Button::new(RichText::new("🔊 Convert").size(12.0))
                                .corner_radius(6.0);
                            if ui
                                .add(btn)
                                .on_hover_text("Convert translation to audio")
                                .clicked()
                            {
                                start_translation_tts = true;
                            }
                        }

                        ui.add_space(8.0);

                        // Cancel TTS button (only shown during conversion)
                        if self.translation_tts_converting {
                            let btn = egui::Button::new(RichText::new("❌ Cancel").size(12.0))
                                .corner_radius(6.0);
                            if ui.add(btn).on_hover_text("Cancel TTS conversion").clicked() {
                                cancel_translation_tts = true;
                            }
                        }

                        ui.add_space(8.0);

                        // Play/Stop audio button
                        if self
                            .create_audio_button(
                                ui,
                                self.translation_tts_converting,
                                self.translation_audio_path.as_deref(),
                                false,
                                translation_tts_enabled,
                            )
                            .clicked()
                        {
                            play_translation_clicked = true;
                            if let Some(path) = self.translation_audio_path.clone() {
                                translation_audio_to_play = Some(path);
                            }
                        }
                    });
                });
                ui.add_space(8.0);

                self.create_text_frame(ui).show(ui, |ui| {
                    ScrollArea::vertical()
                        .max_height(panel_height)
                        .id_salt("translation_scroll")
                        .auto_shrink([false, false])
                        .stick_to_bottom(true) // Auto-scroll to bottom as new content arrives
                        .show(ui, |ui| {
                            // Show error message if present
                            if let Some(error) = &self.error_message {
                                ui.colored_label(
                                    ui.visuals().error_fg_color,
                                    RichText::new(format!("❌ Error: {}", error)).size(font_size),
                                );
                            } else if self.is_translating {
                                // Show loading indicator while translating
                                if self.translation.is_empty() {
                                    ui.horizontal(|ui| {
                                        ui.spinner();
                                        ui.label(
                                            RichText::new("Translating...")
                                                .size(font_size)
                                                .color(ui.visuals().weak_text_color()),
                                        );
                                    });
                                } else {
                                    // Show partial translation
                                    let mut display_text = self.translation.clone();
                                    TextEdit::multiline(&mut display_text)
                                        .font(FontId::new(font_size, FontFamily::Proportional))
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(5)
                                        .frame(false)
                                        .lock_focus(true)
                                        .show(ui);
                                }
                            } else if self.translation.is_empty() {
                                // Show placeholder when empty
                                let display_text = "Translation will appear here...";
                                ui.colored_label(
                                    ui.visuals().weak_text_color(),
                                    RichText::new(display_text).size(font_size * 0.9).italics(),
                                );
                            } else {
                                // Show completed translation
                                let mut display_text = self.translation.clone();
                                TextEdit::multiline(&mut display_text)
                                    .font(FontId::new(font_size, FontFamily::Proportional))
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(5)
                                    .frame(false)
                                    .lock_focus(true)
                                    .show(ui);
                            }
                        });
                });
            });
        });

        (
            play_source_clicked,
            source_audio_to_play,
            play_translation_clicked,
            translation_audio_to_play,
            start_source_tts,
            start_translation_tts,
            cancel_source_tts,
            cancel_translation_tts,
        )
    }
}

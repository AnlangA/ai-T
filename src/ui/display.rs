//! Main display panel for showing source text and translations.
//!
//! This module provides the central UI component that displays
//! the input text and streaming translation results.

use egui::*;

/// Display panel showing source text and translation results.
#[derive(Default)]
pub struct DisplayPanel {
    input_text: String,
    pub translation: String,
    is_translating: bool,
    error_message: Option<String>,
}

impl DisplayPanel {
    /// Sets the input text to display.
    pub fn set_input(&mut self, text: String) {
        self.input_text = text;
        self.error_message = None;
    }

    /// Appends a chunk of translation text (for streaming).
    pub fn update_translation(&mut self, chunk: String) {
        self.translation.push_str(&chunk);
    }

    /// Clears the translation text.
    pub fn clear_translation(&mut self) {
        self.translation.clear();
        self.error_message = None;
    }

    /// Sets whether a translation is in progress.
    pub fn set_translating(&mut self, translating: bool) {
        self.is_translating = translating;
    }

    /// Sets an error message to display.
    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
    }

    /// Creates a styled frame for text display.
    fn create_text_frame(&self, ui: &Ui) -> Frame {
        Frame::NONE
            .stroke(Stroke::new(
                1.0,
                ui.visuals().widgets.noninteractive.bg_stroke.color,
            ))
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(Margin::symmetric(12, 12))
            .corner_radius(4.0)
    }

    /// Renders the display panel UI.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context
    /// * `font_size` - Font size for text display
    pub fn ui(&mut self, ctx: &Context, font_size: f32) {
        CentralPanel::default().show(ctx, |ui| {
            ui.add_space(16.0);

            // Calculate responsive heights based on available space
            let available_height = ui.available_height() - 20.0;
            let panel_height = (available_height / 2.0).max(150.0) - 16.0; // Ensure minimum height

            ui.vertical(|ui| {
                ui.label(RichText::new("Source Text").strong().size(font_size * 1.1));
                ui.add_space(8.0);

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

                ui.label(RichText::new("Translation").strong().size(font_size * 1.1));
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
    }
}

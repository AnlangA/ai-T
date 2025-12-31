use egui::*;

#[derive(Default)]
pub struct DisplayPanel {
    input_text: String,
    pub translation: String,
    is_translating: bool,
}

impl DisplayPanel {
    pub fn set_input(&mut self, text: String) {
        self.input_text = text;
    }

    pub fn update_translation(&mut self, chunk: String) {
        self.translation.push_str(&chunk);
    }

    pub fn clear_translation(&mut self) {
        self.translation.clear();
    }

    pub fn set_translating(&mut self, translating: bool) {
        self.is_translating = translating;
    }

    fn create_text_frame(&self, ui: &Ui) -> Frame {
        Frame::NONE
            .stroke(Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color))
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(Margin::symmetric(12, 12))
            .corner_radius(4.0)
    }

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
                            let mut display_text = self.translation.clone();
                            if self.is_translating && display_text.is_empty() {
                                display_text = "Translating...".to_string();
                            } else if self.translation.is_empty() && !self.is_translating {
                                display_text = "Translation will appear here...".to_string();
                            }

                            TextEdit::multiline(&mut display_text)
                                .font(FontId::new(font_size, FontFamily::Proportional))
                                .desired_width(f32::INFINITY)
                                .desired_rows(5)
                                .frame(false)
                                .lock_focus(true)
                                .show(ui);
                        });
                });
            });
        });
    }
}

use crate::utils::config::AppConfig;
use egui::*;

pub struct Sidebar {
    api_key: String,
    target_language: String,
    source_text: String,
    languages: Vec<&'static str>,
}

impl Default for Sidebar {
    fn default() -> Self {
        let config = AppConfig::default();
        Sidebar {
            api_key: config.api_key,
            target_language: config.target_language,
            source_text: String::new(),
            languages: AppConfig::get_supported_languages(),
        }
    }
}

impl Sidebar {
    pub fn ui(&mut self, ctx: &Context, is_translating: bool) -> (bool, bool, Option<String>) {
        let mut translate_requested = false;
        let mut cancel_requested = false;
        let mut api_key_to_save = None;

        SidePanel::right("sidebar")
            .default_width(300.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Settings");
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                ui.label("API Key:");
                ui.add_space(5.0);

                let key_response = ui.add(
                    TextEdit::singleline(&mut self.api_key)
                        .hint_text("Enter your Z.AI API key")
                        .password(true),
                );

                if key_response.lost_focus() || key_response.has_focus() {
                    api_key_to_save = Some(self.api_key.clone());
                }

                ui.add_space(15.0);

                ui.label("Target Language:");
                ui.add_space(5.0);

                egui::ComboBox::from_id_salt("language_selector")
                    .selected_text(&self.target_language)
                    .show_ui(ui, |ui| {
                        for lang in &self.languages {
                            ui.selectable_value(&mut self.target_language, lang.to_string(), *lang);
                        }
                    });

                ui.add_space(15.0);

                ui.label("Source Text:");
                ui.add_space(5.0);

                ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    let text_edit = TextEdit::multiline(&mut self.source_text)
                        .hint_text("Enter text to translate...")
                        .desired_width(f32::INFINITY);
                    ui.add_sized([ui.available_width(), 150.0], text_edit);
                });

                ui.add_space(15.0);

                ui.vertical_centered(|ui| {
                    if is_translating {
                        // Show cancel button during translation
                        if ui.button("Cancel").clicked() {
                            cancel_requested = true;
                        }
                    } else {
                        // Show translate button when not translating
                        let translate_btn = ui.add_enabled(
                            !self.source_text.is_empty() && !self.api_key.is_empty(),
                            Button::new("Translate"),
                        );

                        if translate_btn.clicked() {
                            translate_requested = true;
                        }
                    }
                });
            });

        (translate_requested, cancel_requested, api_key_to_save)
    }

    pub fn get_source_text(&self) -> String {
        self.source_text.clone()
    }

    pub fn get_api_key(&self) -> String {
        self.api_key.clone()
    }

    pub fn get_target_language(&self) -> String {
        self.target_language.clone()
    }

    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }

    pub fn set_target_language(&mut self, language: String) {
        self.target_language = language;
    }
}

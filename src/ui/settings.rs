use egui::{self, *};

pub struct SettingsPanel {
    pub font_size: f32,
    pub dark_theme: bool,
    show_panel: bool,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        SettingsPanel {
            font_size: 16.0,
            dark_theme: true,
            show_panel: false,
        }
    }
}

impl SettingsPanel {
    pub fn new(font_size: f32, dark_theme: bool) -> Self {
        SettingsPanel {
            font_size,
            dark_theme,
            show_panel: false,
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) -> (bool, Option<(f32, bool)>) {
        let mut theme_changed = None;
        let mut close_requested = false;

        Window::new("Settings")
            .collapsible(true)
            .resizable(false)
            .open(&mut self.show_panel)
            .default_size([300.0, 280.0])
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Appearance Settings");
                    ui.add_space(15.0);
                    ui.separator();
                    ui.add_space(15.0);

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

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);

                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Apply Changes").clicked() {
                                theme_changed = Some((self.font_size, self.dark_theme));
                            }

                            if ui.button("Close").clicked() {
                                close_requested = true;
                            }
                        });
                    });
                });
            });

        if close_requested {
            self.show_panel = false;
        }

        (self.show_panel, theme_changed)
    }

    pub fn toggle_panel(&mut self) {
        self.show_panel = !self.show_panel;
    }
}

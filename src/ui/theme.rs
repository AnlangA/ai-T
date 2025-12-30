use egui::{FontDefinitions, FontFamily, TextStyle, *};

pub struct Theme {
    pub dark: bool,
    pub font_size: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            dark: true,
            font_size: 16.0,
        }
    }
}

impl Theme {
    pub fn setup_fonts(&self, ctx: &Context) {
        let mut fonts = FontDefinitions::default();

        // Load STSong for Chinese and other CJK languages
        fonts.font_data.insert(
            "stsong".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../../fonts/STSong.ttf"))),
        );

        // Load Noto Serif KR for Korean language support
        fonts.font_data.insert(
            "noto_serif_kr".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../../fonts/NotoSerifKR-VariableFont_wght.ttf"))),
        );

        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "noto_serif_kr".to_owned());
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push("stsong".to_owned());

        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("noto_serif_kr".to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("stsong".to_owned());

        ctx.set_fonts(fonts);
    }

    pub fn apply_style(&self, ctx: &Context) {
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(self.font_size * 1.5, FontFamily::Proportional),
            ),
            (
                TextStyle::Body,
                FontId::new(self.font_size, FontFamily::Proportional),
            ),
            (
                TextStyle::Button,
                FontId::new(self.font_size, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(self.font_size * 0.85, FontFamily::Proportional),
            ),
        ]
        .into();

        ctx.set_style(style);
    }

    pub fn set_visuals(&self, ctx: &Context) {
        if self.dark {
            ctx.set_visuals(Visuals::dark());
        } else {
            ctx.set_visuals(Visuals::light());
        }
    }
}

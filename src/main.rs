#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod channel;
mod ui;
mod utils;

use eframe::egui;
use ui::TranslateApp;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([800.0, 500.0])
            .with_app_id("ai-translate"),
        ..Default::default()
    };

    eframe::run_native(
        "AI Translate Tool",
        options,
        Box::new(|cc| Ok(Box::new(TranslateApp::new(cc)))),
    )
}

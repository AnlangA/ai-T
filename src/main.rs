//! AI Translation Tool
//!
//! A cross-platform desktop application for AI-powered text translation
//! with streaming support. Uses the Z.AI API (GLM-4.7) for translations.
//!
//! # Features
//!
//! - Real-time streaming translation
//! - Multiple language support
//! - Configurable UI (font size, dark/light theme)
//! - Persistent configuration storage
//! - Structured logging with tracing
//!
//! # Usage
//!
//! Set the `RUST_LOG` environment variable to control logging:
//!
//! ```bash
//! RUST_LOG=debug ./ai-translate
//! RUST_LOG=ai_translate=trace ./ai-translate
//! ```

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod channel;
mod error;
mod services;
mod ui;
mod utils;

use eframe::egui;
use ui::TranslateApp;

fn main() -> Result<(), eframe::Error> {
    // Initialize tracing with RUST_LOG support
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting AI Translate Tool");

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

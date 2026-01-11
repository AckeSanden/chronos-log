// src/main.rs
// Entry point for the Work Tracker application

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console on Windows in release

mod app;
mod database;
mod models;
mod ui;

use app::WorkTrackerApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    // Initialize logging for debug builds
    #[cfg(debug_assertions)]
    env_logger::init();

    // Configure native options
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "Chronos Log - Work Activity Tracker",
        native_options,
        Box::new(|cc| Ok(Box::new(WorkTrackerApp::new(cc)))),
    )
}

/// Load application icon (returns empty icon data if not available)
fn load_icon() -> egui::IconData {
    // You can replace this with actual icon loading if desired
    egui::IconData::default()
}

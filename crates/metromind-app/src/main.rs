#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! MetroMind: brainstorm on a metro map, through a fisheye lens.

mod app;
mod document;
mod frame_plan;
mod input;
mod render;
mod settings;
mod ui;

use eframe::egui;

/// Usage: `metromind-app [map.metromind]`. Without a path, the last map reopens.
fn main() -> eframe::Result {
    let file = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("bundled icon is a valid PNG");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MetroMind")
            .with_icon(icon)
            .with_app_id(settings::APP_ID)
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([760.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        settings::APP_ID,
        options,
        Box::new(|cc| Ok(Box::new(app::MetroMindApp::new(cc, file)))),
    )
}

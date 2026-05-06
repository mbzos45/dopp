#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)] // it's an example

mod docker;
mod ui;

use ui::{MyApp, DEFAULT_HEIGHT, WINDOW_WIDTH};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, DEFAULT_HEIGHT]),
        ..Default::default()
    };
    eframe::run_native(
        "dopp",
        options,
        Box::new(|_| Ok(Box::new(MyApp::new()))),
    )
}

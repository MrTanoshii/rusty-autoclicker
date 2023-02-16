#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

use eframe::egui;

mod app;
mod defines;
mod gui;
mod types;
mod utils;

use crate::{
    app::RustyAutoClickerApp,
    defines::{APP_NAME, WINDOW_HEIGHT, WINDOW_WIDTH},
    utils::load_icon,
};

// When compiling natively
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions {
        always_on_top: true,
        decorated: true,
        initial_window_size: Some(egui::vec2(WINDOW_WIDTH, WINDOW_HEIGHT)),
        resizable: false,
        transparent: true,
        icon_data: Some(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        &format!("{} v{}", APP_NAME, env!("CARGO_PKG_VERSION")),
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(RustyAutoClickerApp::new(cc))
        }),
    );
}

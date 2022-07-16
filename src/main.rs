#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

use eframe::egui;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions {
        always_on_top: true,
        decorated: true,
        initial_window_size: Some(egui::vec2(550f32, 309f32)),
        resizable: false,
        transparent: true,
        ..Default::default()
    };
    eframe::run_native(
        "Rusty AutoClicker v2.0.0",
        native_options,
        Box::new(|cc| {
            let style = egui::Style {
                visuals: egui::Visuals::dark(),
                ..egui::Style::default()
            };
            cc.egui_ctx.set_style(style);
            Box::new(rusty_autoclicker::RustyAutoClickerApp::new(cc))
        }),
    );
}

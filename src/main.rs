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
    use eframe::egui::ViewportBuilder;

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: ViewportBuilder::default()
            .with_always_on_top()
            .with_decorations(true)
            .with_inner_size(egui::vec2(WINDOW_WIDTH, WINDOW_HEIGHT))
            .with_resizable(true)
            .with_transparent(true)
            .with_icon(load_icon()),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        &format!("{} v{}", APP_NAME, env!("CARGO_PKG_VERSION")),
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(RustyAutoClickerApp::new(cc)))
        }),
    ) {
        native_dialog::DialogBuilder::message()
            .set_level(native_dialog::MessageLevel::Error)
            .set_title("Failed to Initialize Graphics")
            .set_text(
                "Rusty AutoClicker could not start due to a graphics initialization error.\n\n\
                Please ensure your system has a compatible graphics driver installed and supports Vulkan, Metal or DirectX 12.\n\n\
                If the problem persists, try updating your graphics drivers or running the application on a different machine.",
            )
            .alert()
            .show()
            .unwrap();
    };
}

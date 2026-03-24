use eframe::egui::{self};

use crate::{RustyAutoClickerApp, types::AppMode};

impl RustyAutoClickerApp {
    pub fn show_topbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::MenuBar::new().ui(ui, |ui| {
                if self.is_autoclicking {
                    if ui
                        .button(format!("🖱 STOP ({})", self.key_autoclick.unwrap()))
                        .clicked()
                    {
                        self.is_autoclicking = false;
                    };
                } else {
                    if self.hotkey_window_open {
                        ui.disable();
                    }
                    let text: String = if let Some(hotkey) = self.key_autoclick {
                        format!("🖱 START ({hotkey})")
                    } else {
                        "🖱 START".to_string()
                    };
                    if ui.button(text).clicked() {
                        self.is_autoclicking = true
                    }
                }

                ui.separator();
                ui.label("Settings: ");

                if ui
                    .add_enabled(!self.is_autoclicking, egui::Button::new("⌨ Hotkeys"))
                    .clicked()
                {
                    self.hotkey_window_open = true
                };

                ui.separator();
                ui.label("App Mode: ");

                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                ui.selectable_value(&mut self.app_mode, AppMode::Bot, "🖥 Bot")
                    .on_hover_text("Autoclick as fast as possible");
                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                ui.selectable_value(&mut self.app_mode, AppMode::Humanlike, "😆 Humanlike")
                    .on_hover_text("Autoclick emulating human clicking");
            });
        });
    }

    pub fn show_bottombar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                    ui.label(" and ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label("powered by ");
                    ui.hyperlink_to(
                        "rusty-autoclicker",
                        "https://github.com/MrTanoshii/rusty-autoclicker",
                    );
                    ui.separator();
                    egui::warn_if_debug_build(ui);
                });
            });
        });
    }
}

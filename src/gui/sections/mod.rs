use device_query::{Keycode, MouseState};
use eframe::egui::{self};

use crate::RustyAutoClickerApp;

mod bars;
mod buttons;
mod click_config;

impl RustyAutoClickerApp {
    pub fn show_movement_delay(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Movement delay (Humanlike only)");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.label("ms");
                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                ui.add(
                    egui::TextEdit::singleline(&mut self.movement_ms_str)
                        .desired_width(40.0f32)
                        .hint_text("20"),
                );

                ui.label("sec");
                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                ui.add(
                    egui::TextEdit::singleline(&mut self.movement_sec_str)
                        .desired_width(40.0f32)
                        .hint_text("0"),
                );
            });
        });
    }

    pub fn show_infos(&self, ui: &mut egui::Ui, mouse: &MouseState, keys: &[Keycode]) {
        let mouse_txt = format!("Mouse position: {:?}", mouse.coords);
        ui.label(mouse_txt);
        let key_txt = format!("Key pressed: {keys:?}");
        ui.label(key_txt);
        let extra_buttons_pressed = mouse
            .button_pressed
            .iter()
            .enumerate()
            .skip(4)
            .map(|(button_number, pressed)| format!("{button_number:?}-{pressed:?}"))
            .collect::<Vec<String>>()
            .join(" ");

        ui.label(format!(
            "Mouse pressed: L-{:?} R-{:?} M-{:?} {}",
            mouse.button_pressed[1],
            mouse.button_pressed[2],
            mouse.button_pressed[3],
            &extra_buttons_pressed
        ));
    }

    pub fn show_autoclicker(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            if self.is_autoclicking {
                if self.hotkey_window_open {
                    ui.disable();
                }
                if ui
                    .add_sized(
                        [120.0f32, 38.0f32],
                        egui::widgets::Button::new(format!(
                            "🖱 STOP ({})",
                            self.key_autoclick.unwrap()
                        )),
                    )
                    .clicked()
                {
                    self.is_autoclicking = false;
                };
            } else {
                if self.hotkey_window_open {
                    ui.disable();
                }
                let text: String = if self.key_autoclick.is_none() {
                    "🖱 START".to_string()
                } else {
                    format!("🖱 START ({})", self.key_autoclick.unwrap())
                };
                if ui
                    .add_sized([120.0f32, 38.0f32], egui::widgets::Button::new(text))
                    .clicked()
                {
                    // Start autoclick, first click is delayed
                    Self::start_autoclick(self, 0u64);
                    self.is_autoclicking = true
                }
            }
        });
    }
}

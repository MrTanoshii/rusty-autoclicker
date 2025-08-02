use eframe::egui::{self, Context};

use crate::{
    RustyAutoClickerApp,
    types::{ClickPosition, ClickType},
};

impl RustyAutoClickerApp {
    pub fn show_click_interval(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Click Interval");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.label("ms");
                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                ui.add(
                    egui::TextEdit::singleline(&mut self.ms_str)
                        .desired_width(40.0f32)
                        .hint_text("0"),
                );

                ui.label("sec");
                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                ui.add(
                    egui::TextEdit::singleline(&mut self.sec_str)
                        .desired_width(40.0f32)
                        .hint_text("0"),
                );

                ui.label("min");
                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                ui.add(
                    egui::TextEdit::singleline(&mut self.min_str)
                        .desired_width(40.0f32)
                        .hint_text("0"),
                );

                ui.label("hr");
                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                ui.add(
                    egui::TextEdit::singleline(&mut self.hr_str)
                        .desired_width(40.0f32)
                        .hint_text("0"),
                );
            });
        });
    }

    pub fn show_click_type(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Click Type");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                ui.selectable_value(&mut self.click_type, ClickType::Single, "Single");
                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                ui.selectable_value(&mut self.click_type, ClickType::Double, "Double");
                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
            });
        });
    }

    pub fn show_click_amount(&mut self, ui: &mut egui::Ui, click_amount: u64) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Click Amount (0 = forever)");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                ui.add(
                    egui::TextEdit::singleline(&mut self.click_amount_str)
                        .desired_width(40.0f32)
                        .hint_text("0"),
                );
                if self.is_autoclicking && click_amount > 0u64 {
                    let remaining_clicks = click_amount.saturating_sub(self.click_counter);
                    let remaining_text = format!("Remaining {remaining_clicks:?}");
                    ui.label(remaining_text);
                }
            });
        });
    }

    pub fn show_click_position(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Click Position");
            if self.is_autoclicking || self.hotkey_window_open {
                ui.disable();
            };
            if ui
                .add_sized([80.0f32, 16.0f32], egui::widgets::Button::new("Set Coords"))
                .clicked()
            {
                Self::enter_coordinate_setting(self, ctx);
            };
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                ui.add(
                    egui::TextEdit::singleline(&mut self.click_y_str)
                        .desired_width(50.0f32)
                        .hint_text("0"),
                );
                ui.label("Y");
                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                ui.add(
                    egui::TextEdit::singleline(&mut self.click_x_str)
                        .desired_width(50.0f32)
                        .hint_text("0"),
                );
                ui.label("X");

                if self.is_autoclicking || self.hotkey_window_open {
                    ui.disable();
                };
                if ui
                    .selectable_value(&mut self.click_position, ClickPosition::Coord, "Coords")
                    .clicked()
                {
                    self.key_pressed_set_coord = true;
                };
                ui.separator();
                if ui
                    .selectable_value(&mut self.click_position, ClickPosition::Mouse, "Mouse")
                    .clicked()
                {
                    self.key_pressed_set_coord = false;
                };
            });
        });
    }
}

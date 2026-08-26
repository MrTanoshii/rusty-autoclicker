use eframe::egui;

use crate::{
    RustyAutoClickerApp,
    defines::DEFAULT_RANDOM_OFFSET_STR,
    types::{ClickPosition, ClickType},
};

impl RustyAutoClickerApp {
    pub fn show_click_interval(&mut self, ui: &mut egui::Ui) {
        // Base widths are the ORIGINAL fixed pixel values from before any
        // of this responsive work — scaling by `self.ui_scale` reproduces
        // that exact look at the default window size (scale == 1.0) and
        // shrinks/grows every field in lockstep with the font and
        // everything else as the window is resized.
        let field_w = 40.0 * self.ui_scale;
        let label = "Click Interval";

        super::wrapping_field_row(
            ui,
            self,
            |_state, ui| {
                ui.label(label);
            },
            |state, ui| {
                ui.label("ms");
                state.disable_if_busy(ui);
                ui.add(
                    egui::TextEdit::singleline(&mut state.ms_str)
                        .desired_width(field_w)
                        .hint_text("0"),
                );

                ui.label("sec");
                ui.add(
                    egui::TextEdit::singleline(&mut state.sec_str)
                        .desired_width(field_w)
                        .hint_text("0"),
                );

                ui.label("min");
                ui.add(
                    egui::TextEdit::singleline(&mut state.min_str)
                        .desired_width(field_w)
                        .hint_text("0"),
                );

                ui.label("hr");
                ui.add(
                    egui::TextEdit::singleline(&mut state.hr_str)
                        .desired_width(field_w)
                        .hint_text("0"),
                );
            },
        );
    }

    /// "Random Offset (+/-)" — an optional +/- jitter (ms) applied on top
    /// of the base Click Interval, matching the concept from OP Auto
    /// Clicker's "Random offset +-" control. Off by default; the field is
    /// only editable while the checkbox is on. Actually wired into click
    /// timing in `app.rs::roll_next_interval` / `current_interval_ms`, not
    /// just cosmetic.
    pub fn show_random_offset(&mut self, ui: &mut egui::Ui) {
        let field_w = 40.0 * self.ui_scale;
        let label = "Random Offset (+/-)";

        super::wrapping_field_row(
            ui,
            self,
            |state, ui| {
                state.disable_if_busy(ui);
                ui.checkbox(&mut state.random_offset_enabled, label);
            },
            |state, ui| {
                ui.label("ms");
                ui.add_enabled(
                    state.random_offset_enabled,
                    egui::TextEdit::singleline(&mut state.random_offset_str)
                        .desired_width(field_w)
                        .hint_text(DEFAULT_RANDOM_OFFSET_STR),
                );
            },
        );
    }

    pub fn show_click_type(&mut self, ui: &mut egui::Ui) {
        let label = "Click Type";

        super::wrapping_field_row(
            ui,
            self,
            |_state, ui| {
                ui.label(label);
            },
            |state, ui| {
                state.disable_if_busy(ui);
                ui.selectable_value(&mut state.click_type, ClickType::Double, "Double");
                ui.selectable_value(&mut state.click_type, ClickType::Single, "Single");
            },
        );
    }

    pub fn show_click_amount(&mut self, ui: &mut egui::Ui, click_amount: u64) {
        let field_w = 40.0 * self.ui_scale;
        let label = "Click Amount (0 = forever)";

        super::wrapping_field_row(
            ui,
            self,
            |_state, ui| {
                ui.label(label);
            },
            |state, ui| {
                state.disable_if_busy(ui);
                ui.add(
                    egui::TextEdit::singleline(&mut state.click_amount_str)
                        .desired_width(field_w)
                        .hint_text("0"),
                );
                if state.is_autoclicking() && click_amount > 0u64 {
                    let remaining_clicks = click_amount.saturating_sub(state.click_counter);
                    let remaining_text = format!("Remaining {remaining_clicks:?}");
                    ui.label(remaining_text);
                }
            },
        );
    }

    pub fn show_click_position(&mut self, ui: &mut egui::Ui) {
        let scale = self.ui_scale;
        let coord_field_w = 50.0 * scale;
        let set_coords_size = egui::vec2(80.0 * scale, 16.0 * scale);

        super::wrapping_field_row(
            ui,
            self,
            |state, ui| {
                ui.label("Click Mode");
                state.disable_if_busy(ui);

                // Mode toggle lives on the left, where "Set Coords" used to
                // be — this is the actual "will it click at a fixed X/Y or
                // wherever the mouse currently is" switch, so it gets the
                // more prominent left-side spot instead of being easy to
                // mistake for a coordinate readout next to the X/Y fields.
                ui.selectable_value(&mut state.click_position, ClickPosition::Mouse, "Mouse")
                    .on_hover_text(
                        "Click wherever the mouse cursor currently is, instead of a fixed \
                         saved position. Useful for autoclicking under manual mouse control."
                    );

                // Reverted to the original tight spacing (no add_space
                // padding) per feedback. `ui.separator()` only ever draws a
                // 1px hairline with no way to thicken it directly, so this
                // paints a small rect instead to get a genuinely bolder
                // divider.
                let divider_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
                let (divider_rect, _) = ui.allocate_exact_size(
                    egui::vec2(3.0 * scale, 18.0 * scale),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(divider_rect, 0.0, divider_color);

                ui.selectable_value(&mut state.click_position, ClickPosition::Coord, "Fixed")
                    .on_hover_text(
                        "Always click at the saved X/Y coordinates below, regardless of \
                         where the mouse cursor currently is. Use \"Set Coords\" to pick \
                         that fixed position."
                    );
            },
            |state, ui| {
                state.disable_if_busy(ui);
                ui.add(
                    egui::TextEdit::singleline(&mut state.click_y_str)
                        .desired_width(coord_field_w)
                        .hint_text("0"),
                );
                ui.label("Y");
                ui.add(
                    egui::TextEdit::singleline(&mut state.click_x_str)
                        .desired_width(coord_field_w)
                        .hint_text("0"),
                );
                ui.label("X");
                if ui
                    .add_sized(set_coords_size, egui::widgets::Button::new("Set Coords"))
                    .clicked()
                {
                    state.enter_coordinate_setting(ui.ctx());
                };
            },
        );
    }
}

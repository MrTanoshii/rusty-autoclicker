use device_query::{Keycode, MouseState};
use eframe::egui;

use crate::{
    RustyAutoClickerApp,
    defines::{MOUSE_TWEEN_SPEED_MAX_PX_S, MOUSE_TWEEN_SPEED_MIN_PX_S},
    types::InteractionMode,
};

mod bars;
mod buttons;
mod click_config;

/// Renders a settings row as a left side (a label, and optionally a few
/// more widgets) with a right-aligned control group — matching the
/// original fixed layout at normal window sizes.
///
/// The right-aligned group is given an EXPLICIT bounded size via
/// `allocate_ui_with_layout`, sized to whatever's ACTUALLY left on the row
/// after the left side was drawn (`ui.available_width()`, read right after
/// `add_left` runs) — rather than an open-ended
/// `ui.with_layout(Layout::right_to_left(..))`. The latter anchors its
/// content to the full max_rect it inherits, which is NOT reliably
/// "whatever's left after the label" — at small window sizes the
/// right-aligned group could still reach past the window edge and clip
/// even though there was technically enough room for it. This was the
/// actual cause of "Humanlike"/"px/s"/etc. getting cut off at the
/// enforced minimum window size. Bounding the box to the row's real
/// remaining width makes that structurally impossible — the box can
/// never extend past the window edge because it's sized FROM that edge —
/// while still rendering flush-right exactly like the original fixed
/// layout when there's room to spare.
///
/// Deliberately uses a plain `ui.horizontal` here, NOT
/// `ui.horizontal_wrapped` — under a wrapping layout, `available_width()`
/// reports the whole line's width rather than what's left after the
/// cursor's current position (egui can't know in advance whether a given
/// item will wrap), so the "remaining width" read after `add_left` would
/// come back far too large. A plain (non-wrapping) horizontal layout's
/// `available_width()` genuinely shrinks as the cursor advances, so it's
/// the only one of the two that gives an accurate number here. The
/// trade-off is that at the very edge of the enforced minimum window size
/// the right-aligned group can end up squeezed rather than dropping to
/// its own line — but since its box can never exceed the true remaining
/// width, it can never clip off the window either.
///
/// Takes `state` (almost always `self`) as an explicit parameter rather
/// than letting `add_left`/`add_right` capture it by closure — two
/// closures that each capture `self` can't both be constructed as
/// arguments to the same call even though only one of them ever actually
/// runs (the borrow checker can't see that), so `state` is threaded
/// through the callback signature instead.
pub(super) fn wrapping_field_row<T>(
    ui: &mut egui::Ui,
    state: &mut T,
    add_left: impl FnOnce(&mut T, &mut egui::Ui),
    add_right: impl FnOnce(&mut T, &mut egui::Ui),
) {
    let right_height = ui.spacing().interact_size.y;
    ui.horizontal(|ui| {
        add_left(state, ui);
        let right_width = ui.available_width().max(0.0);
        ui.allocate_ui_with_layout(
            egui::vec2(right_width, right_height),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| add_right(state, ui),
        );
    });
}

impl RustyAutoClickerApp {
    pub fn show_movement_speed(&mut self, ui: &mut egui::Ui) {
        // Base field width is the ORIGINAL fixed pixel value from before
        // any of this responsive work — scaling it by `self.ui_scale`
        // reproduces that exact original look at the default window size
        // (scale == 1.0) and shrinks/grows it in lockstep with the font
        // and everything else as the window is resized, rather than
        // computing an independent fraction of whatever width happens to
        // be left in the row.
        let field_w = 40.0 * self.ui_scale;
        let label = "Movement speed (Humanlike only)";

        wrapping_field_row(
            ui,
            self,
            |_state, ui| {
                ui.label(label);
            },
            |state, ui| {
                ui.label("px/s");
                state.disable_if_busy(ui);
                ui.add(
                    egui::TextEdit::singleline(&mut state.speed_max_str)
                        .desired_width(field_w)
                        .hint_text(MOUSE_TWEEN_SPEED_MAX_PX_S.to_string()),
                );

                ui.label("to");
                ui.add(
                    egui::TextEdit::singleline(&mut state.speed_min_str)
                        .desired_width(field_w)
                        .hint_text(MOUSE_TWEEN_SPEED_MIN_PX_S.to_string()),
                );
            },
        );
    }

    pub fn show_infos(&self, ui: &mut egui::Ui, mouse: &MouseState, keys: &[Keycode]) {
        let mouse_txt = format!("Mouse position: {:?}", mouse.coords);
        ui.label(mouse_txt);

        let buttons_pressed = mouse
            .button_pressed
            .iter()
            .enumerate()
            .skip(1)
            .filter(|(_, pressed)| **pressed)
            .map(|(button_number, _)| match button_number {
                1 => "Left".to_string(),
                2 => "Right".to_string(),
                3 => "Middle".to_string(),
                n => n.to_string(),
            })
            .collect::<Vec<String>>();
        ui.label(format!("Mouse pressed: [{}]", buttons_pressed.join(", ")));

        let key_txt = format!(
            "Key pressed: [{}]",
            keys.iter().map(|k| crate::utils::abbreviate_keycode(*k)).collect::<Vec<_>>().join(", ")
        );
        ui.label(key_txt);
    }

    pub fn show_autoclicker(&mut self, ui: &mut egui::Ui) {
        // A SMALL, EQUAL amount of breathing room above AND below the
        // buttons — just enough that they don't sit flush against the
        // separator above or the bottom bar below. Rounded to a whole
        // pixel — a fractional gap here was tripping egui's debug-only
        // "Unaligned" warning overlay (dev/debug builds only; never shows
        // up in a release build) at some window sizes.
        //
        // This is now the ONLY source of spacing on both sides — the
        // fixed (unscaled) frame margins that `CentralPanel` and
        // `show_bottombar`'s panel would otherwise add on top of this are
        // zeroed out at their end (see the notes there), so this `gap`
        // alone controls both, keeping them visually equal instead of the
        // bottom stacking extra fixed margin on top of it.
        let gap = (1.0 * self.ui_scale).round();
        ui.add_space(gap);
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            let autoclick_label = self.autoclick_button_label();
            let hold_label = self.hold_button_label();

            // Measure each label's actual rendered width so the button pair
            // is sized to fit its content (a longer hotkey label such as
            // "STOP (Num Lock)" just makes ITS button wider) rather than a
            // fixed pixel size that clips long labels or leaves dead space
            // around short ones. This is what used to require growing the
            // whole OS window to compensate (`auto_resize_window`, removed) —
            // now the buttons simply size themselves. Both the button's
            // floor size and the font used to measure it scale by
            // `self.ui_scale`, same as the rest of the UI — `TextStyle::
            // Button.resolve(..)` on its own would ignore the global
            // `override_font_id` scaling and measure against the
            // (unscaled) default egui button font, undersizing the button
            // relative to the text actually painted inside it.
            let scale = self.ui_scale;
            let button_height = 38.0 * scale;
            let min_button_width = 100.0 * scale;
            let padding = ui.spacing().button_padding;
            let measure = |ui: &egui::Ui, text: &str| -> egui::Vec2 {
                let font_id = ui
                    .style()
                    .override_font_id
                    .clone()
                    .unwrap_or_else(|| egui::TextStyle::Button.resolve(ui.style()));
                let galley =
                    ui.painter()
                        .layout_no_wrap(text.to_owned(), font_id, egui::Color32::WHITE);
                egui::vec2(
                    (galley.size().x + padding.x * 2.0).max(min_button_width),
                    button_height,
                )
            };
            let autoclick_size = measure(ui, &autoclick_label);
            let hold_size = measure(ui, &hold_label);

            // Allocate a region exactly as wide as the two measured buttons
            // (clamped to whatever space is actually available) so the
            // centered parent layout centers the real content instead of a
            // guessed fixed width.
            let group_width =
                (autoclick_size.x + hold_size.x + ui.spacing().item_spacing.x)
                    .min(ui.available_width());
            let group_height = autoclick_size.y.max(hold_size.y);

            ui.allocate_ui_with_layout(
                egui::vec2(group_width, group_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    // Autoclick start/stop: disabled while holding or the hotkeys window is open
                    let autoclick_enabled = !self.hotkey_window_open && !self.is_holding();
                    if ui
                        .add_enabled(
                            autoclick_enabled,
                            egui::widgets::Button::new(autoclick_label)
                                .min_size(autoclick_size),
                        )
                        .clicked()
                    {
                        if self.is_autoclicking() {
                            self.mode = InteractionMode::Idle;
                        } else {
                            // Start autoclick, first click is delayed
                            self.start_autoclick(0u64);
                        }
                    }

                    // Click & hold: disabled while autoclicking or the hotkeys window is open
                    let hold_enabled = !self.hotkey_window_open && !self.is_autoclicking();
                    if ui
                        .add_enabled(
                            hold_enabled,
                            egui::widgets::Button::new(hold_label).min_size(hold_size),
                        )
                        .clicked()
                    {
                        if self.is_holding() {
                            self.stop_hold();
                        } else {
                            self.start_hold();
                        }
                    }
                },
            );
        });
        ui.add_space(gap);
    }
}

use eframe::egui;
use rdev::{Button, Key};

use crate::{RustyAutoClickerApp, types::ClickButton};

/// Helper macro to add a selectable value for a keyboard key in the UI.
macro_rules! key_option {
    ($ui:expr, $self:expr, $variant:ident) => {
        $ui.selectable_value(
            &mut $self.click_btn,
            ClickButton::Key(Key::$variant),
            crate::utils::abbreviate_key(Key::$variant),
        );
    };
}

impl RustyAutoClickerApp {
    pub fn show_buttons(&mut self, ui: &mut egui::Ui) {
        super::wrapping_field_row(
            ui,
            self,
            |state, ui| {
                ui.label("Buttons");
                ui.add_enabled_ui(!state.is_setting_click_key(), |ui| {
                    if ui
                        .add(egui::RadioButton::new(
                            matches!(state.click_btn, ClickButton::Mouse(_)),
                            "Mouse",
                        ))
                        .clicked()
                    {
                        state.click_btn = ClickButton::Mouse(Button::Left);
                        // Leaving Keyboard mode entirely — a leftover "Set
                        // to X" for the key you just left makes no sense
                        // here.
                        state.click_key_capture_error = None;
                        state.click_key_capture_feedback = None;
                        state.click_key_capture_feedback_set_at = None;
                    }
                    if ui
                        .add(egui::RadioButton::new(
                            matches!(state.click_btn, ClickButton::Key(_)),
                            "Keyboard",
                        ))
                        .clicked()
                    {
                        // Restore whatever key was last used in Keyboard
                        // mode instead of always resetting to Space.
                        state.click_btn = ClickButton::Key(state.last_keyboard_key);
                    }
                });
            },
            |state, ui| match state.click_btn {
                ClickButton::Mouse(_) => state.show_mouse_buttons(ui),
                ClickButton::Key(_) => state.show_keyboard_buttons(ui),
            },
        );

        // Remember the last keyboard key across Mouse<->Keyboard toggles,
        // and give ONE uniform "Set to X" confirmation whether the key
        // came from the dropdown or from Press-Key capture — from the
        // user's perspective both are equally "I successfully set the
        // key", so both should confirm the same way instead of only the
        // capture flow saying anything.
        if let ClickButton::Key(k) = self.click_btn {
            self.last_keyboard_key = k;
            if self.click_btn != self.last_seen_click_btn {
                self.click_key_capture_error = None;
                self.click_key_capture_feedback =
                    Some(format!("Set to \"{}\"", crate::utils::abbreviate_key(k)));
                self.click_key_capture_feedback_set_at = Some(std::time::Instant::now());
            }
        }
        self.last_seen_click_btn = self.click_btn;

        // IMPORTANT: this must NOT be nested inside the `horizontal_wrapped`
        // above. It used to live at the end of `show_keyboard_buttons`,
        // called with that same row's `ui` — i.e. a wrapped horizontal
        // layout nested inside another wrapped horizontal layout sharing one
        // cursor. That combination corrupts egui's row-height bookkeeping
        // and produces a large phantom blank gap that pushes every section
        // below it far down (sometimes off the bottom of the window). It
        // only appeared after a successful "Press Key" capture because that
        // was the only path that set `click_key_capture_feedback`, which is
        // what triggered the nested call — the manual dropdown never sets
        // it, which is why the dropdown path never broke.
        //
        // This row takes up NO space at all when there's nothing to show —
        // it only appears (and the content below it shifts down slightly)
        // while an error or "Set to X" confirmation is actually visible,
        // then collapses back once it clears. The window itself is never
        // resized for this either way; on the rare frame where showing it
        // pushes total content height a few pixels past what fits, the
        // `ScrollArea` safety net in `gui/mod.rs::ui()` absorbs it rather
        // than clipping.
        if let Some(err) = self.click_key_capture_error.clone() {
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(egui::Color32::from_rgb(220, 80, 80), err);
            });
        } else if let Some(msg) = self.click_key_capture_feedback.clone() {
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(egui::Color32::from_rgb(90, 200, 120), msg);
            });
        }
    }

    fn show_mouse_buttons(&mut self, ui: &mut egui::Ui) {
        // Caller (`show_buttons`, via `wrapping_field_row`) already places
        // this closure inside a right-to-left layout — no need to nest
        // another one here.
        egui::ComboBox::from_id_salt("mouse_button")
            .selected_text(format!("{}", self.click_btn))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.click_btn,
                    ClickButton::Mouse(Button::Left),
                    "Left",
                );
                ui.selectable_value(
                    &mut self.click_btn,
                    ClickButton::Mouse(Button::Right),
                    "Right",
                );
                // rdev's macOS backend can only simulate Left/Right.
                if !cfg!(target_os = "macos") {
                    ui.selectable_value(
                        &mut self.click_btn,
                        ClickButton::Mouse(Button::Middle),
                        "Middle",
                    );
                }
            });
    }

    fn show_keyboard_buttons(&mut self, ui: &mut egui::Ui) {
        // Same note as `show_mouse_buttons` above — already inside a
        // right-to-left layout from the caller.
        {
            if self.is_setting_click_key() {
                if ui.button("Cancel").clicked() {
                    self.cancel_click_key_capture();
                }
                ui.label("Press any key...")
                    .on_hover_text(
                        "Num Lock, Print Screen, Scroll Lock, and Pause can't be detected \
                         this way (a limitation of the underlying input library, not this \
                         app) — pick those from the dropdown instead."
                    );
            } else {
                egui::ComboBox::from_id_salt("keyboard_button")
                    .selected_text(format!("{}", self.click_btn))
                    .show_ui(ui, |ui| {
                        // Modifier keys
                        key_option!(ui, self, Alt);
                        key_option!(ui, self, AltGr);
                        key_option!(ui, self, CapsLock);
                        key_option!(ui, self, ControlLeft);
                        key_option!(ui, self, ControlRight);
                        key_option!(ui, self, MetaLeft);
                        key_option!(ui, self, MetaRight);
                        key_option!(ui, self, ShiftLeft);
                        key_option!(ui, self, ShiftRight);
                        key_option!(ui, self, Function);

                        // Navigation
                        key_option!(ui, self, UpArrow);
                        key_option!(ui, self, DownArrow);
                        key_option!(ui, self, LeftArrow);
                        key_option!(ui, self, RightArrow);
                        key_option!(ui, self, Home);
                        key_option!(ui, self, End);
                        key_option!(ui, self, PageUp);
                        key_option!(ui, self, PageDown);
                        key_option!(ui, self, Insert);
                        key_option!(ui, self, Delete);
                        key_option!(ui, self, Backspace);
                        key_option!(ui, self, Escape);
                        key_option!(ui, self, Return);
                        key_option!(ui, self, Tab);
                        key_option!(ui, self, Space);

                        // Function keys
                        key_option!(ui, self, F1);
                        key_option!(ui, self, F2);
                        key_option!(ui, self, F3);
                        key_option!(ui, self, F4);
                        key_option!(ui, self, F5);
                        key_option!(ui, self, F6);
                        key_option!(ui, self, F7);
                        key_option!(ui, self, F8);
                        key_option!(ui, self, F9);
                        key_option!(ui, self, F10);
                        key_option!(ui, self, F11);
                        key_option!(ui, self, F12);

                        // Print/Lock
                        key_option!(ui, self, PrintScreen);
                        key_option!(ui, self, ScrollLock);
                        key_option!(ui, self, Pause);
                        key_option!(ui, self, NumLock);

                        // Top row number keys and symbols
                        key_option!(ui, self, BackQuote);
                        key_option!(ui, self, Num1);
                        key_option!(ui, self, Num2);
                        key_option!(ui, self, Num3);
                        key_option!(ui, self, Num4);
                        key_option!(ui, self, Num5);
                        key_option!(ui, self, Num6);
                        key_option!(ui, self, Num7);
                        key_option!(ui, self, Num8);
                        key_option!(ui, self, Num9);
                        key_option!(ui, self, Num0);
                        key_option!(ui, self, Minus);
                        key_option!(ui, self, Equal);

                        // Letter keys
                        key_option!(ui, self, KeyA);
                        key_option!(ui, self, KeyB);
                        key_option!(ui, self, KeyC);
                        key_option!(ui, self, KeyD);
                        key_option!(ui, self, KeyE);
                        key_option!(ui, self, KeyF);
                        key_option!(ui, self, KeyG);
                        key_option!(ui, self, KeyH);
                        key_option!(ui, self, KeyI);
                        key_option!(ui, self, KeyJ);
                        key_option!(ui, self, KeyK);
                        key_option!(ui, self, KeyL);
                        key_option!(ui, self, KeyM);
                        key_option!(ui, self, KeyN);
                        key_option!(ui, self, KeyO);
                        key_option!(ui, self, KeyP);
                        key_option!(ui, self, KeyQ);
                        key_option!(ui, self, KeyR);
                        key_option!(ui, self, KeyS);
                        key_option!(ui, self, KeyT);
                        key_option!(ui, self, KeyU);
                        key_option!(ui, self, KeyV);
                        key_option!(ui, self, KeyW);
                        key_option!(ui, self, KeyX);
                        key_option!(ui, self, KeyY);
                        key_option!(ui, self, KeyZ);

                        // Punctuation and symbol keys
                        key_option!(ui, self, LeftBracket);
                        key_option!(ui, self, RightBracket);
                        key_option!(ui, self, SemiColon);
                        key_option!(ui, self, Quote);
                        key_option!(ui, self, BackSlash);
                        key_option!(ui, self, IntlBackslash);
                        key_option!(ui, self, Comma);
                        key_option!(ui, self, Dot);
                        key_option!(ui, self, Slash);

                        // Keypad
                        key_option!(ui, self, KpReturn);
                        key_option!(ui, self, KpMinus);
                        key_option!(ui, self, KpPlus);
                        key_option!(ui, self, KpMultiply);
                        key_option!(ui, self, KpDivide);
                        key_option!(ui, self, Kp0);
                        key_option!(ui, self, Kp1);
                        key_option!(ui, self, Kp2);
                        key_option!(ui, self, Kp3);
                        key_option!(ui, self, Kp4);
                        key_option!(ui, self, Kp5);
                        key_option!(ui, self, Kp6);
                        key_option!(ui, self, Kp7);
                        key_option!(ui, self, Kp8);
                        key_option!(ui, self, Kp9);
                        key_option!(ui, self, KpDelete);
                    });

                if ui.button("🎯 Press Key").on_hover_text(
                    "Click, then press the key you want the autoclicker to use.\n\
                     Note: Num Lock, Print Screen, Scroll Lock, and Pause can't be \
                     captured this way — select them from the dropdown."
                ).clicked() {
                    self.enter_click_key_capture();
                }
            }
        }
    }
}

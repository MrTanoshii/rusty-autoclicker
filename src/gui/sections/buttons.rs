use eframe::egui::{self};
use rdev::{Button, Key};

use crate::{RustyAutoClickerApp, types::ClickButton};

/// Helper macro to add a selectable value for a keyboard key in the UI.
macro_rules! key_option {
    ($ui:expr, $self:expr, $variant:ident) => {
        $ui.selectable_value(
            &mut $self.click_btn,
            ClickButton::Key(Key::$variant),
            stringify!($variant),
        );
    };
}

impl RustyAutoClickerApp {
    pub fn show_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Buttons");
            if ui
                .add(egui::RadioButton::new(
                    matches!(self.click_btn, ClickButton::Mouse(_)),
                    "Mouse",
                ))
                .clicked()
            {
                self.click_btn = ClickButton::Mouse(Button::Left);
            }
            if ui
                .add(egui::RadioButton::new(
                    matches!(self.click_btn, ClickButton::Key(_)),
                    "Keyboard",
                ))
                .clicked()
            {
                self.click_btn = ClickButton::Key(Key::Space);
            }

            match self.click_btn {
                ClickButton::Mouse(_) => self.show_mouse_buttons(ui),
                ClickButton::Key(_) => self.show_keyboard_buttons(ui),
            }
        });
    }

    fn show_mouse_buttons(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
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
                    ui.selectable_value(
                        &mut self.click_btn,
                        ClickButton::Mouse(Button::Middle),
                        "Middle",
                    );
                });
        });
    }

    fn show_keyboard_buttons(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
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
        });
    }
}

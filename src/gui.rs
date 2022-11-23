use std::time::Instant;

use device_query::{DeviceQuery, DeviceState, Keycode};
use eframe::egui;
use rdev::Button;

use crate::{
    types::{AppMode, ClickPosition, ClickType},
    utils::{autoclick, sanitize_i64_string, sanitize_string},
    RustyAutoClickerApp,
};

impl eframe::App for RustyAutoClickerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        egui::Rgba::TRANSPARENT
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Print time to between start of old and new frames
        #[cfg(debug_assertions)]
        println!(
            "Frame delta: {:?}",
            Instant::now()
                .checked_duration_since(self.frame_start)
                .unwrap()
        );

        self.frame_start = Instant::now();

        // Get mouse & keyboard states
        let device_state = DeviceState::new();
        let mouse = device_state.get_mouse();
        let keys = device_state.get_keys();

        // Input sanitation
        sanitize_string(&mut self.hr_str, 5usize);
        sanitize_string(&mut self.min_str, 5usize);
        sanitize_string(&mut self.sec_str, 5usize);
        sanitize_string(&mut self.ms_str, 5usize);
        sanitize_string(&mut self.click_amount_str, 5usize);
        sanitize_i64_string(&mut self.click_x_str, 7usize);
        sanitize_i64_string(&mut self.click_y_str, 7usize);
        sanitize_string(&mut self.movement_sec_str, 5usize);
        sanitize_string(&mut self.movement_ms_str, 5usize);

        // Parse time Strings to u64
        let hr: u64 = self.hr_str.parse().unwrap_or_default();
        let min: u64 = self.min_str.parse().unwrap_or_default();
        let sec: u64 = self.sec_str.parse().unwrap_or_default();
        let ms: u64 = self.ms_str.parse().unwrap_or_default();
        // println!("{} hr {} min {} sec {} ms", &hr, min, sec, ms);

        // Parse movement Strings to u64
        let movement_sec: u64 = self.movement_sec_str.parse().unwrap_or_default();
        let movement_ms: u64 = self.movement_ms_str.parse().unwrap_or_default();

        // Calculate movement delay
        let movement_delay_in_ms = (movement_sec * 1000u64) + movement_ms;

        // Parse click amount String to u64
        let click_amount: u64 = self.click_amount_str.parse().unwrap_or_default();

        // Parse mouse coordinates Strings to f64
        let click_x: f64 = self.click_x_str.parse().unwrap_or_default();
        let click_y: f64 = self.click_y_str.parse().unwrap_or_default();

        // Close hotkeys window if escape pressed & released
        if self.hotkey_window_open {
            if keys.contains(&Keycode::Escape) {
                self.key_pressed_esc = true;
            } else if self.key_pressed_esc {
                // Close only if app is not busy
                if !self.is_autoclicking
                    && !self.is_setting_autoclick_key
                    && !self.is_setting_coord
                    && !self.is_setting_set_coord_key
                {
                    self.hotkey_window_open = false;
                }
                self.key_pressed_esc = false;
            }
        };

        // Start cursor follower if setting coordinates
        // if self.is_setting_coord && !self.coord_window_open {
        // } else if self.coord_window_open {
        // }

        // Calculate click interval
        let interval: u64 = (hr * 3600000) + (min * 60000) + (sec * 1000) + ms;

        let update_now = Instant::now();

        // Toggle autoclicking
        if self.key_autoclick.is_some() && keys.contains(&self.key_autoclick.unwrap()) {
            self.key_pressed_autoclick = true;
        } else if self.key_pressed_autoclick {
            self.key_pressed_autoclick = false;
            if self.is_autoclicking {
                self.is_autoclicking = false;
            } else if !self.is_setting_autoclick_key
                && !self.is_setting_coord
                && !self.is_setting_set_coord_key
                && !self.hotkey_window_open
            {
                // Set only if app is not busy
                // Start autoclick, first click is instantaneous
                Self::start_autoclick(self, interval);
            }
        }

        // Send click event
        if self.is_autoclicking
            && update_now
                .checked_duration_since(self.last_now)
                .unwrap()
                .as_millis() as u64
                >= interval
        {
            #[cfg(debug_assertions)]
            println!(
                "{:?} {:?} Click: {:?}",
                self.click_type,
                self.click_btn,
                update_now.checked_duration_since(self.last_now).unwrap(),
            );
            self.last_now = Instant::now();

            autoclick(
                self.app_mode,
                self.click_position,
                (click_x, click_y),
                self.click_type,
                self.click_btn,
                mouse.coords,
                self.is_moving_humanlike,
                movement_delay_in_ms,
                self.rng_thread.clone(),
            );

            // Increment click counter and stop autoclicking if completed
            self.click_counter += 1u64;
            if click_amount != 0u64 && self.click_counter >= click_amount {
                self.is_autoclicking = false;
            }
        }
        // Set hotkey for autoclick
        else if self.is_setting_autoclick_key && self.keys_pressed.is_some() {
            for pressed_key in self.keys_pressed.clone().unwrap().into_iter() {
                if !keys.contains(&pressed_key) {
                    self.key_autoclick = Some(pressed_key);
                    self.is_setting_autoclick_key = false;
                    break;
                }
            }
        }
        // Set hotkey for setting coordinates
        else if self.is_setting_set_coord_key && self.keys_pressed.is_some() {
            for pressed_key in self.keys_pressed.clone().unwrap().into_iter() {
                if !keys.contains(&pressed_key) {
                    self.key_set_coord = Some(pressed_key);
                    self.is_setting_set_coord_key = false;
                    break;
                }
            }
        }
        // Set mouse coordinates
        else if self.is_setting_coord {
            self.click_x_str = mouse.coords.0.to_string();
            self.click_y_str = mouse.coords.1.to_string();

            // Stop if mouse left click
            if mouse.button_pressed[1]
                || (self.key_set_coord.is_some() && keys.contains(&self.key_set_coord.unwrap()))
            {
                Self::exit_coordinate_setting(self, frame);
            }
        }

        // Save state of pressed keys
        self.keys_pressed = Some(keys.clone());

        // Open hotkeys window if hotkeys not set
        if !self.hotkey_window_open
            && (self.key_autoclick.is_none() || self.key_set_coord.is_none())
        {
            self.hotkey_window_open = true
        }

        if self.is_setting_coord {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if self.is_autoclicking || self.hotkey_window_open {
                                ui.set_enabled(false);
                            };
                            ui.add(
                                egui::TextEdit::singleline(&mut self.click_y_str)
                                    .desired_width(50.0f32)
                                    .hint_text("0"),
                            );
                            ui.label("Y");
                            if self.is_autoclicking || self.hotkey_window_open {
                                ui.set_enabled(false);
                            };
                            ui.add(
                                egui::TextEdit::singleline(&mut self.click_x_str)
                                    .desired_width(50.0f32)
                                    .hint_text("0"),
                            );
                            ui.label("X");
                            ui.separator();
                            ui.label(format!(
                                " Set with \"{:}\" / \"L Click\"",
                                self.key_set_coord.unwrap()
                            ));
                        });
                    });
                })
            });
            Self::follow_cursor(self, frame);
        } else {
            // GUI
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                // The top panel is often a good place for a menu bar:
                egui::menu::bar(ui, |ui| {
                    if self.is_autoclicking {
                        if ui
                            .button(format!("🖱 STOP ({})", self.key_autoclick.unwrap()))
                            .clicked()
                        {
                            self.is_autoclicking = false;
                        };
                    } else {
                        if self.hotkey_window_open {
                            ui.set_enabled(false);
                        }
                        let text: String = if self.key_autoclick.is_none() {
                            "🖱 START".to_string()
                        } else {
                            format!("🖱 START ({})", self.key_autoclick.unwrap())
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
                        ui.set_enabled(false);
                    };
                    ui.selectable_value(&mut self.app_mode, AppMode::Bot, "🖥 Bot")
                        .on_hover_text("Autoclick as fast as possible");
                    if self.is_autoclicking || self.hotkey_window_open {
                        ui.set_enabled(false);
                    };
                    ui.selectable_value(&mut self.app_mode, AppMode::Humanlike, "😆 Humanlike")
                        .on_hover_text("Autoclick emulating human clicking");
                });
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's

                ui.horizontal_wrapped(|ui| {
                    ui.label("Click Interval");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.label("ms");
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.ms_str)
                                .desired_width(40.0f32)
                                .hint_text("0"),
                        );

                        ui.label("sec");
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.sec_str)
                                .desired_width(40.0f32)
                                .hint_text("0"),
                        );

                        ui.label("min");
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.min_str)
                                .desired_width(40.0f32)
                                .hint_text("0"),
                        );

                        ui.label("hr");
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.hr_str)
                                .desired_width(40.0f32)
                                .hint_text("0"),
                        );
                    });
                });
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.label("Movement delay (Humanlike only)");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.label("ms");
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.movement_ms_str)
                                .desired_width(40.0f32)
                                .hint_text("20"),
                        );

                        ui.label("sec");
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.movement_sec_str)
                                .desired_width(40.0f32)
                                .hint_text("0"),
                        );
                    });
                });
                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    ui.label("Mouse Button");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.selectable_value(&mut self.click_btn, Button::Right, "Right");
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.selectable_value(&mut self.click_btn, Button::Middle, "Middle");
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.selectable_value(&mut self.click_btn, Button::Left, "Left");
                    });
                });

                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    ui.label("Click Type");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.selectable_value(&mut self.click_type, ClickType::Double, "Double");
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.selectable_value(&mut self.click_type, ClickType::Single, "Single");
                    });
                });

                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    ui.label("Click Amount (0 = forever)");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.click_amount_str)
                                .desired_width(40.0f32)
                                .hint_text("0"),
                        );
                        if self.is_autoclicking && click_amount > 0u64 {
                            let remaining_clicks = click_amount.saturating_sub(self.click_counter);
                            let remaining_text = format!("Remaining {:?}", remaining_clicks);
                            ui.label(remaining_text);
                        }
                    });
                });

                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    ui.label("Click Position");
                    if self.is_autoclicking || self.hotkey_window_open {
                        ui.set_enabled(false);
                    };
                    if ui
                        .add_sized([80.0f32, 16.0f32], egui::widgets::Button::new("Set Coords"))
                        .clicked()
                    {
                        Self::enter_coordinate_setting(self, frame);
                    };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.click_y_str)
                                .desired_width(50.0f32)
                                .hint_text("0"),
                        );
                        ui.label("Y");
                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.click_x_str)
                                .desired_width(50.0f32)
                                .hint_text("0"),
                        );
                        ui.label("X");

                        if self.is_autoclicking || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        if ui
                            .selectable_value(
                                &mut self.click_position,
                                ClickPosition::Coord,
                                "Coords",
                            )
                            .clicked()
                        {
                            self.key_pressed_set_coord = true;
                        };
                        ui.separator();
                        if ui
                            .selectable_value(
                                &mut self.click_position,
                                ClickPosition::Mouse,
                                "Mouse",
                            )
                            .clicked()
                        {
                            self.key_pressed_set_coord = false;
                        };
                    });
                });

                ui.separator();

                let mouse_txt = format!("Mouse position: {:?}", mouse.coords);
                ui.label(mouse_txt);
                let key_txt = format!("Key pressed: {:?}", keys);
                ui.label(key_txt);
                let extra_buttons_pressed = mouse
                    .button_pressed
                    .iter()
                    .enumerate()
                    .skip(4)
                    .map(|(button_number, pressed)| format!("{:?}-{:?}", button_number, pressed))
                    .collect::<Vec<String>>()
                    .join(" ");

                ui.label(format!(
                    "Mouse pressed: L-{:?} R-{:?} M-{:?} {}",
                    mouse.button_pressed[1],
                    mouse.button_pressed[2],
                    mouse.button_pressed[3],
                    &extra_buttons_pressed
                ));

                ui.separator();

                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    if self.is_autoclicking {
                        if self.hotkey_window_open {
                            ui.set_enabled(false);
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
                            ui.set_enabled(false);
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
            });

            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 5.0;
                        ui.hyperlink_to(
                            "eframe",
                            "https://github.com/emilk/egui/tree/master/eframe",
                        );
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

        // Hotkeys window
        if self.hotkey_window_open {
            egui::Window::new("Hotkeys")
                .fixed_size(egui::vec2(220f32, 100f32))
                .anchor(egui::Align2::CENTER_CENTER, [0f32, 0f32])
                .collapsible(false)
                .open(&mut self.hotkey_window_open)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized(
                                [100.0f32, 32.0f32],
                                egui::widgets::Button::new("Start/Stop"),
                            )
                            .clicked()
                        {
                            // Allow keybind only if app is not busy
                            if !self.is_autoclicking
                                && !self.is_setting_autoclick_key
                                && !self.is_setting_coord
                                && !self.is_setting_set_coord_key
                            {
                                self.is_setting_autoclick_key = true;
                                self.key_autoclick = None;
                            }
                        };
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.set_enabled(false);
                            let text: String = if self.key_autoclick.is_none() {
                                "PRESS ANY KEY".to_string()
                            } else {
                                format!("{:}", self.key_autoclick.unwrap())
                            };
                            ui.add_sized([110.0f32, 32.0f32], egui::widgets::Button::new(text));
                        });
                    });
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized(
                                [100.0f32, 32.0f32],
                                egui::widgets::Button::new("Confirm Coords"),
                            )
                            .on_hover_text("Note: L Click cannot be changed")
                            .clicked()
                        {
                            // Allow keybind only if app is not busy
                            if !self.is_autoclicking
                                && !self.is_setting_autoclick_key
                                && !self.is_setting_coord
                                && !self.is_setting_set_coord_key
                            {
                                self.key_set_coord = None;
                                self.is_setting_set_coord_key = true;
                            }
                        };
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.set_enabled(false);
                            let text: String = if self.key_set_coord.is_none() {
                                "PRESS ANY KEY".to_string()
                            } else {
                                format!("{:} / L Click", self.key_set_coord.unwrap())
                            };
                            ui.add_sized([110.0f32, 32.0f32], egui::widgets::Button::new(text));
                        });
                    });
                });
        }

        // Keep updating frame
        ctx.request_repaint();

        // Print time to process frame
        #[cfg(debug_assertions)]
        println!(
            "Frame time: {:?}",
            Instant::now()
                .checked_duration_since(self.frame_start)
                .unwrap()
        );
    }
}

use std::time::{Duration, Instant};

use device_query::{DeviceQuery, DeviceState, Keycode};
use eframe::egui;

use crate::{
    RustyAutoClickerApp,
    types::ClickInfo,
    utils::{autoclick, sanitize_i64_string, sanitize_string},
};

mod sections;
mod windows;

impl eframe::App for RustyAutoClickerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                .unwrap_or(Duration::ZERO)
                .as_millis() as u64
                >= interval
        {
            #[cfg(debug_assertions)]
            println!(
                "{:?} {:?} Click: {:?}",
                self.click_type,
                self.click_btn,
                update_now
                    .checked_duration_since(self.last_now)
                    .unwrap_or(Duration::ZERO),
            );
            self.last_now = Instant::now();

            autoclick(
                self.app_mode,
                ClickInfo {
                    click_btn: self.click_btn,
                    click_coord: (click_x, click_y),
                    click_position: self.click_position,
                    click_type: self.click_type,
                },
                mouse.coords,
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
                Self::exit_coordinate_setting(self, ctx);
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
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
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
                            ui.separator();
                            ui.label(format!(
                                " Set with \"{:}\" / \"L Click\"",
                                self.key_set_coord.unwrap()
                            ));
                        });
                    });
                })
            });
            Self::follow_cursor(self, ctx);
        } else {
            // GUI
            // Top panel with menu bar
            self.show_topbar(ctx);

            egui::CentralPanel::default().show(ctx, |ui| {
                // The central panel the region left after adding TopPanel's and SidePanel's
                self.show_click_interval(ui);
                ui.separator();
                self.show_movement_delay(ui);
                ui.separator();
                self.show_buttons(ui);
                ui.separator();
                self.show_click_type(ui);
                ui.separator();
                self.show_click_amount(ui, click_amount);
                ui.separator();
                self.show_click_position(ui, ctx);
                ui.separator();
                self.show_infos(ui, &mouse, &keys);
                ui.separator();
                self.show_autoclicker(ui);
            });

            self.show_bottombar(ctx);
        }

        // Hotkeys window
        if self.hotkey_window_open {
            self.show_hotkeys_window(ctx);
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

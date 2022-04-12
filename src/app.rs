// #![allow(unused_imports)]
use std::{
    env, str, thread,
    time::{Duration, Instant},
};

use rand::{thread_rng, Rng};

use device_query::{DeviceQuery, DeviceState, Keycode, MouseState};

use rdev::{simulate, Button, EventType, SimulateError};

use eframe::{
    egui,
    epaint::{FontFamily, FontId},
    epi,
};

use sanitizer::prelude::StringSanitizer;

#[derive(PartialEq, Debug, Copy, Clone)]
enum ClickType {
    Single,
    Double,
}

#[derive(PartialEq, Copy, Clone)]
enum ClickPosition {
    Mouse,
    Coord,
}

#[derive(PartialEq, Copy, Clone)]
enum AppMode {
    Bot,
    Humanlike,
}

const DURATION_CLICK_MIN: u64 = 20;
const DURATION_CLICK_MAX: u64 = 40;
const DURATION_DOUBLE_CLICK_MIN: u64 = 30;
const DURATION_DOUBLE_CLICK_MAX: u64 = 60;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct RustyAutoClickerApp {
    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    // Text input strings
    hr_str: String,
    min_str: String,
    sec_str: String,
    ms_str: String,
    click_amount_str: String,
    click_x_str: String,
    click_y_str: String,

    // Time
    last_now: Instant,
    frame_start: Instant,

    // Counter
    click_counter: u64,

    //Hotkeys
    key_autoclick: Option<Keycode>,
    key_set_coord: Option<Keycode>,

    // App state
    is_autoclicking: bool,
    is_setting_coord: bool,
    is_setting_autoclick_key: bool,
    is_setting_set_coord_key: bool,

    // App mode
    app_mode: AppMode,

    // Window state
    hotkey_window_open: bool,
    _coord_window_open: bool,

    // Key states
    key_pressed_autoclick: bool,
    key_pressed_set_coord: bool,
    key_pressed_esc: bool,
    keys_pressed: Option<Vec<Keycode>>,

    // Enums
    click_btn: Button,
    click_type: ClickType,
    click_position: ClickPosition,
}

impl Default for RustyAutoClickerApp {
    fn default() -> Self {
        Self {
            // Text input strings
            hr_str: "0".to_owned(),
            min_str: "0".to_owned(),
            sec_str: "0".to_owned(),
            ms_str: "100".to_owned(),
            click_amount_str: "0".to_owned(),
            click_x_str: "0".to_owned(),
            click_y_str: "0".to_owned(),

            // Time
            last_now: Instant::now(),
            frame_start: Instant::now(),

            // Counter
            click_counter: 0u64,

            // Hotkeys
            key_autoclick: Some(Keycode::F6),
            key_set_coord: Some(Keycode::Escape),

            // App state
            is_autoclicking: false,
            is_setting_coord: false,
            is_setting_autoclick_key: false,
            is_setting_set_coord_key: false,

            // App mode
            app_mode: AppMode::Bot,

            // Window state
            hotkey_window_open: false,
            _coord_window_open: false,

            // Key states
            key_pressed_autoclick: false,
            key_pressed_set_coord: false,
            key_pressed_esc: false,
            keys_pressed: None,

            // Enums
            click_btn: Button::Left,
            click_type: ClickType::Single,
            click_position: ClickPosition::Mouse,
        }
    }
}

// Provides sanitation for input string
fn sanitize_string(string: &mut String, max_length: usize) {
    // Accept numeric only
    let s_slice: &str = &*string;
    let mut sanitizer = StringSanitizer::from(s_slice);
    sanitizer.numeric();
    *string = sanitizer.get();

    // Remove leading 0
    while string.len() > 1 && string.starts_with('0') {
        string.remove(0);
    }

    // Allow max size of `max_length` characters
    if string.len() >= max_length {
        string.truncate(max_length)
    };
}

// Simulate event - rdev crate
fn send(event_type: &EventType) {
    match simulate(event_type) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("We could not send {:?}", event_type);
        }
    }

    // Let ths OS catchup (at least MacOS)
    if env::consts::OS == "macos" {
        thread::sleep(Duration::from_millis(20u64));
    }
}

fn autoclick(
    app_mode: AppMode,
    click_position: ClickPosition,
    click_coord: (f64, f64),
    click_type: ClickType,
    click_btn: Button,
) {
    // Set the amount of runs/clicks required
    let run_amount: u8 = if click_type == ClickType::Single {
        1
    } else if click_type == ClickType::Double {
        2
    } else {
        0
    };

    // Autoclick as fast as possible
    if app_mode == AppMode::Bot {
        for _n in 1..=run_amount {
            // Move mouse to saved coordinates if requested
            if click_position == ClickPosition::Coord {
                send(&EventType::MouseMove {
                    x: click_coord.0,
                    y: click_coord.1,
                })
            }

            send(&EventType::ButtonPress(click_btn));
            send(&EventType::ButtonRelease(click_btn));
        }
    // Autoclick to emulate a humanlike clicks
    } else if app_mode == AppMode::Humanlike {
        let mut rng = thread_rng();
        for n in 1..=run_amount {
            // Sleep between clicks
            if n % 2 == 0 {
                thread::sleep(Duration::from_millis(
                    rng.gen_range(DURATION_DOUBLE_CLICK_MIN..DURATION_DOUBLE_CLICK_MAX),
                ));
            }

            // TODO: Implement non blocking smooth mouse movement (i.e. it should not mouse directly)
            // Move mouse to saved coordinates if requested
            if click_position == ClickPosition::Coord {
                send(&EventType::MouseMove {
                    x: click_coord.0,
                    y: click_coord.1,
                })
            }

            send(&EventType::ButtonPress(click_btn));
            thread::sleep(Duration::from_millis(
                rng.gen_range(DURATION_CLICK_MIN..DURATION_CLICK_MAX),
            ));
            send(&EventType::ButtonRelease(click_btn));
        }
    }
}

impl epi::App for RustyAutoClickerApp {
    fn name(&self) -> &str {
        "Rusty AutoClicker v1.1.0"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        let mut style: egui::Style = (*ctx.style()).clone();
        let font = FontId {
            size: 14.0f32,
            family: FontFamily::Monospace,
        };
        style.override_font_id = Some(font);
        ctx.set_style(style);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        self.frame_start = Instant::now();

        // Get mouse & keyboard states
        let device_state = DeviceState::new();
        let mouse: MouseState = device_state.get_mouse();
        let keys: Vec<Keycode> = device_state.get_keys();

        // Input sanitation
        sanitize_string(&mut self.hr_str, 5usize);
        sanitize_string(&mut self.min_str, 5usize);
        sanitize_string(&mut self.sec_str, 5usize);
        sanitize_string(&mut self.ms_str, 5usize);
        sanitize_string(&mut self.click_amount_str, 5usize);
        sanitize_string(&mut self.click_x_str, 7usize);
        sanitize_string(&mut self.click_y_str, 7usize);

        // Parse time Strings to u64
        let mut hr: u64 = 0u64;
        if !self.hr_str.is_empty() {
            hr = self.hr_str.parse().unwrap();
        }
        let mut min: u64 = 0u64;
        if !self.min_str.is_empty() {
            min = self.min_str.parse().unwrap();
        }
        let mut sec: u64 = 0u64;
        if !self.sec_str.is_empty() {
            sec = self.sec_str.parse().unwrap();
        }
        let mut ms: u64 = 0u64;
        if !self.ms_str.is_empty() {
            ms = self.ms_str.parse().unwrap();
        }
        // println!("{} hr {} min {} sec {} ms", &hr, min, sec, ms);

        // Parse click amount String to u64
        let mut click_amount: u64 = 0u64;
        if !self.click_amount_str.is_empty() {
            click_amount = self.click_amount_str.parse().unwrap();
        }

        // Parse mouse coordinates Strings to f64
        let mut click_x: f64 = 0f64;
        if !self.click_x_str.is_empty() {
            click_x = self.click_x_str.parse().unwrap();
        }
        let mut click_y: f64 = 0f64;
        if !self.click_y_str.is_empty() {
            click_y = self.click_y_str.parse().unwrap();
        }

        // Toggle autoclicking
        if self.key_autoclick.is_some() && keys.contains(&self.key_autoclick.unwrap()) {
            self.key_pressed_autoclick = true;
        } else if self.key_pressed_autoclick {
            self.key_pressed_autoclick = false;
            if self.is_autoclicking {
                self.is_autoclicking = false;
            } else {
                // Set only if app is not busy
                if !self.is_setting_autoclick_key
                    && !self.is_setting_coord
                    && !self.is_setting_set_coord_key
                    && !self.hotkey_window_open
                {
                    self.click_counter = 0u64;
                    self.is_autoclicking = true;
                }
                self.last_now = Instant::now();
            }
        }

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
        let interval: u64 = (hr * 3600000u64) + (min * 60000u64) + (sec * 1000u64) + ms;

        let update_now = Instant::now();

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
            if mouse.button_pressed[1] {
                self.is_setting_coord = false;
            }
            // Stop if coordinate set key pressed & released
            else if self.key_set_coord.is_some() && keys.contains(&self.key_set_coord.unwrap()) {
                self.key_pressed_set_coord = true;
            } else if self.key_pressed_set_coord {
                self.is_setting_coord = false;
                self.key_pressed_set_coord = false;
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
                    .on_disabled_hover_text("Disabled while autoclicking")
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
                    .on_hover_text("Not yet implemented");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.horizontal_wrapped(|ui| {
                ui.label("Click Interval");

                ui.with_layout(egui::Layout::right_to_left(), |ui| {
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
                ui.label("Mouse Button");
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
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
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
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
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    if self.is_autoclicking || self.hotkey_window_open {
                        ui.set_enabled(false);
                    };
                    ui.add(
                        egui::TextEdit::singleline(&mut self.click_amount_str)
                            .desired_width(40.0f32)
                            .hint_text("0"),
                    );
                });
            });

            ui.separator();

            ui.horizontal_wrapped(|ui| {
                ui.label("Click Position");
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
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
                        .selectable_value(&mut self.click_position, ClickPosition::Coord, "Coords")
                        .clicked()
                    {
                        self.is_setting_coord = true;
                        self.key_pressed_set_coord = false;
                    };
                    ui.separator();
                    ui.selectable_value(&mut self.click_position, ClickPosition::Mouse, "Mouse");
                });
            });

            ui.separator();

            let mouse_txt = format!("Mouse position: {:?}", mouse.coords);
            ui.label(mouse_txt);
            let key_txt = format!("Key pressed: {:?}", keys);
            ui.label(key_txt);
            ui.label(format!("Mouse pressed: {:?}", mouse.button_pressed));

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
                        self.is_autoclicking = true
                    }
                }
            });
        });

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
                        ui.with_layout(egui::Layout::right_to_left(), |ui| {
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
                                egui::widgets::Button::new("Set coords"),
                            )
                            .on_hover_text("L Click cannot be changed")
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
                        ui.with_layout(egui::Layout::right_to_left(), |ui| {
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

        #[cfg(debug_assertions)]
        println!(
            "Frame time: {:?}",
            Instant::now()
                .checked_duration_since(self.frame_start)
                .unwrap()
        );
    }
}

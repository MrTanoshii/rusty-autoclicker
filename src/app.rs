// #![allow(unused_imports)]
use rand::{prelude::ThreadRng, thread_rng, Rng};
use std::{
    env, str, thread,
    time::{Duration, Instant},
};

use device_query::{DeviceQuery, DeviceState, Keycode, MouseState};

use rdev::{simulate, Button, EventType, SimulateError};

use eframe::{
    egui,
    emath::Numeric,
    epaint::{FontFamily, FontId},
};

use sanitizer::prelude::StringSanitizer;

use crate::constants::*;

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

// ranges for click durations
const DURATION_CLICK_MIN: u64 = 20;
const DURATION_CLICK_MAX: u64 = 40;
const DURATION_DOUBLE_CLICK_MIN: u64 = 30;
const DURATION_DOUBLE_CLICK_MAX: u64 = 60;

// step widths for human-like mouse movement
pub const MOUSE_STEP_POS_X: f64 = 10.0;
pub const MOUSE_STEP_NEG_X: f64 = -10.0;
pub const MOUSE_STEP_POS_Y: f64 = 10.0;
pub const MOUSE_STEP_NEG_Y: f64 = -10.0;

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
    movement_sec_str: String,
    movement_ms_str: String,

    // Time
    last_now: Instant,
    frame_start: Instant,

    // Counter
    click_counter: u64,

    //Hotkeys
    key_autoclick: Option<Keycode>,
    key_set_coord: Option<Keycode>,

    // App state
    is_executing: bool,
    is_autoclicking: bool,
    is_setting_coord: bool,
    is_setting_autoclick_key: bool,
    is_setting_set_coord_key: bool,
    is_moving_humanlike: bool,
    is_moving: bool,

    // App mode
    app_mode: AppMode,

    // Window state
    hotkey_window_open: bool,
    window_position: egui::Pos2,

    // Key states
    key_pressed_autoclick: bool,
    key_pressed_set_coord: bool,
    key_pressed_esc: bool,
    keys_pressed: Option<Vec<Keycode>>,

    // Enums
    click_btn: Button,
    click_type: ClickType,
    click_position: ClickPosition,

    // RNG
    rng_thread: ThreadRng,
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
            movement_sec_str: "0".to_owned(),
            movement_ms_str: "20".to_owned(),

            // Time
            last_now: Instant::now(),
            frame_start: Instant::now(),

            // Counter
            click_counter: 0u64,

            // Hotkeys
            key_autoclick: Some(Keycode::F6),
            key_set_coord: Some(Keycode::Escape),

            // App state
            is_executing: false,
            is_autoclicking: false,
            is_moving: false,
            is_setting_coord: false,
            is_setting_autoclick_key: false,
            is_setting_set_coord_key: false,
            is_moving_humanlike: true,

            // App mode
            app_mode: AppMode::Bot,

            // Window state
            hotkey_window_open: false,
            window_position: egui::Pos2 { x: 0f32, y: 0f32 },

            // Key states
            key_pressed_autoclick: false,
            key_pressed_set_coord: false,
            key_pressed_esc: false,
            keys_pressed: None,

            // Enums
            click_btn: Button::Left,
            click_type: ClickType::Single,
            click_position: ClickPosition::Mouse,

            // RNG
            rng_thread: thread_rng(),
        }
    }
}

impl RustyAutoClickerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;

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
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn enter_coordinate_setting(&mut self, frame: &mut eframe::Frame) {
        self.is_setting_coord = true;
        self.window_position = frame.info().window_info.position.unwrap();
        frame.set_window_size(egui::vec2(400f32, 30f32));
        frame.set_decorations(false);
    }

    fn follow_cursor(&mut self, frame: &mut eframe::Frame) {
        let offset = egui::Vec2 { x: 15f32, y: 15f32 };
        frame.set_window_pos(
            egui::pos2(
                self.click_x_str.parse().unwrap(),
                self.click_y_str.parse().unwrap(),
            ) + offset,
        );
    }

    fn exit_coordinate_setting(&mut self, frame: &mut eframe::Frame) {
        frame.set_decorations(true);
        frame.set_window_size(egui::vec2(WINDOW_WIDTH, WINDOW_HEIGHT));
        frame.set_window_pos(self.window_position);
        self.is_setting_coord = false;
    }

    fn start_autoclick(&mut self, negative_click_start_offset: u64) {
        self.click_counter = 0u64;
        self.is_autoclicking = !self.is_autoclicking;
        self.rng_thread = thread_rng();

        self.last_now = Instant::now()
            .checked_sub(Duration::from_millis(negative_click_start_offset))
            .unwrap();
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

    // Let the OS catchup (at least MacOS)
    if env::consts::OS == "macos" {
        thread::sleep(Duration::from_millis(20u64));
    }
}

fn parse_string_to_u64(string: String) -> u64 {
    let mut unsigned_int: u64 = 0u64;
    if !string.is_empty() {
        unsigned_int = string.parse().unwrap();
    }
    unsigned_int
}

fn parse_string_to_f64(string: String) -> f64 {
    let mut float: f64 = 0f64;
    if !string.is_empty() {
        float = string.parse().unwrap();
    }
    float
}

fn move_to(
    app_mode: AppMode,
    click_position: ClickPosition,
    click_coord: (f64, f64),
    is_moving_humanlike: bool,
    start_coords: (f64, f64),
    movement_delay_in_ms: u64,
) {
    if app_mode == AppMode::Humanlike {
        // Move mouse slowly to saved coordinates if requested
        if click_position == ClickPosition::Coord && is_moving_humanlike {
            let mut current_x = start_coords.0;
            let mut current_y = start_coords.1;
            for _n in 0..=5 {
                // horizontal movement: determine whether we need to move left, right or not at all
                let delta_x: f64 = if current_x < click_coord.0 {
                    MOUSE_STEP_POS_X.min(click_coord.0 - current_x)
                } else if current_x > click_coord.0 {
                    MOUSE_STEP_NEG_X.max(click_coord.0 - current_x)
                } else {
                    0.0
                };

                // vertical movement: determine whether we need to move up, down or not at all
                let delta_y: f64 = if current_y < click_coord.1 {
                    MOUSE_STEP_POS_Y.min(click_coord.1 - current_y)
                } else if current_y > click_coord.1 {
                    MOUSE_STEP_NEG_Y.max(click_coord.1 - current_y)
                } else {
                    0.0
                };

                current_x += delta_x;
                current_y += delta_y;

                #[cfg(debug_assertions)]
                println!(
                    "Moving by {:?} / {:?}, new pos: {:?} / {:?}",
                    delta_x, delta_y, current_x, current_y
                );
                send(&EventType::MouseMove {
                    x: current_x,
                    y: current_y,
                });

                thread::sleep(Duration::from_millis(movement_delay_in_ms));
            }
        }
    }
}

fn autoclick(
    app_mode: AppMode,
    click_position: ClickPosition,
    click_coord: (f64, f64),
    click_type: ClickType,
    click_btn: Button,
    mut rng_thread: ThreadRng,
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
        // perform clicks
        for n in 1..=run_amount {
            // Sleep between clicks
            if n % 2 == 0 {
                thread::sleep(Duration::from_millis(
                    rng_thread.gen_range(DURATION_DOUBLE_CLICK_MIN..DURATION_DOUBLE_CLICK_MAX),
                ));
            }

            // Move mouse to saved coordinates if requested
            if click_position == ClickPosition::Coord {
                // move to final destination
                send(&EventType::MouseMove {
                    x: click_coord.0,
                    y: click_coord.1,
                });
            }

            send(&EventType::ButtonPress(click_btn));
            thread::sleep(Duration::from_millis(
                rng_thread.gen_range(DURATION_CLICK_MIN..DURATION_CLICK_MAX),
            ));
            send(&EventType::ButtonRelease(click_btn));
        }
    }
}

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
        sanitize_string(&mut self.movement_sec_str, 5usize);
        sanitize_string(&mut self.movement_ms_str, 5usize);

        // Parse time Strings to u64
        let hr: u64 = parse_string_to_u64(self.hr_str.clone());
        let min: u64 = parse_string_to_u64(self.min_str.clone());
        let sec: u64 = parse_string_to_u64(self.sec_str.clone());
        let ms: u64 = parse_string_to_u64(self.ms_str.clone());
        // println!("{} hr {} min {} sec {} ms", &hr, min, sec, ms);

        // Parse movement Strings to u64
        let movement_sec: u64 = parse_string_to_u64(self.movement_sec_str.clone());
        let movement_ms: u64 = parse_string_to_u64(self.movement_ms_str.clone());

        // Calculate movement delay
        let movement_delay_in_ms: u64 = (movement_sec * 1000u64) + movement_ms;

        // Parse click amount String to u64
        let click_amount: u64 = parse_string_to_u64(self.click_amount_str.clone());

        // Parse mouse coordinates Strings to f64
        let click_x: f64 = parse_string_to_f64(self.click_x_str.clone());
        let click_y: f64 = parse_string_to_f64(self.click_y_str.clone());

        // Toggle autoclicking
        if self.key_autoclick.is_some() && keys.contains(&self.key_autoclick.unwrap()) {
            self.key_pressed_autoclick = true;
        } else if self.key_pressed_autoclick {
            self.key_pressed_autoclick = false;
            if self.is_executing {
                self.is_executing = false;
                self.is_moving = false;
                self.is_autoclicking = false;
            } else {
                // Set only if app is not busy
                if !self.is_setting_autoclick_key
                    && !self.is_setting_coord
                    && !self.is_setting_set_coord_key
                    && !self.hotkey_window_open
                {
                    self.click_counter = 0u64;
                    self.is_executing = true;
                    self.rng_thread = thread_rng();
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
                if !self.is_executing
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

        // did we enter execution mode? If yes, start moving
        if self.is_executing && !self.is_moving && !self.is_autoclicking {
            self.is_moving = true;
        }
        // Calculate click interval
        let interval: u64 = (hr * 3600000u64) + (min * 60000u64) + (sec * 1000u64) + ms;

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

        // move to target position
        if self.is_moving {
            #[cfg(debug_assertions)]
            println!(
                "Moving from {:?}/{:?} towards: {:?}/{:?}",
                mouse.coords.0.to_f64(),
                mouse.coords.1.to_f64(),
                click_x,
                click_y
            );
            move_to(
                self.app_mode,
                self.click_position,
                (click_x, click_y),
                self.is_moving_humanlike,
                (mouse.coords.0.to_f64(), mouse.coords.1.to_f64()),
                movement_delay_in_ms,
            );
            if mouse.coords.0.to_f64() == click_x && mouse.coords.1.to_f64() == click_y {
                #[cfg(debug_assertions)]
                println!("Reached destination: {:?}", mouse.coords);
                self.is_moving = false;
                self.is_autoclicking = true;
            };
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
                self.rng_thread.clone(),
            );

            // Increment click counter and stop autoclicking if completed
            self.click_counter += 1u64;
            if click_amount != 0u64 && self.click_counter >= click_amount {
                self.is_autoclicking = false;
                self.is_executing = false;
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
                            if self.is_executing || self.hotkey_window_open {
                                ui.set_enabled(false);
                            };
                            ui.add(
                                egui::TextEdit::singleline(&mut self.click_y_str)
                                    .desired_width(50.0f32)
                                    .hint_text("0"),
                            );
                            ui.label("Y");
                            if self.is_executing || self.hotkey_window_open {
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
                    if self.is_executing {
                        if ui
                            .button(format!("🖱 STOP ({})", self.key_autoclick.unwrap()))
                            .clicked()
                        {
                            self.is_executing = false;
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
                            self.is_executing = true
                        }
                    }

                    ui.separator();
                    ui.label("Settings: ");

                    if ui
                        .add_enabled(!self.is_executing, egui::Button::new("⌨ Hotkeys"))
                        .clicked()
                    {
                        self.hotkey_window_open = true
                    };

                    ui.separator();
                    ui.label("App Mode: ");

                    if self.is_executing || self.hotkey_window_open {
                        ui.set_enabled(false);
                    };
                    ui.selectable_value(&mut self.app_mode, AppMode::Bot, "🖥 Bot")
                        .on_hover_text("Autoclick as fast as possible");
                    if self.is_executing || self.hotkey_window_open {
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
                        if self.is_executing || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.ms_str)
                                .desired_width(40.0f32)
                                .hint_text("0"),
                        );

                        ui.label("sec");
                        if self.is_executing || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.sec_str)
                                .desired_width(40.0f32)
                                .hint_text("0"),
                        );

                        ui.label("min");
                        if self.is_executing || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.min_str)
                                .desired_width(40.0f32)
                                .hint_text("0"),
                        );

                        ui.label("hr");
                        if self.is_executing || self.hotkey_window_open {
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
                ui.label("Movement delay");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.label("ms");
                    if self.is_executing || self.hotkey_window_open {
                        ui.set_enabled(false);
                    };
                    ui.add(
                        egui::TextEdit::singleline(&mut self.movement_ms_str)
                            .desired_width(40.0f32)
                            .hint_text("20"),
                    );

                    ui.label("sec");
                    if self.is_executing || self.hotkey_window_open {
                        ui.set_enabled(false);
                    };
                    ui.add(
                        egui::TextEdit::singleline(&mut self.movement_sec_str)
                            .desired_width(40.0f32)
                            .hint_text("0"),
                    );
                });

                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    ui.label("Mouse Button");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if self.is_executing || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.selectable_value(&mut self.click_btn, Button::Right, "Right");
                        if self.is_executing || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.selectable_value(&mut self.click_btn, Button::Middle, "Middle");
                        if self.is_executing || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.selectable_value(&mut self.click_btn, Button::Left, "Left");
                    });
                });

                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    ui.label("Click Type");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if self.is_executing || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.selectable_value(&mut self.click_type, ClickType::Double, "Double");
                        if self.is_executing || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.selectable_value(&mut self.click_type, ClickType::Single, "Single");
                    });
                });

                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    ui.label("Click Amount (0 = forever)");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if self.is_executing || self.hotkey_window_open {
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
                    if self.is_executing || self.hotkey_window_open {
                        ui.set_enabled(false);
                    };
                    if ui
                        .add_sized([80.0f32, 16.0f32], egui::widgets::Button::new("Set Coords"))
                        .clicked()
                    {
                        Self::enter_coordinate_setting(self, frame);
                    };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        if self.is_executing || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.click_y_str)
                                .desired_width(50.0f32)
                                .hint_text("0"),
                        );
                        ui.label("Y");
                        if self.is_executing || self.hotkey_window_open {
                            ui.set_enabled(false);
                        };
                        ui.add(
                            egui::TextEdit::singleline(&mut self.click_x_str)
                                .desired_width(50.0f32)
                                .hint_text("0"),
                        );
                        ui.label("X");

                        if self.is_executing || self.hotkey_window_open {
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
                    if self.is_executing {
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
                            self.is_executing = false;
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
                            // Self::start_autoclick(self, 0u64);
                            self.is_executing = true
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
                            if !self.is_executing
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
                            if !self.is_executing
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

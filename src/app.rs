// #![allow(unused_imports)]
use std::{
    env, str, thread,
    time::{Duration, Instant},
};

use device_query::{
    device_state, DeviceEvents, DeviceQuery, DeviceState, Keycode, MouseButton, MouseState,
};

use rdev::{simulate, Button, EventType, SimulateError};

use eframe::{
    egui,
    epaint::{FontFamily, FontId},
    epi,
};

#[derive(PartialEq)]
enum ClickType {
    Single,
    Double,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    value: f32,
    hr_str: String,
    min_str: String,
    sec_str: String,
    ms_str: String,
    click_btn: Button,
    click_type: ClickType,
    run_key_pressed: bool,
    is_running: bool,
    run_key: Keycode,
    last_now: Instant,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            hr_str: "0".to_owned(),
            min_str: "0".to_owned(),
            sec_str: "0".to_owned(),
            ms_str: "100".to_owned(),
            click_btn: Button::Left,
            click_type: ClickType::Single,
            run_key_pressed: false,
            is_running: false,
            run_key: Keycode::F6,
            last_now: Instant::now(),
        }
    }
}

// Sanitation to only allow numbers required
fn sanitize_time(string: &mut String) {
    while string.len() > 1 && string.starts_with('0') {
        string.remove(0);
    }
    if string.len() >= 5 {
        string.truncate(5usize)
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

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "Rusty AutoClicker"
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
            size: 16.0,
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
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        let Self {
            label,
            value,
            hr_str,
            min_str,
            sec_str,
            ms_str,
            click_btn,
            click_type,
            run_key_pressed,
            is_running,
            run_key,
            last_now,
        } = self;

        sanitize_time(hr_str);
        sanitize_time(min_str);
        sanitize_time(sec_str);
        sanitize_time(ms_str);

        let mut hr: u64 = 0;
        if !hr_str.is_empty() {
            hr = hr_str.parse().unwrap();
        }
        let mut min: u64 = 0;
        if !min_str.is_empty() {
            min = min_str.parse().unwrap();
        }
        let mut sec: u64 = 0;
        if !sec_str.is_empty() {
            sec = sec_str.parse().unwrap();
        }
        let mut ms: u64 = 0;
        if !ms_str.is_empty() {
            ms = ms_str.parse().unwrap();
        }
        // println!("{} hr {} min {} sec {} ms", &hr, min, sec, ms);

        let device_state = DeviceState::new();

        let mouse: MouseState = device_state.get_mouse();
        let keys: Vec<Keycode> = device_state.get_keys();

        // Toggle clicking
        if keys.contains(&run_key) {
            *run_key_pressed = true;
        } else {
            if *run_key_pressed {
                *run_key_pressed = false;
                if *is_running {
                    *is_running = false;
                } else {
                    *is_running = true;
                    *last_now = Instant::now();
                }
            }
        }

        let delay: u64 = (hr * 3600000u64) + (min * 60000) + (sec * 1000) + ms;

        let update_now = Instant::now();
        if *is_running
            && update_now
                .checked_duration_since(*last_now)
                .unwrap()
                .as_millis() as u64
                >= delay
        {
            #[cfg(debug_assertions)]
            println!(
                "Click {:?}: {:?}",
                click_btn,
                update_now.checked_duration_since(*last_now).unwrap(),
            );
            *last_now = Instant::now();
            send(&EventType::ButtonPress(*click_btn));
            send(&EventType::ButtonRelease(*click_btn));
            if *click_type == ClickType::Double {
                send(&EventType::ButtonPress(*click_btn));
                send(&EventType::ButtonRelease(*click_btn));
            }
        }

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("Click Interval");
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::TextEdit::singleline(hr_str)
                        .desired_width(50.0f32)
                        .hint_text("0"),
                );
                ui.label("hr");
                ui.add(
                    egui::TextEdit::singleline(min_str)
                        .desired_width(50.0f32)
                        .hint_text("0"),
                );
                ui.label("min");
                ui.add(
                    egui::TextEdit::singleline(sec_str)
                        .desired_width(50.0f32)
                        .hint_text("0"),
                );
                ui.label("sec");
                ui.add(
                    egui::TextEdit::singleline(ms_str)
                        .desired_width(50.0f32)
                        .hint_text("0"),
                );
                ui.label("ms");
            });

            ui.separator();

            ui.horizontal_wrapped(|ui| {
                ui.label("Mouse Button");
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    ui.selectable_value(click_btn, Button::Right, "Right");
                    ui.selectable_value(click_btn, Button::Middle, "Middle");
                    ui.selectable_value(click_btn, Button::Left, "Left");
                });
            });

            ui.separator();

            ui.horizontal_wrapped(|ui| {
                ui.label("Click Type");
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    ui.selectable_value(click_type, ClickType::Double, "Double");
                    ui.selectable_value(click_type, ClickType::Single, "Single");
                });
            });

            ui.separator();

            let mouse_txt = format!("Mouse position: {:?}", mouse.coords);
            ui.heading(mouse_txt);
            let key_txt = format!("Key pressed: {:?}", keys);
            ui.heading(key_txt);
            ui.heading(format!("F6 pressed: {}", run_key_pressed));
            ui.heading(format!("Clicking: {}", is_running));

            ui.separator();

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
                });
                ui.horizontal(|ui| {
                    egui::warn_if_debug_build(ui);
                });
            });
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }

        // Keep requesting updates
        ctx.request_repaint();
    }
}

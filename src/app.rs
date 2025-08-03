use std::time::{Duration, Instant};

use device_query::Keycode;
use eframe::{egui, epaint::FontId};
use rand::{prelude::ThreadRng, rng};
use rdev::Button;

use crate::{
    defines::*,
    types::{AppMode, ClickButton, ClickPosition, ClickType},
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct RustyAutoClickerApp {
    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    // Text input strings
    pub hr_str: String,
    pub min_str: String,
    pub sec_str: String,
    pub ms_str: String,
    pub click_amount_str: String,
    pub click_x_str: String,
    pub click_y_str: String,
    pub movement_sec_str: String,
    pub movement_ms_str: String,

    // Time
    pub last_now: Instant,
    pub frame_start: Instant,

    // Counter
    pub click_counter: u64,

    // Hotkeys
    pub key_autoclick: Option<Keycode>,
    pub key_set_coord: Option<Keycode>,

    // App state
    pub is_autoclicking: bool,
    pub is_setting_coord: bool,
    pub is_setting_autoclick_key: bool,
    pub is_setting_set_coord_key: bool,

    // App mode
    pub app_mode: AppMode,

    // Window state
    pub hotkey_window_open: bool,
    pub window_position: egui::Pos2,

    // Key states
    pub key_pressed_autoclick: bool,
    pub key_pressed_set_coord: bool,
    pub key_pressed_esc: bool,
    pub keys_pressed: Option<Vec<Keycode>>,

    // Enums
    pub click_btn: ClickButton,
    pub click_type: ClickType,
    pub click_position: ClickPosition,
    // RNG
    pub rng_thread: ThreadRng,
}

impl Default for RustyAutoClickerApp {
    fn default() -> Self {
        Self {
            // Text input strings
            hr_str: DEFAULT_HR_STR.to_owned(),
            min_str: DEFAULT_MIN_STR.to_owned(),
            sec_str: DEFAULT_SEC_STR.to_owned(),
            ms_str: DEFAULT_MS_STR.to_owned(),
            click_amount_str: DEFAULT_CLICK_AMOUNT_STR.to_owned(),
            click_x_str: DEFAULT_CLICK_X_STR.to_owned(),
            click_y_str: DEFAULT_CLICK_Y_STR.to_owned(),
            movement_sec_str: DEFAULT_MOVEMENT_SEC_STR.to_owned(),
            movement_ms_str: DEFAULT_MOVEMENT_MS_STR.to_owned(),

            // Time
            last_now: Instant::now(),
            frame_start: Instant::now(),

            // Counter
            click_counter: 0u64,

            // Hotkeys
            key_autoclick: HOTKEY_AUTOCLICK,
            key_set_coord: HOTKEY_SET_COORD,

            // App state
            is_autoclicking: false,
            is_setting_coord: false,
            is_setting_autoclick_key: false,
            is_setting_set_coord_key: false,

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
            click_btn: ClickButton::Mouse(Button::Left),
            click_type: ClickType::Single,
            click_position: ClickPosition::Mouse,

            // RNG
            rng_thread: rng(),
        }
    }
}

impl RustyAutoClickerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;

        let mut style = (*ctx.style()).clone();
        let font = FontId {
            size: FONT_SIZE,
            family: FONT_FAMILY,
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

    /// Enter the coordinate setting mode
    ///
    /// # Arguments
    ///
    /// * `ctx` - The ctx to manipulate
    pub fn enter_coordinate_setting(&mut self, ctx: &egui::Context) {
        self.is_setting_coord = true;
        self.window_position =
            ctx.input(|input_state| input_state.viewport().outer_rect.unwrap().min);
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(400f32, 30f32)));
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
    }

    /// Make frame follow cursor with an offset
    ///
    /// # Arguments
    ///
    /// * `ctx` - The ctx to set the window position on
    pub fn follow_cursor(&mut self, ctx: &egui::Context) {
        let offset = egui::Vec2 { x: 15f32, y: 15f32 };
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
            egui::pos2(
                self.click_x_str.parse().unwrap(),
                self.click_y_str.parse().unwrap(),
            ) + offset,
        ));
    }

    /// Exit the coordinate setting mode
    ///
    /// # Arguments
    ///
    /// * `ctx` - The ctx to manipulate
    pub fn exit_coordinate_setting(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        )));
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(self.window_position));

        self.is_setting_coord = false;
        self.click_position = ClickPosition::Coord;
    }

    /// Start the autoclicking process
    ///
    /// # Arguments
    ///
    /// * `negative_click_start_offset` - The offset to start the click counter at
    pub fn start_autoclick(&mut self, negative_click_start_offset: u64) {
        self.click_counter = 0u64;
        self.is_autoclicking = !self.is_autoclicking;
        self.rng_thread = rng();

        self.last_now = Instant::now()
            .checked_sub(Duration::from_millis(negative_click_start_offset))
            .unwrap();
    }
}

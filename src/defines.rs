use device_query::Keycode;
use eframe::egui::FontFamily;

pub const APP_NAME: &str = "Rusty AutoClicker";
pub const APP_ICON: &[u8] = include_bytes!("../assets/icon-64x64.ico");

// Font
pub const FONT_SIZE: f32 = 12.0;
pub const FONT_FAMILY: FontFamily = FontFamily::Monospace;

// dimensions of main window
pub const WINDOW_WIDTH: f32 = 550.0;
pub const WINDOW_HEIGHT: f32 = 341.0;

// ranges for click durations
pub const DURATION_CLICK_MIN: u64 = 20;
pub const DURATION_CLICK_MAX: u64 = 40;
pub const DURATION_DOUBLE_CLICK_MIN: u64 = 30;
pub const DURATION_DOUBLE_CLICK_MAX: u64 = 60;

// step widths for human-like mouse movement
pub const MOUSE_STEP_POS_X: f64 = 10.0;
pub const MOUSE_STEP_NEG_X: f64 = -10.0;
pub const MOUSE_STEP_POS_Y: f64 = 10.0;
pub const MOUSE_STEP_NEG_Y: f64 = -10.0;

// Default input values
pub const DEFAULT_HR_STR: &str = "0";
pub const DEFAULT_MIN_STR: &str = "0";
pub const DEFAULT_SEC_STR: &str = "0";
pub const DEFAULT_MS_STR: &str = "100";
pub const DEFAULT_CLICK_AMOUNT_STR: &str = "0";
pub const DEFAULT_CLICK_X_STR: &str = "0";
pub const DEFAULT_CLICK_Y_STR: &str = "0";
pub const DEFAULT_MOVEMENT_SEC_STR: &str = "0";
pub const DEFAULT_MOVEMENT_MS_STR: &str = "20";

// Hotkeys
pub const HOTKEY_AUTOCLICK: Option<Keycode> = Some(Keycode::F6);
pub const HOTKEY_SET_COORD: Option<Keycode> = Some(Keycode::Escape);

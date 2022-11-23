pub const APP_NAME: &str = "Rusty AutoClicker";
pub const APP_ICON: &[u8] = include_bytes!("../assets/icon-64x64.ico");

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

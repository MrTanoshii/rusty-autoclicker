use std::{fs, path::PathBuf};

use device_query::Keycode;
use serde::{Deserialize, Serialize};

use crate::{
    app::RustyAutoClickerApp,
    types::{AppMode, ClickButton, ClickPosition, ClickType},
};

const SETTINGS_DIR_NAME: &str = "rusty-autoclicker";
const SETTINGS_FILE_NAME: &str = "settings.json";

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Settings {
    // Timing
    pub hr_str: String,
    pub min_str: String,
    pub sec_str: String,
    pub ms_str: String,

    // Click amount
    pub click_amount_str: String,

    // Click coordinates (0,0 by default — separate from window position)
    pub click_x_str: String,
    pub click_y_str: String,

    // Tween speed
    pub speed_min_str: String,
    pub speed_max_str: String,

    // Random offset (+/- jitter on the click interval). `#[serde(default)]`
    // on both so settings.json files saved by older versions (which don't
    // have these keys at all) still deserialize instead of failing to load
    // entirely.
    #[serde(default)]
    pub random_offset_enabled: bool,
    #[serde(default = "default_random_offset_str")]
    pub random_offset_str: String,

    // Hotkeys
    pub key_autoclick: Option<String>,
    pub key_open_set_coord: Option<String>,
    pub key_set_coord: Option<String>,
    pub key_hold: Option<String>,

    // UI-controlled enums
    pub click_btn: String,
    pub click_type: String,
    pub click_position: String,
    pub app_mode: String,

    /// The last keyboard key selected in Keyboard mode, remembered
    /// independently of `click_btn` (which may currently be Mouse mode).
    /// Reused so Mouse -> Keyboard restores this instead of resetting to
    /// Space. Persisted using the same `ClickButton::Key(..)` string form
    /// as `click_btn` above, purely for convenience (reuses
    /// `to_setting_string`/`from_setting_string` rather than needing a
    /// separate key-only serialization helper).
    pub last_keyboard_key: String,

    /// Window inner size [width, height] in logical pixels.
    /// Auto-saved on close; not affected by the Save/Reset buttons.
    pub window_size: Option<[f32; 2]>,

    /// Window outer position [x, y] in logical pixels.
    /// Saved by the Save button and on close; Reset returns to default position.
    pub window_position: Option<[f32; 2]>,
}

impl Settings {
    pub fn from_app(app: &RustyAutoClickerApp) -> Self {
        Self {
            hr_str: app.hr_str.clone(),
            min_str: app.min_str.clone(),
            sec_str: app.sec_str.clone(),
            ms_str: app.ms_str.clone(),

            click_amount_str: app.click_amount_str.clone(),

            click_x_str: app.click_x_str.clone(),
            click_y_str: app.click_y_str.clone(),

            speed_min_str: app.speed_min_str.clone(),
            speed_max_str: app.speed_max_str.clone(),

            random_offset_enabled: app.random_offset_enabled,
            random_offset_str: app.random_offset_str.clone(),

            key_autoclick: app.key_autoclick.map(|k| k.to_string()),
            key_open_set_coord: app.key_open_set_coord.map(|k| k.to_string()),
            key_set_coord: app.key_set_coord.map(|k| k.to_string()),
            key_hold: app.key_hold.map(|k| k.to_string()),

            click_btn: app.click_btn.to_setting_string(),
            click_type: click_type_to_str(app.click_type).to_owned(),
            click_position: click_position_to_str(app.click_position).to_owned(),
            app_mode: app_mode_to_str(app.app_mode).to_owned(),
            last_keyboard_key: ClickButton::Key(app.last_keyboard_key).to_setting_string(),

            window_size: Some(app.last_window_size),
            window_position: Some([app.last_window_pos.x, app.last_window_pos.y]),
        }
    }

    /// Apply this snapshot onto an app instance.
    /// window_size and window_position are handled separately in `new()`.
    pub fn apply_to(&self, app: &mut RustyAutoClickerApp) {
        app.hr_str = self.hr_str.clone();
        app.min_str = self.min_str.clone();
        app.sec_str = self.sec_str.clone();
        app.ms_str = self.ms_str.clone();

        app.click_amount_str = self.click_amount_str.clone();

        app.click_x_str = self.click_x_str.clone();
        app.click_y_str = self.click_y_str.clone();

        app.speed_min_str = self.speed_min_str.clone();
        app.speed_max_str = self.speed_max_str.clone();

        app.random_offset_enabled = self.random_offset_enabled;
        app.random_offset_str = self.random_offset_str.clone();

        app.key_autoclick = parse_keycode(self.key_autoclick.as_deref());
        app.key_open_set_coord = parse_keycode(self.key_open_set_coord.as_deref());
        app.key_set_coord = parse_keycode(self.key_set_coord.as_deref());
        app.key_hold = parse_keycode(self.key_hold.as_deref());

        if let Some(click_btn) = ClickButton::from_setting_string(&self.click_btn) {
            app.click_btn = click_btn;
        }
        app.click_type = str_to_click_type(&self.click_type).unwrap_or(app.click_type);
        app.click_position =
            str_to_click_position(&self.click_position).unwrap_or(app.click_position);
        app.app_mode = str_to_app_mode(&self.app_mode).unwrap_or(app.app_mode);

        if let Some(ClickButton::Key(k)) = ClickButton::from_setting_string(&self.last_keyboard_key) {
            app.last_keyboard_key = k;
        }
    }
}

// ---------- helpers --------------------------------------------------------

fn default_random_offset_str() -> String {
    crate::defines::DEFAULT_RANDOM_OFFSET_STR.to_owned()
}

fn parse_keycode(s: Option<&str>) -> Option<Keycode> {
    s.and_then(|s| s.parse::<Keycode>().ok())
}

fn click_type_to_str(t: ClickType) -> &'static str {
    match t { ClickType::Single => "Single", ClickType::Double => "Double" }
}
fn str_to_click_type(s: &str) -> Option<ClickType> {
    match s { "Single" => Some(ClickType::Single), "Double" => Some(ClickType::Double), _ => None }
}

fn click_position_to_str(p: ClickPosition) -> &'static str {
    match p { ClickPosition::Mouse => "Mouse", ClickPosition::Coord => "Coord" }
}
fn str_to_click_position(s: &str) -> Option<ClickPosition> {
    match s { "Mouse" => Some(ClickPosition::Mouse), "Coord" => Some(ClickPosition::Coord), _ => None }
}

fn app_mode_to_str(m: AppMode) -> &'static str {
    match m { AppMode::Bot => "Bot", AppMode::Humanlike => "Humanlike" }
}
fn str_to_app_mode(s: &str) -> Option<AppMode> {
    match s { "Bot" => Some(AppMode::Bot), "Humanlike" => Some(AppMode::Humanlike), _ => None }
}

// ---------- I/O ------------------------------------------------------------

fn settings_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join(SETTINGS_DIR_NAME))
}

fn settings_path() -> Option<PathBuf> {
    settings_dir().map(|d| d.join(SETTINGS_FILE_NAME))
}

pub fn load_settings() -> Option<Settings> {
    let data = fs::read_to_string(settings_path()?).ok()?;
    serde_json::from_str(&data).ok()
}

/// Write all settings to disk (called by Save button, Reset, and Drop).
pub fn save_settings(settings: &Settings) {
    let Some(dir) = settings_dir() else { return };
    if fs::create_dir_all(&dir).is_err() { return }
    let Some(path) = settings_path() else { return };
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(path, json);
    }
}

/// Targeted patch: update ONLY window_size and window_position in the JSON
/// on disk without touching anything else. Called from Drop so unsaved
/// user settings are never overwritten by the window-geometry auto-save.
pub fn save_window_geometry(width: f32, height: f32, pos_x: f32, pos_y: f32) {
    let Some(dir) = settings_dir() else { return };
    if fs::create_dir_all(&dir).is_err() { return }
    let Some(path) = settings_path() else { return };

    let mut value: serde_json::Value = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::Value::Object(Default::default()));

    if let Some(obj) = value.as_object_mut() {
        obj.insert("window_size".to_string(), serde_json::json!([width, height]));
        obj.insert("window_position".to_string(), serde_json::json!([pos_x, pos_y]));
    }

    if let Ok(json) = serde_json::to_string_pretty(&value) {
        let _ = fs::write(path, json);
    }
}

// ---------- tests ----------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_type_round_trips() {
        assert_eq!(str_to_click_type(click_type_to_str(ClickType::Single)), Some(ClickType::Single));
        assert_eq!(str_to_click_type(click_type_to_str(ClickType::Double)), Some(ClickType::Double));
    }

    #[test]
    fn click_position_round_trips() {
        assert_eq!(str_to_click_position(click_position_to_str(ClickPosition::Mouse)), Some(ClickPosition::Mouse));
        assert_eq!(str_to_click_position(click_position_to_str(ClickPosition::Coord)), Some(ClickPosition::Coord));
    }

    #[test]
    fn app_mode_round_trips() {
        assert_eq!(str_to_app_mode(app_mode_to_str(AppMode::Bot)), Some(AppMode::Bot));
        assert_eq!(str_to_app_mode(app_mode_to_str(AppMode::Humanlike)), Some(AppMode::Humanlike));
    }
}

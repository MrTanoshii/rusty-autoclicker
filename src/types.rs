use std::fmt;

use rdev::{Button, Key};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum AppMode {
    Bot,
    Humanlike,
}

/// Mutually-exclusive interaction states; exactly one is active at a time,
/// which makes "two modes at once" unrepresentable.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum InteractionMode {
    Idle,
    Autoclicking,
    Holding,
    SettingCoord,
    SettingAutoclickKey,
    SettingOpenCoordKey,
    SettingSetCoordKey,
    SettingHoldKey,
    SettingClickKey,
}

#[derive(PartialEq, Copy, Clone)]
pub struct ClickInfo {
    pub click_btn: ClickButton,
    pub click_coord: (f64, f64),
    pub click_position: ClickPosition,
    pub click_type: ClickType,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ClickPosition {
    Mouse,
    Coord,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ClickType {
    Single,
    Double,
}

impl ClickType {
    /// Number of press/release cycles performed per autoclick tick.
    pub const fn run_count(self) -> u8 {
        match self {
            ClickType::Single => 1,
            ClickType::Double => 2,
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ClickButton {
    Mouse(Button),
    Key(Key),
}

impl fmt::Display for ClickButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClickButton::Mouse(button) => write!(f, "{button:?}"),
            ClickButton::Key(key) => write!(f, "{}", crate::utils::abbreviate_key(*key)),
        }
    }
}

impl ClickButton {
    /// Stable, persistence-friendly string form, e.g. `"mouse:Left"` or
    /// `"key:KeyA"`. Neither `rdev::Button` nor `rdev::Key` implement
    /// `serde` traits, so settings persistence round-trips through this
    /// instead of deriving `Serialize`/`Deserialize` directly on
    /// [`ClickButton`].
    pub fn to_setting_string(self) -> String {
        match self {
            ClickButton::Mouse(button) => format!("mouse:{}", mouse_button_name(button)),
            ClickButton::Key(key) => format!("key:{}", key_name(key)),
        }
    }

    /// Parse the string form produced by [`Self::to_setting_string`].
    /// Returns `None` for anything unrecognized (e.g. a settings file from
    /// an incompatible version), so callers can fall back to a default.
    pub fn from_setting_string(s: &str) -> Option<Self> {
        let (kind, value) = s.split_once(':')?;
        match kind {
            "mouse" => mouse_button_from_name(value).map(ClickButton::Mouse),
            "key" => key_from_name(value).map(ClickButton::Key),
            _ => None,
        }
    }
}

fn mouse_button_name(button: Button) -> &'static str {
    match button {
        Button::Left => "Left",
        Button::Right => "Right",
        Button::Middle => "Middle",
        // rdev::Button has a non-exhaustive `Unknown(u8)` variant that the UI
        // never selects; fall back to Left rather than failing to persist.
        _ => "Left",
    }
}

fn mouse_button_from_name(s: &str) -> Option<Button> {
    match s {
        "Left" => Some(Button::Left),
        "Right" => Some(Button::Right),
        "Middle" => Some(Button::Middle),
        _ => None,
    }
}

/// Generates `key_name`/`key_from_name` covering exactly the `rdev::Key`
/// variants offered in the keyboard-button dropdown
/// (`gui/sections/buttons.rs`), so the persisted string always matches a
/// selectable value.
macro_rules! key_names {
    ($($variant:ident),* $(,)?) => {
        fn key_name(key: Key) -> &'static str {
            match key {
                $(Key::$variant => stringify!($variant),)*
                // Variants outside the dropdown's selectable set; persisting
                // these can't happen via the UI, but keep this exhaustive.
                _ => "Space",
            }
        }

        fn key_from_name(s: &str) -> Option<Key> {
            match s {
                $(stringify!($variant) => Some(Key::$variant),)*
                _ => None,
            }
        }
    };
}

key_names!(
    // Modifier keys
    Alt, AltGr, CapsLock, ControlLeft, ControlRight, MetaLeft, MetaRight, ShiftLeft, ShiftRight,
    Function,
    // Navigation
    UpArrow, DownArrow, LeftArrow, RightArrow, Home, End, PageUp, PageDown, Insert, Delete,
    Backspace,
    Escape, Return, Tab, Space,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Print/Lock
    PrintScreen, ScrollLock, Pause, NumLock,
    // Top row number keys and symbols
    BackQuote, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9, Num0, Minus, Equal,
    // Letter keys
    KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN, KeyO,
    KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
    // Punctuation and symbol keys
    LeftBracket, RightBracket, SemiColon, Quote, BackSlash, IntlBackslash, Comma, Dot, Slash,
    // Keypad
    KpReturn, KpMinus, KpPlus, KpMultiply, KpDivide, Kp0, Kp1, Kp2, Kp3, Kp4, Kp5, Kp6, Kp7, Kp8,
    Kp9, KpDelete,
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_count_matches_click_type() {
        assert_eq!(ClickType::Single.run_count(), 1);
        assert_eq!(ClickType::Double.run_count(), 2);
    }
}
